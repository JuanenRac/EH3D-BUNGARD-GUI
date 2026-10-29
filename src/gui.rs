//! The window of the modern interface: the launcher and the viewer, drawn on one canvas with the colours of the theme.
//!
//! The machine is driven by the engines of the classic programs (`ClassicEngine` of DRILLPRO and of ROUTEPRO) with the same machine
//! file, so the computer's RS-232, a USB-RS232 adapter and the EH3D node (directly or through the EH3D server) work as they do there.
//! What the views show of a running job comes from the engine: how many holes are done and where the head is.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use eframe::egui::{
    self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, Vec2,
};
use eh3d_drillpro::classic::engine::{ClassicEngine, Question};
use eh3d_drillpro::job::JobOptions;
use eh3d_view::scene::{Iso, Scene, Table};
use eh3d_view::texts::{tr, Lang, Text};
use eh3d_view::theme::{Corner, Rgb, Theme, ThemeId};

use crate::cli::{Options, Screen};
use crate::jobs;
use crate::launcher::{self, Choice, Machine, Program, Settings};
use crate::machines;

/// The design size of the canvas; everything is laid out in these units and scaled to the window.
const DESIGN_W: f32 = 1600.0;
const DESIGN_H: f32 = 900.0;

fn col(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn shade(c: Color32, k: f32) -> Color32 {
    let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

/// The classic engines speak the language type of DRILLPRO.
fn drill_lang(l: Lang) -> eh3d_drillpro::i18n::Lang {
    eh3d_drillpro::i18n::Lang::from_code(l.code()).unwrap_or(eh3d_drillpro::i18n::Lang::En)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Act {
    Choose(Choice),
    Remember,
    NextTheme,
    NextLang,
    Layer(usize),
    Start,
    Stop,
    Back,
    Answer(bool),
}

/// One of the two programs inside the window: its engine and what the views show of its file.
struct Side {
    engine: Box<dyn ClassicEngine>,
    scene: Scene,
    last_done: usize,
    /// Where the work starts on the table (the offset of the configuration), added to every point of the file.
    offset: (f64, f64),
}

impl Side {
    fn new(engine: Box<dyn ClassicEngine>) -> Side {
        Side {
            scene: Scene::new(jobs::DEFAULT_TABLE),
            engine,
            last_done: 0,
            offset: (0.0, 0.0),
        }
    }

    fn table(&self) -> Table {
        self.engine
            .settings()
            .filter(|s| s.clip_mm.0 > 0.0 && s.clip_mm.1 > 0.0)
            .map_or(jobs::DEFAULT_TABLE, |s| {
                Table::new(s.clip_mm.0, s.clip_mm.1)
            })
    }
}

struct Shot {
    file: PathBuf,
    wait: u32,
    done: Arc<AtomicBool>,
}

pub struct App {
    settings: Settings,
    settings_path: PathBuf,
    lang: Lang,
    theme: Theme,
    screen: Screen,
    drill: Side,
    route: Side,
    table: Table,
    layers: [bool; 4],
    remember: bool,
    running: Option<Program>,
    message: Option<String>,
    connection: String,
    hits: Vec<(Rect, Act)>,
    frame: u32,
    start_at_frame: Option<u32>,
    shot: Option<Shot>,
    close: bool,
    auto_ok: bool,
    base_dir: PathBuf,
}

impl App {
    pub fn new(o: &Options) -> App {
        let settings_path = launcher::settings_path();
        let settings = Settings::load(&settings_path);
        let lang = o.lang.unwrap_or(settings.lang);
        let theme_id = o.theme.unwrap_or(settings.theme);
        let dlang = drill_lang(lang);
        let data = |file: &str| {
            o.data
                .clone()
                .unwrap_or_else(|| eh3d_drillpro::config::find_data_dir(file))
        };
        let mut drill = eh3d_drillpro::app::App::new(dlang, &data("DRILLPRO.DCF"));
        let mut route = eh3d_routepro::app::App::new(dlang, &data("ROUTEPRO.RCF"));
        let mut connection = String::from("demo");
        match eh3d_drillpro::protocol::connection::load(o.connection.as_deref()) {
            Ok(c) => {
                connection = c.describe();
                drill.set_connection(c.clone());
                route.set_connection(c);
            }
            Err(e) => {
                drill.set_error(e.clone());
                route.set_error(e);
            }
        }
        let mut app = App {
            screen: o.screen.unwrap_or(match settings.choice {
                Some(Choice::Modern) => Screen::Viewer,
                _ => Screen::Launcher,
            }),
            settings,
            settings_path,
            lang,
            theme: Theme::for_id(theme_id),
            drill: Side::new(Box::new(drill)),
            route: Side::new(Box::new(route)),
            table: jobs::DEFAULT_TABLE,
            layers: [true; 4],
            remember: false,
            running: None,
            message: None,
            connection,
            hits: Vec::new(),
            frame: 0,
            start_at_frame: o.start.then_some(4),
            shot: o.shot.clone().map(|file| Shot {
                file,
                wait: o.wait,
                done: Arc::new(AtomicBool::new(false)),
            }),
            close: false,
            auto_ok: o.auto_ok,
            base_dir: std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(Path::to_path_buf))
                .unwrap_or_else(|| PathBuf::from(".")),
        };
        if let Some(f) = &o.drill {
            app.load_drill(f);
        }
        if let Some(f) = &o.route {
            app.load_route(f);
        }
        app.refit_table();
        app
    }

    fn tr(&self, t: Text) -> &'static str {
        tr(self.lang, t)
    }

    fn refit_table(&mut self) {
        self.table = self.drill.table();
        self.drill.scene.table = self.table;
        self.route.scene.table = self.table;
    }

    fn load_drill(&mut self, path: &Path) {
        self.drill.engine.load(path);
        let file = jobs::read_latin1(path)
            .ok()
            .and_then(|t| eh3d_drillpro::ncd::parse(&t).ok());
        let (unit, offset) = self
            .drill
            .engine
            .settings()
            .map_or((0.0254, (0.0, 0.0)), |s| (s.unit.mm(), s.offset_mm));
        match file {
            Some(f) => {
                self.drill.scene = jobs::scene_from_drill(&f, self.table, unit);
                self.drill.offset = offset;
                for h in &mut self.drill.scene.holes {
                    h.x_mm += offset.0;
                    h.y_mm += offset.1;
                }
                self.message = None;
            }
            None => self.message = Some(self.drill.engine.error().unwrap_or("?").to_string()),
        }
        self.drill.last_done = 0;
    }

    fn load_route(&mut self, path: &Path) {
        self.route.engine.load(path);
        let plot = jobs::read_latin1(path)
            .ok()
            .map(|t| eh3d_routepro::hpgl::parse(&t));
        let offset = self
            .route
            .engine
            .settings()
            .map_or((0.0, 0.0), |s| s.offset_mm);
        if let Some(p) = plot {
            self.route.scene = jobs::scene_from_plot(&p, self.table);
            self.route.offset = offset;
            for path in &mut self.route.scene.paths {
                for pt in &mut path.points {
                    pt.0 += offset.0;
                    pt.1 += offset.1;
                }
            }
        }
        self.route.last_done = 0;
    }

    fn remember_settings(&mut self) {
        self.settings.lang = self.lang;
        self.settings.theme = self.theme.id;
        let _ = self.settings.save(&self.settings_path);
    }

    /// The sizes of the holes of both files, from the smallest.
    fn sizes(&self) -> Vec<f64> {
        let mut all = self.drill.scene.clone();
        all.overlay(&self.route.scene);
        all.sizes()
    }

    fn active(&mut self) -> Option<(Program, &mut Side)> {
        if self.drill.engine.file_loaded() {
            Some((Program::Drill, &mut self.drill))
        } else if self.route.engine.file_loaded() {
            Some((Program::Route, &mut self.route))
        } else {
            None
        }
    }

    fn start_job(&mut self) {
        if let Some((p, side)) = self.active() {
            side.engine.set_job_options(JobOptions {
                ask_operator: true,
                ..Default::default()
            });
            side.engine.start();
            side.last_done = 0;
            for h in &mut side.scene.holes {
                h.done = false;
            }
            for pth in &mut side.scene.paths {
                pth.reached = 0;
            }
            self.running = Some(p);
        }
    }

    fn stop_job(&mut self) {
        if let Some(p) = self.running.take() {
            self.side_mut(p).engine.stop();
        }
    }

    fn side_mut(&mut self, p: Program) -> &mut Side {
        match p {
            Program::Drill => &mut self.drill,
            Program::Route => &mut self.route,
        }
    }

    /// Advances the engine and takes from it what the views show.
    fn tick(&mut self) {
        let Some(p) = self.running else { return };
        let auto_ok = self.auto_ok;
        let side = self.side_mut(p);
        if side.engine.question().is_some() {
            if auto_ok {
                side.engine.answer(true);
            }
            return;
        }
        let more = side.engine.tick(1);
        let done = side.engine.done();
        let view = side.engine.job_view();
        let head = view.as_ref().map(|v| (v.x_mm, v.y_mm));
        let percent = view.as_ref().map_or(0, |v| v.percent);
        if p == Program::Drill {
            // the hole that was drilled is the one nearest to the head (the engine drills them in its own order)
            while side.last_done < done {
                let (hx, hy) = head.unwrap_or((0.0, 0.0));
                let best = side
                    .scene
                    .holes
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| !h.done)
                    .min_by(|a, b| {
                        let da = (a.1.x_mm - hx).hypot(a.1.y_mm - hy);
                        let db = (b.1.x_mm - hx).hypot(b.1.y_mm - hy);
                        da.total_cmp(&db)
                    })
                    .map(|(i, _)| i);
                if let Some(i) = best {
                    side.scene.holes[i].done = true;
                }
                side.last_done += 1;
            }
        } else {
            let total: usize = side.scene.paths.iter().map(|q| q.points.len()).sum();
            let mut left = total * percent as usize / 100;
            for q in &mut side.scene.paths {
                q.reached = left.min(q.points.len());
                left = left.saturating_sub(q.points.len());
            }
        }
        if !more {
            if p == Program::Drill {
                for h in &mut side.scene.holes {
                    h.done = true;
                }
            } else {
                for q in &mut side.scene.paths {
                    q.reached = q.points.len();
                }
            }
            self.running = None;
        }
    }

    fn choose(&mut self, c: Choice) {
        if self.remember {
            self.settings.choice = Some(c);
            self.remember_settings();
        }
        match c {
            Choice::Modern => self.screen = Screen::Viewer,
            classic => match launcher::classic_launch(classic, &self.base_dir) {
                Some(l) => match std::process::Command::new(&l.exe).args(&l.args).spawn() {
                    Ok(_) => self.close = true,
                    Err(e) => self.message = Some(format!("{}: {e}", l.exe.display())),
                },
                None => self.message = Some(self.tr(Text::ClassicMissing).to_string()),
            },
        }
    }

    fn act(&mut self, a: Act) {
        match a {
            Act::Choose(c) => self.choose(c),
            Act::Remember => self.remember = !self.remember,
            Act::NextTheme => {
                let i = ThemeId::ALL
                    .iter()
                    .position(|t| *t == self.theme.id)
                    .unwrap_or(0);
                self.theme = Theme::for_id(ThemeId::ALL[(i + 1) % ThemeId::ALL.len()]);
                self.remember_settings();
            }
            Act::NextLang => {
                let i = Lang::ALL.iter().position(|l| *l == self.lang).unwrap_or(0);
                self.lang = Lang::ALL[(i + 1) % Lang::ALL.len()];
                let dl = drill_lang(self.lang);
                self.drill.engine.set_lang(dl);
                self.route.engine.set_lang(dl);
                self.remember_settings();
            }
            Act::Layer(i) => self.layers[i] = !self.layers[i],
            Act::Start => self.start_job(),
            Act::Stop => self.stop_job(),
            Act::Back => {
                self.stop_job();
                self.screen = Screen::Launcher;
            }
            Act::Answer(ok) => {
                if let Some(p) = self.running {
                    self.side_mut(p).engine.answer(ok);
                    if !ok {
                        self.running = None;
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------------------------------------ drawing

/// A canvas of the design size scaled into the window: every coordinate is in design units.
struct Cv<'a> {
    p: &'a egui::Painter,
    o: Pos2,
    u: f32,
    t: &'a Theme,
    hits: Vec<(Rect, Act)>,
}

impl<'a> Cv<'a> {
    fn pt(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.o.x + x * self.u, self.o.y + y * self.u)
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.pt(x, y), Vec2::new(w * self.u, h * self.u))
    }

    fn text(&self, x: f32, y: f32, al: Align2, s: &str, size: f32, c: Color32) -> Rect {
        self.p
            .text(self.pt(x, y), al, s, FontId::proportional(size * self.u), c)
    }

    fn line(&self, a: (f32, f32), b: (f32, f32), w: f32, c: Color32) {
        self.p.line_segment(
            [self.pt(a.0, a.1), self.pt(b.0, b.1)],
            Stroke::new(w * self.u, c),
        );
    }

    fn poly(&self, pts: &[(f32, f32)], fill: Color32, stroke: Stroke) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::convex_polygon(v, fill, stroke));
    }

    fn polyline(&self, pts: &[(f32, f32)], w: f32, c: Color32, dashed: bool) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        if v.len() < 2 {
            return;
        }
        if dashed {
            self.p.extend(Shape::dashed_line(
                &v,
                Stroke::new(w * self.u, c),
                7.0 * self.u,
                5.0 * self.u,
            ));
        } else {
            self.p.add(Shape::line(v, Stroke::new(w * self.u, c)));
        }
    }

    /// A card in the corner style of the theme.
    fn card(&mut self, x: f32, y: f32, w: f32, h: f32, hot: bool, act: Option<Act>) {
        let t = self.t;
        let fill = col(t.panel);
        let line = if hot {
            col(t.line)
        } else {
            alpha(Color32::WHITE, 34)
        };
        let stroke = Stroke::new(if hot { 1.6 } else { 1.0 } * self.u, line);
        match t.corner {
            Corner::Cut => {
                let c = 16.0;
                self.poly(
                    &[
                        (x + c, y),
                        (x + w, y),
                        (x + w, y + h - c),
                        (x + w - c, y + h),
                        (x, y + h),
                        (x, y + c),
                    ],
                    fill,
                    stroke,
                );
            }
            Corner::Round | Corner::Square => {
                let r = if t.corner == Corner::Round { 14.0 } else { 0.0 } * self.u;
                let rect = self.rect(x, y, w, h);
                self.p.rect_filled(rect, r, fill);
                self.p.rect_stroke(rect, r, stroke, StrokeKind::Inside);
            }
        }
        if let Some(a) = act {
            self.hits.push((self.rect(x, y, w, h), a));
        }
    }

    fn title(&self, x: f32, y: f32, s: &str) {
        self.text(
            x,
            y,
            Align2::LEFT_BOTTOM,
            &s.to_uppercase(),
            14.0,
            col(self.t.accent),
        );
    }

    /// A button: filled with a colour, a label in the middle.
    #[allow(clippy::too_many_arguments)]
    fn button(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str, c: Color32, act: Act) {
        let rect = self.rect(x, y, w, h);
        let ink = if self.t.corner == Corner::Square {
            Color32::BLACK
        } else {
            col(self.t.background.0)
        };
        if self.t.corner == Corner::Cut {
            self.poly(
                &[
                    (x + 12.0, y),
                    (x + w, y),
                    (x + w, y + h - 12.0),
                    (x + w - 12.0, y + h),
                    (x, y + h),
                    (x, y + 12.0),
                ],
                c,
                Stroke::NONE,
            );
        } else {
            let r = if self.t.corner == Corner::Round {
                12.0
            } else {
                0.0
            } * self.u;
            self.p.rect_filled(rect, r, c);
        }
        self.text(
            x + w / 2.0,
            y + h / 2.0 + 6.0,
            Align2::CENTER_BOTTOM,
            &label.to_uppercase(),
            17.0,
            ink,
        );
        self.hits.push((rect, act));
    }
}

