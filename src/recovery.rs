//! The work that is saved while a job runs: if the computer, the window or the machine's link goes down in the middle of a job, the file that is left tells at the
//! next start what was being done (the files, the panel, the holes that were drilled), and the person chooses to go on from there or to leave it.

use std::path::{Path, PathBuf};

/// What a job that was running needs to go on.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Recovery {
    /// The program that was running: `drill` or `route`.
    pub program: String,
    pub drill: Option<PathBuf>,
    pub route: Option<PathBuf>,
    pub columns: u32,
    pub rows: u32,
    /// `gap`, `pitch` or `fit`.
    pub mode: String,
    pub spacing: (f64, f64),
    pub tabs_on: bool,
    pub tab_count: u32,
    pub tab_width: f64,
    /// `even`, `ends` or `middle`.
    pub tab_place: String,
    /// How many holes (or paths) the job had done, and how many it has in all.
    pub done: usize,
    pub total: usize,
    /// Where the holes that were drilled are, in mm on the table.
    pub drilled: Vec<(f64, f64)>,
}

impl Recovery {
    pub fn to_text(&self) -> String {
        let mut t = String::new();
        let mut put = |k: &str, v: String| {
            t.push_str(k);
            t.push('=');
            t.push_str(&v);
            t.push('\n');
        };
        put("program", self.program.clone());
        if let Some(p) = &self.drill {
            put("drill", p.display().to_string());
        }
        if let Some(p) = &self.route {
            put("route", p.display().to_string());
        }
        put("columns", self.columns.to_string());
        put("rows", self.rows.to_string());
        put("mode", self.mode.clone());
        put("spacing", format!("{},{}", self.spacing.0, self.spacing.1));
        put("tabs_on", self.tabs_on.to_string());
        put("tab_count", self.tab_count.to_string());
        put("tab_width", self.tab_width.to_string());
        put("tab_place", self.tab_place.clone());
        put("done", self.done.to_string());
        put("total", self.total.to_string());
        for (x, y) in &self.drilled {
            put("hole", format!("{x:.3},{y:.3}"));
        }
        t
    }

    /// Reads the file back; `None` when it is not the file of a job.
    pub fn from_text(text: &str) -> Option<Recovery> {
        let mut r = Recovery::default();
        let pair = |v: &str| -> Option<(f64, f64)> {
            let (a, b) = v.split_once(',')?;
            Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
        };
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match k {
                "program" => r.program = v.to_string(),
                "drill" => r.drill = Some(PathBuf::from(v)),
                "route" => r.route = Some(PathBuf::from(v)),
                "columns" => r.columns = v.parse().ok()?,
                "rows" => r.rows = v.parse().ok()?,
                "mode" => r.mode = v.to_string(),
                "spacing" => r.spacing = pair(v)?,
                "tabs_on" => r.tabs_on = v == "true",
                "tab_count" => r.tab_count = v.parse().ok()?,
                "tab_width" => r.tab_width = v.parse().ok()?,
                "tab_place" => r.tab_place = v.to_string(),
                "done" => r.done = v.parse().ok()?,
                "total" => r.total = v.parse().ok()?,
                "hole" => r.drilled.push(pair(v)?),
                _ => {}
            }
        }
        (matches!(r.program.as_str(), "drill" | "route")
            && (r.drill.is_some() || r.route.is_some()))
        .then_some(r)
    }

    pub fn save(&self, path: &Path) {
        // written whole to a side file and renamed over the other, so a crash in the middle of the writing leaves the last good one
        let tmp = path.with_extension("tmp");
        if std::fs::write(&tmp, self.to_text()).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    pub fn load(path: &Path) -> Option<Recovery> {
        Recovery::from_text(&std::fs::read_to_string(path).ok()?)
    }

    pub fn clear(path: &Path) {
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(path.with_extension("tmp"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Recovery {
        Recovery {
            program: "drill".into(),
            drill: Some(PathBuf::from("C:/jobs/board.ncd")),
            route: Some(PathBuf::from("C:/jobs/board.plt")),
            columns: 3,
            rows: 2,
            mode: "gap".into(),
            spacing: (5.0, 7.5),
            tabs_on: true,
            tab_count: 2,
            tab_width: 3.0,
            tab_place: "ends".into(),
            done: 41,
            total: 120,
            drilled: vec![(10.0, 20.5), (12.25, 20.5)],
        }
    }

    #[test]
    fn what_is_saved_comes_back_the_same() {
        let r = sample();
        assert_eq!(Recovery::from_text(&r.to_text()), Some(r));
    }

    #[test]
    fn a_file_that_is_not_a_job_is_not_taken_for_one() {
        assert!(Recovery::from_text("").is_none());
        assert!(Recovery::from_text("program=drill\ncolumns=x\n").is_none());
        assert!(Recovery::from_text("program=other\ndrill=a\n").is_none());
        assert!(Recovery::from_text("program=drill\n").is_none());
    }

    #[test]
    fn it_is_written_read_and_cleared_on_disk() {
        let dir = std::env::temp_dir().join(format!("eh3d-recovery-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("recovery.cfg");
        let r = sample();
        r.save(&path);
        assert_eq!(Recovery::load(&path), Some(r));
        Recovery::clear(&path);
        assert!(Recovery::load(&path).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
