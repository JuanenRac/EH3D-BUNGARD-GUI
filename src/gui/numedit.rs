//! Typing a number: every value that has buttons to step it can also be clicked and written with the keyboard (Enter or a click
//! elsewhere accepts it, Esc leaves it as it was), and the tools can be given their default values or any value of the operator.

use super::board::{specs, CellSpec};
use super::extra::Field;
use super::*;
use crate::boards::Core;

/// A value that can be typed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EditTarget {
    Field(Field),
    /// A cell of the tool table: the program, the row and the column.
    ToolCell(Program, usize, usize),
    /// The width (true) or the depth (false) of the board.
    BoardSize(bool),
}

/// The box that is being typed in: where it is on the canvas (design units), how wide and what it holds.
pub(crate) struct NumEdit {
    pub target: EditTarget,
    pub text: String,
    pub pos: (f32, f32),
    pub w: f32,
    pub fresh: bool,
}

impl App {
    /// The value now, as a number.
    fn edit_value(&self, t: EditTarget) -> Option<f64> {
        Some(match t {
            EditTarget::Field(f) => match f {
                Field::Columns => self.panel.columns as f64,
                Field::Rows => self.panel.rows as f64,
                Field::SpacingX => self.len_to_display(self.panel.spacing.0),
                Field::SpacingY => self.len_to_display(self.panel.spacing.1),
                Field::TableW => self.len_to_display(self.table.width_mm),
                Field::TableD => self.len_to_display(self.table.depth_mm),
            },
            EditTarget::ToolCell(p, row, c) => self
                .used_tools(p)
                .get(row)?
                .cells
                .get(c)?
                .trim()
                .parse()
                .ok()?,
            EditTarget::BoardSize(width) => {
                let (w, d) = self
                    .board
                    .size_mm
                    .or_else(|| self.board_rect().map(|(a, b, c, e)| (c - a, e - b)))?;
                self.len_to_display(if width { w } else { d })
            }
        })
    }

    /// Opens the box over a value, with its text selected.
    pub(crate) fn start_edit(&mut self, target: EditTarget, x: f32, y: f32, w: f32) {
        self.commit_edit();
        let Some(v) = self.edit_value(target) else {
            return;
        };
        let text = if (v - v.round()).abs() < 1e-9 {
            format!("{v:.0}")
        } else {
            format!("{v:.2}")
        };
        self.edit = Some(NumEdit {
            target,
            text,
            pos: (x, y),
            w,
            fresh: true,
        });
    }

    /// Takes what was typed: a number (the comma is taken as the decimal point) becomes the value, anything else leaves it as it was.
    pub(crate) fn commit_edit(&mut self) {
        let Some(ed) = self.edit.take() else {
            return;
        };
        let Ok(v) = ed.text.trim().replace(',', ".").parse::<f64>() else {
            return;
        };
        if !v.is_finite() {
            return;
        }
        match ed.target {
            EditTarget::Field(f) => {
                let now = self.edit_value(ed.target).unwrap_or(v);
                self.adjust(f, (v - now) as f32);
            }
            EditTarget::ToolCell(p, row, c) => self.tool_set(p, row, c, v),
            EditTarget::BoardSize(width) => {
                let now = self.edit_value(ed.target).unwrap_or(v);
                self.board_adjust(width, (v - now) as f32);
            }
        }
    }

    /// The box over the window, drawn by egui (the canvas has no text boxes of its own).
    pub(super) fn edit_overlay(&mut self, ctx: &egui::Context) {
        let Some(mut ed) = self.edit.take() else {
            return;
        };
        let u = self.ui_scale;
        let pos = Pos2::new(
            self.ui_origin.x + ed.pos.0 * u,
            self.ui_origin.y + ed.pos.1 * u,
        );
        let mut finished = false;
        egui::Area::new(egui::Id::unique("number-edit"))
            .fixed_pos(pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let mut out = egui::TextEdit::singleline(&mut ed.text)
                    .desired_width(ed.w * u)
                    .font(FontId::proportional(19.0 * u))
                    .align(Align2::CENTER_CENTER)
                    .show(ui);
                if ed.fresh {
                    out.response.request_focus();
                    let n = ed.text.chars().count();
                    out.state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(0),
                            egui::text::CCursor::new(n),
                        )));
                    out.state.store(ui.ctx(), out.response.id);
                    ed.fresh = false;
                } else if out.response.lost_focus() {
                    finished = true;
                }
            });
        self.edit = Some(ed);
        if finished {
            self.commit_edit();
        }
    }

    // ------------------------------------------------------------------------------------------ the tools

    /// Gives a cell of the tool table the typed value, kept inside what the cell can hold.
    pub(crate) fn tool_set(&mut self, p: Program, row: usize, col_i: usize, v: f64) {
        let kind = self.engine_of(p).kind();
        let Some(spec) = specs(kind).get(col_i) else {
            return;
        };
        let all = self.engine_of(p).tool_rows();
        let Some(mut r) = all.into_iter().filter(|r| r.name.is_some()).nth(row) else {
            return;
        };
        if let Some(cell) = r.cells.get_mut(col_i) {
            *cell = format_cell(spec, v);
        }
        self.side_mut(p).engine.apply_tool_rows(&[r]);
        self.saved = false;
    }

    /// The default values of one tool (by the diameter of its drill, or a router of 2 mm, and the material of the board).
    pub(crate) fn tool_default(&mut self, p: Program, row: usize) {
        let core: Core = self.board.material().core;
        let rows = self.used_tools(p);
        let Some(tool) = rows.get(row).and_then(|r| r.name) else {
            return;
        };
        let values: Vec<f64> = match p {
            Program::Drill => {
                let d = self
                    .drill
                    .scene
                    .holes
                    .iter()
                    .find(|h| h.tool == tool)
                    .map_or(1.0, |h| h.diameter_mm);
                let (e, f, r) = crate::defaults::drill(d, core);
                vec![e, f as f64, r as f64]
            }
            Program::Route => {
                let (d, n, xy, z, r) = crate::defaults::route(self.board.thickness_mm, core);
                vec![d, n as f64, xy as f64, z as f64, r as f64]
            }
        };
        for (c, v) in values.into_iter().enumerate() {
            self.tool_set(p, row, c, v);
        }
    }

    /// The default values of every tool of the job.
    pub(crate) fn tools_defaults(&mut self) {
        for p in [Program::Drill, Program::Route] {
            for row in 0..self.used_tools(p).len() {
                self.tool_default(p, row);
            }
        }
    }
}

