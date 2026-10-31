//! The job that was running when the window, the computer or the link went down: while a job runs, what it needs to go on is written to a file (see
//! [`crate::recovery`]); at the next start a window asks whether to go on from there or to leave it, and a job that ends or is stopped on purpose clears the file.

use super::extra::{wrap, PanelCfg, PanelMode};
use super::*;
use crate::recovery::Recovery;
use eh3d_drillpro::job::TabPlacement;

/// How often the file is written at the most while the job goes on (it is also written at once when the count of the work done changes).
const SAVE_EVERY: Duration = Duration::from_secs(2);

fn program_name(p: Program) -> &'static str {
    match p {
        Program::Drill => "drill",
        Program::Route => "route",
    }
}

impl App {
    /// The place of the file of the job that was running: next to the settings, and nowhere in the tests.
    pub(super) fn recovery_file() -> Option<PathBuf> {
        if cfg!(test) {
            return None;
        }
        launcher::settings_path()
            .parent()
            .map(|d| d.join("recovery.cfg"))
    }

    /// How much of the job is done: the holes drilled, or the paths of the routing that were reached to their end.
    fn work_done(&self, p: Program) -> usize {
        let side = match p {
            Program::Drill => &self.drill,
            Program::Route => &self.route,
        };
        match p {
            Program::Drill => self.resume_skip + side.engine.done(),
            Program::Route => side
                .scene
                .paths
                .iter()
                .filter(|q| q.reached >= q.points.len())
                .count(),
        }
    }

    /// What the job needs to go on from here.
    fn recovery_now(&self, p: Program) -> Recovery {
        let side = match p {
            Program::Drill => &self.drill,
            Program::Route => &self.route,
        };
        let c = self.panel;
        Recovery {
            program: match p {
                Program::Drill => "drill",
                Program::Route => "route",
            }
            .into(),
            // the file of a program that waits in the queue is kept too, to go on with it after this one
            drill: (p == Program::Drill || self.job_queue.contains(&Program::Drill))
                .then(|| self.drill_path.clone())
                .flatten(),
            route: (p == Program::Route || self.job_queue.contains(&Program::Route))
                .then(|| self.route_path.clone())
                .flatten(),
            columns: c.columns,
            rows: c.rows,
            mode: match c.mode {
                PanelMode::Gap => "gap",
                PanelMode::Pitch => "pitch",
                PanelMode::Fit => "fit",
            }
            .into(),
            spacing: c.spacing,
            tabs_on: c.tabs_on,
            tab_count: c.tab_count,
            tab_width: c.tab_width,
            tab_place: match c.tab_place {
                TabPlacement::Even => "even",
                TabPlacement::Ends => "ends",
                TabPlacement::Middle => "middle",
            }
            .into(),
            done: self.work_done(p),
            total: match p {
                Program::Drill => side.scene.holes.len(),
                Program::Route => side.scene.paths.len(),
            },
            queue: self
                .job_queue
                .iter()
                .map(|q| program_name(*q).to_string())
                .collect(),
            off: self
                .off_tools
                .iter()
                .map(|(q, t)| (program_name(*q).to_string(), *t))
                .collect(),
            drilled: side
                .scene
                .holes
                .iter()
                .filter(|h| h.done)
                .map(|h| (h.x_mm, h.y_mm))
                .collect(),
        }
    }

    /// Writes the file while a job runs, when the work done changed or some time has passed.
    pub(super) fn save_recovery(&mut self) {
        let (Some(path), Some(p)) = (self.recovery_path.clone(), self.running) else {
            return;
        };
        let done = self.work_done(p);
        let due = self.recovery_at.is_none_or(|t| t.elapsed() >= SAVE_EVERY);
        if done != self.recovery_done || due {
            if done != self.recovery_done || self.recovery_at.is_none() {
                self.recovery_now(p).save(&path);
                self.recovery_done = done;
            }
            self.recovery_at = Some(Instant::now());
        }
    }

    /// A job that ended, or that the person stopped: nothing is left to go on from.
    pub(super) fn clear_recovery(&mut self) {
        if let Some(path) = &self.recovery_path {
            Recovery::clear(path);
        }
        self.recovery_done = usize::MAX;
        self.recovery_at = None;
        self.resume_skip = 0;
    }

