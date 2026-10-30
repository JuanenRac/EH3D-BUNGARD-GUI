//! The calibration of the table: a probe measures the height of the table in the points of a grid over the work area, and the window shows the map (blue where the
//! table is low, red where it is high), how much it is tilted and how far apart the lowest and the highest point are. The arithmetic is in `eh3d-levelling`; here is
//! the window, the order of the points and the file where the map is kept. Until a probe node is connected the probe is simulated, with a table that is tilted and has a bump.

use super::*;
use eh3d_levelling::{simulated_table, Grid, HeightMap};

/// How often the simulated probe gives a point.
const POINT_EVERY: Duration = Duration::from_millis(150);

/// The map of the table and where the measuring is.
pub(crate) struct LevelState {
    pub cols: usize,
    pub rows: usize,
    pub map: HeightMap,
    pub measuring: bool,
    /// The Z of the jobs follows the map (only when the person asks for it, and only a map that is complete).
    pub apply: bool,
    pub at: Option<Instant>,
    /// Where the map is kept (nowhere in the tests).
    pub path: Option<PathBuf>,
}

impl LevelState {
    pub fn new(table: Table) -> LevelState {
        let path = if cfg!(test) {
            None
        } else {
            launcher::settings_path()
                .parent()
                .map(|d| d.join("table_map.txt"))
        };
        let (cols, rows) = (7, 5);
        let map = path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| HeightMap::from_text(&t).ok())
            .unwrap_or_else(|| Self::empty(table, cols, rows));
        LevelState {
            cols: map.grid.cols,
            rows: map.grid.rows,
            map,
            measuring: false,
            apply: false,
            at: None,
            path,
        }
    }

    fn empty(table: Table, cols: usize, rows: usize) -> HeightMap {
        HeightMap::new(Grid::new(
            (0.0, 0.0, table.width_mm, table.depth_mm),
            cols,
            rows,
        ))
    }
}

impl App {
    /// A new, empty map over the table with `cols` by `rows` points.
    pub(crate) fn level_reset(&mut self, cols: usize, rows: usize) {
        self.level.cols = cols.clamp(2, 15);
        self.level.rows = rows.clamp(2, 15);
        self.level.map = LevelState::empty(self.table, self.level.cols, self.level.rows);
        self.level.measuring = false;
        self.level.at = None;
        self.level_save();
    }

    /// Starts the measuring (a map that is complete is measured again from the start), or stops it.
    pub(crate) fn level_toggle(&mut self) {
        if self.level.measuring {
            self.level.measuring = false;
            return;
        }
        if self.level.map.is_complete() {
            self.level_reset(self.level.cols, self.level.rows);
        }
        self.level.measuring = true;
        self.level.at = None;
    }

    pub(crate) fn level_clear(&mut self) {
        self.level_reset(self.level.cols, self.level.rows);
    }

    /// The map that the jobs follow: the one measured, when the person asked for it and it is complete.
    pub(crate) fn level_for_jobs(&self) -> Option<eh3d_levelling::HeightMap> {
        (self.level.apply && self.level.map.is_complete()).then(|| self.level.map.clone())
    }

    fn level_save(&self) {
        if let Some(p) = &self.level.path {
            let _ = std::fs::write(p, self.level.map.to_text());
        }
    }

    /// The probe gives the next point of the route.
    pub(crate) fn level_tick(&mut self, now: Instant, every_call: bool) {
        if !self.level.measuring {
            return;
        }
        if !every_call
            && self
                .level
                .at
                .is_some_and(|t| now.duration_since(t) < POINT_EVERY)
        {
            return;
        }
        self.level.at = Some(now);
        match self.level.map.next_point() {
            Some(i) => {
                let (x, y) = self.level.map.grid.point(i);
                let _ = self.level.map.set(i, simulated_table(x, y));
                if self.level.map.is_complete() {
                    self.level.measuring = false;
                    self.level_save();
                }
            }
            None => self.level.measuring = false,
        }
    }

