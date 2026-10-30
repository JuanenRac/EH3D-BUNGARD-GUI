//! The board of the job and the tools of the job: the library of base materials with its window, the tool table with its window, and the
//! list of tools of the left card (drilling and routing, which one is working).

use super::extra::wrap;
use super::*;
use crate::boards::{self, Sides, MATERIALS};
use eh3d_drillpro::protocol::sound::Cue;

/// The materials of a maker that the Board window shows at once.
pub(super) const MAT_ROWS: usize = 9;
use eh3d_drillpro::classic::engine::{Kind as EngineKind, ToolRow};

/// A choice in the window of the board.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BoardPick {
    Maker(usize),
    Material(usize),
    Thickness(usize),
    Copper(usize),
    Layers(u8),
    /// A standard format of the material (by its place), or the size of the files of the job (`None`).
    Size(Option<usize>),
}

/// One tool of the job as the left card shows it.
pub(crate) struct ToolLine {
    pub prog: Program,
    pub tool: u8,
    pub main: String,
    pub detail: String,
    pub n: usize,
    pub done: usize,
    pub color: usize,
}

/// How a cell of the tool table is edited: the smallest and the largest value, whether it has decimals, and the size of a step.
pub(super) struct CellSpec {
    pub(super) label: Text,
    pub(super) min: f64,
    pub(super) max: f64,
    pub(super) decimals: bool,
    pub(super) step: f64,
}

pub(super) fn specs(kind: EngineKind) -> &'static [CellSpec] {
    match kind {
        EngineKind::Drill => &[
            CellSpec {
                label: Text::LblExtraStroke,
                min: 0.0,
                max: 20.0,
                decimals: true,
                step: 0.1,
            },
            CellSpec {
                label: Text::LblFeed,
                min: 10.0,
                max: 9999.0,
                decimals: false,
                step: 50.0,
            },
            CellSpec {
                label: Text::LblSpeedRpm,
                min: 1000.0,
                max: 99999.0,
                decimals: false,
                step: 1000.0,
            },
        ],
        EngineKind::Route => &[
            CellSpec {
                label: Text::LblDepthMm,
                min: 0.0,
                max: 20.0,
                decimals: true,
                step: 0.1,
            },
            CellSpec {
                label: Text::LblPasses,
                min: 1.0,
                max: 99.0,
                decimals: false,
                step: 1.0,
            },
            CellSpec {
                label: Text::LblFeedXy,
                min: 10.0,
                max: 9999.0,
                decimals: false,
                step: 50.0,
            },
            CellSpec {
                label: Text::LblFeedZ,
                min: 10.0,
                max: 9999.0,
                decimals: false,
                step: 50.0,
            },
            CellSpec {
                label: Text::LblSpeedRpm,
                min: 1000.0,
                max: 99999.0,
                decimals: false,
                step: 1000.0,
            },
        ],
    }
}

impl App {
    pub(super) fn engine_of(&self, p: Program) -> &dyn ClassicEngine {
        match p {
            Program::Drill => &*self.drill.engine,
            Program::Route => &*self.route.engine,
        }
    }

    /// The tools the file of a program uses, with their values.
    pub(crate) fn used_tools(&self, p: Program) -> Vec<ToolRow> {
        self.engine_of(p)
            .tool_rows()
            .into_iter()
            .filter(|r| r.name.is_some())
            .collect()
    }

    /// The tools of the job, drilling first and then routing, one line each.
    pub(crate) fn tool_lines(&self) -> Vec<ToolLine> {
        let mut lines = Vec::new();
        let rows = self.used_tools(Program::Drill);
        let mut tools: Vec<u8> = self.drill.scene.holes.iter().map(|h| h.tool).collect();
        tools.sort_unstable();
        tools.dedup();
        for t in tools {
            let mine: Vec<_> = self
                .drill
                .scene
                .holes
                .iter()
                .filter(|h| h.tool == t)
                .collect();
            let row = rows.iter().find(|r| r.name == Some(t));
            let cell = |i: usize| {
                row.and_then(|r| r.cells.get(i))
                    .cloned()
                    .unwrap_or_default()
            };
            lines.push(ToolLine {
                prog: Program::Drill,
                tool: t,
                main: format!("{:.2} mm", mine[0].diameter_mm),
                detail: format!(
                    "{} {}  {}/min  {} rpm",
                    mine.len(),
                    self.tr(Text::Holes),
                    cell(1).trim(),
                    cell(2).trim()
                ),
                n: mine.len(),
                done: mine.iter().filter(|h| h.done).count(),
                color: lines.len(),
            });
        }
        let rows = self.used_tools(Program::Route);
        let mut tools: Vec<u8> = self.route.scene.paths.iter().map(|q| q.tool).collect();
        tools.sort_unstable();
        tools.dedup();
        for t in tools {
            let mine: Vec<_> = self
                .route
                .scene
                .paths
                .iter()
                .filter(|q| q.tool == t)
                .collect();
            let row = rows.iter().find(|r| r.name == Some(t));
            let cell = |i: usize| {
                row.and_then(|r| r.cells.get(i))
                    .cloned()
                    .unwrap_or_default()
            };
            let points: usize = mine.iter().map(|q| q.points.len()).sum();
            lines.push(ToolLine {
                prog: Program::Route,
                tool: t,
                main: format!("{} {t}", self.tr(Text::Routing)),
                detail: format!(
                    "{}  {}/min  {} rpm",
                    cell(0).trim(),
                    cell(2).trim(),
                    cell(4).trim()
                ),
                n: points,
                done: mine.iter().map(|q| q.reached).sum(),
                color: lines.len(),
            });
        }
        lines
    }