fn draw_boxes(
    cv: &Cv,
    iso: &Iso,
    o: (f32, f32),
    shift: (f64, f64),
    faces: &[(f64, f64, f64, f64, f64, f64, Color32)],
    stroke: Color32,
) {
    for &(x, y, z, w, d, h, top) in faces {
        let p = |a: f64, b: f64, c: f64| {
            let (px, py) = iso.project(a + shift.0, b + shift.1, c);
            (o.0 + px as f32, o.1 + py as f32)
        };
        let sk = Stroke::new(1.0 * cv.u, stroke);
        cv.poly(
            &[
                p(x, y + d, z),
                p(x + w, y + d, z),
                p(x + w, y + d, z + h),
                p(x, y + d, z + h),
            ],
            shade(top, 0.55),
            sk,
        );
        cv.poly(
            &[
                p(x + w, y, z),
                p(x + w, y + d, z),
                p(x + w, y + d, z + h),
                p(x + w, y, z + h),
            ],
            shade(top, 0.38),
            sk,
        );
        cv.poly(
            &[
                p(x, y, z + h),
                p(x + w, y, z + h),
                p(x + w, y + d, z + h),
                p(x, y + d, z + h),
            ],
            top,
            sk,
        );
    }
}

impl App {
    fn paint(&mut self, p: &egui::Painter, rect: Rect) {
        let u = (rect.width() / DESIGN_W).min(rect.height() / DESIGN_H);
        // the design area is centred; the background covers the whole window
        let o = Pos2::new(
            rect.min.x + (rect.width() - DESIGN_W * u) / 2.0,
            rect.min.y + (rect.height() - DESIGN_H * u) / 2.0,
        );
        let t = self.theme.clone();
        p.rect_filled(rect, 0.0, col(t.background.0));
        let strip = Rect::from_min_max(
            Pos2::new(rect.min.x, o.y),
            Pos2::new(rect.max.x, o.y + DESIGN_H * u),
        );
        p.rect_filled(strip, 0.0, col(t.background.1));
        let mut cv = Cv {
            p,
            o,
            u,
            t: &t,
            hits: Vec::new(),
        };
        match self.screen {
            Screen::Launcher => self.paint_launcher(&mut cv),
            Screen::Viewer => self.paint_viewer(&mut cv),
        }
        if let Some(m) = &self.message {
            cv.text(
                DESIGN_W / 2.0,
                DESIGN_H - 24.0,
                Align2::CENTER_BOTTOM,
                m,
                16.0,
                col(t.danger),
            );
        }
        self.hits = std::mem::take(&mut cv.hits);
    }

