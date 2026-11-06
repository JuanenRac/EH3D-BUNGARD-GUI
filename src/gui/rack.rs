//! The tool changer of the ATC models. The rack holds a number of tools (the test tool is in slot 0 and in the spindle, so the slots count for
//! the tools alone); a job with more tools than that is done in loads: the tools that fit, then a stop to load the rack again, then the rest.
//! Before every load a window shows what goes in each slot, as the operator has to put it there.

use super::*;
use crate::machines::ToolChange;
use eh3d_bungard_formats::config::{rack_line, AtcSlot};

impl App {
    /// Does the machine of the settings change the tools by itself?
    pub(crate) fn has_changer(&self) -> bool {
        crate::machines::models()
            .get(self.model())
            .is_some_and(|m| m.change == ToolChange::Automatic)
    }

    /// The places of the bar a program of the job is planned with when its own machine file does not carry them: the routing takes those of the drilling
    /// file (the bar is one, the two programs share it), and a bar with no places anywhere is planned with the ones it is drawn in, but only on a
    /// simulated machine (on a real one a bar nobody has measured is not driven to: the operator changes the tools). `None`: the program is planned
    /// with what its file has.
    pub(crate) fn bar_for_job(&self, p: Program) -> Option<Vec<AtcSlot>> {
        if !self.has_changer() {
            return None;
        }
        let drill_atc = self
            .drill
            .engine
            .settings()
            .map(|s| s.atc.clone())
            .unwrap_or_default();
        let (own, real) = match p {
            Program::Drill => (&drill_atc, self.drill.engine.connection().is_real()),
            Program::Route => (
                &self
                    .route
                    .engine
                    .settings()
                    .map(|s| s.atc.clone())
                    .unwrap_or_default(),
                self.route.engine.connection().is_real(),
            ),
        };
        if !own.is_empty() {
            return None;
        }
        if !drill_atc.is_empty() {
            return Some(drill_atc);
        }
        if real {
            return None;
        }
        let (a, b, n) = self.bar_default();
        Some(rack_line(a, b, n))
    }

    /// Puts the places of the bar into the file of a program for the start of a job (and the automatic change on), and gives back what was there, to
    /// be put back by [`App::restore_bar`] once the job is planned: the machine file keeps only what the person set.
    pub(crate) fn lend_bar(&mut self, p: Program, line: &[AtcSlot]) -> (Vec<AtcSlot>, bool) {
        let side = self.side_mut(p);
        let Some(st) = side.engine.settings_mut() else {
            return (Vec::new(), false);
        };
        let before = (
            std::mem::replace(&mut st.atc, line.to_vec()),
            st.auto_change,
        );
        st.auto_change = true;
        before
    }

    pub(crate) fn restore_bar(&mut self, p: Program, before: (Vec<AtcSlot>, bool)) {
        if let Some(st) = self.side_mut(p).engine.settings_mut() {
            st.atc = before.0;
            st.auto_change = before.1;
        }
    }

    /// How many tools the rack holds at once.
    pub(crate) fn rack_places(&self) -> usize {
        crate::machines::models()
            .get(self.model())
            .map_or(0, |m| m.rack_places() as usize)
    }

    /// The tools of a program that the job does, in the order it does them.
    pub(crate) fn tools_to_do(&self, p: Program) -> Vec<u8> {
        let off = self.tools_off(p);
        self.job_tool_order(p)
            .into_iter()
            .filter(|t| !off.contains(t))
            .collect()
    }

    /// The loads of the rack for the programs of a job: each program's tools in pieces that fit the rack, and consecutive pieces together while
    /// they fit. Programs with no tools to do are left out.
    pub(crate) fn rack_plan(&self, programs: &[Program]) -> Vec<Vec<(Program, Vec<u8>)>> {
        self.rack_plan_for(programs, self.rack_places().max(1))
    }