/// The text of a value in a cell: with two decimals where the cell has them, whole otherwise, kept inside its limits.
pub(crate) fn format_cell(spec: &CellSpec, v: f64) -> String {
    let v = v.clamp(spec.min, spec.max);
    if spec.decimals {
        format!("{v:.2}")
    } else {
        format!("{v:.0}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;
    use std::path::{Path, PathBuf};

    fn testdata() -> PathBuf {
        // a copy of the data, so that saving never touches the files of the repository
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("eh3d-bg-numedit-{}-{n}", std::process::id()));
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
        dir
    }

    fn app() -> App {
        let dir = testdata();
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
    fn a_typed_value_becomes_the_value() {
        let mut a = app();
        a.start_edit(EditTarget::Field(Field::TableW), 0.0, 0.0, 100.0);
        a.edit.as_mut().unwrap().text = "612".into();
        a.commit_edit();
        assert_eq!(a.table.width_mm, 612.0);
        // the comma is a decimal point; text that is not a number leaves the value as it was
        a.start_edit(EditTarget::Field(Field::SpacingX), 0.0, 0.0, 100.0);
        a.edit.as_mut().unwrap().text = "7,5".into();
        a.commit_edit();
        assert_eq!(a.panel.spacing.0, 7.5);
        a.start_edit(EditTarget::Field(Field::SpacingX), 0.0, 0.0, 100.0);
        a.edit.as_mut().unwrap().text = "abc".into();
        a.commit_edit();
        assert_eq!(a.panel.spacing.0, 7.5);
    }

    #[test]
    fn the_cells_of_the_tools_take_typed_values_within_their_limits() {
        let mut a = app();
        let before = a.used_tools(Program::Drill)[0].cells[1].clone();
        a.start_edit(EditTarget::ToolCell(Program::Drill, 0, 1), 0.0, 0.0, 100.0);
        assert_eq!(
            a.edit.as_ref().unwrap().text.trim(),
            before.trim().trim_start_matches('0')
        );
        a.edit.as_mut().unwrap().text = "1234".into();
        a.commit_edit();
        assert_eq!(
            a.used_tools(Program::Drill)[0].cells[1]
                .trim()
                .parse::<f64>()
                .unwrap(),
            1234.0
        );
        a.tool_set(Program::Drill, 0, 2, 5_000_000.0);
        assert_eq!(
            a.used_tools(Program::Drill)[0].cells[2]
                .trim()
                .parse::<f64>()
                .unwrap(),
            99_999.0
        );
    }

    #[test]
    fn the_default_values_come_from_the_diameter_and_the_material() {
        let mut a = app();
        a.tools_defaults();
        let rows = a.used_tools(Program::Drill);
        let big = rows.iter().find(|r| r.name == Some(32)).unwrap();
        let small = rows.iter().find(|r| r.name == Some(8)).unwrap();
        let rpm =
            |r: &eh3d_drillpro::classic::engine::ToolRow| r.cells[2].trim().parse::<u32>().unwrap();
        assert!(rpm(small) > rpm(big));
        assert_eq!(
            a.used_tools(Program::Route)[0].cells[1]
                .trim()
                .parse::<u32>()
                .unwrap(),
            2
        );
    }

    #[test]
    fn the_simulation_follows_the_speed_chosen() {
        let steps = |speed: f32| {
            let mut a = app();
            a.settings.sim_speed = speed;
            a.auto_ok = true;
            a.fixed_dt = Some(0.016);
            a.start_job();
            let mut n = 0;
            while a.running.is_some() && n < 5_000_000 {
                a.tick();
                n += 1;
            }
            n
        };
        assert!(steps(10.0) < steps(1.0));
    }
}
