//! Editing the job from the plan view: choose holes and paths with a click or a box dragged over them (from left to right takes what is
//! wholly inside, from right to left takes whatever the box touches, as the drawing programs do; Ctrl or Shift adds to the choice),
//! copy, paste and delete them, and take a whole tool out of the job. An edit writes the file again (in a folder of its own, the original
//! is never touched) and reads it as any other file, so the engines, the views and the panel all see the same job.

use std::collections::BTreeSet;

use super::view2d::{rotated_drill_text, rotated_plot_text, Rotation};
use super::*;
use crate::jobs;

/// What was copied: the holes (position in mm and tool) and the paths (pen and points in mm).
#[derive(Debug, Clone, Default)]
pub(crate) struct Clip {
    pub holes: Vec<(f64, f64, u8)>,
    pub paths: Vec<(u8, Vec<(f64, f64)>)>,
}

/// How far a pasted copy lands from the original, in mm.
const PASTE_STEP: f64 = 5.0;

/// The state of the job to come back to: its two files and the turn of the work.
#[derive(Debug, Clone)]
pub(crate) struct Snapshot {
    pub drill: Option<PathBuf>,
    pub route: Option<PathBuf>,
    pub rotation: u8,
}

fn identity() -> Rotation {
    Rotation {
        quarters: 0,
        center: (0.0, 0.0),
        shift: (0.0, 0.0),
    }
}