    fn rack_plan_for(&self, programs: &[Program], cap: usize) -> Vec<Vec<(Program, Vec<u8>)>> {
        let tools: Vec<crate::jobflow::Piece> =
            programs.iter().map(|&p| (p, self.tools_to_do(p))).collect();
        crate::jobflow::plan_loads(&tools, cap)
    }

    /// The tool of each slot of the bar: the slot, the program and the tool. Where the person gave the tools their places (always, to start a job on the changer) these are the
    /// places, and every tool of the job is in the bar from the first moment to the last, whatever has run. Without places, slot 1 holds the first tool the job uses and the
    /// rest follow in the order of the job.
    pub(crate) fn rack_slot_tools(&self) -> Vec<(u8, Program, u8)> {
        if !self.has_changer() {
            return Vec::new();
        }
        let given = self.slot_tools_given();
        if !given.is_empty() {
            return given;
        }
        let live = if !self.rack_show.is_empty() {
            // the window of the rack is open: what goes in the rack
            Some(self.rack_show.clone())
        } else if self.rack_active && !self.rack_load.is_empty() {
            // a job is going on: the whole load, the programs that ran before this one included
            Some(self.rack_load.clone())
        } else {
            None
        };
        let pieces = live.unwrap_or_else(|| {
            self.rack_plan(&[Program::Drill, Program::Route])
                .into_iter()
                .next()
                .unwrap_or_default()
        });
        crate::jobflow::slot_tools(&pieces)
    }

    /// Shows the next load of the rack and waits for the operator.
    pub(crate) fn rack_next(&mut self) {
        if self.rack_loads.is_empty() {
            return;
        }
        self.rack_show = self.rack_loads.remove(0);
        self.rack_load = self.rack_show.clone();
        self.rack_no.0 += 1;
        self.show_modal(Modal::Rack);
    }

    /// The operator has put the tools in the rack: the load starts with its first piece.
    pub(crate) fn rack_go(&mut self) {
        if self.rack_show.is_empty() {
            self.modal = None;
            return;
        }
        self.modal = None;
        let mut pieces = std::mem::take(&mut self.rack_show);
        let first = pieces.remove(0);
        self.rack_pieces = pieces;
        self.rack_only = Some(first.clone());
        let skip = std::mem::take(&mut self.rack_skip);
        self.rack_first = 0;
        self.rack_chain = false;
        self.rack_last = None;
        self.begin_program(first.0, skip);
    }

    pub(crate) fn rack_cancel(&mut self) {
        self.rack_clear();
        self.modal = None;
    }

    pub(crate) fn rack_clear(&mut self) {
        self.rack_active = false;
        self.rack_loads.clear();
        self.rack_show.clear();
        self.rack_pieces.clear();
        self.rack_only = None;
        self.rack_skip = 0;
        self.rack_first = 0;
        self.rack_chain = false;
        self.rack_last = None;
        self.rack_load.clear();
    }

    /// The tool of a program with its diameter (drills) or its number (routers).
    fn rack_label(&self, p: Program, tool: u8) -> String {
        let line = self
            .tool_lines()
            .into_iter()
            .find(|l| l.prog == p && l.tool == tool);
        match (p, line) {
            (Program::Drill, Some(l)) => format!("T{tool:02}  {}", l.main),
            (Program::Route, Some(l)) => format!("T{tool:02}  {}", l.main),
            _ => format!("T{tool:02}"),
        }
    }