    fn top_bar(&self, cv: &mut Cv, back: bool) {
        let t = cv.t.clone();
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::new(0.0, cv.o.y), Vec2::new(10_000.0, 64.0 * cv.u)),
            0.0,
            alpha(col(t.panel), 235),
        );
        cv.line((0.0, 64.0), (DESIGN_W, 64.0), 1.0, alpha(col(t.line), 150));
        let x = if back { 70.0 } else { 28.0 };
        if back {
            cv.text(28.0, 42.0, Align2::LEFT_BOTTOM, "<", 28.0, col(t.dim));
            cv.hits.push((cv.rect(14.0, 10.0, 48.0, 44.0), Act::Back));
        }
        cv.text(x, 42.0, Align2::LEFT_BOTTOM, "EH3D", 26.0, col(t.text));
        cv.text(
            x + 78.0,
            42.0,
            Align2::LEFT_BOTTOM,
            "BUNGARD",
            26.0,
            col(t.accent),
        );
        // theme and language: a click takes the next one
        let name = match t.id {
            ThemeId::Forge => Text::ThemeForge,
            ThemeId::Neon => Text::ThemeNeon,
            ThemeId::Glass => Text::ThemeGlass,
            ThemeId::WindowsClassic => Text::ThemeWindowsClassic,
            ThemeId::System => Text::ThemeSystem,
        };
        let theme_label = format!("{}: {}", self.tr(Text::ThemeLabel), self.tr(name));
        let r = cv.text(
            DESIGN_W - 28.0,
            28.0,
            Align2::RIGHT_BOTTOM,
            &theme_label,
            14.0,
            col(t.text),
        );
        cv.hits.push((r.expand(6.0), Act::NextTheme));
        let lang_label = format!(
            "{}: {}",
            self.tr(Text::Language),
            self.lang.code().to_uppercase()
        );
        let r = cv.text(
            DESIGN_W - 28.0,
            50.0,
            Align2::RIGHT_BOTTOM,
            &lang_label,
            14.0,
            col(t.dim),
        );
        cv.hits.push((r.expand(6.0), Act::NextLang));
    }

    fn paint_launcher(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        self.top_bar(cv, false);
        cv.text(
            DESIGN_W / 2.0,
            160.0,
            Align2::CENTER_BOTTOM,
            "EH3D-BUNGARD",
            44.0,
            col(t.text),
        );
        cv.text(
            DESIGN_W / 2.0,
            200.0,
            Align2::CENTER_BOTTOM,
            &self.tr(Text::LauncherAsk).to_uppercase(),
            16.0,
            col(t.dim),
        );
        // the modern interface: the big card
        let (mx, my, mw, mh) = (150.0, 250.0, 560.0, 430.0);
        cv.card(mx, my, mw, mh, true, Some(Act::Choose(Choice::Modern)));
        cv.text(
            mx + mw / 2.0,
            my + 70.0,
            Align2::CENTER_BOTTOM,
            "MODERN",
            16.0,
            col(t.accent),
        );
        cv.text(
            mx + mw / 2.0,
            my + 130.0,
            Align2::CENTER_BOTTOM,
            "MTC + ATC",
            48.0,
            col(t.text),
        );
        // a small preview of the viewer
        cv.p.rect_filled(
            cv.rect(mx + 50.0, my + 170.0, mw - 100.0, 170.0),
            6.0 * cv.u,
            col(t.table),
        );
        for i in 0..24 {
            let (cx, cy) = (
                mx + 80.0 + (i % 12) as f32 * 36.0,
                my + 210.0 + (i / 12) as f32 * 40.0,
            );
            cv.p.circle_filled(
                cv.pt(cx, cy),
                (3.0 + (i % 4) as f32 * 1.4) * cv.u,
                col(t.size_color(i % 4)),
            );
        }
        cv.polyline(
            &[
                (mx + 60.0, my + 320.0),
                (mx + 260.0, my + 250.0),
                (mx + 500.0, my + 322.0),
            ],
            2.0,
            col(t.accent2),
            true,
        );
        cv.text(
            mx + mw / 2.0,
            my + mh - 40.0,
            Align2::CENTER_BOTTOM,
            &self.tr(Text::PressEnter).to_uppercase(),
            14.0,
            col(t.ok),
        );
        // the classic programs: four small cards
        let kinds = [
            Choice::Classic(Program::Drill, Machine::Mtc),
            Choice::Classic(Program::Drill, Machine::Atc),
            Choice::Classic(Program::Route, Machine::Mtc),
            Choice::Classic(Program::Route, Machine::Atc),
        ];
        for (i, c) in kinds.into_iter().enumerate() {
            let (x, y) = (
                760.0 + (i % 2) as f32 * 360.0,
                250.0 + (i / 2) as f32 * 220.0,
            );
            cv.card(x, y, 330.0, 200.0, false, Some(Act::Choose(c)));
            let Choice::Classic(prog, mach) = c else {
                continue;
            };
            let program = self.tr(if prog == Program::Drill {
                Text::Drilling
            } else {
                Text::Routing
            });
            let machine = if mach == Machine::Mtc { "MTC" } else { "ATC" };
            cv.text(
                x + 165.0,
                y + 56.0,
                Align2::CENTER_BOTTOM,
                &self
                    .tr(Text::LauncherClassicMtc)
                    .split(' ')
                    .next()
                    .unwrap_or("")
                    .to_uppercase(),
                14.0,
                col(t.dim),
            );
            cv.text(
                x + 165.0,
                y + 112.0,
                Align2::CENTER_BOTTOM,
                machine,
                40.0,
                col(t.text),
            );
            cv.text(
                x + 165.0,
                y + 160.0,
                Align2::CENTER_BOTTOM,
                program,
                18.0,
                col(t.dim),
            );
        }
        // remember
        let on = self.remember;
        let (bx, by) = (DESIGN_W / 2.0 - 140.0, 730.0);
        let box_rect = cv.rect(bx, by, 22.0, 22.0);
        cv.p.rect_filled(
            box_rect,
            4.0 * cv.u,
            if on {
                col(t.accent)
            } else {
                Color32::TRANSPARENT
            },
        );
        cv.p.rect_stroke(
            box_rect,
            4.0 * cv.u,
            Stroke::new(1.5 * cv.u, col(t.accent)),
            StrokeKind::Inside,
        );
        let r = cv.text(
            bx + 34.0,
            by + 17.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::Remember),
            16.0,
            col(t.dim),
        );
        cv.hits.push((
            Rect::from_min_max(box_rect.min, r.max).expand(6.0),
            Act::Remember,
        ));
    }

    fn paint_viewer(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        self.top_bar(cv, true);
        let sizes = self.sizes();
        let right_w = (DESIGN_W * 0.38).clamp(420.0, 680.0);
        let mid_w = DESIGN_W - 316.0 - 20.0 - right_w - 16.0;
        let (top, hud_y) = (84.0, DESIGN_H - 196.0);
        let card_h = hud_y - top - 18.0;

        // ---- left: the job
        cv.card(20.0, top, 280.0, card_h, true, None);
        cv.title(38.0, top + 32.0, "JOB");
        let name = self
            .drill
            .engine
            .file_name()
            .or_else(|| self.route.engine.file_name())
            .unwrap_or_else(|| self.tr(Text::NoFile).to_string());
        cv.text(
            38.0,
            top + 70.0,
            Align2::LEFT_BOTTOM,
            &name,
            19.0,
            col(t.text),
        );
        let holes = self.drill.scene.holes.len();
        cv.text(
            38.0,
            top + 94.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{holes} {} - {} {}",
                self.tr(Text::Holes),
                sizes.len(),
                self.tr(Text::Tools)
            ),
            13.0,
            col(t.dim),
        );
        let mut all = self.drill.scene.clone();
        all.overlay(&self.route.scene);
        let mut y = top + 116.0;
        for (i, d) in sizes.iter().enumerate().take(6) {
            let (n, done) = all.size_count(*d);
            let c = col(t.size_color(i));
            cv.p.rect_filled(
                cv.rect(38.0, y, 244.0, 60.0),
                6.0 * cv.u,
                col(t.panel_raised),
            );
            cv.p.circle_filled(cv.pt(66.0, y + 30.0), (5.0 + *d as f32 * 3.5) * cv.u, c);
            cv.text(
                98.0,
                y + 26.0,
                Align2::LEFT_BOTTOM,
                &format!("{d:.1} mm"),
                17.0,
                col(t.text),
            );
            cv.text(
                98.0,
                y + 47.0,
                Align2::LEFT_BOTTOM,
                &format!("{n} {}", self.tr(Text::Holes)),
                13.0,
                col(t.dim),
            );
            cv.p.rect_filled(
                cv.rect(186.0, y + 38.0, 84.0, 6.0),
                3.0 * cv.u,
                alpha(Color32::WHITE, 26),
            );
            if n > 0 {
                cv.p.rect_filled(
                    cv.rect(186.0, y + 38.0, 84.0 * done as f32 / n as f32, 6.0),
                    3.0 * cv.u,
                    c,
                );
            }
            y += 68.0;
        }
        y += 14.0;
        cv.text(38.0, y, Align2::LEFT_BOTTOM, "LAYERS", 12.0, col(t.dim));
        let layer_names = [
            Text::LayerHoles,
            Text::LayerPaths,
            Text::LayerMachine,
            Text::LayerGrid,
        ];
        for (i, n) in layer_names.into_iter().enumerate() {
            let ly = y + 12.0 + i as f32 * 26.0;
            let r = cv.rect(38.0, ly, 16.0, 16.0);
            cv.p.rect_filled(
                r,
                4.0 * cv.u,
                if self.layers[i] {
                    col(t.accent)
                } else {
                    Color32::TRANSPARENT
                },
            );
            cv.p.rect_stroke(
                r,
                4.0 * cv.u,
                Stroke::new(1.2 * cv.u, col(t.accent)),
                StrokeKind::Inside,
            );
            cv.text(
                64.0,
                ly + 13.0,
                Align2::LEFT_BOTTOM,
                self.tr(n),
                14.0,
                col(t.text),
            );
            cv.hits
                .push((cv.rect(38.0, ly - 4.0, 200.0, 24.0), Act::Layer(i)));
        }

        // ---- centre: the plan from above
        let (cx, cw) = (316.0, mid_w);
        cv.card(cx, top, cw, card_h, true, None);
        cv.title(cx + 18.0, top + 32.0, self.tr(Text::ViewPlan));
        let (vx, vy, vw, vh) = (cx + 14.0, top + 48.0, cw - 28.0, card_h - 96.0);
        let plan = Scene::new(self.table).view2d(vw as f64, vh as f64, 10.0);
        let screen = |x: f64, y: f64| {
            let (px, py) = plan.to_screen(&self.table, x, y);
            (vx + px as f32, vy + py as f32)
        };
        let (tx0, ty0) = screen(0.0, self.table.depth_mm);
        let (tx1, ty1) = screen(self.table.width_mm, 0.0);
        cv.p.rect_filled(
            cv.rect(tx0, ty0, tx1 - tx0, ty1 - ty0),
            4.0 * cv.u,
            col(t.table),
        );
        if self.layers[3] {
            let mut gx = 0.0;
            while gx <= self.table.width_mm {
                let (a, _) = screen(gx, 0.0);
                cv.line(
                    (a, ty0),
                    (a, ty1),
                    if gx % 100.0 == 0.0 { 1.4 } else { 0.6 },
                    col(t.grid),
                );
                gx += 25.0;
            }
            let mut gy = 0.0;
            while gy <= self.table.depth_mm {
                let (_, b) = screen(0.0, gy);
                cv.line(
                    (tx0, b),
                    (tx1, b),
                    if gy % 100.0 == 0.0 { 1.4 } else { 0.6 },
                    col(t.grid),
                );
                gy += 25.0;
            }
        }
        cv.p.rect_stroke(
            cv.rect(tx0, ty0, tx1 - tx0, ty1 - ty0),
            4.0 * cv.u,
            Stroke::new(1.2 * cv.u, alpha(col(t.line), 140)),
            StrokeKind::Inside,
        );
        if let Some((x0, y0, x1, y1)) = all.bounds() {
            let (a, b) = screen(x0 - 3.0, y1 + 3.0);
            let (c, d) = screen(x1 + 3.0, y0 - 3.0);
            cv.p.rect_filled(
                cv.rect(a, b, c - a, d - b),
                4.0 * cv.u,
                alpha(col(t.board), 200),
            );
        }
        if self.layers[1] {
            for path in &self.route.scene.paths {
                let pts: Vec<(f32, f32)> = path.points.iter().map(|&(x, y)| screen(x, y)).collect();
                cv.polyline(&pts, 2.0, col(t.accent2), true);
                if path.reached > 1 {
                    cv.polyline(&pts[..path.reached.min(pts.len())], 2.4, col(t.ok), false);
                }
            }
        }
        if self.layers[0] {
            for h in &self.drill.scene.holes {
                let idx = sizes
                    .iter()
                    .position(|d| (*d - h.diameter_mm).abs() < 0.005)
                    .unwrap_or(0);
                let c = col(t.size_color(idx));
                let r = plan.hole_radius(h.diameter_mm, 2.0) as f32;
                let (px, py) = screen(h.x_mm, h.y_mm);
                if h.done {
                    cv.p.circle_filled(cv.pt(px, py), r * cv.u, c);
                } else {
                    cv.p.circle_stroke(
                        cv.pt(px, py),
                        r * cv.u,
                        Stroke::new(1.4 * cv.u, alpha(c, 220)),
                    );
                }
            }
        }
        let head = self.head();
        if let Some((hx, hy)) = head {
            let (px, py) = screen(hx, hy);
            cv.p.circle_stroke(
                cv.pt(px, py),
                13.0 * cv.u,
                Stroke::new(2.0 * cv.u, col(t.ok)),
            );
            cv.line((px - 22.0, py), (px + 22.0, py), 1.5, col(t.ok));
            cv.line((px, py - 22.0), (px, py + 22.0), 1.5, col(t.ok));
        }
        cv.text(
            cx + 18.0,
            top + card_h - 16.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{} {:.0} x {:.0} mm",
                self.tr(Text::TableFromConfig),
                self.table.width_mm,
                self.table.depth_mm
            ),
            13.0,
            col(t.dim),
        );

        // ---- right: the machine
        let rx = DESIGN_W - 20.0 - right_w;
        cv.card(rx, top, right_w, card_h, false, None);
        cv.title(rx + 18.0, top + 32.0, self.tr(Text::ViewMachine));
        if self.layers[2] {
            self.paint_machine(cv, rx, top + 44.0, right_w, card_h - 70.0, &sizes, head);
        }

        // ---- bottom: the numbers and the buttons
        cv.card(20.0, hud_y, DESIGN_W - 40.0, 176.0, false, None);
        let view = self.running_view();
        let vals = [
            (
                "X",
                format!("{:07.2}", view.as_ref().map_or(0.0, |v| v.x_mm)),
            ),
            (
                "Y",
                format!("{:07.2}", view.as_ref().map_or(0.0, |v| v.y_mm)),
            ),
            (
                "FEED Z",
                format!("{}", view.as_ref().map_or(0, |v| v.z_feed)),
            ),
            ("SPINDLE", format!("{}", view.as_ref().map_or(0, |v| v.rpm))),
            (
                "TOOL",
                view.as_ref()
                    .and_then(|v| v.tool)
                    .map_or("--".into(), |n| format!("T{n:02}")),
            ),
            (self.tr(Text::LayerMachine), self.connection.clone()),
        ];
        for (i, (lab, val)) in vals.iter().enumerate() {
            let x = 44.0 + i as f32 * 156.0;
            cv.text(
                x,
                hud_y + 42.0,
                Align2::LEFT_BOTTOM,
                &lab.to_uppercase(),
                12.0,
                col(t.dim),
            );
            cv.text(
                x,
                hud_y + 78.0,
                Align2::LEFT_BOTTOM,
                val,
                if i == 5 { 20.0 } else { 28.0 },
                col(t.text),
            );
        }
        let progress = self.progress();
        cv.text(
            44.0,
            hud_y + 118.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::Progress).to_uppercase(),
            12.0,
            col(t.dim),
        );
        cv.p.rect_filled(
            cv.rect(44.0, hud_y + 128.0, 900.0, 16.0),
            8.0 * cv.u,
            alpha(Color32::WHITE, 26),
        );
        cv.p.rect_filled(
            cv.rect(44.0, hud_y + 128.0, 900.0 * progress as f32, 16.0),
            8.0 * cv.u,
            col(t.ok),
        );
        cv.text(
            960.0,
            hud_y + 142.0,
            Align2::LEFT_BOTTOM,
            &format!("{} %", (progress * 100.0).round()),
            18.0,
            col(t.text),
        );
        cv.button(
            1250.0,
            hud_y + 28.0,
            150.0,
            86.0,
            self.tr(Text::Start),
            col(t.ok),
            Act::Start,
        );
        cv.button(
            1416.0,
            hud_y + 28.0,
            150.0,
            86.0,
            self.tr(Text::Stop),
            col(t.danger),
            Act::Stop,
        );
        cv.text(
            1250.0,
            hud_y + 152.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::SimulatedJob),
            12.0,
            col(t.dim),
        );

        // ---- a question of the machine for the operator
        if let Some(p) = self.running {
            let side = if p == Program::Drill {
                &self.drill
            } else {
                &self.route
            };
            if let Some(q) = side.engine.question() {
                let text = match q {
                    Question::SpindleStart => self.tr(Text::AskSpindle).to_string(),
                    Question::ToolChange(n) => format!("{} {n:02}", self.tr(Text::AskTool)),
                };
                cv.p.rect_filled(
                    Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10_000.0, 10_000.0)),
                    0.0,
                    alpha(Color32::BLACK, 150),
                );
                cv.card(500.0, 300.0, 600.0, 260.0, true, None);
                cv.text(
                    800.0,
                    390.0,
                    Align2::CENTER_BOTTOM,
                    &text,
                    28.0,
                    col(t.text),
                );
                cv.button(
                    560.0,
                    450.0,
                    220.0,
                    70.0,
                    self.tr(Text::Ok),
                    col(t.ok),
                    Act::Answer(true),
                );
                cv.button(
                    820.0,
                    450.0,
                    220.0,
                    70.0,
                    self.tr(Text::Abort),
                    col(t.danger),
                    Act::Answer(false),
                );
            }
        }
    }

    fn running_view(&self) -> Option<eh3d_drillpro::classic::engine::JobView> {
        let p = self.running?;
        if p == Program::Drill {
            self.drill.engine.job_view()
        } else {
            self.route.engine.job_view()
        }
    }

    /// Where the head is: the running job says it; otherwise the next hole to drill.
    fn head(&self) -> Option<(f64, f64)> {
        if let Some(v) = self.running_view() {
            return Some((v.x_mm, v.y_mm));
        }
        self.drill
            .scene
            .holes
            .iter()
            .find(|h| !h.done)
            .map(|h| (h.x_mm, h.y_mm))
    }

    fn progress(&self) -> f64 {
        let mut all = self.drill.scene.clone();
        all.overlay(&self.route.scene);
        all.progress()
    }

    /// The machine in 3D, built as the Bungard CCD machines are and as big as the one the table of the configuration belongs to: two
    /// aluminium rails with black end caps and adjustable feet, the white perforated plate between them, the gantry that runs along the
    /// rails (its motor at one end) with the head and the spindle, and, on the models with the tool changer, the rack at the short edge
    /// with the tools at the positions of the configuration. The published work area is drawn thin and the usable area of the
    /// configuration (which may be a few centimetres bigger or smaller) dashed; the frame grows when the usable area is bigger.
    #[allow(clippy::too_many_arguments)]
    fn paint_machine(
        &self,
        cv: &mut Cv,
        x0: f32,
        y0: f32,
        w: f32,
        h: f32,
        sizes: &[f64],
        head: Option<(f64, f64)>,
    ) {
        let t = cv.t.clone();
        let (tw, td) = (self.table.width_mm, self.table.depth_mm);
        let profile = machines::for_table(&self.table);
        let (wx, wy) = (profile.work_mm.0.max(tw), profile.work_mm.1.max(td));
        let (px_m, py_m) = profile.plate_margins((tw, td));
        // the view holds the whole frame: the plate, the rails, the motor and the rack
        let shift = (px_m + 60.0, py_m + 60.0);
        let iso = Iso::fit(
            &Table::new(wx + 2.0 * shift.0, wy + 2.0 * shift.1 + 80.0),
            150.0,
            w as f64,
            h as f64,
        );
        let o = (x0, y0);
        let p = |x: f64, y: f64, z: f64| {
            let (px, py) = iso.project(x + shift.0, y + shift.1, z);
            (o.0 + px as f32, o.1 + py as f32)
        };
        let edge = alpha(col(t.line), 170);
        let alu = Color32::from_rgb(196, 200, 206);
        let black = Color32::from_rgb(34, 34, 37);
        let plate = Color32::from_rgb(214, 215, 210);
        let steel = Color32::from_rgb(168, 174, 182);
        let motor = Color32::from_rgb(70, 72, 78);
        let boxes = |cv: &Cv, list: &[(f64, f64, f64, f64, f64, f64, Color32)]| {
            draw_boxes(cv, &iso, o, shift, list, edge);
        };
        let (hx, hy) = head.unwrap_or((tw / 2.0, td / 2.0));
        let board_z = 30.0;
        // the frame: the plate goes from -px_m to wx + px_m along the rails and from -py_m to wy + py_m across them
        let (fx0, fx1) = (-px_m, wx + px_m);
        let (fy0, fy1) = (-py_m, wy + py_m);
        let flen = fx1 - fx0;

        // the rail at the back and its feet
        boxes(
            cv,
            &[
                (fx0 + 10.0, fy0 - 34.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx1 - 36.0, fy0 - 34.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx0 - 34.0, fy0 - 40.0, 0.0, flen + 68.0, 40.0, 40.0, alu),
                (fx0 - 38.0, fy0 - 40.0, 0.0, 4.0, 40.0, 40.0, black),
                (fx1 + 34.0, fy0 - 40.0, 0.0, 4.0, 40.0, 40.0, black),
            ],
        );
        // the cable chain along the rail
        boxes(
            cv,
            &[(fx0 + 20.0, fy0 - 8.0, 40.0, flen - 40.0, 12.0, 8.0, black)],
        );
        // the plate with its grid of holes, three pins and two discs
        boxes(cv, &[(fx0, fy0, 22.0, flen, wy + 2.0 * py_m, 8.0, plate)]);
        let mut gx = fx0 + 25.0;
        while gx < fx1 - 15.0 {
            let mut gy = fy0 + 25.0;
            while gy < fy1 - 15.0 {
                let (px, py) = p(gx, gy, board_z + 0.2);
                let r = (4.0 * iso.k) as f32;
                let pts: Vec<(f32, f32)> = (0..8)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / 8.0;
                        (px + r * 1.15 * a.cos(), py + r * 0.62 * a.sin())
                    })
                    .collect();
                cv.poly(&pts, alpha(Color32::BLACK, 120), Stroke::NONE);
                gy += 40.0;
            }
            gx += 40.0;
        }
        for (cx, cy) in [(wx * 0.62, wy * 0.22), (wx * 0.30, wy * 0.74)] {
            let (px, py) = p(cx, cy, board_z + 0.4);
            let r = (22.0 * iso.k) as f32;
            let pts: Vec<(f32, f32)> = (0..14)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 14.0;
                    (px + r * 1.15 * a.cos(), py + r * 0.62 * a.sin())
                })
                .collect();
            cv.poly(
                &pts,
                Color32::from_rgb(244, 244, 240),
                Stroke::new(1.0 * cv.u, edge),
            );
        }
        boxes(
            cv,
            &[
                (fx0 + 40.0, fy1 - 40.0, board_z, 12.0, 12.0, 6.0, steel),
                (wx * 0.5, fy0 + 30.0, board_z, 12.0, 12.0, 6.0, steel),
                (fx1 - 50.0, fy1 - 60.0, board_z, 12.0, 12.0, 6.0, steel),
            ],
        );
        // the work area of the maker (thin) and the usable area of the configuration (dashed)
        let ring = |x0r: f64, y0r: f64, x1r: f64, y1r: f64| -> Vec<(f32, f32)> {
            [(x0r, y0r), (x1r, y0r), (x1r, y1r), (x0r, y1r), (x0r, y0r)]
                .iter()
                .map(|&(x, y)| p(x, y, board_z + 0.6))
                .collect()
        };
        cv.polyline(
            &ring(0.0, 0.0, profile.work_mm.0, profile.work_mm.1),
            1.0,
            alpha(col(t.dim), 200),
            false,
        );
        cv.polyline(&ring(0.0, 0.0, tw, td), 1.6, col(t.accent), true);
        // the rail at the front and its feet
        boxes(
            cv,
            &[
                (fx0 + 10.0, fy1 + 8.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx1 - 36.0, fy1 + 8.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx0 - 34.0, fy1, 0.0, flen + 68.0, 40.0, 40.0, alu),
                (fx0 - 38.0, fy1, 0.0, 4.0, 40.0, 40.0, black),
                (fx1 + 34.0, fy1, 0.0, 4.0, 40.0, 40.0, black),
            ],
        );
        // the board and the work on it
        let mut all = self.drill.scene.clone();
        all.overlay(&self.route.scene);
        if let Some((x0b, y0b, x1b, y1b)) = all.bounds() {
            boxes(
                cv,
                &[(
                    x0b - 3.0,
                    y0b - 3.0,
                    board_z,
                    x1b - x0b + 6.0,
                    y1b - y0b + 6.0,
                    3.0,
                    col(t.board),
                )],
            );
        }
        let top = board_z + 3.2;
        if self.layers[0] {
            for hole in &self.drill.scene.holes {
                let idx = sizes
                    .iter()
                    .position(|d| (*d - hole.diameter_mm).abs() < 0.005)
                    .unwrap_or(0);
                let c = col(t.size_color(idx));
                let (px, py) = p(hole.x_mm, hole.y_mm, top);
                let r = ((hole.diameter_mm * iso.k * 0.9).max(1.4)) as f32;
                let pts: Vec<(f32, f32)> = (0..10)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / 10.0;
                        (px + r * 1.15 * a.cos(), py + r * 0.62 * a.sin())
                    })
                    .collect();
                cv.poly(
                    &pts,
                    if hole.done {
                        c
                    } else {
                        alpha(Color32::BLACK, 140)
                    },
                    Stroke::new(1.0 * cv.u, c),
                );
            }
        }
        if self.layers[1] {
            for path in &self.route.scene.paths {
                let pts: Vec<(f32, f32)> = path.points.iter().map(|&(x, y)| p(x, y, top)).collect();
                cv.polyline(&pts, 1.8, col(t.accent2), true);
                if path.reached > 1 {
                    cv.polyline(&pts[..path.reached.min(pts.len())], 2.2, col(t.ok), false);
                }
            }
        }
        // the gantry: two carriages on the rails, the beam across, its motor at the end, the head and the spindle
        boxes(
            cv,
            &[
                (hx - 34.0, fy0 - 40.0, 40.0, 64.0, 40.0, 26.0, alu),
                (hx - 34.0, fy1, 40.0, 64.0, 40.0, 26.0, alu),
                (
                    hx - 64.0,
                    fy0 - 40.0,
                    66.0,
                    36.0,
                    fy1 - fy0 + 80.0,
                    36.0,
                    alu,
                ),
                (hx - 64.0, fy1 + 40.0, 62.0, 44.0, 44.0, 44.0, motor),
                (hx - 24.0, hy - 22.0, 44.0, 44.0, 44.0, 76.0, alu),
                (hx - 9.0, hy - 9.0, 36.0, 18.0, 18.0, 72.0, steel),
            ],
        );
        let tip = p(hx, hy, top);
        let spindle_foot = p(hx, hy, 40.0);
        cv.line(spindle_foot, tip, 2.2, col(t.ok));
        cv.p.circle_filled(cv.pt(tip.0, tip.1), 5.0 * cv.u, col(t.ok));
        // the tool changer: the rack at the short edge with the tools where the configuration puts them
        let atc = self
            .drill
            .engine
            .settings()
            .filter(|s| s.auto_change && !s.atc.is_empty())
            .map(|s| s.atc.clone())
            .unwrap_or_default();
        if !atc.is_empty() {
            let (y_min, y_max) = atc.iter().fold((f64::MAX, f64::MIN), |(a, b), s| {
                (a.min(s.y_mm), b.max(s.y_mm))
            });
            let rack_x = atc.iter().map(|s| s.x_mm).fold(f64::MIN, f64::max);
            boxes(
                cv,
                &[(
                    rack_x - 26.0,
                    y_min - 14.0,
                    board_z,
                    46.0,
                    y_max - y_min + 28.0,
                    10.0,
                    alu,
                )],
            );
            for s in &atc {
                let c = match s.index {
                    0 => steel,
                    15 => Color32::from_rgb(90, 200, 190),
                    n => col(t.size_color((n as usize).saturating_sub(1))),
                };
                boxes(
                    cv,
                    &[(
                        s.x_mm - 3.0,
                        s.y_mm - 3.0,
                        board_z + 10.0,
                        6.0,
                        6.0,
                        26.0,
                        c,
                    )],
                );
            }
        }
        cv.text(
            x0 + 18.0,
            y0 + h + 6.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{} - {:.0} x {:.0} x {:.0} mm - {:.0} kg",
                profile.name,
                profile.work_mm.0,
                profile.work_mm.1,
                profile.work_mm.2,
                profile.weight_kg
            ),
            12.0,
            col(t.dim),
        );
    }
}

