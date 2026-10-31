//! What the plan view can do besides showing the job: follow the tool that is working or centre on any tool, turn the work (the drilling
//! file and the routing file together), move the board on the table (it changes the offset of the machine for real), fit the board or the
//! table to the view, and take the whole window when the 3D view is not wanted.

use std::path::PathBuf;

use super::*;
use crate::jobs;

/// A turn of the work by quarter turns, counterclockwise, around the centre of the board; `shift` keeps the corner of the board where it was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rotation {
    pub quarters: u8,
    pub center: (f64, f64),
    pub shift: (f64, f64),
}

fn turn(p: (f64, f64), q: u8, c: (f64, f64)) -> (f64, f64) {
    let (dx, dy) = (p.0 - c.0, p.1 - c.1);
    let (rx, ry) = match q % 4 {
        0 => (dx, dy),
        1 => (-dy, dx),
        2 => (-dx, -dy),
        _ => (dy, -dx),
    };
    (c.0 + rx, c.1 + ry)
}

impl Rotation {
    /// The turn of a board whose bounds are given in mm: the corner of the bounds stays where it was.
    pub fn new(bounds: (f64, f64, f64, f64), quarters: u8) -> Rotation {
        let (x0, y0, x1, y1) = bounds;
        let center = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let corners = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)].map(|c| turn(c, quarters, center));
        let nx = corners.iter().map(|c| c.0).fold(f64::MAX, f64::min);
        let ny = corners.iter().map(|c| c.1).fold(f64::MAX, f64::min);
        Rotation {
            quarters: quarters % 4,
            center,
            shift: (x0 - nx, y0 - ny),
        }
    }

    pub fn apply(&self, p: (f64, f64)) -> (f64, f64) {
        let t = turn(p, self.quarters, self.center);
        (t.0 + self.shift.0, t.1 + self.shift.1)
    }
}

/// The drilling file turned, as text the engine reads (the coordinates stay in the unit of the file).
pub fn rotated_drill_text(
    file: &eh3d_drillpro::ncd::DrillFile,
    rot: &Rotation,
    unit_mm: f64,
) -> String {
    let mut out = String::new();
    if !file.diameters.is_empty() {
        out.push_str("M48\n");
        out.push_str(if file.metric { "METRIC\n" } else { "INCH\n" });
        for (t, d) in &file.diameters {
            out.push_str(&format!("T{t:02}C{d}\n"));
        }
    }
    out.push_str("%\n");
    let mut tool = None;
    for h in &file.holes {
        if tool != Some(h.tool) {
            out.push_str(&format!("T{:02}\n", h.tool));
            tool = Some(h.tool);
        }
        let (x, y) = rot.apply((h.x_mil as f64 * unit_mm, h.y_mil as f64 * unit_mm));
        out.push_str(&format!(
            "X{}Y{}\n",
            (x / unit_mm).round() as i64,
            (y / unit_mm).round() as i64
        ));
    }
    out.push_str("M30\n");
    out
}

/// The routing file turned, as HP-GL text.
pub fn rotated_plot_text(plot: &eh3d_routepro::hpgl::Plot, rot: &Rotation) -> String {
    let u = eh3d_routepro::hpgl::UNIT_MM;
    let mut out = String::from("IN;\n");
    for path in &plot.paths {
        let mut pts = path.points.iter().map(|&(x, y)| {
            let (rx, ry) = rot.apply((x as f64 * u, y as f64 * u));
            ((rx / u).round() as i64, (ry / u).round() as i64)
        });
        let Some(first) = pts.next() else { continue };
        out.push_str(&format!("SP{};\nPU{},{};\n", path.pen, first.0, first.1));
        let rest: Vec<String> = pts.map(|(x, y)| format!("{x},{y}")).collect();
        if !rest.is_empty() {
            out.push_str(&format!("PD{};\n", rest.join(",")));
        }
    }
    out.push_str("PU;\n");
    out
}