    pub(super) fn paint_rack(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let tools: Vec<(Program, u8)> = self
            .rack_show
            .iter()
            .flat_map(|(p, ts)| ts.iter().map(move |&x| (*p, x)))
            .collect();
        let rows = tools.len().div_ceil(2).max(1);
        let tips = super::extra::wrap(self.tr(Text::RackTips), 96);
        let h = 250.0 + rows as f32 * 34.0 + tips.len() as f32 * 22.0 + 12.0;
        let title = if self.rack_no.1 > 1 {
            format!(
                "{}  -  {} {}/{}",
                self.tr(Text::RackTitle),
                self.tr(Text::RackGroup),
                self.rack_no.0,
                self.rack_no.1
            )
        } else {
            self.tr(Text::RackTitle).to_string()
        };
        let (x, y) = self.modal_card(cv, &title, h);
        cv.text_fit(
            x + 30.0,
            y + 94.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::RackTestTool),
            16.0,
            col(t.warn),
            680.0,
        );
        cv.text_fit(
            x + 30.0,
            y + 124.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::RackReload),
            15.0,
            col(t.dim),
            680.0,
        );
        // the slots, in two columns: slot 1 is the first tool the job uses
        for (i, (p, tool)) in tools.iter().enumerate() {
            let (col_x, row) = (x + 30.0 + (i / rows) as f32 * 350.0, i % rows);
            let ry = y + 144.0 + row as f32 * 34.0;
            let r = cv.rect(col_x, ry, 330.0, 30.0);
            cv.p.rect_filled(r, 6.0 * cv.u, col(t.panel_raised));
            // the place the person gave the tool, or the next one of the load when it has none
            let place = crate::slots::slot_of(&self.slot_assign, *p, *tool).unwrap_or(i + 1);
            cv.text(
                col_x + 12.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                &format!("{place}"),
                15.0,
                col(t.accent),
            );
            cv.text_fit(
                col_x + 44.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                &self.rack_label(*p, *tool),
                15.0,
                col(t.text),
                270.0,
            );
        }
        // the things to look at when the rack is loaded (the length of the tools, the air, the depth limiter)
        for (i, line) in tips.iter().enumerate() {
            cv.text(
                x + 30.0,
                y + 144.0 + rows as f32 * 34.0 + 24.0 + i as f32 * 22.0,
                Align2::LEFT_BOTTOM,
                line,
                14.0,
                col(t.dim),
            );
        }
        let by = y + h - 70.0;
        cv.chip(
            x + 30.0,
            by,
            200.0,
            44.0,
            self.tr(if self.rack_no.0 > 1 {
                Text::Resume
            } else {
                Text::Start
            }),
            true,
            Act::RackGo,
        );
        cv.chip(
            x + 250.0,
            by,
            170.0,
            44.0,
            self.tr(Text::Close),
            false,
            Act::RackCancel,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("eh3d-bg-rack-{}-{n}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        for f in [
            "DRILLPRO.DCF",
            "DRILLPRO.DTD",
            "ROUTEPRO.RCF",
            "ROUTEPRO.RTD",
            "05_small_board.ncd",
            "05_board_outline.plt",
        ] {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("testdata")
                    .join(f),
                dir.join(f),
            )
            .unwrap();
        }
        let mut a = App::new(&Options {
            data: Some(dir.clone()),
            screen: Some(Screen::Viewer),
            ..Default::default()
        });
        a.open_path(&dir.join("05_small_board.ncd"));
        a.open_path(&dir.join("05_board_outline.plt"));
        a
    }

    #[test]
    fn a_machine_by_hand_has_no_rack_and_a_job_starts_at_once() {
        let mut a = app();
        a.settings.model = "CCD MTC".to_string();
        assert!(!a.has_changer());
        a.act(Act::Start);
        assert!(a.modal.is_none());
        assert!(a.running.is_some());
    }

    #[test]
    fn a_machine_with_the_changer_shows_the_slots_before_the_job_starts() {
        let mut a = app();
        a.settings.model = "CCD/2 ATC".to_string();
        assert!(a.has_changer() && a.rack_places() > 0);
        a.assign_job_order();
        a.act(Act::Start);
        assert_eq!(a.modal, Some(Modal::Rack));
        assert!(a.running.is_none());
        assert!(!a.rack_show.is_empty());
        a.act(Act::RackGo);
        assert!(a.modal.is_none());
        assert!(a.running.is_some());
    }

    #[test]
    fn each_slot_of_the_rack_is_told_its_tool_in_the_order_of_the_job() {
        let mut a = app();
        a.settings.model = "CCD MTC".to_string();
        assert!(a.rack_slot_tools().is_empty());
        a.settings.model = "CCD/2 ATC".to_string();
        let slots = a.rack_slot_tools();
        let tools = a.tools_to_do(Program::Drill).len() + a.tools_to_do(Program::Route).len();
        assert_eq!(slots.len(), tools.min(a.rack_places()));
        assert_eq!(slots[0].0, 1);
        let first_drill = a.tools_to_do(Program::Drill)[0];
        assert_eq!((slots[0].1, slots[0].2), (Program::Drill, first_drill));
    }

    #[test]
    fn more_tools_than_slots_are_done_in_loads_with_a_stop_to_load_the_rack_again() {
        let mut a = app();
        a.settings.model = "CCD/2 ATC".to_string();
        let n = a.tools_to_do(Program::Drill).len();
        assert!(n >= 3);
        // a rack of two slots: the drills go two at a time, the routing with the last pair if it fits
        let drills = a.tools_to_do(Program::Drill);
        let routes = a.tools_to_do(Program::Route);
        let plan = a.rack_plan_for(&[Program::Drill, Program::Route], 2);
        assert!(plan.len() >= 2);
        for load in &plan {
            let held: usize = load.iter().map(|(_, t)| t.len()).sum();
            assert!(held <= 2, "{load:?}");
        }
        // every tool is in a load, once, in the order of the job
        let all: Vec<(Program, u8)> = plan
            .iter()
            .flatten()
            .flat_map(|(p, ts)| ts.iter().map(move |&t| (*p, t)))
            .collect();
        let want: Vec<(Program, u8)> = drills
            .iter()
            .map(|&t| (Program::Drill, t))
            .chain(routes.iter().map(|&t| (Program::Route, t)))
            .collect();
        assert_eq!(all, want);
        // the real rack holds every tool in one load
        let plan = a.rack_plan(&[Program::Drill, Program::Route]);
        assert_eq!(plan.len(), 1);
    }

    /// Plays a whole job and says whether the operator was ever asked for a tool and how far along X the head went.
    fn play_whole_job(a: &mut App) -> (bool, f64) {
        a.assign_job_order();
        a.auto_ok = true;
        a.fixed_dt = Some(2.0);
        a.act(Act::Start);
        if a.modal == Some(Modal::Rack) {
            a.act(Act::RackGo);
        }
        let (mut asked, mut max_x) = (false, 0.0f64);
        for _ in 0..200_000 {
            for p in [Program::Drill, Program::Route] {
                let side = match p {
                    Program::Drill => &a.drill,
                    Program::Route => &a.route,
                };
                if matches!(
                    side.engine.question(),
                    Some(eh3d_drillpro::classic::engine::Question::ToolChange(_))
                ) {
                    asked = true;
                }
            }
            if let Some(p) = a.running {
                let side = match p {
                    Program::Drill => &a.drill,
                    Program::Route => &a.route,
                };
                if let Some(h) = side.engine.head_mm() {
                    max_x = max_x.max(h.0);
                }
            }
            a.tick();
            if a.modal == Some(Modal::Rack) {
                a.act(Act::RackGo);
            }
            if a.stage_wait {
                a.act(Act::Start);
            }
            if a.running.is_none() && a.modal.is_none() && a.rack_loads.is_empty() && !a.stage_wait
            {
                break;
            }
        }
        (asked, max_x)
    }

    #[test]
    fn with_the_changer_the_operator_is_never_asked_for_a_tool_and_the_head_goes_to_the_bar() {
        for moved in [false, true] {
            let mut a = app();
            a.apply_model(crate::machines::Model::from_name("CCD/2 ATC").unwrap());
            a.assign_job_order();
            if moved {
                a.shift_board(60.0, 40.0);
                // the job drills where the board was moved to, as the views draw it (and not where the file was when it was opened)
                a.auto_ok = true;
                a.act(Act::Start);
                a.act(Act::RackGo);
                let mut job = a.drill.engine.hole_order();
                let mut seen: Vec<(f64, f64)> = a
                    .drill
                    .scene
                    .holes
                    .iter()
                    .map(|h| (h.x_mm, h.y_mm))
                    .collect();
                let key =
                    |p: &(f64, f64)| ((p.0 * 100.0).round() as i64, (p.1 * 100.0).round() as i64);
                job.sort_by_key(key);
                seen.sort_by_key(key);
                assert_eq!(job.len(), seen.len());
                for (j, s) in job.iter().zip(&seen) {
                    assert!(
                        (j.0 - s.0).abs() < 0.011 && (j.1 - s.1).abs() < 0.011,
                        "{j:?} {s:?}"
                    );
                }
                a.stop_job();
            }
            let (asked, max_x) = play_whole_job(&mut a);
            assert!(
                !asked,
                "the operator was asked to change a tool (moved: {moved})"
            );
            assert!(
                max_x > a.table.width_mm,
                "the head never went to the bar: {max_x} (moved: {moved})"
            );
            assert!(a.drill.scene.holes.iter().all(|h| h.done), "moved: {moved}");
        }
    }

    #[test]
    fn a_mixed_job_with_the_changer_drills_and_then_routes_taking_every_tool_from_the_bar() {
        let mut a = app();
        a.apply_model(crate::machines::Model::from_name("CCD/2 ATC").unwrap());
        assert_eq!(a.job_programs(), vec![Program::Drill, Program::Route]);
        assert!(!a.tools_to_do(Program::Route).is_empty());
        let (asked, max_x) = play_whole_job(&mut a);
        // nobody was asked for a tool, the head went to the bar, and both files are done at the end
        assert!(!asked);
        assert!(max_x > a.table.width_mm);
        assert!(a.drill.scene.holes.iter().all(|h| h.done));
        assert!(!a.route.scene.paths.is_empty());
        assert!(a
            .route
            .scene
            .paths
            .iter()
            .all(|q| q.reached == q.points.len()));
        assert!(a.running.is_none() && a.rack_loads.is_empty());
    }

    #[test]
    fn with_the_changer_the_routing_takes_its_places_after_the_drills_and_gets_the_chuck_from_the_last_drill(
    ) {
        let mut a = app();
        a.apply_model(crate::machines::Model::from_name("CCD/2 ATC").unwrap());
        a.auto_ok = true;
        a.fixed_dt = Some(2.0);
        let drills = a.tools_to_do(Program::Drill).len();
        assert!(drills > 0 && !a.tools_to_do(Program::Route).is_empty());
        a.assign_job_order();
        a.act(Act::Start);
        a.act(Act::RackGo);
        assert_eq!(a.rack_first, 0);
        // the drilling hands its chuck on: the routing is the next piece of the same load
        assert_eq!(a.rack_pieces.first().map(|x| x.0), Some(Program::Route));
        for _ in 0..200_000 {
            if a.stage_wait {
                break;
            }
            a.tick();
        }
        assert!(a.stage_wait, "the drilling did not end");
        // the rack is not asked for again and the window that follows the routing is shown
        assert!(a.modal.is_none());
        a.act(Act::Start);
        assert_eq!(a.running, Some(Program::Route));
        assert_eq!(a.rack_first, drills);
    }

    #[test]
    fn by_hand_the_operator_is_asked_for_each_tool() {
        let mut a = app();
        a.apply_model(crate::machines::Model::from_name("CCD MTC").unwrap());
        let (asked, max_x) = play_whole_job(&mut a);
        assert!(asked);
        assert!(max_x <= a.table.width_mm);
    }

    #[test]
    fn the_places_proposed_for_a_bar_without_places_are_not_kept_in_the_machine_file() {
        let mut a = app();
        a.apply_model(crate::machines::Model::from_name("CCD/2 ATC").unwrap());
        assert!(a.bar_for_job(Program::Drill).is_some());
        assert!(a.bar_for_job(Program::Route).is_some());
        let _ = play_whole_job(&mut a);
        assert!(a.drill.engine.settings().unwrap().atc.is_empty());
    }
}