fn install_fonts(ctx: &egui::Context) {
    let candidates = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/simsun.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/System/Library/Fonts/PingFang.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = FontDefinitions::default();
            fonts
                .font_data
                .insert("cjk".into(), Arc::new(FontData::from_owned(bytes)));
            for family in [FontFamily::Monospace, FontFamily::Proportional] {
                fonts.families.entry(family).or_default().push("cjk".into());
            }
            ctx.set_fonts(fonts);
            return;
        }
    }
}

fn save_png(img: &egui::ColorImage, path: &Path) -> std::io::Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, img.size[0] as u32, img.size[1] as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(std::io::Error::other)?;
    let data: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
    w.write_image_data(&data).map_err(std::io::Error::other)
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frame += 1;
        if self.start_at_frame == Some(self.frame) {
            self.start_job();
        }
        // the keys: Enter takes the modern interface at the launcher; Esc stops the job
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && self.screen == Screen::Launcher {
            self.act(Act::Choose(Choice::Modern));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && self.running.is_some() {
            self.stop_job();
        }
        self.tick();
        let shot_file = self
            .shot
            .as_ref()
            .map(|s| (s.file.clone(), s.wait, s.done.clone()));
        if let Some((file, wait, done)) = shot_file {
            if self.frame == 3 + wait {
                let (f2, d2, c2) = (file.clone(), done.clone(), ctx.clone());
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                    egui::ScreenshotCallback::new(move |img| {
                        if let Err(e) = save_png(&img, &f2) {
                            eprintln!("cannot save {}: {e}", f2.display());
                        }
                        d2.store(true, Ordering::SeqCst);
                        c2.request_repaint();
                    }),
                ));
            }
            if done.load(Ordering::SeqCst) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        let mut clicked: Option<Pos2> = None;
        egui::Frame::NONE.show(ui, |ui| {
            let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click());
            let painter = ui.painter_at(rect);
            self.paint(&painter, rect);
            if resp.clicked() {
                clicked = resp.interact_pointer_pos();
            }
        });
        if let Some(pos) = clicked {
            // the last card or button drawn on top is the one that is hit
            let hit = self
                .hits
                .iter()
                .rev()
                .find(|(r, _)| r.contains(pos))
                .map(|(_, a)| *a);
            if let Some(a) = hit {
                self.act(a);
            }
        }
        ctx.request_repaint_after(Duration::from_millis(if self.running.is_some() {
            16
        } else {
            100
        }));
    }
}

/// Opens the window.
pub fn run(o: &Options) -> eframe::Result {
    let app = App::new(o);
    let size = if o.shot.is_some() {
        Vec2::new(1600.0, 900.0)
    } else {
        Vec2::new(1400.0, 820.0)
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size([900.0, 560.0])
            .with_resizable(true)
            .with_maximize_button(true)
            .with_title("EH3D-BUNGARD-GUI"),
        ..Default::default()
    };
    eframe::run_native(
        "EH3D-BUNGARD-GUI",
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