    pub(super) fn paint_levelling(&self, cv: &mut Cv, x: f32, top: f32) {
        let t = cv.t.clone();
        let m = &self.level.map;
        cv.text(
            x + 30.0,
            top + 4.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::LevDemo),
            14.0,
            col(t.dim),
        );
        self.stepper(
            cv,
            top + 20.0,
            self.tr(Text::LevCols),
            &format!("{}", self.level.cols),
            Field::LevCols,
            [-5.0, -1.0, 1.0, 5.0],
            !self.level.measuring,
        );
        self.stepper(
            cv,
            top + 80.0,
            self.tr(Text::LevRows),
            &format!("{}", self.level.rows),
            Field::LevRows,
            [-5.0, -1.0, 1.0, 5.0],
            !self.level.measuring,
        );
        cv.chip(
            x + 30.0,
            top + 140.0,
            220.0,
            44.0,
            if self.level.measuring {
                self.tr(Text::LevStop)
            } else {
                self.tr(Text::LevMeasure)
            },
            self.level.measuring,
            Act::LevelToggle,
        );
        cv.chip(
            x + 270.0,
            top + 140.0,
            220.0,
            44.0,
            self.tr(Text::LevClear),
            false,
            Act::LevelClear,
        );
        cv.checkbox(
            x + 510.0,
            top + 146.0,
            300.0,
            self.tr(Text::LevApply),
            self.level.apply,
            Act::LevelApply,
        );
        // the map: one cell for each point
        let (mx, my, mw, mh) = (x + 30.0, top + 204.0, 780.0, 300.0);
        cv.p.rect_filled(
            cv.rect(mx, my, mw, mh),
            4.0 * cv.u,
            alpha(col(t.panel_raised), 255),
        );
        let (cols, rows) = (m.grid.cols, m.grid.rows);
        let (cw, ch) = (mw / cols as f32, mh / rows as f32);
        let (lo, hi, _) = m.spread().unwrap_or((0.0, 0.0, 0.0));
        let next = m.next_point();
        for r in 0..rows {
            for c in 0..cols {
                let i = r * cols + c;
                // the first row of the grid (Y at its lowest) is drawn at the bottom, as the table is seen from above
                let rect = cv.rect(
                    mx + c as f32 * cw + 2.0,
                    my + (rows - 1 - r) as f32 * ch + 2.0,
                    cw - 4.0,
                    ch - 4.0,
                );
                match m.get(i) {
                    Some(z) => {
                        let k = if hi > lo {
                            ((z - lo) / (hi - lo)) as f32
                        } else {
                            0.5
                        };
                        let colour = Color32::from_rgb(
                            (60.0 + 195.0 * k) as u8,
                            (140.0 - 70.0 * k) as u8,
                            (255.0 - 195.0 * k) as u8,
                        );
                        cv.p.rect_filled(rect, 3.0 * cv.u, colour);
                        cv.text(
                            mx + (c as f32 + 0.5) * cw,
                            my + (rows - 1 - r) as f32 * ch + ch / 2.0 + 6.0,
                            Align2::CENTER_BOTTOM,
                            &format!("{z:+.3}"),
                            13.0,
                            Color32::BLACK,
                        );
                    }
                    None => {
                        let now = self.level.measuring && next == Some(i);
                        cv.p.rect_stroke(
                            rect,
                            3.0 * cv.u,
                            Stroke::new(
                                if now { 2.0 } else { 1.0 },
                                if now {
                                    col(t.accent)
                                } else {
                                    alpha(col(t.dim), 120)
                                },
                            ),
                            egui::StrokeKind::Inside,
                        );
                    }
                }
            }
        }
        // what the map says
        let mut line = format!(
            "{}: {} / {}",
            self.tr(Text::LevPoints),
            m.measured(),
            m.grid.len()
        );
        if let Some((_, _, spread)) = m.spread() {
            line += &format!("   -   {}: {:.3} mm", self.tr(Text::LevSpread), spread);
        }
        if let Some((tx, ty)) = m.tilt_mm_per_m().filter(|_| m.is_complete()) {
            line += &format!(
                "   -   {}: {:+.2} / {:+.2} mm/m",
                self.tr(Text::LevTilt),
                tx,
                ty
            );
        }
        cv.text(
            x + 30.0,
            my + mh + 34.0,
            Align2::LEFT_BOTTOM,
            &line,
            16.0,
            col(t.text),
        );
    }
}
