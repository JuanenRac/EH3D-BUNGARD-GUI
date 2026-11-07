//! The window of the report of the job: the pieces, the holes and the paths by tool, the time the job is expected to take and what looks wrong, with a button that
//! copies it all as text. It is worked out when the window opens (`report`), from what the views draw and the tools and programs that are on; its words are in
//! the files of `locales/report/` and its figures in `bungard-gui.report.cfg` next to the settings file (over `report_defaults.cfg`).

use super::*;
use crate::report;

/// The lines the window shows; the text that is copied has all of them.
const SHOWN_LINES: usize = 20;

impl App {
    /// The report of what the job would do now: the parts that are on, with the tools that are on, as the views draw them (with the copies of the panel).
    pub(crate) fn report_now(&self) -> report::Report {
        let cfg = report::Config::load(&self.settings_path.with_extension("report.cfg"));
        let mut scene = Scene::new(self.table);
        let mut pieces = 1;
        for (p, side) in [(Program::Drill, &self.drill), (Program::Route, &self.route)] {
            if !self.program_runs(p) {
                continue;
            }
            let from = |s: &Scene| Scene {
                table: self.table,
                holes: s
                    .holes
                    .iter()
                    .filter(|h| self.tool_is_on(Program::Drill, h.tool))
                    .cloned()
                    .collect(),
                paths: s
                    .paths
                    .iter()
                    .filter(|q| self.tool_is_on(Program::Route, q.tool))
                    .cloned()
                    .collect(),
            };
            scene.overlay(&from(&side.scene));
            // the copies of the panel: the work as it is drawn over the work as it is on the table
            let (all, one) = match p {
                Program::Drill => (side.scene.holes.len(), side.base.holes.len()),
                Program::Route => (side.scene.paths.len(), side.base.paths.len()),
            };
            if one > 0 && all >= one {
                pieces = pieces.max((all / one) as u32);
            }
        }
        report::build(&scene, pieces, &cfg)
    }

    /// The lines that go with the warnings of the report: what the window itself knows (a board that does not fit the table, a warning of an engine).
    fn report_notes(&self) -> Vec<String> {
        let mut notes = Vec::new();
        if let Some((head, detail)) = self.board_fit_warning() {
            notes.push(format!("{head}: {detail}"));
        }
        for (p, side) in [(Program::Drill, &self.drill), (Program::Route, &self.route)] {
            if self.program_runs(p) {
                if let Some(w) = side.engine.warning() {
                    notes.push(w.to_string());
                }
            }
        }
        notes
    }

    /// Works the report out and opens its window (the lines stay as they were drawn until it is opened again).
    pub(crate) fn report_open(&mut self) {
        let r = self.report_now();
        self.report_lines = report::lines(&r, self.lang, &self.report_notes());
        crate::applog::info(
            "report",
            format!(
                "report opened: {} pieces, {} holes, {} paths, {} s",
                r.pieces, r.holes, r.paths, r.seconds
            ),
        );
    }

    /// Puts the report on the clipboard (the window does it at the start of the next frame) and says so.
    pub(crate) fn report_copy(&mut self) {
        self.pending_copy = Some(self.report_lines.join("\n"));
        self.message = Some(report::word(self.lang, "Copied"));
    }

    pub(super) fn paint_report(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let body: Vec<&String> = self.report_lines.iter().skip(1).collect();
        let shown = body.len().min(SHOWN_LINES);
        let h = 170.0 + 28.0 * shown as f32;
        let (x, y) = self.modal_card(cv, &report::word(self.lang, "Title"), h);
        for (i, line) in body.iter().take(shown).enumerate() {
            // the first lines of each block are the values, the ones that start with a space are the tools and the warnings under them
            let ink = if line.starts_with(' ') { t.dim } else { t.text };
            cv.text_fit(
                x + 30.0,
                y + 96.0 + 28.0 * i as f32,
                Align2::LEFT_BOTTOM,
                line,
                17.0,
                col(ink),
                680.0,
            );
        }
        if body.len() > shown {
            cv.text(
                x + 30.0,
                y + 96.0 + 28.0 * shown as f32,
                Align2::LEFT_BOTTOM,
                "...",
                17.0,
                col(t.dim),
            );
        }
        cv.chip(
            x + 30.0,
            y + h - 60.0,
            320.0,
            38.0,
            &report::word(self.lang, "Copy"),
            false,
            Act::ReportCopy,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("eh3d-bg-reportpane-{}-{n}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        for f in [
            "DRILLPRO.DCF",
            "DRILLPRO.DTD",
            "ROUTEPRO.RCF",
            "ROUTEPRO.RTD",
        ] {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("testdata")
                    .join(f),
                dir.join(f),
            )
            .unwrap();
        }
        App::new(&Options {
            data: Some(dir),
            screen: Some(Screen::Viewer),
            ..Default::default()
        })
    }

    #[test]
    fn a_job_with_nothing_loaded_reports_the_warning_of_an_empty_job() {
        let mut a = app();
        a.report_open();
        let text = a.report_lines.join("\n");
        assert!(text.contains("Pieces: 1"), "{text}");
        assert!(
            text.contains(&report::word(Lang::En, "WarnEmpty")),
            "{text}"
        );
    }

    #[test]
    fn the_report_counts_the_holes_the_views_draw_and_leaves_out_the_tools_that_are_off() {
        let mut a = app();
        a.lang = Lang::En;
        let ncd = std::env::temp_dir().join(format!("eh3d-bg-report-{}.ncd", std::process::id()));
        std::fs::write(&ncd, "%\nT10\nX1000Y1000\nX2000Y1000\nT30\nX100Y100\nM30\n").unwrap();
        a.open_path(&ncd);
        a.report_open();
        let all = a.report_lines.join("\n");
        assert!(all.contains("Holes: 3 (3 per piece)"), "{all}");
        a.off_tools.push((Program::Drill, 30));
        a.report_open();
        let less = a.report_lines.join("\n");
        assert!(less.contains("Holes: 2 (2 per piece)"), "{less}");
        let _ = std::fs::remove_file(ncd);
    }

    #[test]
    fn the_report_is_handed_to_the_clipboard_whole_and_the_window_says_so() {
        let mut a = app();
        a.lang = Lang::Es;
        a.report_open();
        a.report_copy();
        let copied = a.pending_copy.clone().unwrap();
        assert_eq!(copied, a.report_lines.join("\n"));
        assert!(copied.starts_with(&report::word(Lang::Es, "Title")));
        assert_eq!(a.message, Some(report::word(Lang::Es, "Copied")));
    }

    #[test]
    fn the_window_opens_with_its_report_worked_out() {
        let mut a = app();
        a.show_modal(Modal::Report);
        assert_eq!(a.modal, Some(Modal::Report));
        assert!(a.report_lines.len() >= 5);
    }
}