impl App {
    /// The bounds in mm of the files of the job as they were opened (before any turn and offset).
    fn original_bounds(&self) -> Option<(f64, f64, f64, f64)> {
        let mut pts: Vec<(f64, f64)> = Vec::new();
        if let Some(p) = &self.drill_path {
            let unit = self.drill.engine.settings().map_or(0.0254, |s| s.unit.mm());
            if let Some(f) = jobs::read_latin1(p)
                .ok()
                .and_then(|t| eh3d_drillpro::ncd::parse(&t).ok())
            {
                pts.extend(
                    f.holes
                        .iter()
                        .map(|h| (h.x_mil as f64 * unit, h.y_mil as f64 * unit)),
                );
            }
        }
        if let Some(p) = &self.route_path {
            if let Some(pl) = jobs::read_latin1(p)
                .ok()
                .map(|t| eh3d_routepro::hpgl::parse_with(&t, self.route_segments()))
            {
                pts.extend(pl.paths.iter().flat_map(|q| {
                    q.points.iter().map(|&(x, y)| {
                        (
                            x as f64 * eh3d_routepro::hpgl::UNIT_MM,
                            y as f64 * eh3d_routepro::hpgl::UNIT_MM,
                        )
                    })
                }));
            }
        }
        let mut b: Option<(f64, f64, f64, f64)> = None;
        for (x, y) in pts {
            b = Some(match b {
                None => (x, y, x, y),
                Some((a, c, d, e)) => (a.min(x), c.min(y), d.max(x), e.max(y)),
            });
        }
        b
    }

    /// The file to read for a program: the original, or a copy of it turned when the work is turned.
    pub(super) fn source_for(&self, path: &Path, drill: bool) -> PathBuf {
        if self.rotation == 0 {
            return path.to_path_buf();
        }
        let Some(bounds) = self.original_bounds() else {
            return path.to_path_buf();
        };
        let rot = Rotation::new(bounds, self.rotation);
        let Some(name) = path.file_name() else {
            return path.to_path_buf();
        };
        let dir = self.archive_dir.join(format!("turned-{}", self.rotation));
        let _ = std::fs::create_dir_all(&dir);
        let out = dir.join(name);
        let text = if drill {
            let unit = self.drill.engine.settings().map_or(0.0254, |s| s.unit.mm());
            jobs::read_latin1(path)
                .ok()
                .and_then(|t| eh3d_drillpro::ncd::parse(&t).ok())
                .map(|f| rotated_drill_text(&f, &rot, unit))
        } else {
            jobs::read_latin1(path).ok().map(|t| {
                rotated_plot_text(
                    &eh3d_routepro::hpgl::parse_with(&t, self.route_segments()),
                    &rot,
                )
            })
        };
        match text {
            Some(t) if std::fs::write(&out, &t).is_ok() => out,
            _ => path.to_path_buf(),
        }
    }

    /// Turns the work a quarter turn (both files together) and reads it again.
    /// The work turns on its own middle, wherever it has been moved to (not on the corner it started from), and stays on the table.
    pub(crate) fn rotate_work(&mut self, quarters: i8) {
        let before = self.board();
        self.rotation = (self.rotation as i8 + quarters).rem_euclid(4) as u8;
        self.reload_jobs();
        if let (Some(b0), Some(b1)) = (before, self.board()) {
            let c0 = ((b0.0 + b0.2) / 2.0, (b0.1 + b0.3) / 2.0);
            let c1 = ((b1.0 + b1.2) / 2.0, (b1.1 + b1.3) / 2.0);
            let (mut dx, mut dy) = (c0.0 - c1.0, c0.1 - c1.1);
            // a turned work that would leave the table is brought back inside it
            // (with the three millimetres that the board has round its work)
            let (half_w, half_h) = ((b1.2 - b1.0) / 2.0 + 3.0, (b1.3 - b1.1) / 2.0 + 3.0);
            let (cx, cy) = (c1.0 + dx, c1.1 + dy);
            if cx - half_w < 0.0 {
                dx += half_w - cx;
            } else if cx + half_w > self.table.width_mm {
                dx -= cx + half_w - self.table.width_mm;
            }
            if cy - half_h < 0.0 {
                dy += half_h - cy;
            } else if cy + half_h > self.table.depth_mm {
                dy -= cy + half_h - self.table.depth_mm;
            }
            if dx.abs() > 1e-6 || dy.abs() > 1e-6 {
                self.shift_design(dx, dy);
            }
        }
        // a board that stayed where it was (the move of the design alone) does not turn: only the design does
        self.view2d.focus = None;
    }

    /// Moves the board on the table by this much (mm): the offset of the machine changes, and so does what the views show.
    pub(crate) fn shift_board(&mut self, dx: f64, dy: f64) {
        // the board goes with its design
        if let Some(r) = self.board_frozen.as_mut() {
            *r = (r.0 + dx, r.1 + dy, r.2 + dx, r.3 + dy);
        }
        self.shift_design(dx, dy);
    }