    /// At the start: a job that was left in the middle is offered.
    pub(super) fn offer_resume(&mut self) {
        let Some(path) = self.recovery_path.clone() else {
            return;
        };
        match Recovery::load(&path) {
            Some(r)
                if r.drill
                    .as_ref()
                    .or(r.route.as_ref())
                    .is_some_and(|f| f.exists()) =>
            {
                self.resume = Some(r);
                self.screen = Screen::Viewer;
                self.modal = Some(Modal::Resume);
            }
            _ => Recovery::clear(&path),
        }
    }

    /// The person goes on with the job that was left: the files and the panel come back, what was drilled is marked and the job starts at the rest.
    pub(super) fn resume_job(&mut self) {
        let Some(r) = self.resume.take() else { return };
        self.modal = None;
        self.screen = Screen::Viewer;
        let program = if r.program == "route" {
            Program::Route
        } else {
            Program::Drill
        };
        if let Some(f) = &r.drill {
            self.load_drill(f);
        }
        if let Some(f) = &r.route {
            self.load_route(f);
        }
        self.panel = PanelCfg {
            columns: r.columns.max(1),
            rows: r.rows.max(1),
            spacing: r.spacing,
            mode: match r.mode.as_str() {
                "pitch" => PanelMode::Pitch,
                "fit" => PanelMode::Fit,
                _ => PanelMode::Gap,
            },
            tabs_on: r.tabs_on,
            tab_count: r.tab_count.max(1),
            tab_width: r.tab_width,
            tab_place: match r.tab_place.as_str() {
                "ends" => TabPlacement::Ends,
                "middle" => TabPlacement::Middle,
                _ => TabPlacement::Even,
            },
        };
        self.apply_panel();
        let side = self.side_mut(program);
        match program {
            Program::Drill => {
                for (x, y) in &r.drilled {
                    let nearest = side
                        .scene
                        .holes
                        .iter()
                        .enumerate()
                        .filter(|(_, h)| !h.done)
                        .min_by(|a, b| {
                            let da = (a.1.x_mm - x).hypot(a.1.y_mm - y);
                            let db = (b.1.x_mm - x).hypot(b.1.y_mm - y);
                            da.total_cmp(&db)
                        })
                        .map(|(i, _)| i);
                    if let Some(i) = nearest {
                        side.scene.holes[i].done = true;
                    }
                }
            }
            Program::Route => {
                for q in side.scene.paths.iter_mut().take(r.done) {
                    q.reached = q.points.len();
                }
            }
        }
        // the tools that were off stay off, and the programs that were waiting their turn go on after this one
        self.off_tools = r
            .off
            .iter()
            .map(|(q, t)| {
                (
                    if q == "route" {
                        Program::Route
                    } else {
                        Program::Drill
                    },
                    *t,
                )
            })
            .collect();
        let mut programs = vec![program];
        for q in &r.queue {
            let next = if q == "route" {
                Program::Route
            } else {
                Program::Drill
            };
            if !programs.contains(&next) {
                programs.push(next);
            }
        }
        self.start_programs(programs, r.done);
    }

    /// The person leaves the job that was left.
    pub(super) fn resume_cancel(&mut self) {
        self.resume = None;
        self.modal = None;
        self.clear_recovery();
    }

    pub(super) fn paint_resume(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y) = self.modal_card(cv, self.tr(Text::ResumeTitle), 360.0);
        let Some(r) = &self.resume else { return };
        for (i, l) in wrap(self.tr(Text::ResumeBody), 62).iter().enumerate() {
            cv.text(
                x + 30.0,
                y + 100.0 + i as f32 * 26.0,
                Align2::LEFT_BOTTOM,
                l,
                18.0,
                col(t.text),
            );
        }
        let file = r.drill.as_ref().or(r.route.as_ref());
        cv.text(
            x + 30.0,
            y + 200.0,
            Align2::LEFT_BOTTOM,
            &file
                .and_then(|f| f.file_name())
                .map_or(String::new(), |n| n.to_string_lossy().to_string()),
            22.0,
            col(t.accent),
        );
        cv.text(
            x + 30.0,
            y + 236.0,
            Align2::LEFT_BOTTOM,
            &format!("{} / {}", r.done, r.total),
            22.0,
            col(t.text),
        );
        cv.chip(
            x + 30.0,
            y + 280.0,
            320.0,
            44.0,
            self.tr(Text::ResumeContinue),
            true,
            Act::ResumeContinue,
        );
        cv.chip(
            x + 370.0,
            y + 280.0,
            200.0,
            44.0,
            self.tr(Text::ResumeCancel),
            false,
            Act::ResumeCancel,
        );
    }
}
