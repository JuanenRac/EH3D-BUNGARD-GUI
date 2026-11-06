//! The window of the places of the tools in the bar of the tool changer. The tools of a job can be in the bar in any order, so each tool has a drop-down list with the
//! place it is in; the job does not start while a tool has no place, two tools share one, or a place is not defined in the machine file. What is chosen is kept for the
//! machine, by the size and the kind of the tool and by its number, so the next job with the same tools finds the places as they were left. The logic is in
//! `crate::slots`; this file keeps the glue with the window and the drawing.

use super::*;
use crate::slots::{self, JobTool, Problem};

/// A tool of the job as the window lists it: what it is, the colour of its size, and where it is in the job.
pub(crate) struct SlotRow {
    pub tool: JobTool,
    pub color: usize,
    pub done: bool,
    pub using: bool,
}

/// How many rows the window shows (the bar holds 14 tools).
const ROWS_SHOWN: usize = 14;

impl App {
    /// The tools the job uses, drilling first and then routing, each with its size: the ones of the programs that are on and the tools that are on.
    pub(crate) fn slot_rows(&self) -> Vec<SlotRow> {
        let working = self
            .running
            .and_then(|p| self.running_view().and_then(|v| v.tool).map(|n| (p, n)));
        self.tool_lines()
            .into_iter()
            .filter(|l| self.program_runs(l.prog) && self.tool_is_on(l.prog, l.tool))
            .map(|l| {
                let diameter_um = match l.prog {
                    Program::Drill => self
                        .drill
                        .scene
                        .holes
                        .iter()
                        .find(|h| h.tool == l.tool)
                        .map_or(0, |h| (h.diameter_mm * 1000.0).round() as u32),
                    Program::Route => 0,
                };
                SlotRow {
                    tool: JobTool {
                        prog: l.prog,
                        tool: l.tool,
                        diameter_um,
                    },
                    color: l.color,
                    done: l.count > 0 && l.count_done >= l.count,
                    using: working == Some((l.prog, l.tool)),
                }
            })
            .collect()
    }

    pub(crate) fn job_slot_tools(&self) -> Vec<JobTool> {
        self.slot_rows().into_iter().map(|r| r.tool).collect()
    }

    /// The places of the bar of the machine file (or of the model while the file has none).
    pub(crate) fn slot_defined(&self) -> Vec<usize> {
        self.bar_view()
            .map(|v| v.slots.iter().map(|s| s.index as usize).collect())
            .unwrap_or_default()
    }

    /// What is wrong with the places of the tools of the job; empty when it can start.
    pub(crate) fn slot_problems(&self) -> Vec<Problem> {
        slots::check(
            &self.job_slot_tools(),
            &self.slot_assign,
            &self.slot_defined(),
        )
    }

    /// The tool and the place it is in, for each tool of the job that has a place, by place: what the bar shows.
    pub(crate) fn slot_tools_given(&self) -> Vec<(u8, Program, u8)> {
        let mut v: Vec<(u8, Program, u8)> = self
            .slot_rows()
            .iter()
            .filter_map(|r| {
                slots::slot_of(&self.slot_assign, r.tool.prog, r.tool.tool)
                    .map(|s| (s as u8, r.tool.prog, r.tool.tool))
            })
            .collect();
        v.sort_by_key(|(s, _, _)| *s);
        v
    }

    /// The places the planner of a program takes its tools from, and the place of the tool the chuck comes holding. Nothing when the job is not on the changer.
    pub(crate) fn slots_for_program(
        &self,
        program: Program,
        carry_in: bool,
    ) -> (Vec<(u8, usize)>, Option<usize>) {
        if !self.rack_active {
            return (Vec::new(), None);
        }
        let mine: Vec<u8> = self
            .job_slot_tools()
            .iter()
            .filter(|t| t.prog == program)
            .map(|t| t.tool)
            .collect();
        let given = slots::engine_slots(&self.slot_assign, program)
            .into_iter()
            .filter(|(t, _)| mine.contains(t))
            .collect();
        let carried = if carry_in {
            self.rack_last
                .and_then(|(p, t)| slots::slot_of(&self.slot_assign, p, t))
        } else {
            None
        };
        (given, carried)
    }