/// Does the segment touch the box?
fn segment_in_box(a: (f64, f64), b: (f64, f64), lo: (f64, f64), hi: (f64, f64)) -> bool {
    let inside = |p: (f64, f64)| p.0 >= lo.0 && p.0 <= hi.0 && p.1 >= lo.1 && p.1 <= hi.1;
    if inside(a) || inside(b) {
        return true;
    }
    // Liang-Barsky
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let (mut t0, mut t1) = (0.0f64, 1.0f64);
    for (p, q) in [
        (-dx, a.0 - lo.0),
        (dx, hi.0 - a.0),
        (-dy, a.1 - lo.1),
        (dy, hi.1 - a.1),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

fn dist_to_segment(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 <= 1e-12 {
        0.0
    } else {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    };
    ((p.0 - (a.0 + t * dx)).powi(2) + (p.1 - (a.1 + t * dy)).powi(2)).sqrt()
}

impl App {
    /// A point of the screen as a point of the table in mm (the plan view as it was drawn last).
    pub(crate) fn plan_point(&self, pos: Pos2) -> Option<(f64, f64)> {
        let (scale, ox, oy, vx, vy) = self.plan_xf.get();
        if scale <= 0.0 || self.screen != Screen::Viewer {
            return None;
        }
        let u = self.ui_scale.max(0.01) as f64;
        let px = (pos.x - self.ui_origin.x) as f64 / u - vx;
        let py = (pos.y - self.ui_origin.y) as f64 / u - vy;
        Some(((px - ox) / scale, self.table.depth_mm - (py - oy) / scale))
    }

    pub(crate) fn selection_count(&self) -> usize {
        self.sel_holes.len() + self.sel_paths.len()
    }

    pub(crate) fn clear_selection(&mut self) {
        self.sel_holes.clear();
        self.sel_paths.clear();
        self.band = None;
    }

    pub(crate) fn toggle_select_mode(&mut self) {
        self.select_mode = !self.select_mode;
        if self.select_mode {
            self.move_board = false;
            self.move_design = false;
        } else {
            self.band = None;
        }
    }

    pub(crate) fn select_all(&mut self) {
        self.sel_holes = (0..self.drill.scene.holes.len()).collect();
        self.sel_paths = (0..self.route.scene.paths.len()).collect();
    }

    /// Chooses what a box takes; `crossing` takes whatever it touches, otherwise only what is wholly inside. `add` keeps the earlier choice.
    pub(crate) fn select_in_box(
        &mut self,
        a: (f64, f64),
        b: (f64, f64),
        crossing: bool,
        add: bool,
    ) {
        let lo = (a.0.min(b.0), a.1.min(b.1));
        let hi = (a.0.max(b.0), a.1.max(b.1));
        let inside = |p: (f64, f64)| p.0 >= lo.0 && p.0 <= hi.0 && p.1 >= lo.1 && p.1 <= hi.1;
        if !add {
            self.sel_holes.clear();
            self.sel_paths.clear();
        }
        if self.layers[0] {
            for (i, h) in self.drill.scene.holes.iter().enumerate() {
                if inside((h.x_mm, h.y_mm)) {
                    self.sel_holes.insert(i);
                }
            }
        }
        if self.layers[1] {
            for (i, p) in self.route.scene.paths.iter().enumerate() {
                let hit = if crossing {
                    p.points.iter().any(|&q| inside(q))
                        || p.points
                            .windows(2)
                            .any(|w| segment_in_box(w[0], w[1], lo, hi))
                } else {
                    !p.points.is_empty() && p.points.iter().all(|&q| inside(q))
                };
                if hit {
                    self.sel_paths.insert(i);
                }
            }
        }
    }

    /// A click: the nearest hole or path within a few pixels is chosen (with `add`, it is added or taken away); a click on nothing clears the choice.
    pub(crate) fn select_at(&mut self, p: (f64, f64), add: bool) {
        let scale = self.plan_xf.get().0.max(1e-6);
        let tol = 7.0 / scale;
        let mut best: Option<(f64, bool, usize)> = None;
        if self.layers[0] {
            for (i, h) in self.drill.scene.holes.iter().enumerate() {
                let d =
                    ((h.x_mm - p.0).powi(2) + (h.y_mm - p.1).powi(2)).sqrt() - h.diameter_mm / 2.0;
                if d <= tol && best.is_none_or(|b| d < b.0) {
                    best = Some((d, true, i));
                }
            }
        }
        if self.layers[1] {
            for (i, path) in self.route.scene.paths.iter().enumerate() {
                for w in path.points.windows(2) {
                    let d = dist_to_segment(p, w[0], w[1]);
                    if d <= tol && best.is_none_or(|b| d < b.0) {
                        best = Some((d, false, i));
                    }
                }
            }
        }
        match best {
            None => {
                if !add {
                    self.sel_holes.clear();
                    self.sel_paths.clear();
                }
            }
            Some((_, hole, i)) => {
                if !add {
                    self.sel_holes.clear();
                    self.sel_paths.clear();
                }
                let set = if hole {
                    &mut self.sel_holes
                } else {
                    &mut self.sel_paths
                };
                if !set.remove(&i) || !add {
                    set.insert(i);
                }
            }
        }
    }

    /// The hole or path nearest to a point of the table within a few pixels: (is a hole, index).
    fn nearest(&self, p: (f64, f64)) -> Option<(bool, usize)> {
        let tol = 7.0 / self.plan_xf.get().0.max(1e-6);
        let mut best: Option<(f64, bool, usize)> = None;
        if self.layers[0] {
            for (i, h) in self.drill.scene.holes.iter().enumerate() {
                let d =
                    ((h.x_mm - p.0).powi(2) + (h.y_mm - p.1).powi(2)).sqrt() - h.diameter_mm / 2.0;
                if d <= tol && best.is_none_or(|b| d < b.0) {
                    best = Some((d, true, i));
                }
            }
        }
        if self.layers[1] {
            for (i, path) in self.route.scene.paths.iter().enumerate() {
                for w in path.points.windows(2) {
                    let d = dist_to_segment(p, w[0], w[1]);
                    if d <= tol && best.is_none_or(|b| d < b.0) {
                        best = Some((d, false, i));
                    }
                }
            }
        }
        best.map(|(_, hole, i)| (hole, i))
    }

    /// Is the point over something of the choice? (A drag that starts there moves the choice; anywhere else it draws a box.)
    pub(crate) fn is_over_choice(&self, p: (f64, f64)) -> bool {
        match self.nearest(p) {
            Some((true, i)) => self.sel_holes.contains(&i),
            Some((false, i)) => self.sel_paths.contains(&i),
            None => false,
        }
    }

    /// The holes and paths of the board file (not of the copies of the panel) that the choice stands for: a panel repeats the board, so a hole of any copy
    /// is the same hole of the file. Paths are left out when the tabs have cut them into pieces.
    fn file_sets(&self) -> (BTreeSet<usize>, BTreeSet<usize>) {
        let (bh, bp) = (self.drill.base.holes.len(), self.route.base.paths.len());
        let holes = self
            .sel_holes
            .iter()
            .filter(|_| bh > 0)
            .map(|i| i % bh)
            .collect();
        let copies = self
            .panel_applied
            .map_or(1, |(c, r, _)| (c * r).max(1) as usize);
        let paths = if bp > 0 && self.route.scene.paths.len() == bp * copies {
            self.sel_paths.iter().map(|i| i % bp).collect()
        } else {
            BTreeSet::new()
        };
        (holes, paths)
    }

    /// Where the copy of the panel that holds item `i` of a scene of `n` items per board sits, in mm.
    fn copy_offset(&self, i: usize, n: usize) -> (f64, f64) {
        match self.panel_applied {
            Some((c, _, pitch)) if n > 0 && c > 0 => {
                let copy = (i / n) as u32;
                ((copy % c) as f64 * pitch.0, (copy / c) as f64 * pitch.1)
            }
            _ => (0.0, 0.0),
        }
    }

    /// Takes the state of the job as it is now, to come back to it.
    fn snapshot(&self) -> Snapshot {
        Snapshot {
            drill: self.drill_path.clone(),
            route: self.route_path.clone(),
            rotation: self.rotation,
        }
    }

    fn push_undo(&mut self) {
        let s = self.snapshot();
        self.undo.push(s);
        self.redo.clear();
        if self.undo.len() > 50 {
            self.undo.remove(0);
        }
    }

    fn restore(&mut self, s: Snapshot) {
        self.drill_path = s.drill;
        self.route_path = s.route;
        self.rotation = s.rotation;
        self.reload_jobs();
        self.clear_selection();
        self.saved = false;
    }

    pub(crate) fn undo_edit(&mut self) {
        if let Some(s) = self.undo.pop() {
            let now = self.snapshot();
            self.redo.push(now);
            self.restore(s);
        }
    }

    pub(crate) fn redo_edit(&mut self) {
        if let Some(s) = self.redo.pop() {
            let now = self.snapshot();
            self.undo.push(now);
            self.restore(s);
        }
    }

    /// Moves what is chosen by this much (mm): the holes and paths change place in the file; the choice stays.
    pub(crate) fn move_selection(&mut self, dx: f64, dy: f64) {
        let (holes, paths) = self.file_sets();
        if holes.is_empty() && paths.is_empty() {
            return;
        }
        self.push_undo();
        let mut changed = false;
        if !holes.is_empty() {
            changed |= self.edit_drill(|f, unit, _| {
                for (i, h) in f.holes.iter_mut().enumerate() {
                    if holes.contains(&i) {
                        h.x_mil += (dx / unit).round() as i32;
                        h.y_mil += (dy / unit).round() as i32;
                    }
                }
            });
        }
        if !paths.is_empty() {
            changed |= self.edit_route(|p, _| {
                let u = eh3d_routepro::hpgl::UNIT_MM;
                for (i, q) in p.paths.iter_mut().enumerate() {
                    if paths.contains(&i) {
                        for pt in &mut q.points {
                            pt.0 += (dx / u).round() as i32;
                            pt.1 += (dy / u).round() as i32;
                        }
                    }
                }
            });
        }
        if changed {
            self.saved = false;
            self.refit_table();
            self.apply_panel();
        }
    }

    pub(crate) fn copy_selection(&mut self) {
        if self.selection_count() == 0 {
            return;
        }
        let mut c = Clip::default();
        let (bh, bp) = (self.drill.base.holes.len(), self.route.base.paths.len());
        for &i in &self.sel_holes {
            if let Some(h) = self.drill.scene.holes.get(i) {
                // in the place of the board, not of the copy of the panel it was chosen in
                let (ox, oy) = self.copy_offset(i, bh);
                c.holes.push((h.x_mm - ox, h.y_mm - oy, h.tool));
            }
        }
        for &i in &self.sel_paths {
            if let Some(p) = self.route.scene.paths.get(i) {
                let (ox, oy) = self.copy_offset(i, bp);
                c.paths.push((
                    p.tool,
                    p.points.iter().map(|&(x, y)| (x - ox, y - oy)).collect(),
                ));
            }
        }
        self.clip = c;
        self.paste_n = 0;
    }

    pub(crate) fn cut_selection(&mut self) {
        self.copy_selection();
        self.delete_selection();
    }

    /// A folder of its own for the next file written by an edit.
    fn edit_dir(&mut self) -> PathBuf {
        // the folder is the program's own (two windows, or two tests, never write the same file)
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        self.edit_serial = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst) as u32;
        let d = self.archive_dir.join(format!(
            "edited-{}-{}",
            std::process::id(),
            self.edit_serial
        ));
        let _ = std::fs::create_dir_all(&d);
        d
    }

    /// The turn of the work becomes part of the files, so that an edit and the positions on the screen agree.
    fn bake_rotation(&mut self) {
        if self.rotation == 0 {
            return;
        }
        let d = self.drill_path.clone().map(|p| self.source_for(&p, true));
        let r = self.route_path.clone().map(|p| self.source_for(&p, false));
        let mut keep = |from: Option<PathBuf>| -> Option<PathBuf> {
            let from = from?;
            let to = self.edit_dir().join(from.file_name()?);
            std::fs::copy(&from, &to).ok().map(|_| to)
        };
        let (d, r) = (keep(d), keep(r));
        if d.is_some() {
            self.drill_path = d;
        }
        if r.is_some() {
            self.route_path = r;
        }
        self.rotation = 0;
    }

    /// Reads the drilling file, lets `f` change it (with the unit of its coordinates and the offset of the machine), writes it again and loads it.
    fn edit_drill(
        &mut self,
        f: impl FnOnce(&mut eh3d_drillpro::ncd::DrillFile, f64, (f64, f64)),
    ) -> bool {
        self.bake_rotation();
        let Some(path) = self.drill_path.clone() else {
            return false;
        };
        let Some(mut file) = jobs::read_latin1(&path)
            .ok()
            .and_then(|t| eh3d_drillpro::ncd::parse(&t).ok())
        else {
            return false;
        };
        let (unit, offset) = self
            .drill
            .engine
            .settings()
            .map_or((0.0254, (0.0, 0.0)), |s| (s.unit.mm(), s.offset_mm));
        let was_empty = self.drill.scene.holes.is_empty();
        f(&mut file, unit, offset);
        let Some(name) = path.file_name() else {
            return false;
        };
        let out = self.edit_dir().join(name);
        if std::fs::write(&out, rotated_drill_text(&file, &identity(), unit)).is_err() {
            return false;
        }
        self.load_drill(&out);
        if self.drill.scene.holes.is_empty() {
            self.job_drill = false;
        } else if was_empty {
            self.job_drill = true;
        }
        true
    }

    fn edit_route(&mut self, f: impl FnOnce(&mut eh3d_routepro::hpgl::Plot, (f64, f64))) -> bool {
        self.bake_rotation();
        let Some(path) = self.route_path.clone() else {
            return false;
        };
        let Some(mut plot) = jobs::read_latin1(&path)
            .ok()
            .map(|t| eh3d_routepro::hpgl::parse_with(&t, self.route_segments()))
        else {
            return false;
        };
        let offset = self
            .route
            .engine
            .settings()
            .map_or((0.0, 0.0), |s| s.offset_mm);
        let was_empty = self.route.scene.paths.is_empty();
        f(&mut plot, offset);
        let Some(name) = path.file_name() else {
            return false;
        };
        let out = self.edit_dir().join(name);
        if std::fs::write(&out, rotated_plot_text(&plot, &identity())).is_err() {
            return false;
        }
        self.load_route(&out);
        if self.route.scene.paths.is_empty() {
            self.job_route = false;
        } else if was_empty {
            self.job_route = true;
        }
        true
    }

    /// What has to be done after an edit so that everything shows the new job.
    fn edited(&mut self) {
        self.saved = false;
        self.refit_table();
        self.apply_panel();
    }

    pub(crate) fn delete_selection(&mut self) {
        let (holes, paths) = self.file_sets();
        if holes.is_empty() && paths.is_empty() {
            return;
        }
        self.push_undo();
        let mut changed = false;
        if !holes.is_empty() {
            changed |= self.edit_drill(|f, _, _| {
                let mut i = 0;
                f.holes.retain(|_| {
                    i += 1;
                    !holes.contains(&(i - 1))
                });
            });
        }
        if !paths.is_empty() {
            changed |= self.edit_route(|p, _| {
                let mut i = 0;
                p.paths.retain(|_| {
                    i += 1;
                    !paths.contains(&(i - 1))
                });
            });
        }
        self.clear_selection();
        if changed {
            self.edited();
        }
    }

    /// Takes a whole tool out of the job: its holes or its paths go from the file, the views and the panel.
    pub(crate) fn remove_tool(&mut self, prog: Program, tool: u8) {
        self.push_undo();
        let changed = match prog {
            Program::Drill => self.edit_drill(|f, _, _| {
                f.holes.retain(|h| h.tool != tool);
                f.tools.retain(|t| *t != tool);
                f.diameters.retain(|(t, _)| *t != tool);
            }),
            Program::Route => self.edit_route(|p, _| p.paths.retain(|q| q.pen != tool)),
        };
        self.off_tools.retain(|x| *x != (prog, tool));
        self.start_at.retain(|x| *x != (prog, tool));
        self.clear_selection();
        if changed {
            self.edited();
        }
    }

    /// Pastes the copy a few millimetres from where it was (each paste a step further), and chooses the new ones.
    pub(crate) fn paste_clip(&mut self) {
        let clip = self.clip.clone();
        if clip.holes.is_empty() && clip.paths.is_empty() {
            return;
        }
        self.push_undo();
        self.paste_n += 1;
        let (dx, dy) = (
            PASTE_STEP * self.paste_n as f64,
            -PASTE_STEP * self.paste_n as f64,
        );
        let (mut new_holes, mut new_paths) = (BTreeSet::new(), BTreeSet::new());
        let mut changed = false;
        if !clip.holes.is_empty() {
            let before = self.drill.base.holes.len();
            changed |= self.edit_drill(|f, unit, off| {
                for &(x, y, tool) in &clip.holes {
                    f.holes.push(eh3d_drillpro::ncd::Hole {
                        tool,
                        x_mil: ((x + dx - off.0) / unit).round() as i32,
                        y_mil: ((y + dy - off.1) / unit).round() as i32,
                    });
                    if !f.tools.contains(&tool) {
                        f.tools.push(tool);
                    }
                }
            });
            new_holes.extend(before..self.drill.base.holes.len());
        }
        if !clip.paths.is_empty() {
            let before = self.route.base.paths.len();
            changed |= self.edit_route(|p, off| {
                let u = eh3d_routepro::hpgl::UNIT_MM;
                for (pen, pts) in &clip.paths {
                    p.paths.push(eh3d_routepro::hpgl::Polyline {
                        pen: *pen,
                        points: pts
                            .iter()
                            .map(|&(x, y)| {
                                (
                                    ((x + dx - off.0) / u).round() as i32,
                                    ((y + dy - off.1) / u).round() as i32,
                                )
                            })
                            .collect(),
                    });
                }
            });
            new_paths.extend(before..self.route.base.paths.len());
        }
        if changed {
            self.sel_holes = new_holes;
            self.sel_paths = new_paths;
            self.edited();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("eh3d-bg-edit-{}-{n}", std::process::id()));
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
    fn a_box_chooses_holes_and_paths_and_delete_takes_them_out_of_the_job() {
        let mut a = app();
        let (nh, np) = (a.drill.scene.holes.len(), a.route.scene.paths.len());
        assert!(nh > 2 && np > 0);
        // a box over everything, from right to left (it touches), takes it all
        a.select_in_box((-1e4, -1e4), (1e4, 1e4), true, false);
        assert_eq!((a.sel_holes.len(), a.sel_paths.len()), (nh, np));
        // a small box takes the first hole only
        let h0 = a.drill.scene.holes[0].clone();
        a.select_in_box(
            (h0.x_mm - 0.05, h0.y_mm - 0.05),
            (h0.x_mm + 0.05, h0.y_mm + 0.05),
            false,
            false,
        );
        assert!(a.sel_holes.contains(&0) && a.sel_paths.is_empty());
        a.delete_selection();
        assert!(a.drill.scene.holes.len() < nh);
        assert_eq!(a.selection_count(), 0);
    }

    #[test]
    fn copy_and_paste_make_new_holes_further_on_and_choose_them() {
        let mut a = app();
        let nh = a.drill.scene.holes.len();
        a.select_in_box((-1e4, -1e4), (1e4, 1e4), true, false);
        a.sel_paths.clear();
        a.copy_selection();
        a.paste_clip();
        assert_eq!(a.drill.scene.holes.len(), nh * 2);
        assert_eq!(a.sel_holes.len(), nh);
        let (o, c) = (&a.drill.scene.holes[0], &a.drill.scene.holes[nh]);
        assert!((c.x_mm - o.x_mm - 5.0).abs() < 0.1 && (o.y_mm - c.y_mm - 5.0).abs() < 0.1);
        assert_eq!(o.tool, c.tool);
    }

    #[test]
    fn a_tool_removed_leaves_the_job_and_the_views() {
        let mut a = app();
        let tool = a.drill.scene.holes[0].tool;
        let with = a
            .drill
            .scene
            .holes
            .iter()
            .filter(|h| h.tool == tool)
            .count();
        let total = a.drill.scene.holes.len();
        a.remove_tool(Program::Drill, tool);
        assert_eq!(a.drill.scene.holes.len(), total - with);
        assert!(a.drill.scene.holes.iter().all(|h| h.tool != tool));
        assert!(a
            .tool_lines()
            .iter()
            .all(|l| !(l.prog == Program::Drill && l.tool == tool)));
    }

    #[test]
    fn what_is_chosen_can_be_moved_and_every_edit_can_be_undone_and_redone() {
        let mut a = app();
        let (nh, h0) = (a.drill.scene.holes.len(), a.drill.scene.holes[0].clone());
        a.sel_holes.insert(0);
        a.move_selection(2.0, -1.0);
        assert_eq!(a.drill.scene.holes.len(), nh);
        let h = &a.drill.scene.holes[0];
        assert!((h.x_mm - h0.x_mm - 2.0).abs() < 0.05 && (h.y_mm - h0.y_mm + 1.0).abs() < 0.05);
        assert!(a.sel_holes.contains(&0), "the choice stays on what moved");
        a.delete_selection();
        assert_eq!(a.drill.scene.holes.len(), nh - 1);
        a.undo_edit();
        assert_eq!(a.drill.scene.holes.len(), nh);
        a.undo_edit();
        let h = &a.drill.scene.holes[0];
        assert!((h.x_mm - h0.x_mm).abs() < 0.05 && (h.y_mm - h0.y_mm).abs() < 0.05);
        a.redo_edit();
        a.redo_edit();
        assert_eq!(a.drill.scene.holes.len(), nh - 1);
        // nothing left to undo or redo
        a.redo_edit();
        assert_eq!(a.drill.scene.holes.len(), nh - 1);
    }

    #[test]
    fn a_panel_edits_the_board_and_every_copy_follows() {
        let mut a = app();
        let nh = a.drill.scene.holes.len();
        a.panel.columns = 2;
        a.panel.rows = 1;
        a.apply_panel();
        assert_eq!(a.drill.scene.holes.len(), nh * 2);
        // a hole of the second copy is the same hole of the file
        a.sel_holes.insert(nh);
        a.delete_selection();
        assert_eq!(a.drill.scene.holes.len(), (nh - 1) * 2);
    }

    #[test]
    fn a_job_that_starts_at_a_tool_leaves_out_the_ones_before_it() {
        let mut a = app();
        let order = a.job_tool_order(Program::Drill);
        assert!(order.len() >= 3);
        a.toggle_start_at(Program::Drill, order[2]);
        assert_eq!(a.tools_off(Program::Drill), vec![order[0], order[1]]);
        a.toggle_start_at(Program::Drill, order[2]);
        assert!(a.tools_off(Program::Drill).is_empty());
    }

    #[test]
    fn a_job_emptied_of_holes_still_loads() {
        let mut a = app();
        a.select_all();
        a.delete_selection();
        assert!(a.drill.scene.holes.is_empty() && a.route.scene.paths.is_empty());
        assert!(!a.job_drill && !a.job_route);
    }
}
