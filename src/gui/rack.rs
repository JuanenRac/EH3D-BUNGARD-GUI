//! The tool changer of the ATC models. The rack holds a number of tools (the test tool is in slot 0 and in the spindle, so the slots count for
//! the tools alone); a job with more tools than that is done in loads: the tools that fit, then a stop to load the rack again, then the rest.
//! Before every load a window shows what goes in each slot, as the operator has to put it there.

use super::*;
use crate::machines::ToolChange;

impl App {
    /// Does the machine of the settings change the tools by itself?
    pub(crate) fn has_changer(&self) -> bool {
        crate::machines::models()
            .get(self.model())
            .is_some_and(|m| m.change == ToolChange::Automatic)
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
        let pieces: Vec<(Program, Vec<u8>)> = programs
            .iter()
            .flat_map(|&p| {
                self.tools_to_do(p)
                    .chunks(cap)
                    .map(|c| (p, c.to_vec()))
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut loads: Vec<Vec<(Program, Vec<u8>)>> = Vec::new();
        let (mut cur, mut n): (Vec<(Program, Vec<u8>)>, usize) = (Vec::new(), 0);
        for piece in pieces {
            if n + piece.1.len() > cap && !cur.is_empty() {
                loads.push(std::mem::take(&mut cur));
                n = 0;
            }
            n += piece.1.len();
            cur.push(piece);
        }
        if !cur.is_empty() {
            loads.push(cur);
        }
        loads
    }

    /// The tool of each slot of the first load of the job (slot 1 holds the first tool the job uses), for the plan view.
    pub(crate) fn rack_assignment(&self) -> Vec<(u8, String)> {
        if !self.has_changer() {
            return Vec::new();
        }
        let live = if self.rack_only.is_some() || !self.rack_show.is_empty() {
            // a job is going on or the window of the rack is open: what is in the rack now
            let mut pieces: Vec<(Program, Vec<u8>)> = self.rack_show.clone();
            if let Some(p) = &self.rack_only {
                pieces.insert(0, p.clone());
            }
            pieces.extend(self.rack_pieces.clone());
            Some(pieces)
        } else {
            None
        };
        let pieces = live.unwrap_or_else(|| {
            self.rack_plan(&[Program::Drill, Program::Route])
                .into_iter()
                .next()
                .unwrap_or_default()
        });
        pieces
            .iter()
            .flat_map(|(p, ts)| ts.iter().map(move |&t| (*p, t)))
            .enumerate()
            .map(|(i, (_, t))| ((i + 1) as u8, format!("T{t:02}")))
            .collect()
    }

    /// Shows the next load of the rack and waits for the operator.
    pub(crate) fn rack_next(&mut self) {
        if self.rack_loads.is_empty() {
            return;
        }
        self.rack_show = self.rack_loads.remove(0);
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
            cv.text(
                col_x + 12.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                &format!("{}", i + 1),
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
        assert!(a.rack_assignment().is_empty());
        a.settings.model = "CCD/2 ATC".to_string();
        let slots = a.rack_assignment();
        let tools = a.tools_to_do(Program::Drill).len() + a.tools_to_do(Program::Route).len();
        assert_eq!(slots.len(), tools.min(a.rack_places()));
        assert_eq!(slots[0].0, 1);
        assert!(slots.iter().all(|(_, t)| t.starts_with('T')));
        let first_drill = a.tools_to_do(Program::Drill)[0];
        assert_eq!(slots[0].1, format!("T{first_drill:02}"));
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
}