    fn slot_book_key(&self) -> String {
        launcher::model_key(&self.settings.model)
    }

    /// Fills in the places of the tools that have none from what was remembered for this machine. A tool whose size is not the one it had when it was given a place
    /// loses it (the tool that had that place is another one).
    pub(crate) fn slots_recall(&mut self) {
        let tools = self.job_slot_tools();
        for t in &tools {
            let changed = self
                .slot_seen
                .iter()
                .any(|s| s.prog == t.prog && s.tool == t.tool && s.diameter_um != t.diameter_um);
            if changed {
                slots::set(&mut self.slot_assign, t.prog, t.tool, None);
            }
        }
        let defined = self.slot_defined();
        let recalled = self
            .settings
            .slot_books
            .get(&self.slot_book_key())
            .map(|b| b.recall(&tools, &defined))
            .unwrap_or_default();
        for (p, t, s) in recalled {
            if slots::slot_of(&self.slot_assign, p, t).is_none() {
                slots::set(&mut self.slot_assign, p, t, Some(s));
            }
        }
        self.slot_seen = tools;
    }

    /// Looks again at the places when the tools of the job are not the ones they were last matched with (a file was opened, a tool was switched off...). Not while a job runs.
    pub(crate) fn slots_sync(&mut self) {
        if self.running.is_some() || !self.has_changer() {
            return;
        }
        if self.job_slot_tools() != self.slot_seen {
            self.slots_recall();
        }
    }

    /// Keeps the places of the tools of the job for this machine, in the settings file.
    fn slots_remember(&mut self) {
        let tools = self.job_slot_tools();
        let key = self.slot_book_key();
        let book = self.settings.slot_books.entry(key).or_default();
        book.remember(&tools, &self.slot_assign);
        self.remember_settings();
    }

    /// Forgets what is kept for the tools of the job that were left without a place.
    fn slots_forget(&mut self, tools: &[JobTool]) {
        let key = self.slot_book_key();
        if let Some(book) = self.settings.slot_books.get_mut(&key) {
            book.forget(tools);
            if book.is_empty() {
                self.settings.slot_books.remove(&key);
            }
        }
        self.remember_settings();
    }

    /// Gives a place to each tool that has none, in order (the drills first, the smaller first).
    pub(crate) fn slots_auto(&mut self) {
        let tools = self.job_slot_tools();
        let defined = self.slot_defined();
        slots::fill_in_order(&tools, &mut self.slot_assign, &defined);
        self.slots_remember();
    }

    /// Takes the place away from every tool of the job, and forgets it.
    pub(crate) fn slots_clear(&mut self) {
        let tools = self.job_slot_tools();
        for t in &tools {
            slots::set(&mut self.slot_assign, t.prog, t.tool, None);
        }
        self.slots_forget(&tools);
    }

    /// The rows of the list of places of the tool in a row of the window: no place, and each place a tool can have (with the tool that is in it, if there is one).
    pub(crate) fn slot_items(&self, row: usize) -> Vec<String> {
        let rows = self.slot_rows();
        let ok = slots::selectable(&self.slot_defined());
        let mut v = vec![self.tr(Text::SlotsNone).to_string()];
        for s in ok {
            let who = rows.iter().enumerate().find(|(i, r)| {
                *i != row && slots::slot_of(&self.slot_assign, r.tool.prog, r.tool.tool) == Some(s)
            });
            v.push(match who {
                Some((_, r)) => format!("{s}  ({})", tool_tag(r.tool.prog, r.tool.tool)),
                None => format!("{s}"),
            });
        }
        v
    }