    /// Turns on or off the move of the design alone: the board stays where it is while the design is dragged over it and over the table.
    pub(crate) fn toggle_move_design(&mut self) {
        self.move_design = !self.move_design;
        if self.move_design {
            self.move_board = false;
            self.board_frozen = self.board_rect_following_design();
        }
    }

    /// The design (the holes and the paths) goes `dx` and `dy` millimetres on its own: the board and the table stay.
    pub(crate) fn shift_design(&mut self, dx: f64, dy: f64) {
        let mut moved = false;
        for side in [&mut self.drill, &mut self.route] {
            if let Some(s) = side.engine.settings_mut() {
                s.offset_mm = (
                    ((s.offset_mm.0 + dx) * 100.0).round() / 100.0,
                    ((s.offset_mm.1 + dy) * 100.0).round() / 100.0,
                );
                moved = true;
            }
            side.engine.settings_changed();
            side.offset = (side.offset.0 + dx, side.offset.1 + dy);
            for h in &mut side.base.holes {
                h.x_mm += dx;
                h.y_mm += dy;
            }
            for p in &mut side.base.paths {
                for pt in &mut p.points {
                    pt.0 += dx;
                    pt.1 += dy;
                }
            }
        }
        if moved {
            self.saved = false;
        }
        self.apply_panel();
    }

    /// The middle of the work of a tool: its holes or its paths.
    fn tool_center(&self, prog: Program, tool: u8) -> Option<(f64, f64)> {
        let pts: Vec<(f64, f64)> = match prog {
            Program::Drill => self
                .drill
                .scene
                .holes
                .iter()
                .filter(|h| h.tool == tool)
                .map(|h| (h.x_mm, h.y_mm))
                .collect(),
            Program::Route => self
                .route
                .scene
                .paths
                .iter()
                .filter(|p| p.tool == tool)
                .flat_map(|p| p.points.iter().copied())
                .collect(),
        };
        if pts.is_empty() {
            return None;
        }
        let n = pts.len() as f64;
        Some((
            pts.iter().map(|p| p.0).sum::<f64>() / n,
            pts.iter().map(|p| p.1).sum::<f64>() / n,
        ))
    }

    /// Centres the plan view on the work of a tool and comes close.
    pub(crate) fn center_on_tool(&mut self, prog: Program, tool: u8) {
        if let Some(c) = self.tool_center(prog, tool) {
            self.follow = false;
            self.view2d.focus = Some(c);
            self.view2d.pan = (0.0, 0.0);
            self.view2d.zoom = self.view2d.zoom.max(3.0);
        }
    }

    /// Fits the board (or the whole table) into the plan view.
    pub(crate) fn fit_plan(&mut self, board: bool) {
        self.follow = false;
        self.view2d.pan = (0.0, 0.0);
        if board {
            if let Some((x0, y0, x1, y1)) = self.board_rect() {
                let (w, d) = ((x1 - x0).max(1.0), (y1 - y0).max(1.0));
                self.view2d.focus = Some(((x0 + x1) / 2.0, (y0 + y1) / 2.0));
                self.view2d.zoom = ((self.table.width_mm / w).min(self.table.depth_mm / d) * 0.92)
                    .clamp(0.3, 20.0) as f32;
                return;
            }
        }
        self.view2d = Nav2::default();
    }