    /// The top of the left card below the name of the file: the board and then the tools, each marked as drilling or routing and the
    /// one that is working lit. Returns where the next thing goes.
    pub(super) fn paint_job_tools(&self, cv: &mut Cv, y0: f32) -> f32 {
        let t = cv.t.clone();
        let lines = self.tool_lines();
        let (drills, routes) = (
            lines.iter().filter(|l| l.prog == Program::Drill).count(),
            lines.iter().filter(|l| l.prog == Program::Route).count(),
        );
        let kind = match (drills > 0, routes > 0) {
            (true, true) => format!("{} + {}", self.tr(Text::Drilling), self.tr(Text::Routing)),
            (true, false) => self.tr(Text::Drilling).to_string(),
            (false, true) => self.tr(Text::Routing).to_string(),
            (false, false) => String::new(),
        };
        cv.text(38.0, y0, Align2::LEFT_BOTTOM, &kind, 13.0, col(t.accent));
        cv.text(
            38.0,
            y0 + 20.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{} {}",
                self.board.material().maker,
                self.board.material().name
            ),
            13.0,
            col(t.text),
        );
        cv.text(
            38.0,
            y0 + 38.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{:.2} mm - {:.0} um Cu - {}",
                self.board.thickness_mm,
                self.board.copper_um,
                if self.board.layers == 1 {
                    self.tr(Text::LblOneSide)
                } else {
                    self.tr(Text::LblTwoSides)
                }
            ),
            12.0,
            col(t.dim),
        );
        cv.hits.push((
            cv.rect(30.0, y0 - 16.0, 260.0, 60.0),
            Act::Show(Modal::Board),
        ));
        let working = self
            .running
            .and_then(|p| self.running_view().and_then(|v| v.tool).map(|n| (p, n)));
        let mut y = y0 + 52.0;
        for l in lines.iter().take(7) {
            let c = col(t.size_color(l.color));
            let lit = working == Some((l.prog, l.tool));
            let r = cv.rect(38.0, y, 244.0, 48.0);
            cv.p.rect_filled(
                r,
                6.0 * cv.u,
                if lit {
                    alpha(c, 70)
                } else {
                    col(t.panel_raised)
                },
            );
            if lit {
                cv.p.rect_stroke(
                    r,
                    6.0 * cv.u,
                    Stroke::new(1.6 * cv.u, c),
                    StrokeKind::Inside,
                );
            }
            cv.p.circle_filled(cv.pt(56.0, y + 24.0), 7.0 * cv.u, c);
            let tag = if l.prog == Program::Drill { "D" } else { "R" };
            cv.text(
                72.0,
                y + 20.0,
                Align2::LEFT_BOTTOM,
                &format!("{tag}  T{:02}  {}", l.tool, l.main),
                15.0,
                col(t.text),
            );
            cv.text(
                72.0,
                y + 36.0,
                Align2::LEFT_BOTTOM,
                &l.detail,
                11.0,
                col(t.dim),
            );
            cv.p.rect_filled(
                cv.rect(72.0, y + 40.0, 200.0, 4.0),
                2.0 * cv.u,
                alpha(Color32::WHITE, 26),
            );
            if l.n > 0 {
                cv.p.rect_filled(
                    cv.rect(72.0, y + 40.0, 200.0 * l.done as f32 / l.n as f32, 4.0),
                    2.0 * cv.u,
                    c,
                );
            }
            y += 52.0;
        }
        // a click on a tool takes the plan view to its work
        for (i, l) in lines.iter().take(7).enumerate() {
            cv.hits.push((
                cv.rect(38.0, y0 + 52.0 + 52.0 * i as f32, 244.0, 48.0),
                Act::CenterTool(l.prog, l.tool),
            ));
        }
        y
    }

    // ------------------------------------------------------------------------------------------ the board

    /// The board as it is on the table: its size is the one chosen or the one of the files.
    pub(crate) fn board_rect(&self) -> Option<(f64, f64, f64, f64)> {
        let mut all = self.drill.base.clone();
        all.overlay(&self.route.base);
        let (x0, y0, x1, y1) = all.bounds().unwrap_or((0.0, 0.0, 0.0, 0.0));
        match self.board.size_mm {
            Some((w, d)) => Some((x0 - 3.0, y0 - 3.0, x0 - 3.0 + w, y0 - 3.0 + d)),
            None => all
                .bounds()
                .map(|_| (x0 - 3.0, y0 - 3.0, x1 + 3.0, y1 + 3.0)),
        }
    }

    fn apply_board(&mut self) {
        self.drill.engine.set_board(self.board.thickness_mm);
        self.route.engine.set_board(self.board.thickness_mm);
        self.settings.board = self.board;
        self.remember_settings();
    }

    pub(crate) fn board_pick(&mut self, p: BoardPick) {
        let m = self.board.material();
        match p {
            BoardPick::Maker(i) => {
                self.maker = i;
                self.mat_scroll = 0;
                return;
            }
            BoardPick::Material(i) => self.board = self.board.with_material(i),
            BoardPick::Thickness(i) => {
                if let Some(&v) = m.thickness_mm.get(i) {
                    self.board.thickness_mm = v;
                }
            }
            BoardPick::Copper(i) => {
                if let Some(&v) = m.copper_um.get(i) {
                    self.board.copper_um = v;
                }
            }
            BoardPick::Layers(n) => {
                if (n == 1 && m.sides != Sides::Two) || (n == 2 && m.sides != Sides::One) {
                    self.board.layers = n;
                }
            }
            BoardPick::Size(None) => self.board.size_mm = None,
            BoardPick::Size(Some(i)) => self.board.size_mm = m.formats_mm.get(i).copied(),
        }
        self.apply_board();
    }

    pub(crate) fn mat_scroll_by(&mut self, delta: i32) {
        let of_maker = MATERIALS
            .iter()
            .filter(|m| m.maker == boards::makers()[self.maker.min(boards::makers().len() - 1)])
            .count();
        let max = of_maker.saturating_sub(MAT_ROWS);
        self.mat_scroll = (self.mat_scroll as i64 + delta as i64).clamp(0, max as i64) as usize;
    }

    pub(crate) fn board_adjust(&mut self, width: bool, d: f32) {
        let d = self.len_from_display(d as f64) as f32;
        let (w, h) = self.board.size_mm.unwrap_or_else(|| {
            self.board_rect()
                .map_or((100.0, 80.0), |(a, b, c, e)| (c - a, e - b))
        });
        let d = d as f64;
        self.board.size_mm = Some(if width {
            ((w + d).clamp(10.0, 2000.0), h)
        } else {
            (w, (h + d).clamp(10.0, 2000.0))
        });
        self.apply_board();
    }

    // ------------------------------------------------------------------------------------------ the tools

    pub(crate) fn tool_adjust(&mut self, p: Program, row: usize, col_i: usize, dir: f32) {
        let kind = self.engine_of(p).kind();
        let Some(spec) = specs(kind).get(col_i) else {
            return;
        };
        let all = self.engine_of(p).tool_rows();
        let Some(mut r) = all.into_iter().filter(|r| r.name.is_some()).nth(row) else {
            return;
        };
        let Some(cell) = r.cells.get_mut(col_i) else {
            return;
        };
        let v: f64 = cell.trim().parse().unwrap_or(spec.min);
        let nv = (v + spec.step * dir as f64).clamp(spec.min, spec.max);
        *cell = if spec.decimals {
            format!("{nv:.2}")
        } else {
            format!("{nv:.0}")
        };
        self.side_mut(p).engine.apply_tool_rows(&[r]);
        self.saved = false;
    }

    pub(crate) fn tools_save(&mut self) {
        let a = self.drill.engine.save_tools();
        let b = self.route.engine.save_tools();
        match (a, b) {
            (Ok(_), Ok(_)) => {
                self.saved = true;
                self.message = None;
            }
            (Err(e), _) | (_, Err(e)) => self.message = Some(e),
        }
    }

    // ------------------------------------------------------------------------------------------ the windows

    pub(super) fn paint_board_modal(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y, w, h) = (150.0, 70.0, 1300.0, 770.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 44.0, self.tr(Text::BoardTitle));
        let b = self.board;
        let m = b.material();
        // the makers
        let makers = boards::makers();
        for (i, mk) in makers.iter().enumerate() {
            let (col_i, row_i) = (i % 4, i / 4);
            cv.chip(
                x + 30.0 + col_i as f32 * 152.0,
                y + 70.0 + row_i as f32 * 46.0,
                144.0,
                38.0,
                mk,
                *mk == makers[self.maker.min(makers.len() - 1)],
                Act::Board(BoardPick::Maker(i)),
            );
        }
        // the materials of the maker
        let maker = makers[self.maker.min(makers.len() - 1)];
        let mut ly = y + 70.0 + (makers.len().div_ceil(4)) as f32 * 46.0 + 12.0;
        let of_maker: Vec<(usize, &crate::boards::Material)> = MATERIALS
            .iter()
            .enumerate()
            .filter(|(_, mt)| mt.maker == maker)
            .collect();
        let shown = self.mat_scroll.min(of_maker.len().saturating_sub(MAT_ROWS));
        if of_maker.len() > MAT_ROWS {
            cv.chip(x + 596.0, ly, 36.0, 36.0, "^", false, Act::MatScroll(-3));
            cv.chip(
                x + 596.0,
                ly + 42.0,
                36.0,
                36.0,
                "v",
                false,
                Act::MatScroll(3),
            );
            cv.text(
                x + 600.0,
                ly + 110.0,
                Align2::LEFT_BOTTOM,
                &format!(
                    "{}/{}",
                    (shown + MAT_ROWS).min(of_maker.len()),
                    of_maker.len()
                ),
                12.0,
                col(t.dim),
            );
        }
        for &(i, mat) in of_maker.iter().skip(shown).take(MAT_ROWS) {
            let on = i == b.material;
            let r = cv.rect(x + 30.0, ly, 556.0, 34.0);
            cv.p.rect_filled(
                r,
                6.0 * cv.u,
                if on {
                    alpha(col(t.accent), 90)
                } else {
                    col(t.panel_raised)
                },
            );
            cv.text(
                x + 44.0,
                ly + 24.0,
                Align2::LEFT_BOTTOM,
                &format!(
                    "{}  -  {}{}",
                    mat.name,
                    mat.core.name(),
                    mat.dk.map_or(String::new(), |d| format!("  Dk {d}"))
                ),
                15.0,
                col(t.text),
            );
            cv.hits.push((r, Act::Board(BoardPick::Material(i))));
            ly += 38.0;
        }
        // what the chosen material is
        let rx = x + 640.0;
        let mut ry = y + 70.0;
        cv.text(
            rx,
            ry + 14.0,
            Align2::LEFT_BOTTOM,
            &format!("{} {}", m.maker, m.name),
            22.0,
            col(t.text),
        );
        ry += 28.0;
        let facts = format!(
            "{}{}{}{}",
            m.core.name(),
            m.tg_c
                .map_or(String::new(), |g| format!("  -  Tg {g:.0} C")),
            m.dk.map_or(String::new(), |d| format!("  -  Dk {d}")),
            m.df.map_or(String::new(), |d| format!("  -  Df {d}")),
        );
        cv.text(rx, ry + 14.0, Align2::LEFT_BOTTOM, &facts, 14.0, col(t.dim));
        ry += 22.0;
        for l in wrap(m.note, 70) {
            cv.text(rx, ry + 14.0, Align2::LEFT_BOTTOM, &l, 13.0, col(t.dim));
            ry += 18.0;
        }
        if m.photoresist {
            cv.text(
                rx,
                ry + 14.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::LblPhotoresist),
                13.0,
                col(t.ok),
            );
            ry += 18.0;
        }
        ry += 14.0;
        cv.text(
            rx,
            ry + 14.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LblThickness).to_uppercase(),
            12.0,
            col(t.dim),
        );
        ry += 20.0;
        for (i, v) in m.thickness_mm.iter().enumerate() {
            cv.chip(
                rx + (i % 7) as f32 * 92.0,
                ry + (i / 7) as f32 * 42.0,
                84.0,
                34.0,
                &format!("{v}"),
                (b.thickness_mm - v).abs() < 1e-9,
                Act::Board(BoardPick::Thickness(i)),
            );
        }
        ry += (m.thickness_mm.len().div_ceil(7)) as f32 * 42.0 + 10.0;
        cv.text(
            rx,
            ry + 14.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LblCopper).to_uppercase(),
            12.0,
            col(t.dim),
        );
        ry += 20.0;
        for (i, v) in m.copper_um.iter().enumerate() {
            cv.chip(
                rx + i as f32 * 92.0,
                ry,
                84.0,
                34.0,
                &format!("{v}"),
                (b.copper_um - v).abs() < 1e-9,
                Act::Board(BoardPick::Copper(i)),
            );
        }
        ry += 54.0;
        cv.text(
            rx,
            ry + 14.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LblCopperSides).to_uppercase(),
            12.0,
            col(t.dim),
        );
        ry += 20.0;
        for (n, label) in [
            (1u8, self.tr(Text::LblOneSide)),
            (2u8, self.tr(Text::LblTwoSides)),
        ] {
            cv.chip(
                rx + (n - 1) as f32 * 132.0,
                ry,
                124.0,
                34.0,
                label,
                b.layers == n,
                Act::Board(BoardPick::Layers(n)),
            );
        }
        ry += 54.0;
        cv.text(
            rx,
            ry + 14.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LblSize).to_uppercase(),
            12.0,
            col(t.dim),
        );
        ry += 20.0;
        cv.chip(
            rx,
            ry,
            150.0,
            34.0,
            self.tr(Text::LblFromFiles),
            b.size_mm.is_none(),
            Act::Board(BoardPick::Size(None)),
        );
        for (i, (fw, fd)) in m.formats_mm.iter().enumerate().take(6) {
            cv.chip(
                rx + 160.0 + (i % 4) as f32 * 118.0,
                ry + (i / 4) as f32 * 42.0,
                110.0,
                34.0,
                &format!("{fw:.0}x{fd:.0}"),
                b.size_mm == Some((*fw, *fd)),
                Act::Board(BoardPick::Size(Some(i))),
            );
        }
        ry += 92.0;
        let (sw, sd) = b
            .size_mm
            .or_else(|| self.board_rect().map(|(a, bb, c, e)| (c - a, e - bb)))
            .unwrap_or((0.0, 0.0));
        for (k, (label, val, width)) in [
            (self.tr(Text::LblWidth).to_uppercase(), sw, true),
            (self.tr(Text::LblDepth).to_uppercase(), sd, false),
        ]
        .into_iter()
        .enumerate()
        {
            let yy = ry + k as f32 * 46.0;
            cv.text(
                rx,
                yy + 28.0,
                Align2::LEFT_BOTTOM,
                &label,
                15.0,
                col(t.text),
            );
            for (j, s) in [-10.0f32, -1.0, 1.0, 10.0].into_iter().enumerate() {
                let xx = rx + [130.0, 190.0, 380.0, 440.0][j];
                cv.chip(
                    xx,
                    yy,
                    54.0,
                    38.0,
                    &format!("{s:+}"),
                    false,
                    Act::BoardAdj(width, s),
                );
            }
            cv.text(
                rx + 320.0,
                yy + 28.0,
                Align2::CENTER_BOTTOM,
                &format!("{val:.1}"),
                20.0,
                col(t.text),
            );
            let rv = cv.rect(rx + 270.0, yy, 100.0, 38.0);
            cv.p.rect_stroke(
                rv,
                6.0 * cv.u,
                Stroke::new(cv.u, alpha(col(t.line), 70)),
                StrokeKind::Inside,
            );
            cv.hits.push((
                rv,
                Act::Edit(EditTarget::BoardSize(width), rx + 272.0, yy + 2.0, 96.0),
            ));
        }
        cv.chip(
            x + w - 200.0,
            y + h - 60.0,
            170.0,
            42.0,
            self.tr(Text::Close),
            true,
            Act::CloseModal,
        );
    }

    pub(super) fn paint_tools_modal(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y, w, h) = (110.0, 90.0, 1380.0, 720.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 44.0, self.tr(Text::Tools));
        let mut ry = y + 70.0;
        let mut index = 0;
        for p in [Program::Drill, Program::Route] {
            let kind = self.engine_of(p).kind();
            let sp = specs(kind);
            let rows = self.used_tools(p);
            if rows.is_empty() {
                continue;
            }
            let heading = if p == Program::Drill {
                self.tr(Text::Drilling)
            } else {
                self.tr(Text::Routing)
            };
            cv.text(
                x + 30.0,
                ry + 16.0,
                Align2::LEFT_BOTTOM,
                &heading.to_uppercase(),
                13.0,
                col(t.accent),
            );
            for (i, s) in sp.iter().enumerate() {
                cv.text(
                    x + 230.0 + i as f32 * 215.0 + 95.0,
                    ry + 16.0,
                    Align2::CENTER_BOTTOM,
                    self.tr(s.label),
                    12.0,
                    col(t.dim),
                );
            }
            if p == Program::Drill {
                cv.text(
                    x + 1250.0,
                    ry + 16.0,
                    Align2::RIGHT_BOTTOM,
                    self.tr(Text::HolesDrilled),
                    12.0,
                    col(t.dim),
                );
            }
            ry += 24.0;
            for (ri, r) in rows.iter().enumerate() {
                let n = r.name.unwrap_or(0);
                let c = col(t.size_color(index));
                cv.p.circle_filled(cv.pt(x + 44.0, ry + 20.0), 7.0 * cv.u, c);
                let diameter = if p == Program::Drill {
                    let count = self
                        .drill
                        .scene
                        .holes
                        .iter()
                        .filter(|hh| hh.tool == n)
                        .count();
                    self.drill
                        .scene
                        .holes
                        .iter()
                        .find(|hh| hh.tool == n)
                        .map_or(String::new(), |hh| {
                            format!("  {:.2} mm  \u{d7}{count}", hh.diameter_mm)
                        })
                } else {
                    String::new()
                };
                cv.text(
                    x + 64.0,
                    ry + 28.0,
                    Align2::LEFT_BOTTOM,
                    &format!("T{n:02}{diameter}"),
                    17.0,
                    col(t.text),
                );
                for (ci, s) in sp.iter().enumerate() {
                    let v = r.cells.get(ci).map_or("", |c| c.trim());
                    let bx = x + 230.0 + ci as f32 * 215.0;
                    cv.chip(
                        bx,
                        ry,
                        40.0,
                        38.0,
                        "-",
                        false,
                        Act::ToolAdj(p, ri, ci, -1.0),
                    );
                    cv.text(
                        bx + 95.0,
                        ry + 28.0,
                        Align2::CENTER_BOTTOM,
                        v,
                        19.0,
                        col(t.text),
                    );
                    cv.chip(
                        bx + 150.0,
                        ry,
                        40.0,
                        38.0,
                        "+",
                        false,
                        Act::ToolAdj(p, ri, ci, 1.0),
                    );
                    let _ = s;
                    let rv = cv.rect(bx + 45.0, ry, 100.0, 38.0);
                    cv.p.rect_stroke(
                        rv,
                        6.0 * cv.u,
                        Stroke::new(cv.u, alpha(col(t.line), 70)),
                        StrokeKind::Inside,
                    );
                    cv.hits.push((
                        rv,
                        Act::Edit(EditTarget::ToolCell(p, ri, ci), bx + 47.0, ry + 2.0, 96.0),
                    ));
                }
                if p == Program::Drill {
                    let total = self.settings.holes_drilled.get(&n).copied().unwrap_or(0);
                    cv.text(
                        x + 1250.0,
                        ry + 28.0,
                        Align2::RIGHT_BOTTOM,
                        &total.to_string(),
                        19.0,
                        col(t.dim),
                    );
                }
                cv.chip(
                    x + 1290.0,
                    ry,
                    76.0,
                    38.0,
                    self.tr(Text::LblDefault),
                    false,
                    Act::ToolDefault(p, ri),
                );
                ry += 46.0;
                index += 1;
            }
            ry += 16.0;
        }
        if index == 0 {
            cv.text(
                x + 30.0,
                y + 120.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::NoFile),
                20.0,
                col(t.dim),
            );
        }
        if self.saved {
            cv.text(
                x + 300.0,
                y + h - 30.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::Saved),
                18.0,
                col(t.ok),
            );
        }
        cv.text(
            x + 30.0,
            y + h - 30.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::TypeValueHint),
            14.0,
            col(t.dim),
        );
        cv.chip(
            x + w - 860.0,
            y + h - 60.0,
            220.0,
            42.0,
            self.tr(Text::ResetCounts),
            false,
            Act::ResetHoles,
        );
        cv.chip(
            x + w - 620.0,
            y + h - 60.0,
            200.0,
            42.0,
            self.tr(Text::DefaultValues),
            false,
            Act::ToolsDefaults,
        );
        cv.chip(
            x + w - 400.0,
            y + h - 60.0,
            170.0,
            42.0,
            self.tr(Text::Save),
            false,
            Act::ToolsSave,
        );
        cv.chip(
            x + w - 200.0,
            y + h - 60.0,
            170.0,
            42.0,
            self.tr(Text::Close),
            true,
            Act::CloseModal,
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------- the settings

const LANG_NAMES: [(Lang, &str); 8] = [
    (Lang::En, "English"),
    (Lang::Es, "Español"),
    (Lang::It, "Italiano"),
    (Lang::De, "Deutsch"),
    (Lang::Fr, "Français"),
    (Lang::Pt, "Português"),
    (Lang::Zh, "中文"),
    (Lang::Ja, "日本語"),
];

impl App {
    pub(crate) fn set_theme(&mut self, id: ThemeId) {
        self.theme = Theme::for_id(id);
        self.remember_settings();
    }

    pub(crate) fn set_lang(&mut self, l: Lang) {
        self.lang = l;
        let dl = drill_lang(l);
        self.drill.engine.set_lang(dl);
        self.route.engine.set_lang(dl);
        self.remember_settings();
    }

    pub(crate) fn toggle_ask(&mut self) {
        self.settings.ask_operator = !self.settings.ask_operator;
        self.remember_settings();
    }

    /// Opens this window directly the next time, or asks again at the launcher.
    pub(crate) fn toggle_skip(&mut self) {
        self.settings.choice = if self.settings.choice == Some(Choice::Modern) {
            None
        } else {
            Some(Choice::Modern)
        };
        self.remember_settings();
    }

    pub(super) fn paint_settings_modal(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y, w, h) = (380.0, 30.0, 840.0, 840.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 44.0, self.tr(Text::SettingsTitle));
        // the tabs
        for (i, n) in [Text::TabGeneral, Text::LayerMachine, Text::TabJob]
            .into_iter()
            .enumerate()
        {
            cv.chip(
                x + 30.0 + i as f32 * 210.0,
                y + 62.0,
                200.0,
                40.0,
                self.tr(n),
                self.settings_tab as usize == i,
                Act::Tab(i as u8),
            );
        }
        let top = y + 120.0;
        match self.settings_tab {
            0 => {
                // the theme
                cv.text(
                    x + 30.0,
                    top + 8.0,
                    Align2::LEFT_BOTTOM,
                    &self.tr(Text::ThemeLabel).to_uppercase(),
                    12.0,
                    col(t.dim),
                );
                for (i, id) in ThemeId::ALL.into_iter().enumerate() {
                    cv.chip(
                        x + 30.0 + (i % 3) as f32 * 262.0,
                        top + 20.0 + (i / 3) as f32 * 46.0,
                        254.0,
                        38.0,
                        self.tr(id.text()),
                        self.theme.id == id,
                        Act::SetTheme(id),
                    );
                }
                // the language
                let ly = top + 20.0 + (ThemeId::ALL.len() as f32 / 3.0).ceil() * 46.0 + 16.0;
                cv.text(
                    x + 30.0,
                    ly,
                    Align2::LEFT_BOTTOM,
                    &self.tr(Text::Language).to_uppercase(),
                    12.0,
                    col(t.dim),
                );
                for (i, (l, n)) in LANG_NAMES.into_iter().enumerate() {
                    cv.chip(
                        x + 30.0 + (i % 4) as f32 * 196.0,
                        ly + 12.0 + (i / 4) as f32 * 46.0,
                        188.0,
                        38.0,
                        n,
                        self.lang == l,
                        Act::SetLang(l),
                    );
                }
                // the units of the lengths on the screen
                let uy = ly + 124.0;
                cv.text(
                    x + 30.0,
                    uy,
                    Align2::LEFT_BOTTOM,
                    &self.tr(Text::Units).to_uppercase(),
                    12.0,
                    col(t.dim),
                );
                cv.chip(
                    x + 30.0,
                    uy + 12.0,
                    380.0,
                    38.0,
                    self.tr(Text::UnitMm),
                    !self.settings.display_mil,
                    Act::DisplayMil(false),
                );
                cv.chip(
                    x + 420.0,
                    uy + 12.0,
                    380.0,
                    38.0,
                    self.tr(Text::UnitMil),
                    self.settings.display_mil,
                    Act::DisplayMil(true),
                );
                // the speed of the simulation
                let sy = uy + 78.0;
                cv.text(
                    x + 30.0,
                    sy,
                    Align2::LEFT_BOTTOM,
                    &self.tr(Text::SimSpeed).to_uppercase(),
                    12.0,
                    col(t.dim),
                );
                for (i, v) in [0.5f32, 1.0, 2.0, 5.0, 10.0, 50.0].into_iter().enumerate() {
                    cv.chip(
                        x + 30.0 + i as f32 * 130.0,
                        sy + 12.0,
                        122.0,
                        38.0,
                        &format!("x{v}"),
                        (self.settings.sim_speed - v).abs() < 1e-3,
                        Act::SetSpeed(v),
                    );
                }
            }
            1 => {
                // the model of the machine
                cv.text(
                    x + 30.0,
                    top + 8.0,
                    Align2::LEFT_BOTTOM,
                    &self.tr(Text::ModelLabel).to_uppercase(),
                    12.0,
                    col(t.dim),
                );
                let models = crate::machines::models();
                for (i, m) in models.iter().enumerate() {
                    cv.chip(
                        x + 30.0 + (i % 4) as f32 * 196.0,
                        top + 20.0 + (i / 4) as f32 * 46.0,
                        188.0,
                        38.0,
                        &m.name(),
                        self.model() == i,
                        Act::Model(i),
                    );
                }
                let m = models[self.model().min(models.len() - 1)];
                let pr = m.profile();
                let ty = top + 20.0 + (models.len().div_ceil(4)) as f32 * 46.0 + 14.0;
                let tc = if m.change == crate::machines::ToolChange::Automatic {
                    self.tr(Text::TcAuto)
                } else {
                    self.tr(Text::TcManual)
                };
                cv.text(
                    x + 30.0,
                    ty + 16.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{}: {}", self.tr(Text::ToolChange), tc),
                    16.0,
                    col(t.text),
                );
                let (ww, wd, wz) = pr.work_mm;
                let (ow, od, oz) = pr.overall_mm;
                cv.text(
                    x + 30.0,
                    ty + 40.0,
                    Align2::LEFT_BOTTOM,
                    &format!(
                        "{}: {} x {} x {} {u} | {}: {} x {} x {} {u} | {:.0} kg{}",
                        self.tr(Text::LblWorkArea),
                        self.len_text(ww),
                        self.len_text(wd),
                        self.len_text(wz),
                        self.tr(Text::LblWholeMachine),
                        self.len_text(ow),
                        self.len_text(od),
                        self.len_text(oz),
                        pr.weight_kg,
                        if m.rack_places() > 0 {
                            format!(" | {} {}", m.rack_places(), self.tr(Text::RackPlaces))
                        } else {
                            String::new()
                        },
                        u = self.len_unit()
                    ),
                    14.0,
                    col(t.dim),
                );
                // the rack of the machine file
                let rack = self.drill.engine.settings().map_or(0, |s| s.atc.len());
                if m.change == crate::machines::ToolChange::Automatic && rack == 0 {
                    cv.text(
                        x + 30.0,
                        ty + 64.0,
                        Align2::LEFT_BOTTOM,
                        self.tr(Text::LblNoRack),
                        14.0,
                        col(t.danger),
                    );
                }
                // the usable area, which can be bigger than the published one
                self.stepper(
                    cv,
                    ty + 80.0,
                    self.tr(Text::TableWidth),
                    &format!("{} {}", self.len_text(self.table.width_mm), self.len_unit()),
                    Field::TableW,
                    [-10.0, -1.0, 1.0, 10.0],
                    true,
                );
                self.stepper(
                    cv,
                    ty + 138.0,
                    self.tr(Text::TableDepth),
                    &format!("{} {}", self.len_text(self.table.depth_mm), self.len_unit()),
                    Field::TableD,
                    [-10.0, -1.0, 1.0, 10.0],
                    true,
                );
                // the unit of the coordinates of the files, as the classic programs have it
                let fy = ty + 232.0;
                cv.text(
                    x + 30.0,
                    fy,
                    Align2::LEFT_BOTTOM,
                    &self.tr(Text::LblFileCoords).to_uppercase(),
                    12.0,
                    col(t.dim),
                );
                let cur = self.file_unit();
                for (i, n) in ["0.1 mil", "1 mil", "0.025 mm", "0.01 mm"]
                    .into_iter()
                    .enumerate()
                {
                    cv.chip(
                        x + 30.0 + i as f32 * 196.0,
                        fy + 12.0,
                        188.0,
                        38.0,
                        n,
                        cur == Some(i as u8),
                        Act::FileUnit(i as u8),
                    );
                }
                cv.text(
                    x + 30.0,
                    fy + 76.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{}: {}", self.tr(Text::Connection), self.connection),
                    15.0,
                    col(t.dim),
                );
            }
            _ => {
                cv.checkbox(
                    x + 30.0,
                    top + 10.0,
                    780.0,
                    self.tr(Text::SetAskOperator),
                    self.settings.ask_operator,
                    Act::ToggleAsk,
                );
                cv.checkbox(
                    x + 30.0,
                    top + 58.0,
                    780.0,
                    self.tr(Text::SetSkipLauncher),
                    self.settings.choice == Some(Choice::Modern),
                    Act::ToggleSkip,
                );
                cv.checkbox(
                    x + 30.0,
                    top + 106.0,
                    780.0,
                    self.tr(Text::EmergencyShow),
                    self.settings.emergency,
                    Act::ToggleEmergency,
                );
                // the sounds: each one on or off, with the file of the operator's or the default tone
                for (i, (cue, label)) in [
                    (Cue::Finished, Text::SoundFinish),
                    (Cue::ToolChange, Text::SoundTool),
                ]
                .into_iter()
                .enumerate()
                {
                    let on = match cue {
                        Cue::Finished => self.conn.sound_finish.is_on(),
                        Cue::ToolChange => self.conn.sound_tool_change.is_on(),
                    };
                    let yy = top + 154.0 + i as f32 * 96.0;
                    cv.checkbox(
                        x + 30.0,
                        yy,
                        600.0,
                        self.tr(label),
                        on,
                        Act::ToggleSound(cue),
                    );
                    if on {
                        cv.chip(
                            x + 650.0,
                            yy + 50.0,
                            160.0,
                            36.0,
                            self.tr(Text::SoundTest),
                            false,
                            Act::SoundTest(cue),
                        );
                    }
                }
                cv.text(
                    x + 30.0,
                    top + 372.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{}: {}", self.tr(Text::BoardTitle), self.board.describe()),
                    15.0,
                    col(t.dim),
                );
                cv.text(
                    x + 30.0,
                    top + 398.0,
                    Align2::LEFT_BOTTOM,
                    &format!(
                        "{} - {}",
                        crate::launcher::settings_path().display(),
                        env!("CARGO_PKG_VERSION")
                    ),
                    12.0,
                    col(t.dim),
                );
            }
        }
        cv.text(
            x + 30.0,
            y + h - 90.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::TypeValueHint),
            13.0,
            col(t.dim),
        );
        if self.saved {
            cv.text(
                x + 30.0,
                y + h - 30.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::Saved),
                18.0,
                col(t.ok),
            );
        }
        cv.chip(
            x + 440.0,
            y + h - 66.0,
            180.0,
            44.0,
            self.tr(Text::Save),
            false,
            Act::SaveSettings,
        );
        cv.chip(
            x + 630.0,
            y + h - 66.0,
            180.0,
            44.0,
            self.tr(Text::Close),
            true,
            Act::CloseModal,
        );
    }

    /// The model of the machine in use.
    pub(crate) fn model(&self) -> usize {
        crate::machines::Model::from_name(&self.settings.model).unwrap_or(0)
    }

    /// The unit of the coordinates of the files in the machine file (0.1 mil, mil, plotter step, 0.01 mm), as an index.
    pub(crate) fn file_unit(&self) -> Option<u8> {
        self.drill.engine.settings().map(|s| s.unit.index() as u8)
    }

    /// Takes a model: the usable area becomes the published one, the tool change is the manual or the automatic one, and the choice is
    /// remembered. The rack positions are the machine file's own.
    pub(crate) fn apply_model(&mut self, i: usize) {
        let models = crate::machines::models();
        let Some(m) = models.get(i).copied() else {
            return;
        };
        let (w, d) = m.profile().work_mm_xy();
        let automatic = m.change == crate::machines::ToolChange::Automatic;
        for (k, side) in [&mut self.drill, &mut self.route].into_iter().enumerate() {
            if let Some(s) = side.engine.settings_mut() {
                s.clip_mm = (w, d);
                // only the drilling program has the tool changer
                if k == 0 {
                    s.auto_change = automatic;
                }
            }
            side.engine.settings_changed();
        }
        self.settings.model = m.name();
        self.table = Table::new(w, d);
        self.drill.scene.table = self.table;
        self.route.scene.table = self.table;
        // both views show the new table from the start: the zoom and the shift of the old machine would not fit it
        self.view2d = Default::default();
        self.view3d = Default::default();
        self.saved = false;
        self.apply_panel();
        self.remember_settings();
    }

    /// Changes how the coordinates of the files are read (as the Parameter window of the classic programs does) and reads the files again.
    pub(crate) fn set_file_unit(&mut self, i: u8) {
        let Some(unit) = eh3d_drillpro::config::CoordUnit::from_index(i as u32) else {
            return;
        };
        for side in [&mut self.drill, &mut self.route] {
            if let Some(s) = side.engine.settings_mut() {
                s.unit = unit;
            }
            side.engine.settings_changed();
        }
        self.saved = false;
        self.reload_jobs();
    }

    /// Reads the files of the job again (after a change of the unit of the coordinates).
    pub(crate) fn reload_jobs(&mut self) {
        if let Some(p) = self.drill_path.clone() {
            self.load_drill(&p);
        }
        if let Some(p) = self.route_path.clone() {
            self.load_route(&p);
        }
        self.apply_panel();
    }

    /// A length as it is shown: millimetres with a decimal, or mils (0.001 inch) whole.
    pub(crate) fn len_text(&self, mm: f64) -> String {
        if self.settings.display_mil {
            format!("{:.0}", mm / 0.0254)
        } else {
            format!("{mm:.1}")
        }
    }

    pub(crate) fn len_unit(&self) -> &'static str {
        if self.settings.display_mil {
            "mil"
        } else {
            "mm"
        }
    }

    /// A typed length in the units on the screen, in millimetres.
    pub(crate) fn len_from_display(&self, v: f64) -> f64 {
        if self.settings.display_mil {
            v * 0.0254
        } else {
            v
        }
    }

    pub(crate) fn len_to_display(&self, mm: f64) -> f64 {
        if self.settings.display_mil {
            mm / 0.0254
        } else {
            mm
        }
    }
}