    /// The row of the list that is chosen for the tool in a row of the window.
    pub(crate) fn slot_current(&self, row: usize) -> usize {
        let rows = self.slot_rows();
        let Some(r) = rows.get(row) else { return 0 };
        let ok = slots::selectable(&self.slot_defined());
        slots::slot_of(&self.slot_assign, r.tool.prog, r.tool.tool)
            .and_then(|s| ok.iter().position(|x| *x == s))
            .map_or(0, |i| i + 1)
    }

    /// A row of the list of places is chosen for the tool in a row of the window (the first row is no place).
    pub(crate) fn slot_pick(&mut self, row: usize, pick: usize) {
        let rows = self.slot_rows();
        let Some(r) = rows.get(row) else { return };
        let ok = slots::selectable(&self.slot_defined());
        let slot = pick.checked_sub(1).and_then(|i| ok.get(i)).copied();
        let tool = r.tool;
        slots::set(&mut self.slot_assign, tool.prog, tool.tool, slot);
        if slot.is_some() {
            self.slots_remember();
        } else {
            self.slots_forget(&[tool]);
        }
    }

    /// The lines that say what stops the job: the tools with no place, the places the machine file does not define and the places two tools share.
    pub(crate) fn slot_problem_lines(&self, problems: &[Problem]) -> Vec<String> {
        let mut none: Vec<String> = Vec::new();
        let mut lines: Vec<String> = Vec::new();
        for p in problems {
            match p {
                Problem::Unassigned(prog, t) => none.push(tool_tag(*prog, *t)),
                Problem::Undefined(prog, t, s) => lines.push(format!(
                    "{}: {s} ({})",
                    self.tr(Text::SlotsUndefined),
                    tool_tag(*prog, *t)
                )),
                Problem::Shared(s, who) => lines.push(format!(
                    "{}: {s} ({})",
                    self.tr(Text::SlotsShared),
                    who.iter()
                        .map(|(p, t)| tool_tag(*p, *t))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            }
        }
        if !none.is_empty() {
            lines.insert(
                0,
                format!("{}: {}", self.tr(Text::SlotsNoSlot), none.join(", ")),
            );
        }
        lines
    }

    /// Whether the job on the changer can start: every tool has a place of its own that the machine file defines. If not, the window of the places opens and says which.
    pub(crate) fn slots_ready(&mut self) -> bool {
        self.slots_recall();
        let problems = self.slot_problems();
        if problems.is_empty() {
            return true;
        }
        self.message = Some(self.slot_problem_lines(&problems).join("  -  "));
        self.show_modal(Modal::Slots);
        false
    }

    /// The window.
    pub(super) fn paint_slots(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let h = 760.0;
        let (x, y) = self.modal_card(cv, self.tr(Text::SlotsTitle), h);
        let rows = self.slot_rows();
        let problems = self.slot_problems();
        let bad: Vec<(Program, u8)> = problems
            .iter()
            .flat_map(|p| match p {
                Problem::Unassigned(prog, tool) | Problem::Undefined(prog, tool, _) => {
                    vec![(*prog, *tool)]
                }
                Problem::Shared(_, who) => who.clone(),
            })
            .collect();
        // the headings of the columns
        for (dx, align, text) in [
            (30.0, Align2::LEFT_BOTTOM, self.tr(Text::HudTool)),
            (110.0, Align2::LEFT_BOTTOM, self.tr(Text::ColType)),
            (330.0, Align2::RIGHT_BOTTOM, "\u{d8} mm"),
            (500.0, Align2::LEFT_BOTTOM, self.tr(Text::HudRackSlot)),
        ] {
            cv.text(x + dx, y + 84.0, align, text, 13.0, col(t.dim));
        }
        for (i, r) in rows.iter().take(ROWS_SHOWN).enumerate() {
            let ry = y + 92.0 + i as f32 * 34.0;
            let is_bad = bad.contains(&(r.tool.prog, r.tool.tool));
            cv.p.circle_filled(
                cv.pt(x + 36.0, ry + 15.0),
                6.0 * cv.u,
                col(t.size_color(r.color)),
            );
            cv.text(
                x + 50.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                &format!("T{:02}", r.tool.tool),
                15.0,
                col(t.text),
            );
            cv.text(
                x + 110.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                self.tr(if r.tool.prog == Program::Drill {
                    Text::HudKindDrill
                } else {
                    Text::HudKindCutter
                }),
                15.0,
                col(t.text),
            );
            if r.tool.diameter_um > 0 {
                cv.text(
                    x + 330.0,
                    ry + 22.0,
                    Align2::RIGHT_BOTTOM,
                    &format!("{:.2}", r.tool.diameter_um as f64 / 1000.0),
                    15.0,
                    col(t.text),
                );
            }
            let (state, ink) = if r.using {
                (self.tr(Text::SlotsUsing), col(t.accent))
            } else if r.done {
                (self.tr(Text::SlotsDone), col(t.ok))
            } else {
                (self.tr(Text::SlotsPending), col(t.dim))
            };
            cv.text_fit(
                x + 360.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                state,
                14.0,
                ink,
                120.0,
            );
            // the list of places of this tool
            let id = ComboId::Slot(i);
            let items = self.slot_items(i);
            let current = items.get(self.slot_current(i)).cloned().unwrap_or_default();
            let (cx, cw) = (x + 500.0, 200.0);
            let open = self.combo.is_some_and(|c| c.id == id);
            cv.chip(cx, ry, cw, 30.0, &current, open, Act::ComboOpen(id));
            let (ax, ay) = (cx + cw - 22.0, ry + 15.0);
            cv.poly(
                &[(ax - 6.0, ay - 3.0), (ax + 6.0, ay - 3.0), (ax, ay + 5.0)],
                col(t.text),
                Stroke::NONE,
            );
            cv.combos.push((id, cx, ry + 32.0, cw));
            if is_bad {
                cv.text(
                    cx - 16.0,
                    ry + 22.0,
                    Align2::LEFT_BOTTOM,
                    "!",
                    18.0,
                    col(t.danger),
                );
            }
        }
        // what stops the job, in words
        for (i, line) in self
            .slot_problem_lines(&problems)
            .iter()
            .take(3)
            .enumerate()
        {
            cv.text_fit(
                x + 30.0,
                y + 598.0 + i as f32 * 18.0,
                Align2::LEFT_BOTTOM,
                line,
                14.0,
                col(t.danger),
                680.0,
            );
        }
        for (i, line) in super::extra::wrap(self.tr(Text::SlotsNote), 96)
            .iter()
            .take(3)
            .enumerate()
        {
            cv.text(
                x + 30.0,
                y + 660.0 + i as f32 * 16.0,
                Align2::LEFT_BOTTOM,
                line,
                12.0,
                col(t.dim),
            );
        }
        let by = y + h - 64.0;
        cv.chip(
            x + 30.0,
            by,
            220.0,
            44.0,
            self.tr(Text::SlotsAuto),
            false,
            Act::SlotsAuto,
        );
        cv.chip(
            x + 270.0,
            by,
            220.0,
            44.0,
            self.tr(Text::SlotsClear),
            false,
            Act::SlotsClear,
        );
        cv.chip(
            x + 510.0,
            by,
            170.0,
            44.0,
            self.tr(Text::Close),
            false,
            Act::CloseModal,
        );
    }
}

/// How a tool is named in the lines of the window: `D T03` for a drill and `R T03` for a cutter.
fn tool_tag(p: Program, tool: u8) -> String {
    format!("{} T{tool:02}", if p == Program::Drill { "D" } else { "R" })
}

#[cfg(test)]
impl App {
    /// Gives the tools the places the order of the job gives them (the first tool the job uses in the place 1, and so on), as a job on the changer had them before the
    /// person could choose.
    pub(crate) fn assign_job_order(&mut self) {
        if !self.has_changer() {
            return;
        }
        let pieces = self
            .rack_plan(&[Program::Drill, Program::Route])
            .into_iter()
            .next()
            .unwrap_or_default();
        for (slot, p, t) in crate::jobflow::slot_tools(&pieces) {
            slots::set(&mut self.slot_assign, p, t, Some(slot as usize));
        }
        self.slot_seen = self.job_slot_tools();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("eh3d-bg-slots-{}-{n}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        for f in [
            "DRILLPRO.DCF",
            "DRILLPRO.DTD",
            "ROUTEPRO.RCF",
            "ROUTEPRO.RTD",
            "05_small_board.ncd",
            "05_board_outline.plt",
            "05_board_tracks.plt",
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
        a.apply_model(crate::machines::Model::from_name("CCD/2 ATC").unwrap());
        a
    }

    /// The tools of the job: the drills and then the cutters.
    fn job_tools(a: &App) -> Vec<(Program, u8)> {
        a.job_slot_tools()
            .iter()
            .map(|t| (t.prog, t.tool))
            .collect()
    }

    /// The places the order of the job gives the tools, turned round: the first tool goes to the last place used and so on.
    fn turned_round(a: &mut App) {
        a.assign_job_order();
        let places: Vec<usize> = a.slot_assign.iter().map(|e| e.2).collect();
        for (e, s) in a.slot_assign.iter_mut().zip(places.iter().rev()) {
            e.2 = *s;
        }
    }

    /// The distance from a point to the nearest stretch of a path, in mm.
    fn distance_to(paths: &[Vec<(f64, f64)>], p: (f64, f64)) -> f64 {
        let mut best = f64::MAX;
        for path in paths {
            for w in path.windows(2) {
                let (a, b) = (w[0], w[1]);
                let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                let len2 = dx * dx + dy * dy;
                let k = if len2 > 1e-12 {
                    (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                best = best.min((a.0 + k * dx - p.0).hypot(a.1 + k * dy - p.1));
            }
            if path.len() == 1 {
                best = best.min((path[0].0 - p.0).hypot(path[0].1 - p.1));
            }
        }
        best
    }

    /// What a whole job did, seen in short steps.
    struct Played {
        /// The tools the head held at some moment.
        held: Vec<(Program, u8)>,
        /// The bar as it was shown the first time the router worked.
        bar_in_routing: Vec<(u8, Program, u8)>,
        /// How many looks were taken while the router was cutting, and the farthest the head was from the design at one of them.
        cutting: usize,
        worst_mm: f64,
        /// Where the head was (x, y, z in mm) at the look that was the farthest from the design.
        worst_at: (f64, f64, f64),
        /// The farthest the head was, while cutting, from the corner of the table along X and along Y.
        reach: (f64, f64),
    }

    fn play(a: &mut App) -> Played {
        a.auto_ok = true;
        a.fixed_dt = Some(0.02);
        let paths: Vec<Vec<(f64, f64)>> = a
            .route
            .scene
            .paths
            .iter()
            .map(|q| q.points.clone())
            .collect();
        // the bar is deep in the travel of Z, as deep as the board is thick below the surface: a head that is at the bar, or on its way to a place of it, is not cutting
        let bar: Option<(f64, f64, f64, f64, f64)> = a.bar_view().map(|v| {
            let xs = v.slots.iter().map(|s| s.x_mm);
            let ys = v.slots.iter().map(|s| s.y_mm);
            let zs = v.slots.iter().map(|s| s.z_mm);
            (
                xs.clone().fold(f64::MAX, f64::min),
                xs.fold(f64::MIN, f64::max),
                ys.clone().fold(f64::MAX, f64::min),
                ys.fold(f64::MIN, f64::max),
                zs.fold(f64::MAX, f64::min),
            )
        });
        a.act(Act::Start);
        if a.modal == Some(Modal::Rack) {
            a.act(Act::RackGo);
        }
        let mut out = Played {
            held: Vec::new(),
            bar_in_routing: Vec::new(),
            cutting: 0,
            worst_mm: 0.0,
            worst_at: (0.0, 0.0, 0.0),
            reach: (0.0, 0.0),
        };
        for _ in 0..400_000 {
            a.tick();
            if let Some(c) = a.rack_carried {
                if !out.held.contains(&c) {
                    out.held.push(c);
                }
            }
            if a.running == Some(Program::Route) {
                if out.bar_in_routing.is_empty() {
                    out.bar_in_routing = a.rack_slot_tools();
                }
                let min_depth = a.settings.route_mark_min_depth_mm;
                let deep = a
                    .route
                    .engine
                    .depth_below_surface_mm()
                    .is_some_and(|d| d >= min_depth);
                let at_bar = |h: (f64, f64, f64)| {
                    bar.is_some_and(|(x0, x1, y0, y1, z)| {
                        (h.0 >= x0 - 1.0 && h.0 <= x1 + 1.0 && h.1 >= y0 - 1.0 && h.1 <= y1 + 1.0)
                            || h.2 >= z - 1.0
                    })
                };
                if let (true, Some(h)) = (deep, a.route.engine.head_mm().filter(|h| !at_bar(*h))) {
                    out.cutting += 1;
                    let far = distance_to(&paths, (h.0, h.1));
                    if far > out.worst_mm {
                        out.worst_mm = far;
                        out.worst_at = h;
                    }
                    out.reach = (out.reach.0.max(h.0), out.reach.1.max(h.1));
                }
            }
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
        out
    }

    #[test]
    fn a_job_with_tools_that_have_no_place_does_not_start_and_says_which_and_opens_the_window() {
        let mut a = app();
        assert!(a.slot_assign.is_empty());
        let tools = job_tools(&a);
        assert!(tools.len() >= 2, "{tools:?}");
        a.act(Act::Start);
        assert!(a.running.is_none() && !a.rack_active);
        assert_eq!(a.modal, Some(Modal::Slots));
        let said = a.message.clone().unwrap_or_default();
        assert!(said.contains(a.tr(Text::SlotsNoSlot)), "{said}");
        for (p, t) in &tools {
            assert!(said.contains(&tool_tag(*p, *t)), "{said}");
        }
        // the places are given in order from the window, and then the job starts: the rack is asked for first
        a.act(Act::SlotsAuto);
        assert!(a.slot_problems().is_empty());
        a.act(Act::CloseModal);
        a.act(Act::Start);
        assert_eq!(a.modal, Some(Modal::Rack));
        a.act(Act::RackGo);
        assert!(a.running.is_some());
        a.act(Act::Stop);
    }

    #[test]
    fn two_tools_in_one_place_and_a_place_the_machine_file_does_not_define_stop_the_job() {
        let mut a = app();
        a.act(Act::SlotsAuto);
        assert!(a.slot_problems().is_empty());
        let rows = a.slot_rows();
        let (first, second) = (rows[0].tool, rows[1].tool);
        // the second tool takes the place of the first
        let place = slots::slot_of(&a.slot_assign, first.prog, first.tool).unwrap();
        slots::set(&mut a.slot_assign, second.prog, second.tool, Some(place));
        let problems = a.slot_problems();
        assert!(
            problems.contains(&Problem::Shared(
                place,
                vec![(first.prog, first.tool), (second.prog, second.tool)]
            )),
            "{problems:?}"
        );
        a.act(Act::Start);
        assert!(a.running.is_none());
        assert_eq!(a.modal, Some(Modal::Slots));
        assert!(a
            .message
            .clone()
            .unwrap_or_default()
            .contains(a.tr(Text::SlotsShared)));
        // a place that the file of the machine does not give a position to
        a.act(Act::CloseModal);
        slots::set(&mut a.slot_assign, first.prog, first.tool, Some(3));
        slots::set(&mut a.slot_assign, second.prog, second.tool, Some(4));
        if let Some(s) = a.drill.engine.settings_mut() {
            s.atc = (0..=16)
                .filter(|i| *i != 3)
                .map(|i| eh3d_bungard_formats::config::AtcSlot {
                    index: i,
                    x_mm: 330.0,
                    y_mm: 20.0 + 18.0 * i as f64,
                    z_mm: 30.0,
                })
                .collect();
        }
        let problems = a.slot_problems();
        assert!(
            problems.contains(&Problem::Undefined(first.prog, first.tool, 3)),
            "{problems:?}"
        );
        a.act(Act::Start);
        assert!(a.running.is_none());
        assert!(a
            .message
            .clone()
            .unwrap_or_default()
            .contains(a.tr(Text::SlotsUndefined)));
    }

    #[test]
    fn the_list_of_places_of_a_tool_offers_no_place_and_each_place_of_the_bar_with_who_is_in_it() {
        let mut a = app();
        a.act(Act::SlotsAuto);
        let items = a.slot_items(0);
        // no place, and the places 1 to 14
        assert_eq!(items.len(), 1 + slots::MAX_TOOL_SLOT);
        assert_eq!(items[0], a.tr(Text::SlotsNone));
        // the place of the second tool is told in the list of the first
        let rows = a.slot_rows();
        let second = slots::slot_of(&a.slot_assign, rows[1].tool.prog, rows[1].tool.tool).unwrap();
        assert!(items[second].contains('('), "{:?}", items[second]);
        // choosing a row gives that place, choosing the first takes it away
        a.combo_open(ComboId::Slot(0));
        a.combo_pick(9);
        let t = rows[0].tool;
        assert_eq!(slots::slot_of(&a.slot_assign, t.prog, t.tool), Some(9));
        assert_eq!(a.slot_current(0), 9);
        a.combo_open(ComboId::Slot(0));
        a.combo_pick(0);
        assert_eq!(slots::slot_of(&a.slot_assign, t.prog, t.tool), None);
        assert_eq!(a.slot_current(0), 0);
    }

    #[test]
    fn the_places_are_kept_for_the_machine_and_come_back_with_the_same_tools_and_not_after_clearing(
    ) {
        let mut a = app();
        turned_round(&mut a);
        a.slots_remember();
        let key = launcher::model_key(&a.settings.model);
        assert_eq!(key, "ccd_2_atc");
        assert!(a.settings.slot_books.contains_key(&key));
        // through the text of the settings file
        let back = Settings::parse(&a.settings.to_text());
        assert_eq!(back.slot_books, a.settings.slot_books);
        let line = a
            .settings
            .to_text()
            .lines()
            .find(|l| l.starts_with("bar_slots_ccd_2_atc = "))
            .map(str::to_string)
            .unwrap();
        assert!(line.contains("= v1;d"), "{line}");
        // the next job with the same tools: nothing is assigned, and it all comes back
        let given = a.slot_assign.clone();
        a.slot_assign.clear();
        a.slot_seen.clear();
        a.slots_recall();
        let mut want = given.clone();
        want.sort_by_key(|e| (e.0 == Program::Route, e.1));
        let mut got = a.slot_assign.clone();
        got.sort_by_key(|e| (e.0 == Program::Route, e.1));
        assert_eq!(got, want);
        // another machine does not find them
        a.settings.model = "CCD XL ATC".to_string();
        a.slot_assign.clear();
        a.slot_seen.clear();
        a.slots_recall();
        assert!(a.slot_assign.is_empty());
        a.settings.model = "CCD/2 ATC".to_string();
        // clearing forgets them
        a.slots_recall();
        assert!(!a.slot_assign.is_empty());
        a.act(Act::SlotsClear);
        assert!(a.slot_assign.is_empty());
        a.slot_seen.clear();
        a.slots_recall();
        assert!(a.slot_assign.is_empty());
        assert!(!a.settings.slot_books.contains_key(&key));
    }

    #[test]
    fn the_bar_shows_every_tool_of_the_job_in_the_place_it_was_given_before_during_and_after_the_routing(
    ) {
        let mut a = app();
        turned_round(&mut a);
        let tools = job_tools(&a);
        let before = a.rack_slot_tools();
        assert_eq!(before.len(), tools.len());
        let played = play(&mut a);
        // the routing began with the drills still in the bar, each in its place and not renumbered from 1
        assert_eq!(played.bar_in_routing, before);
        for (slot, p, t) in &played.bar_in_routing {
            assert_eq!(
                slots::slot_of(&a.slot_assign, *p, *t),
                Some(*slot as usize),
                "{p:?} {t}"
            );
        }
        assert!(played
            .bar_in_routing
            .iter()
            .any(|(_, p, _)| *p == Program::Drill));
        assert!(played
            .bar_in_routing
            .iter()
            .any(|(_, p, _)| *p == Program::Route));
        // and when it is over, still all of them
        assert_eq!(a.rack_slot_tools(), before);
    }

    #[test]
    fn with_the_places_turned_round_the_head_takes_every_drill_and_cutter_from_the_place_it_was_given(
    ) {
        let mut a = app();
        turned_round(&mut a);
        let tools = job_tools(&a);
        let played = play(&mut a);
        for t in &tools {
            assert!(
                played.held.contains(t),
                "the head never held {t:?}; it held {:?}",
                played.held
            );
        }
        assert!(a.drill.scene.holes.iter().all(|h| h.done));
    }

    #[test]
    fn the_router_cuts_on_the_design_and_inside_the_table_in_a_job_with_the_changer_and_no_panel() {
        // the outline and then the tracks of the example board, with the places in the order of the job and turned round
        for (file, turn) in [
            ("05_board_outline.plt", false),
            ("05_board_outline.plt", true),
            ("05_board_tracks.plt", true),
        ] {
            let mut a = app();
            let path = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join(file);
            a.open_path(&path);
            assert!(a.panel_applied.is_none(), "a single board has no panel");
            if turn {
                turned_round(&mut a);
            } else {
                a.assign_job_order();
            }
            let (w, d) = (a.table.width_mm, a.table.depth_mm);
            let played = play(&mut a);
            assert!(played.cutting > 0, "{file}: the router never cut");
            assert!(
                played.worst_mm < 0.5,
                "{file}: the head cut {} mm from the design, at {:?}",
                played.worst_mm,
                played.worst_at
            );
            assert!(
                played.reach.0 <= w && played.reach.1 <= d,
                "{file}: the head cut at {:?} on a table of {w} x {d}",
                played.reach
            );
            assert!(a
                .route
                .scene
                .paths
                .iter()
                .all(|q| q.reached == q.points.len()));
        }
    }

    #[test]
    fn the_window_of_the_places_paints_in_every_theme_and_language_with_and_without_places() {
        let ctx = egui::Context::default();
        let mut a = app();
        a.animate = false;
        a.screen = Screen::Viewer;
        for (lang, theme) in [
            (Lang::En, ThemeId::Forge),
            (Lang::De, ThemeId::Paper),
            (Lang::Ja, ThemeId::Contrast),
        ] {
            a.set_lang(lang);
            a.set_theme(theme);
            for given in [false, true] {
                a.slot_assign.clear();
                a.settings.slot_books.clear();
                if given {
                    a.act(Act::SlotsAuto);
                }
                a.show_modal(Modal::Slots);
                let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(design_w(), DESIGN_H));
                let mut out = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(rect),
                        ..Default::default()
                    },
                    |ui| {
                        let p = ui.ctx().layer_painter(egui::LayerId::background());
                        a.paint(&p, rect);
                    },
                );
                out.textures_delta.clear();
                assert_eq!(a.slot_problems().is_empty(), given);
            }
        }
    }
}