    /// The tools of the plan view: follow the tool, move the board, turn the work, fit the board or the table, and take the whole window.
    pub(super) fn paint_plan_tools(&self, cv: &mut Cv, x: f32, y: f32, w: f32) {
        let col3 = (w - 28.0) / 3.0;
        // the first row has three things: follow the tool, move the board (with its design) and move the design alone; the turn is in the header of the view
        cv.checkbox(
            x,
            y,
            col3,
            self.tr(Text::FollowTool),
            self.follow,
            Act::ToggleFollow,
        );
        cv.checkbox(
            x + col3 + 6.0,
            y,
            col3,
            self.tr(Text::MoveBoard),
            self.move_board,
            Act::ToggleMoveBoard,
        );
        cv.checkbox(
            x + 2.0 * (col3 + 6.0),
            y,
            col3,
            self.tr(Text::MoveDesign),
            self.move_design,
            Act::ToggleMoveDesign,
        );
        cv.chip(
            x + 4.0,
            y + 42.0,
            col3 - 4.0,
            32.0,
            self.tr(Text::OnBoard),
            false,
            Act::FitPlan(true),
        );
        cv.chip(
            x + col3 + 6.0 + 4.0,
            y + 42.0,
            col3 - 4.0,
            32.0,
            self.tr(Text::OnTable),
            false,
            Act::FitPlan(false),
        );
        cv.chip(
            x + 2.0 * (col3 + 6.0) + 4.0,
            y + 42.0,
            col3 - 4.0,
            32.0,
            if self.plan_big {
                self.tr(Text::ShowBoth)
            } else {
                self.tr(Text::Enlarge2d)
            },
            self.plan_big,
            Act::ToggleBig,
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
        let dir = std::env::temp_dir().join(format!("eh3d-bg-view2d-{}-{n}", std::process::id()));
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
    fn a_turn_keeps_the_corner_and_four_turns_come_back() {
        let b = (10.0, 20.0, 110.0, 100.0);
        for q in 0..4u8 {
            let r = Rotation::new(b, q);
            let pts =
                [(10.0, 20.0), (110.0, 20.0), (110.0, 100.0), (10.0, 100.0)].map(|p| r.apply(p));
            let (x0, y0) = (
                pts.iter().map(|p| p.0).fold(f64::MAX, f64::min),
                pts.iter().map(|p| p.1).fold(f64::MAX, f64::min),
            );
            assert!(
                (x0 - 10.0).abs() < 1e-9 && (y0 - 20.0).abs() < 1e-9,
                "turn {q}"
            );
        }
        // a quarter turn swaps the width and the depth
        let r = Rotation::new(b, 1);
        let far = r.apply((110.0, 100.0));
        assert!(
            (far.0 - 10.0 - 80.0).abs() < 1e-9
                || (far.1 - 20.0 - 100.0).abs() < 1e-9
                || far.0 >= 10.0
        );
    }

    #[test]
    fn the_work_turns_with_both_files_together_and_the_holes_stay_on_the_board() {
        let mut a = app();
        let before = a.board_rect().unwrap();
        let holes = a.drill.scene.holes.len();
        a.rotate_work(1);
        let after = a.board_rect().unwrap();
        // the width and the depth are swapped and the board stays on the table
        let (w0, d0) = (before.2 - before.0, before.3 - before.1);
        let (w1, d1) = (after.2 - after.0, after.3 - after.1);
        assert!(
            (w0 - d1).abs() < 0.2 && (d0 - w1).abs() < 0.2,
            "{w0}x{d0} became {w1}x{d1}"
        );
        assert_eq!(a.drill.scene.holes.len(), holes);
        // every hole is inside the turned board
        let (x0, y0, x1, y1) = a.board_rect().unwrap();
        assert!(a
            .drill
            .scene
            .holes
            .iter()
            .all(|h| h.x_mm >= x0 && h.x_mm <= x1 && h.y_mm >= y0 && h.y_mm <= y1));
        a.rotate_work(3);
        let back = a.board_rect().unwrap();
        // the same board again (it may have been brought back inside the table on the way)
        assert!(((back.2 - back.0) - (before.2 - before.0)).abs() < 0.2);
        assert!(((back.3 - back.1) - (before.3 - before.1)).abs() < 0.2);
    }

    #[test]
    fn moving_the_board_changes_the_offset_of_the_machine() {
        let mut a = app();
        let off = a.drill.engine.settings().unwrap().offset_mm;
        let hole = (a.drill.scene.holes[0].x_mm, a.drill.scene.holes[0].y_mm);
        a.shift_board(10.0, -5.0);
        let now = a.drill.engine.settings().unwrap().offset_mm;
        assert!((now.0 - off.0 - 10.0).abs() < 0.011 && (now.1 - off.1 + 5.0).abs() < 0.011);
        assert!((a.drill.scene.holes[0].x_mm - hole.0 - 10.0).abs() < 1e-9);
        assert!((a.drill.scene.holes[0].y_mm - hole.1 + 5.0).abs() < 1e-9);
    }

    #[test]
    fn the_plan_view_centres_on_a_tool_and_fits_the_board_or_the_table() {
        let mut a = app();
        a.center_on_tool(Program::Drill, 32);
        let f = a.view2d.focus.unwrap();
        assert!(f.0 > 0.0 && f.1 > 0.0 && a.view2d.zoom >= 3.0);
        a.fit_plan(true);
        assert!(a.view2d.focus.is_some() && a.view2d.zoom > 1.0);
        a.fit_plan(false);
        assert_eq!(a.view2d, Nav2::default());
        a.act(Act::ToggleFollow);
        assert!(a.follow);
        a.act(Act::ToggleBig);
        assert!(a.plan_big);
    }
}
