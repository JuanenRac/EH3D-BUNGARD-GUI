//! The controls that have life: the check boxes (the box fills and the mark is drawn when it is switched, and it answers to the pointer),
//! the text boxes (they take the colours of the theme and light up when they are the focus), the buttons that cannot be pressed now, and
//! the emergency button, which can be dragged to wherever the operator wants it.

use super::extra::Zone;
use super::*;
use eh3d_drillpro::protocol::sound::{Cue, SoundSetting};

/// The colours of the text boxes of egui, made from the theme: the box is the colour of the raised panels, its edge is the accent colour
/// and brightens under the pointer and in the focus, and the selected text has the accent colour behind it.
pub(crate) fn apply_visuals(ctx: &egui::Context, t: &Theme) {
    let mut v = egui::Visuals::dark();
    let (bg, text, accent) = (col(t.panel_raised), col(t.text), col(t.accent));
    v.extreme_bg_color = bg;
    v.override_text_color = Some(text);
    v.selection.bg_fill = alpha(accent, 110);
    v.selection.stroke = Stroke::new(2.0, accent);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, alpha(accent, 120));
    v.widgets.hovered.bg_stroke = Stroke::new(1.6, shade(accent, 1.15));
    v.widgets.active.bg_stroke = Stroke::new(2.0, accent);
    v.widgets.inactive.corner_radius = 8.0.into();
    v.widgets.hovered.corner_radius = 8.0.into();
    v.widgets.active.corner_radius = 8.0.into();
    v.text_cursor.stroke = Stroke::new(2.0, accent);
    ctx.set_visuals(v);
}

impl<'a> Cv<'a> {
    /// A button that cannot be pressed now: dim, flat, with no answer to the pointer.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn button_off(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str, c: Color32) {
        let rect = self.rect(x, y, w, h);
        let r = if self.t.corner == Corner::Round {
            12.0 * self.u
        } else {
            0.0
        };
        self.p.rect_filled(rect, r, alpha(shade(c, 0.45), 120));
        self.p.rect_stroke(
            rect,
            r,
            Stroke::new(self.u, alpha(c, 50)),
            StrokeKind::Inside,
        );
        self.text_fit(
            x + w / 2.0,
            y + h / 2.0 + 6.0,
            Align2::CENTER_BOTTOM,
            &label.to_uppercase(),
            17.0,
            alpha(col(self.t.text), 90),
            w - 16.0,
        );
    }

    /// A check box with its label: the box fills and the mark is drawn in a moment when it is switched, it grows and glows under the pointer
    /// and sinks when it is pressed.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn checkbox(&mut self, x: f32, y: f32, w: f32, label: &str, on: bool, act: Act) {
        let row = self.rect(x, y, w, 36.0);
        let (hover, down) = self.touch(row);
        let ctx = self.p.ctx().clone();
        let id = egui::Id::unique(("checkbox", label));
        let k = ctx.animate_bool_with_time(id, on, 0.2);
        let h =
            ctx.animate_bool_with_time(egui::Id::unique(("checkbox-hover", label)), hover, 0.12);
        let accent = col(self.t.accent);
        // the row lights up under the pointer
        if h > 0.0 {
            self.p
                .rect_filled(row, 8.0 * self.u, alpha(accent, (28.0 * h) as u8));
        }
        let grow = 1.0 + 0.08 * h - if down { 0.1 } else { 0.0 };
        let (bx, by, bs) = (x + 6.0, y + 5.0, 26.0);
        let c = (bx + bs / 2.0, by + bs / 2.0);
        let half = bs / 2.0 * grow;
        let boxr = self.rect(c.0 - half, c.1 - half, half * 2.0, half * 2.0);
        let r = if self.t.corner == Corner::Square {
            0.0
        } else {
            7.0 * self.u
        };
        // the glow
        if h > 0.0 || k > 0.0 {
            self.p.rect_filled(
                boxr.expand(4.0 * self.u),
                r + 4.0 * self.u,
                alpha(accent, (50.0 * h.max(k * 0.5)) as u8),
            );
        }
        let off = col(self.t.panel_raised);
        let fill = Color32::from_rgb(
            (off.r() as f32 + (accent.r() as f32 - off.r() as f32) * k) as u8,
            (off.g() as f32 + (accent.g() as f32 - off.g() as f32) * k) as u8,
            (off.b() as f32 + (accent.b() as f32 - off.b() as f32) * k) as u8,
        );
        // a box in relief when it is on, sunk into the panel when it is off
        if k > 0.5 {
            self.raised(boxr, r, fill, hover, down);
        } else {
            self.sunk(boxr, r, fill, hover);
        }
        // the mark is drawn as it is switched: first the short stroke and then the long one
        if k > 0.0 {
            let ink = col(self.t.background.0);
            let (a, b, d) = (
                (c.0 - 8.0 * grow, c.1 + 0.5 * grow),
                (c.0 - 2.5 * grow, c.1 + 6.0 * grow),
                (c.0 + 8.5 * grow, c.1 - 6.5 * grow),
            );
            let first = (k * 2.0).min(1.0);
            let second = (k * 2.0 - 1.0).clamp(0.0, 1.0);
            let mid = (a.0 + (b.0 - a.0) * first, a.1 + (b.1 - a.1) * first);
            self.line(a, mid, 3.0, ink);
            if second > 0.0 {
                let end = (b.0 + (d.0 - b.0) * second, b.1 + (d.1 - b.1) * second);
                self.line(b, end, 3.0, ink);
            }
        }
        self.text_fit(
            x + 44.0,
            y + 25.0,
            Align2::LEFT_BOTTOM,
            label,
            16.0,
            if on || hover {
                col(self.t.text)
            } else {
                alpha(col(self.t.text), 200)
            },
            w - 52.0,
        );
        self.hits.push((row, act));
    }
}

impl App {
    /// Gives the engines the connection and the sounds of the window as they are now.
    pub(crate) fn apply_conn(&mut self) {
        self.drill.engine.set_connection(self.conn.clone());
        self.route.engine.set_connection(self.conn.clone());
        self.connection = self.conn.describe();
    }

    fn sound_mut(&mut self, cue: Cue) -> &mut SoundSetting {
        match cue {
            Cue::Finished => &mut self.conn.sound_finish,
            Cue::ToolChange => &mut self.conn.sound_tool_change,
            Cue::CheckOk => &mut self.conn.sound_check_ok,
            Cue::CheckNotOk => &mut self.conn.sound_check_bad,
        }
    }

    /// The sound chosen in its list: none, the standard tone, a file of the person's or one of the catalogue. Choosing never plays it: it is heard with the Test button.
    pub(crate) fn sound_choose(&mut self, cue: Cue, setting: SoundSetting) {
        *self.sound_mut(cue) = setting;
        self.saved = false;
        self.apply_conn();
    }

    /// The path typed for a sound: empty is the default tone.
    pub(crate) fn sound_set_path(&mut self, cue: Cue, text: &str) {
        let t = text.trim();
        let s = self.sound_mut(cue);
        if matches!(s, SoundSetting::Default | SoundSetting::File(_)) {
            *s = if t.is_empty() {
                SoundSetting::Default
            } else {
                SoundSetting::File(PathBuf::from(t))
            };
        }
        self.saved = false;
        self.apply_conn();
    }

    /// Plays a sound to hear it.
    pub(crate) fn sound_test(&self, cue: Cue) {
        let s = match cue {
            Cue::Finished => &self.conn.sound_finish,
            Cue::ToolChange => &self.conn.sound_tool_change,
            Cue::CheckOk => &self.conn.sound_check_ok,
            Cue::CheckNotOk => &self.conn.sound_check_bad,
        };
        eh3d_drillpro::protocol::sound::play(s, cue);
    }

    /// The boxes of the paths of the sounds, over the Job tab of the settings.
    pub(super) fn sound_fields(&mut self, ctx: &egui::Context) {
        let u = self.ui_scale;
        let (x, y) = (380.0, 30.0 + 120.0);
        for (i, cue) in [
            Cue::Finished,
            Cue::ToolChange,
            Cue::CheckOk,
            Cue::CheckNotOk,
        ]
        .into_iter()
        .enumerate()
        {
            let s = match cue {
                Cue::Finished => self.conn.sound_finish.clone(),
                Cue::ToolChange => self.conn.sound_tool_change.clone(),
                Cue::CheckOk => self.conn.sound_check_ok.clone(),
                Cue::CheckNotOk => self.conn.sound_check_bad.clone(),
            };
            // the box of the path is there only while the sound is a file of the person's
            if !matches!(s, SoundSetting::File(_)) {
                continue;
            }
            let mut text = match &s {
                SoundSetting::File(p) => p.display().to_string(),
                _ => String::new(),
            };
            let before = text.clone();
            let pos = Pos2::new(
                self.ui_origin.x + (x + 60.0) * u,
                self.ui_origin.y + (y + 202.0 + 50.0 + i as f32 * 84.0) * u,
            );
            egui::Area::new(egui::Id::unique(("sound-path", i)))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    super::look::text_box(
                        ui,
                        egui::TextEdit::singleline(&mut text)
                            .desired_width(560.0 * u)
                            .hint_text(self.tr(Text::SoundFileHint))
                            .font(FontId::proportional(16.0 * u)),
                    );
                });
            if text != before {
                self.sound_set_path(cue, &text);
            }
        }
    }

    /// The state of the buttons of the job: Start works when nothing runs or the job is paused; Pause and Stop only while it runs.
    pub(super) fn job_buttons(&self) -> (bool, bool, bool) {
        let running = self.running.is_some();
        (
            (!running || self.paused) && !self.estop,
            running && !self.paused,
            running,
        )
    }

    /// The emergency button: a red cap on a yellow plate, with a shadow and a ring that glows under the pointer and pulses when the stop is
    /// on. It can be dragged to where the operator wants it (the place is remembered).
    pub(super) fn paint_estop(&self, cv: &mut Cv) {
        if !self.settings.emergency {
            return;
        }
        let t = cv.t.clone();
        let (ex, ey) = self.settings.estop_pos;
        let r = 46.0;
        let rect = cv.rect(ex - r, ey - r, r * 2.0, r * 2.0);
        let (hover, down) = cv.touch(rect);
        let centre = cv.pt(ex, ey);
        let pulse = if self.estop {
            let time = cv.p.ctx().input(|i| i.time) as f32;
            0.5 + 0.5 * (time * 6.0).sin()
        } else {
            0.0
        };
        cv.p.circle_filled(
            centre + Vec2::new(0.0, 4.0 * cv.u),
            (r + 8.0) * cv.u,
            alpha(Color32::BLACK, 110),
        );
        if hover || self.estop {
            cv.p.circle_filled(
                centre,
                (r + 14.0 + 6.0 * pulse) * cv.u,
                alpha(
                    col(t.danger),
                    (50.0 + 90.0 * pulse.max(if hover { 0.4 } else { 0.0 })) as u8,
                ),
            );
        }
        // the yellow plate and the dark ring
        cv.p.circle_filled(centre, (r + 6.0) * cv.u, Color32::from_rgb(244, 196, 36));
        cv.p.circle_stroke(
            centre,
            (r + 6.0) * cv.u,
            Stroke::new(1.5 * cv.u, Color32::from_rgb(120, 90, 0)),
        );
        cv.p.circle_filled(centre, (r - 2.0) * cv.u, Color32::from_rgb(40, 38, 34));
        // the red cap sinks when it is pressed
        let cap = if down { r - 13.0 } else { r - 8.0 };
        let red = if self.estop {
            Color32::from_rgb(120, 20, 20)
        } else if down {
            Color32::from_rgb(150, 20, 28)
        } else if hover {
            Color32::from_rgb(255, 60, 60)
        } else {
            Color32::from_rgb(220, 34, 40)
        };
        cv.p.circle_filled(
            centre + Vec2::new(0.0, if down { 2.0 } else { 0.0 } * cv.u),
            cap * cv.u,
            red,
        );
        cv.p.circle_stroke(
            centre,
            cap * cv.u,
            Stroke::new(1.2 * cv.u, alpha(Color32::WHITE, 70)),
        );
        cv.p.circle_filled(
            centre - Vec2::new(cap * 0.3, cap * 0.35) * cv.u,
            (cap * 0.28) * cv.u,
            alpha(Color32::WHITE, if down { 20 } else { 60 }),
        );
        cv.draw_icon(ex, ey + 1.0, 30.0, Icon::Alert, Color32::WHITE);
        cv.zones.push((rect, Zone::Estop));
        cv.hits.push((rect, Act::Emergency));
        if self.estop {
            cv.text(
                ex,
                ey - r - 16.0,
                Align2::CENTER_BOTTOM,
                self.tr(Text::EmergencyStop),
                14.0,
                col(t.danger),
            );
        }
    }

    /// The emergency stop: the job and the machine stop at once; the button must be pressed again to release the stop before a job can start.
    pub(crate) fn emergency(&mut self) {
        if self.estop {
            self.estop = false;
            self.message = None;
            return;
        }
        self.estop = true;
        self.stop_job();
        self.message = Some(self.tr(Text::EmergencyStop).to_string());
    }
}

/// The pictures of the buttons that start, pause and stop the job and of the emergency button: drawn with shapes, so they scale with the window and take the
/// colour of the theme.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Icon {
    Play,
    Pause,
    Stop,
    /// A warning triangle with an exclamation mark, for the emergency button.
    Alert,
    Select,
    Copy,
    Paste,
    Trash,
    Rotate,
    FitBoard,
    FitTable,
    Target,
    Enlarge,
    Back,
    Up,
    Cut,
    Undo,
    Redo,
    Next,
    Down,
}

impl Cv<'_> {
    /// The icon centred at (`cx`, `cy`), `size` canvas units high, in colour `c`.
    pub(crate) fn draw_icon(&self, cx: f32, cy: f32, size: f32, icon: Icon, c: Color32) {
        let h = size / 2.0;
        match icon {
            Icon::Play => {
                // a triangle pointing right, its centre of weight on the centre of the button
                self.poly(
                    &[
                        (cx - h * 0.62, cy - h),
                        (cx - h * 0.62, cy + h),
                        (cx + h * 0.95, cy),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::Pause => {
                for dx in [-0.52_f32, 0.52] {
                    let x = cx + dx * h;
                    self.poly(
                        &[
                            (x - h * 0.34, cy - h),
                            (x + h * 0.34, cy - h),
                            (x + h * 0.34, cy + h),
                            (x - h * 0.34, cy + h),
                        ],
                        c,
                        Stroke::NONE,
                    );
                }
            }
            Icon::Stop => {
                let s = h * 0.92;
                self.poly(
                    &[
                        (cx - s, cy - s),
                        (cx + s, cy - s),
                        (cx + s, cy + s),
                        (cx - s, cy + s),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::Alert => {
                // the triangle, an outline, and the exclamation mark inside it
                let top = (cx, cy - h * 1.05);
                let (l, r, b) = (
                    (cx - h * 1.15, cy + h * 0.85),
                    (cx + h * 1.15, cy + h * 0.85),
                    cy + h * 0.85,
                );
                self.polyline(&[top, r, l, top], 3.0, c, false);
                self.poly(
                    &[
                        (cx - h * 0.1, cy - h * 0.42),
                        (cx + h * 0.1, cy - h * 0.42),
                        (cx + h * 0.07, cy + h * 0.28),
                        (cx - h * 0.07, cy + h * 0.28),
                    ],
                    c,
                    Stroke::NONE,
                );
                let dot = self.pt(cx, b - h * 0.32);
                self.p.circle_filled(dot, h * 0.1 * self.u, c);
            }
            other => self.draw_more_icon(cx, cy, size, other, c),
        }
    }

    /// A button with a picture in place of its label (see [`Cv::button`]); the picture sinks with the button when it is pressed.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn icon_button(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        icon: Icon,
        c: Color32,
        act: Act,
    ) {
        self.button(x, y, w, h, "", c, act);
        let (_, down) = self.touch(self.rect(x, y, w, h));
        let ink = if self.t.corner == Corner::Square {
            Color32::BLACK
        } else {
            col(self.t.background.0)
        };
        self.draw_icon(
            x + w / 2.0,
            y + h / 2.0 + if down { 3.0 } else { 0.0 },
            34.0,
            icon,
            ink,
        );
    }

    /// The same button when it does not answer: dim, with the picture dim too.
    pub(crate) fn icon_button_off(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        icon: Icon,
        c: Color32,
    ) {
        self.button_off(x, y, w, h, "", c);
        self.draw_icon(
            x + w / 2.0,
            y + h / 2.0,
            34.0,
            icon,
            alpha(col(self.t.text), 90),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;
    use std::path::{Path, PathBuf};

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir: PathBuf =
            std::env::temp_dir().join(format!("eh3d-bg-controls-{}-{n}", std::process::id()));
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
        a.fixed_dt = Some(0.05);
        a.open_path(&dir.join("05_small_board.ncd"));
        a
    }

    /// Paints a screen of the app in a language with no window and gives back the texts that did not fit their room, even made smaller.
    fn clipped_on(a: &mut App, ctx: &egui::Context) -> Vec<String> {
        CLIPPED.with(|c| c.borrow_mut().clear());
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(DESIGN_W, DESIGN_H));
        let input = egui::RawInput {
            screen_rect: Some(rect),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            let p = ui.ctx().layer_painter(egui::LayerId::background());
            a.paint(&p, rect);
        });
        out.textures_delta.clear();
        CLIPPED.with(|c| std::mem::take(&mut *c.borrow_mut()))
    }

    #[test]
    fn no_text_of_a_button_a_switch_or_a_list_is_cut_in_any_language() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.animate = false;
        // the tool changer: the window of the rack with the tools of the job in its slots
        a.settings.model = "CCD/2 ATC".to_string();
        a.rack_show = a.rack_plan(&[Program::Drill, Program::Route]).remove(0);
        a.rack_no = (2, 3);
        a.select_mode = true;
        let mut found: Vec<String> = Vec::new();
        for lang in Lang::ALL {
            a.set_lang(lang);
            let mut screens: Vec<(String, Option<extra::Modal>, Screen)> = vec![
                ("launcher".into(), None, Screen::Launcher),
                ("viewer".into(), None, Screen::Viewer),
            ];
            for m in [
                extra::Modal::About,
                extra::Modal::Open,
                extra::Modal::Panel,
                extra::Modal::Settings,
                extra::Modal::Connection,
                extra::Modal::Board,
                extra::Modal::Tools,
                extra::Modal::Catalogue,
                extra::Modal::Head,
                extra::Modal::Inspect,
                extra::Modal::InspectTune,
                extra::Modal::Rack,
                extra::Modal::Bar,
            ] {
                screens.push((format!("{m:?}"), Some(m), Screen::Viewer));
            }
            for (_name, modal, screen) in screens {
                a.screen = screen;
                a.modal = None;
                if let Some(m) = modal {
                    a.show_modal(m);
                }
                // two frames: the first one draws what the second one needs to be settled
                clipped_on(&mut a, &ctx);
                for t in clipped_on(&mut a, &ctx) {
                    found.push(format!("{}: {t}", lang.code()));
                }
            }
        }
        found.sort();
        found.dedup();
        assert!(
            found.is_empty(),
            "{} texts are cut:\n{}",
            found.len(),
            found.join("\n")
        );
    }

    #[test]
    fn the_board_window_paints_the_sheets_without_copper_in_any_language() {
        use super::super::board::BoardPick;
        use super::super::combo::ComboId;
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.animate = false;
        a.screen = Screen::Viewer;
        // one of each kind of sheet without copper (empty copper list, no formats) and, between them, a laminate with copper
        let picks: Vec<usize> = crate::boards::MATERIALS
            .iter()
            .enumerate()
            .filter(|(_, m)| m.sides == crate::boards::Sides::NoCopper)
            .map(|(i, _)| i)
            .collect();
        assert!(picks.len() >= 15);
        let copper = a.board.material;
        let mut found: Vec<String> = Vec::new();
        for lang in Lang::ALL {
            a.set_lang(lang);
            for &i in &picks {
                a.board = a.board.with_material(i);
                assert!(a.board.has_no_copper() && a.board.copper_um == 0.0);
                a.show_modal(extra::Modal::Board);
                clipped_on(&mut a, &ctx);
                for t in clipped_on(&mut a, &ctx) {
                    found.push(format!("{}: {t}", lang.code()));
                }
                // the lists of the window do not break with an empty copper or an empty size
                for id in [ComboId::Thickness, ComboId::BoardSize] {
                    let items = a.combo_items(id);
                    assert!(!items.is_empty());
                    assert!(a.combo_current(id) < items.len());
                }
                assert!(a.board.material().copper_um.is_empty());
                // the sides and the copper cannot be picked on a sheet without copper
                let before = a.board;
                a.board_pick(BoardPick::Layers(1));
                a.board_pick(BoardPick::Copper(0));
                assert_eq!(a.board.copper_um, before.copper_um);
                a.modal = None;
            }
            a.board = a.board.with_material(copper);
        }
        found.sort();
        found.dedup();
        assert!(
            found.is_empty(),
            "{} texts are cut:\n{}",
            found.len(),
            found.join("\n")
        );
    }

    #[test]
    fn the_open_window_paints_with_the_preview_on_and_off() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.animate = false;
        a.screen = Screen::Viewer;
        a.last_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
        for on in [true, false, true] {
            a.settings.open_preview = on;
            a.show_modal(extra::Modal::Open);
            let i = a
                .browser
                .as_ref()
                .unwrap()
                .entries
                .iter()
                .position(|e| e.name == "05_small_board.ncd")
                .unwrap();
            a.browse(i);
            assert_eq!(a.browser.as_ref().unwrap().preview.is_some(), on);
            clipped_on(&mut a, &ctx);
            clipped_on(&mut a, &ctx);
            assert!(a.modal_card.is_some());
        }
    }

    #[test]
    fn every_theme_paints_the_relief_the_choice_and_the_icons() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.animate = false;
        a.screen = Screen::Viewer;
        // the slots of a tool changer, as its machine file has them
        a.settings.model = "CCD/2 ATC".to_string();
        if let Some(s) = a.drill.engine.settings_mut() {
            s.auto_change = true;
            s.atc = (0..=15)
                .map(|i| eh3d_bungard_formats::config::AtcSlot {
                    index: i,
                    x_mm: 330.0 + 5.0 * i as f64,
                    y_mm: 20.0 + 18.0 * i as f64,
                    z_mm: 30.0,
                })
                .collect();
        }
        for id in ThemeId::ALL {
            a.set_theme(id);
            a.select_mode = true;
            a.select_in_box((-1e4, -1e4), (1e4, 1e4), true, false);
            let _ = clipped_on(&mut a, &ctx);
            let _ = clipped_on(&mut a, &ctx);
            for m in [
                extra::Modal::Open,
                extra::Modal::Settings,
                extra::Modal::Board,
                extra::Modal::Tools,
                extra::Modal::Bar,
            ] {
                a.show_modal(m);
                let _ = clipped_on(&mut a, &ctx);
                let _ = clipped_on(&mut a, &ctx);
                assert!(a.modal_card.is_some(), "{id:?} {m:?}");
            }
            a.modal = None;
        }
    }

    #[test]
    fn every_window_has_a_top_to_move_it_and_an_x_in_its_corner_to_close_it() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.animate = false;
        for m in [
            extra::Modal::About,
            extra::Modal::Open,
            extra::Modal::Panel,
            extra::Modal::Settings,
            extra::Modal::Connection,
            extra::Modal::Board,
            extra::Modal::Tools,
            extra::Modal::Catalogue,
            extra::Modal::Head,
            extra::Modal::Inspect,
            extra::Modal::InspectTune,
            extra::Modal::Bar,
        ] {
            a.screen = Screen::Viewer;
            a.show_modal(m);
            clipped_on(&mut a, &ctx);
            clipped_on(&mut a, &ctx);
            let (x, y, w, _) = a.modal_card.unwrap_or_else(|| panic!("{m:?} has no card"));
            let drag = a
                .modal_drag
                .unwrap_or_else(|| panic!("{m:?} has no top to drag"));
            let s = a.ui_scale;
            let o = a.ui_origin;
            // the strip of the top is over the title and ends before the X
            assert!(
                (drag.min.x - (o.x + x * s)).abs() < 1.0
                    && (drag.min.y - (o.y + y * s)).abs() < 1.0,
                "{m:?}"
            );
            assert!(drag.max.x < o.x + (x + w) * s - 40.0 * s, "{m:?}");
            // the X is a button of the corner that closes the window
            let corner = Pos2::new(o.x + (x + w - 32.0) * s, o.y + (y + 28.0) * s);
            let close = a
                .hits
                .iter()
                .rev()
                .find(|(r, _)| r.contains(corner))
                .map(|(_, act)| *act);
            assert!(matches!(close, Some(Act::CloseModal)), "{m:?}: {close:?}");
            // dragged, the window is somewhere else and its X goes with it
            a.drag(Zone::ModalMove, (120.0, 60.0), false);
            clipped_on(&mut a, &ctx);
            let moved = Pos2::new(
                o.x + (x + w - 32.0 + 120.0) * s,
                o.y + (y + 28.0 + 60.0) * s,
            );
            let close = a
                .hits
                .iter()
                .rev()
                .find(|(r, _)| r.contains(moved))
                .map(|(_, act)| *act);
            assert!(
                matches!(close, Some(Act::CloseModal)),
                "{m:?} moved: {close:?}"
            );
            a.act(Act::CloseModal);
            assert!(a.modal.is_none());
        }
    }

    #[test]
    fn the_warning_of_a_catalogue_bit_and_the_notes_of_the_material_fit_in_every_language() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.animate = false;
        a.board = a
            .board
            .with_material(board_of(crate::boards::Core::Ceramic));
        let mut short = *a
            .cat_bits()
            .iter()
            .find(|b| b.kind.is_drill() && b.diameter_mm <= 1.0)
            .expect("a drill");
        short.flute_mm = 1.0;
        let tool = a.used_tools(Program::Drill)[0].name.unwrap();
        a.tool_bits.push((Program::Drill, tool, short));
        let mut found: Vec<String> = Vec::new();
        for lang in Lang::ALL {
            a.set_lang(lang);
            a.screen = Screen::Viewer;
            a.modal = None;
            a.show_modal(extra::Modal::Tools);
            clipped_on(&mut a, &ctx);
            for t in clipped_on(&mut a, &ctx) {
                found.push(format!("{}: {t}", lang.code()));
            }
        }
        found.sort();
        found.dedup();
        assert!(
            found.is_empty(),
            "texts cut:
{}",
            found.join(
                "
"
            )
        );
    }

    /// The string literals of the code that paints the interface (what is before the tests of each file) that look like words of the interface.
    fn words_in_literals(src: &str) -> Vec<String> {
        let code = src.split("\n#[cfg(test)]").next().unwrap_or(src);
        let mut found = Vec::new();
        for line in code.lines() {
            let t = line.trim_start();
            // messages of errors, of the log and of the checks are not for the screen
            if t.starts_with("//")
                || [
                    "expect(",
                    "assert",
                    "panic!",
                    "Err(",
                    "eprintln!",
                    "println!",
                    "write!",
                    "unreachable!",
                    "debug_assert",
                    "env!(",
                    "include_str!",
                    "tracing::",
                    "log::",
                ]
                .iter()
                .any(|k| t.contains(k))
            {
                continue;
            }
            let chars: Vec<char> = line.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                if chars[i] == '"' {
                    let mut lit = String::new();
                    i += 1;
                    while i < chars.len() && chars[i] != '"' {
                        if chars[i] == '\\' {
                            i += 1;
                        }
                        if i < chars.len() {
                            lit.push(chars[i]);
                        }
                        i += 1;
                    }
                    let words: Vec<&str> = lit
                        .split(|c: char| !c.is_ascii_alphabetic())
                        .filter(|w| w.len() >= 2)
                        .collect();
                    let sentence = words.len() >= 2 && lit.contains(' ') && !lit.contains('{');
                    let shouting = lit.len() >= 3
                        && lit.chars().all(|c| c.is_ascii_uppercase() || c == ' ')
                        && !["MTC", "ATC", "CCD", "DTD", "DCF", "RTD", "RCF"]
                            .contains(&lit.as_str());
                    if sentence || shouting {
                        found.push(lit);
                    }
                }
                i += 1;
            }
        }
        found
    }

    #[test]
    fn no_word_of_the_interface_is_written_in_the_painting_code() {
        // the brand names and the units are the same in every language; everything else goes through the texts of eh3d-view
        let ok = ["MTC + ATC", "Z mm/min", "USERPROFILE", "HOME"];
        let files = [
            ("gui.rs", include_str!("../gui.rs")),
            ("board.rs", include_str!("board.rs")),
            ("catalogue.rs", include_str!("catalogue.rs")),
            ("combo.rs", include_str!("combo.rs")),
            ("controls.rs", include_str!("controls.rs")),
            ("extra.rs", include_str!("extra.rs")),
            ("fleet.rs", include_str!("fleet.rs")),
            ("head.rs", include_str!("head.rs")),
            ("inspect.rs", include_str!("inspect.rs")),
            ("levelling.rs", include_str!("levelling.rs")),
            ("machine3d.rs", include_str!("machine3d.rs")),
            ("numedit.rs", include_str!("numedit.rs")),
            ("opener.rs", include_str!("opener.rs")),
            ("resume.rs", include_str!("resume.rs")),
            ("view2d.rs", include_str!("view2d.rs")),
        ];
        let mut bad = Vec::new();
        for (name, src) in files {
            for lit in words_in_literals(src) {
                if !ok.contains(&lit.as_str()) {
                    bad.push(format!("{name}: \"{lit}\""));
                }
            }
        }
        assert!(
            bad.is_empty(),
            "words of the interface outside the texts:\n{}",
            bad.join("\n")
        );
    }

    #[test]
    fn a_bit_of_the_catalogue_goes_to_the_tool_with_the_numbers_the_spindle_can_give() {
        use crate::spindle::{Fit, SpindleId};
        let mut a = app();
        a.settings.spindle = SpindleId::Vfd;
        let bits = a.cat_bits();
        let (i, b) = bits
            .iter()
            .enumerate()
            .find(|(_, b)| {
                b.kind.is_drill()
                    && matches!(a.settings.current_spindle().fit(b), Fit::Adjusted { .. })
            })
            .expect("a drill that the VFD spindle takes with other numbers");
        let Fit::Adjusted { bit, .. } = a.settings.current_spindle().fit(b) else {
            unreachable!()
        };
        a.act(Act::ToolFromCat(Program::Drill, 0));
        assert_eq!(a.modal, Some(Modal::Catalogue));
        a.act(Act::CatSelect(i));
        a.act(Act::CatUse);
        assert_eq!(a.modal, Some(Modal::Tools));
        let row = &a.used_tools(Program::Drill)[0];
        let rpm: u32 = row.cells[2].trim().parse().unwrap();
        assert_eq!(rpm, bit.rpm);
        assert!(rpm <= 24_000);
        // a router does not go to a drill tool, and the catalogue forgets the target when opened alone
        a.act(Act::Show(Modal::Catalogue));
        assert!(a.cat_target.is_none());
    }

    fn board_of(core: crate::boards::Core) -> usize {
        crate::boards::MATERIALS
            .iter()
            .position(|m| m.core == core)
            .expect("a material of the core")
    }

    #[test]
    fn a_bit_of_the_catalogue_takes_the_factors_of_the_material_of_the_board_to_the_tool() {
        use crate::boards::Core;
        use crate::spindle::SpindleId;
        let mut a = app();
        a.settings.spindle = SpindleId::Original;
        a.board = a.board.with_material(board_of(Core::Ptfe));
        let bits = a.cat_bits();
        let (i, b) = bits
            .iter()
            .enumerate()
            .find(|(_, b)| b.kind.is_drill() && b.rpm <= 60_000 && b.diameter_mm <= 1.0)
            .map(|(i, b)| (i, *b))
            .expect("a small drill");
        a.act(Act::ToolFromCat(Program::Drill, 0));
        a.act(Act::CatSelect(i));
        a.act(Act::CatUse);
        let row = &a.used_tools(Program::Drill)[0];
        let rpm: u32 = row.cells[2].trim().parse().unwrap();
        let feed: u32 = row.cells[1].trim().parse().unwrap();
        // PTFE wants 0.8 of the turns and of the feed
        assert!(
            (rpm as f64 - b.rpm as f64 * 0.8).abs() <= 500.0,
            "{rpm} {b:?}"
        );
        assert!(
            (feed as f64 - b.plunge_mm_min as f64 * 0.8).abs() <= 10.0,
            "{feed}"
        );
        assert!(rpm < b.rpm && feed < b.plunge_mm_min);
        // and the bit is remembered for the tool, to warn when it is not suitable
        let tool = row.name.unwrap();
        assert!(a
            .tool_bits
            .iter()
            .any(|(p, n, x)| *p == Program::Drill && *n == tool && *x == b));
    }

    #[test]
    fn a_router_of_the_catalogue_gets_its_passes_from_the_board_with_the_extra_and_its_bite() {
        use crate::boards::Core;
        let mut a = app_with_both();
        a.board = a.board.with_material(board_of(Core::Fr4));
        a.board.thickness_mm = 1.6;
        let bits = a.cat_bits();
        let (i, b) = bits
            .iter()
            .enumerate()
            .find(|(_, b)| !b.kind.is_drill() && b.pass_mm > 0.0 && b.pass_mm < 1.0)
            .map(|(i, b)| (i, *b))
            .expect("a router with a small bite");
        a.act(Act::ToolFromCat(Program::Route, 0));
        a.act(Act::CatSelect(i));
        a.act(Act::CatUse);
        let row = &a.used_tools(Program::Route)[0];
        let depth: f64 = row.cells[0].trim().parse().unwrap();
        let passes: u32 = row.cells[1].trim().parse().unwrap();
        // at least the board and the extra, and the passes cover that depth with the bite of the bit
        assert!(
            depth >= 1.6 + crate::defaults::ROUTE_EXTRA_MM - 0.06,
            "{depth}"
        );
        assert_eq!(passes, (depth / b.pass_mm).ceil() as u32, "{depth} {b:?}");
        assert!(passes as f64 * b.pass_mm >= depth - 1e-6);
    }

    #[test]
    fn a_tool_whose_catalogue_bit_does_not_suit_the_board_is_marked_and_the_default_clears_it() {
        let mut a = app();
        a.board.thickness_mm = 1.6;
        let mut short = *a
            .cat_bits()
            .iter()
            .find(|b| b.kind.is_drill() && b.diameter_mm <= 1.0)
            .expect("a drill");
        short.flute_mm = 1.0;
        let tool = a.used_tools(Program::Drill)[0].name.unwrap();
        assert!(!a.tool_bit_unsuitable(Program::Drill, tool));
        a.tool_bits.push((Program::Drill, tool, short));
        assert!(
            a.tool_bit_unsuitable(Program::Drill, tool),
            "the flute is shorter than the board"
        );
        // a longer cutting part is suitable on the same board, and a thin board is enough for the short one
        a.tool_bits[0].2.flute_mm = 6.0;
        assert!(!a.tool_bit_unsuitable(Program::Drill, tool));
        a.tool_bits[0].2.flute_mm = 1.0;
        a.board.thickness_mm = 0.4;
        assert!(!a.tool_bit_unsuitable(Program::Drill, tool));
        a.board.thickness_mm = 1.6;
        a.act(Act::ToolDefault(Program::Drill, 0));
        assert!(!a.tool_bit_unsuitable(Program::Drill, tool));
        assert!(a.tool_bits.is_empty());
    }

    #[test]
    fn a_bit_that_the_spindle_cannot_work_with_does_not_go_to_the_tool() {
        use crate::spindle::{Fit, SpindleId};
        let mut a = app();
        a.settings.spindle = SpindleId::Original;
        let bits = a.cat_bits();
        let Some((i, _)) = bits.iter().enumerate().find(|(_, b)| {
            matches!(a.settings.current_spindle().fit(b), Fit::Incompatible(_)) && b.kind.is_drill()
        }) else {
            return;
        };
        let before = a.used_tools(Program::Drill)[0].cells.clone();
        a.act(Act::ToolFromCat(Program::Drill, 0));
        a.act(Act::CatSelect(i));
        a.act(Act::CatUse);
        assert_eq!(a.modal, Some(Modal::Catalogue));
        assert_eq!(a.used_tools(Program::Drill)[0].cells, before);
    }

    #[test]
    fn the_spindle_of_the_machine_is_chosen_and_kept() {
        use crate::spindle::SpindleId;
        let mut a = app();
        a.act(Act::SetSpindle(SpindleId::Vfd));
        assert_eq!(a.settings.current_spindle().max_rpm, 24_000);
        a.act(Act::CatSpindle(1));
        assert_eq!(a.settings.spindle, SpindleId::Original);
        a.act(Act::CatSpindle(-1));
        assert_eq!(a.settings.spindle, SpindleId::Vfd);
    }

    #[test]
    fn the_machine_of_the_3d_view_does_not_follow_the_board_and_starts_before_it_works() {
        use super::super::machine3d::Phase;
        let mut a = app();
        let rest = a.head3d().unwrap();
        assert_eq!(a.m3d.phase(), Phase::Parked);
        // moving the design in the plan view never moves the machine, with a board or without it
        a.shift_design(25.0, 17.0);
        a.shift_design(-10.0, -5.0);
        assert_eq!(a.head3d().unwrap(), rest);
        // the job: the machine powers up, goes home and travels to the first point while the work waits
        a.auto_ok = true;
        a.act(Act::Start);
        assert_eq!(a.m3d.phase(), Phase::Startup);
        let done = |a: &App| a.drill.scene.holes.iter().filter(|h| h.done).count();
        for _ in 0..40 {
            a.tick();
        }
        assert_ne!(
            a.m3d.phase(),
            Phase::Working,
            "two seconds is not enough to start and travel"
        );
        assert_eq!(done(&a), 0, "no hole is drilled while the machine travels");
        let mut moved = false;
        for _ in 0..4000 {
            a.tick();
            moved |= a.head3d().unwrap() != rest;
            if a.m3d.phase() == Phase::Working {
                break;
            }
        }
        assert_eq!(a.m3d.phase(), Phase::Working);
        assert!(moved, "the head travelled to the first point");
        // from then on the head is where the job says
        a.tick();
        let view = a.running_view().expect("the job runs");
        assert_eq!(a.head3d().unwrap(), (view.x_mm, view.y_mm));
        // moving the board in the plan view while the job runs does not make the machine jump either (its position is the job's)
        // and a stop sends the head home
        a.act(Act::Stop);
        assert_eq!(a.m3d.phase(), Phase::Returning);
        for _ in 0..4000 {
            a.tick();
        }
        assert_eq!((a.m3d.phase(), a.head3d().unwrap()), (Phase::Parked, rest));
    }

    #[test]
    fn start_goes_out_while_the_job_runs_and_comes_back_with_pause_or_stop() {
        let mut a = app();
        assert_eq!(a.job_buttons(), (true, false, false));
        a.auto_ok = true;
        a.act(Act::Start);
        assert_eq!(a.job_buttons(), (false, true, true));
        a.act(Act::Pause);
        assert_eq!(a.job_buttons(), (true, false, true));
        // Start while it is paused goes on
        a.act(Act::Start);
        assert_eq!(a.job_buttons(), (false, true, true));
        a.act(Act::Stop);
        assert_eq!(a.job_buttons(), (true, false, false));
    }

    #[test]
    fn the_emergency_stop_stops_the_job_and_blocks_the_start_until_it_is_released() {
        let mut a = app();
        a.auto_ok = true;
        a.act(Act::Start);
        assert!(a.running.is_some());
        a.act(Act::Emergency);
        assert!(a.running.is_none() && a.estop);
        assert!(!a.job_buttons().0);
        a.act(Act::Start);
        assert!(
            a.running.is_none(),
            "a job started with the emergency stop on"
        );
        a.act(Act::Emergency);
        assert!(!a.estop);
        a.act(Act::Start);
        assert!(a.running.is_some());
    }

    #[test]
    fn the_outline_of_a_gerber_board_opens_as_a_routing_job_and_a_layer_of_copper_does_not() {
        let mut a = app();
        let dir = std::env::temp_dir().join(format!("bungard-gerber-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let outline = "%FSLAX24Y24*%\n%MOMM*%\nG01*\nX0Y0D02*\nX500000Y0D01*\nX500000Y300000D01*\nX0Y300000D01*\nX0Y0D01*\nM02*\n";
        std::fs::write(dir.join("board.gko"), outline).unwrap();
        std::fs::write(dir.join("copper.gtl"), outline).unwrap();
        a.open_path(&dir.join("board.gko"));
        let (x0, y0, x1, y1) = a.route.scene.bounds().unwrap();
        assert!(((x1 - x0) - 50.0).abs() < 0.01 && ((y1 - y0) - 30.0).abs() < 0.01);
        // a copper layer is not an outline
        a.message = None;
        a.open_path(&dir.join("copper.gtl"));
        assert!(a.message.is_some());
        // the drawing of a board in DXF opens the same way
        let dxf = "0
SECTION
2
ENTITIES
0
LINE
10
0
20
0
11
40
21
0
0
LINE
10
40
20
0
11
40
21
20
0
ENDSEC
0
EOF
";
        std::fs::write(dir.join("board.dxf"), dxf).unwrap();
        a.open_path(&dir.join("board.dxf"));
        let (x0, y0, x1, y1) = a.route.scene.bounds().unwrap();
        assert!(((x1 - x0) - 40.0).abs() < 0.01 && ((y1 - y0) - 20.0).abs() < 0.01);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_circle_of_a_plotter_file_is_cut_by_the_chord_angle_of_the_configuration() {
        let mut a = app();
        let dir = std::env::temp_dir().join(format!("bungard-circle-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ring.plt"), "IN;SP1;PU2000,2000;CI400;").unwrap();
        a.open_path(&dir.join("ring.plt"));
        let segments = a.route_segments() as usize;
        assert!(segments >= 6);
        assert_eq!(a.route.scene.paths.len(), 1);
        assert_eq!(a.route.scene.paths[0].points.len(), segments + 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_holes_of_each_drill_are_counted_across_jobs_and_can_be_started_again() {
        let mut a = app();
        a.auto_ok = true;
        // a long step of time, not a faster simulation: the settings are saved and the other tests read them
        a.fixed_dt = Some(60.0);
        let holes = a.drill.scene.holes.len() as u64;
        assert!(holes > 0);
        for _ in 0..2 {
            a.act(Act::Start);
            for _ in 0..20_000 {
                if a.running.is_none() {
                    break;
                }
                a.tick();
            }
            assert!(a.running.is_none(), "the job did not end");
        }
        let total: u64 = a.settings.holes_drilled.values().sum();
        assert_eq!(total, 2 * holes);
        // what is counted stays after a restart: the settings file has it
        let back = crate::launcher::Settings::parse(&a.settings.to_text());
        assert_eq!(back.holes_drilled, a.settings.holes_drilled);
        a.act(Act::ResetHoles);
        assert!(a.settings.holes_drilled.is_empty());
    }

    #[test]
    fn the_professional_examples_open_with_their_tools_and_fit_the_table() {
        for (name, tools) in [
            ("10_mcu_board", 8),
            ("11_analog_frontend", 6),
            ("12_rf_lna_filter", 5),
            ("13_power_supply", 6),
        ] {
            let mut a = app();
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata");
            a.open_path(&dir.join(format!("{name}.ncd")));
            a.open_path(&dir.join(format!("{name}.plt")));
            assert!(a.drill.scene.holes.len() > 100, "{name}");
            assert!(!a.route.scene.paths.is_empty(), "{name}");
            let mut used: Vec<u8> = a.drill.scene.holes.iter().map(|h| h.tool).collect();
            used.sort_unstable();
            used.dedup();
            assert_eq!(used.len(), tools, "{name}");
            // the diameters are the declared ones (a via of 0.3 mm is not taken for a drill of 1 mm)
            for h in &a.drill.scene.holes {
                assert!(
                    (h.diameter_mm - h.tool as f64 / 10.0).abs() < 0.01,
                    "{name} T{}",
                    h.tool
                );
            }
            let (holes, paths) = a.drill.scene.outside_table();
            assert!(holes.is_empty() && paths.is_empty(), "{name}");
            assert!(a.route.scene.outside_table().1.is_empty(), "{name}");
        }
    }

    /// The distance from the head of the 3D view to the nearest hole (or path point) of the job as the views draw it.
    fn head_to_work(a: &App) -> f64 {
        let (hx, hy) = a.head3d().unwrap();
        let mut best = f64::MAX;
        for h in &a.drill.scene.holes {
            best = best.min((h.x_mm - hx).hypot(h.y_mm - hy));
        }
        for q in &a.route.scene.paths {
            for p in &q.points {
                best = best.min((p.0 - hx).hypot(p.1 - hy));
            }
        }
        best
    }

    fn run_until_working(a: &mut App) {
        a.auto_ok = true;
        a.fixed_dt = Some(0.05);
        a.act(Act::Start);
        for _ in 0..4000 {
            if a.m3d.phase() == super::super::machine3d::Phase::Working {
                break;
            }
            a.tick();
        }
        // a few holes in
        for _ in 0..200 {
            a.tick();
        }
    }

    #[test]
    fn the_job_works_where_the_board_was_moved_to_and_where_it_was_turned() {
        // the board moved to the middle of the table
        let mut a = app_with_both();
        a.shift_board(150.0, 100.0);
        run_until_working(&mut a);
        let d = head_to_work(&a);
        assert!(
            d < 3.0,
            "the head is {d} mm from the work as drawn after moving the board"
        );
        a.act(Act::Stop);
        // and turned
        let mut b = app_with_both();
        b.rotate_work(1);
        b.shift_board(60.0, 40.0);
        run_until_working(&mut b);
        let d = head_to_work(&b);
        assert!(
            d < 3.0,
            "the head is {d} mm from the work as drawn after turning the board"
        );
    }

    fn work_bounds(a: &App) -> (f64, f64, f64, f64) {
        let mut all = a.drill.scene.clone();
        all.overlay(&a.route.scene);
        all.bounds().unwrap()
    }

    #[test]
    fn the_work_turns_on_its_own_middle_where_it_was_moved_to_and_stays_on_the_table() {
        let mut a = app_with_both();
        a.shift_board(150.0, 100.0);
        let b0 = work_bounds(&a);
        let c0 = ((b0.0 + b0.2) / 2.0, (b0.1 + b0.3) / 2.0);
        for _ in 0..4 {
            a.rotate_work(1);
            let b = work_bounds(&a);
            let c = ((b.0 + b.2) / 2.0, (b.1 + b.3) / 2.0);
            assert!(
                (c.0 - c0.0).abs() < 0.6 && (c.1 - c0.1).abs() < 0.6,
                "the middle moved: {c0:?} -> {c:?}"
            );
            assert!(b.0 >= 0.0 && b.1 >= 0.0 && b.2 <= a.table.width_mm && b.3 <= a.table.depth_mm);
        }
        // at the edge of the table the turned work is brought back inside it
        let mut e = app_with_both();
        let w = e.table.width_mm;
        let b = work_bounds(&e);
        e.shift_board(w - b.2 - 1.0, 0.0);
        e.rotate_work(1);
        let b = work_bounds(&e);
        assert!(b.2 <= w + 1e-6 && b.0 >= 0.0, "{b:?}");
        // a board that stayed where it was does not turn with the design
        let mut f = app_with_both();
        f.toggle_move_design();
        let r0 = f.board_frozen.unwrap();
        f.rotate_work(1);
        let r1 = f.board_frozen.unwrap();
        assert_eq!(r1, r0);
    }

    #[test]
    fn a_window_is_moved_by_its_top_and_stays_on_the_canvas_and_a_new_one_starts_in_its_place() {
        let mut a = app();
        a.show_modal(Modal::Tools);
        a.modal_card = Some((430.0, 130.0, 740.0, 420.0));
        a.drag(Zone::ModalMove, (100.0, 50.0), false);
        assert_eq!(a.modal_off, (100.0, 50.0));
        // never out of the canvas: a hand of it stays in view
        a.drag(Zone::ModalMove, (50_000.0, 50_000.0), false);
        assert!(a.modal_off.0 < DESIGN_W && a.modal_off.1 < DESIGN_H);
        assert!(430.0 + a.modal_off.0 <= DESIGN_W - 160.0 + 1e-3);
        a.drag(Zone::ModalMove, (-90_000.0, -90_000.0), false);
        assert!(
            430.0 + 740.0 + a.modal_off.0 >= 160.0 - 1e-3 && 130.0 + a.modal_off.1 >= 70.0 - 1e-3
        );
        // the reset of the view puts it back, and so does opening another window
        a.reset_view(Zone::ModalMove);
        assert_eq!(a.modal_off, (0.0, 0.0));
        a.drag(Zone::ModalMove, (40.0, 40.0), false);
        a.show_modal(Modal::Catalogue);
        assert_eq!(a.modal_off, (0.0, 0.0));
    }

    fn app_with_both() -> App {
        let mut a = app();
        a.open_path(&Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/05_board_outline.plt"));
        a.auto_ok = true;
        a.fixed_dt = Some(60.0);
        a
    }

    fn run_to_the_end(a: &mut App) {
        for _ in 0..40_000 {
            if a.running.is_none() && a.job_queue.is_empty() {
                return;
            }
            a.tick();
        }
        panic!("the job did not end");
    }

    #[test]
    fn the_drilling_and_the_routing_that_are_on_run_one_after_the_other() {
        let mut a = app_with_both();
        assert!(a.drill.engine.file_loaded() && a.route.engine.file_loaded());
        a.act(Act::Start);
        assert_eq!(a.running, Some(Program::Drill));
        assert_eq!(a.job_queue, vec![Program::Route]);
        let mut routed_after_drilling = false;
        for _ in 0..40_000 {
            if a.running == Some(Program::Route) {
                routed_after_drilling = a.drill.scene.holes.iter().all(|h| h.done);
                break;
            }
            a.tick();
        }
        assert!(
            routed_after_drilling,
            "the routing starts when the drilling is done"
        );
        run_to_the_end(&mut a);
        assert!(a
            .route
            .scene
            .paths
            .iter()
            .all(|q| q.reached == q.points.len()));
    }

    #[test]
    fn only_the_routing_or_only_the_drilling_can_be_done() {
        let mut a = app_with_both();
        a.act(Act::JobPart(Program::Drill));
        a.act(Act::Start);
        assert_eq!(a.running, Some(Program::Route));
        assert!(a.job_queue.is_empty());
        run_to_the_end(&mut a);
        assert!(
            a.drill.scene.holes.iter().all(|h| !h.done),
            "the drilling was left out"
        );
        // the other way round
        a.act(Act::JobPart(Program::Drill));
        a.act(Act::JobPart(Program::Route));
        for q in a.route.scene.paths.iter_mut() {
            q.reached = 0;
        }
        a.act(Act::Start);
        assert_eq!(a.running, Some(Program::Drill));
        run_to_the_end(&mut a);
        assert!(a.route.scene.paths.iter().all(|q| q.reached == 0));
        // everything off: nothing starts and the person is told
        a.act(Act::JobPart(Program::Drill));
        a.act(Act::Start);
        assert!(a.running.is_none() && a.message.is_some());
    }

    #[test]
    fn a_tool_that_is_off_is_left_out_of_the_job() {
        let mut a = app_with_both();
        a.settings.holes_drilled.clear();
        let tools: Vec<u8> = {
            let mut v: Vec<u8> = a.drill.scene.holes.iter().map(|h| h.tool).collect();
            v.sort_unstable();
            v.dedup();
            v
        };
        assert!(tools.len() > 1);
        a.act(Act::ToolOff(Program::Drill, tools[0]));
        assert!(!a.tool_is_on(Program::Drill, tools[0]));
        a.act(Act::JobPart(Program::Route));
        a.act(Act::Start);
        run_to_the_end(&mut a);
        for h in &a.drill.scene.holes {
            assert_eq!(h.done, h.tool != tools[0], "tool {}", h.tool);
        }
        // switched on again
        a.act(Act::ToolOff(Program::Drill, tools[0]));
        assert!(a.tool_is_on(Program::Drill, tools[0]));
        // all the tools of the drilling off: the drilling has nothing to do
        for t in &tools {
            a.act(Act::ToolOff(Program::Drill, *t));
        }
        assert!(!a.program_runs(Program::Drill));
    }

    #[test]
    fn a_job_left_in_the_middle_is_offered_again_and_goes_on_from_where_it_was() {
        let mut a = app();
        let file = std::env::temp_dir().join(format!("eh3d-bg-resume-{}.cfg", std::process::id()));
        a.recovery_path = Some(file.clone());
        a.auto_ok = true;
        a.fixed_dt = Some(2.0);
        let holes = a.drill.scene.holes.len();
        assert!(holes > 4);
        a.act(Act::Start);
        // the file is there as soon as the job starts and follows the holes that are drilled
        assert!(file.exists());
        let mut done = 0;
        for _ in 0..20_000 {
            a.tick();
            done = a.drill.scene.holes.iter().filter(|h| h.done).count();
            if done >= 3 || a.running.is_none() {
                break;
            }
        }
        assert!(a.running.is_some() && done >= 3, "{done} holes");
        let saved = crate::recovery::Recovery::load(&file).expect("the file of the job");
        assert_eq!((saved.program.as_str(), saved.total), ("drill", holes));
        assert_eq!(saved.drilled.len(), done);
        // the window goes down: a new one finds the job and asks
        let mut b = app();
        b.recovery_path = Some(file.clone());
        b.offer_resume();
        assert_eq!(b.modal, Some(Modal::Resume));
        b.auto_ok = true;
        b.fixed_dt = Some(2.0);
        b.act(Act::ResumeContinue);
        assert!(b.running.is_some() && b.modal.is_none());
        // the holes that were drilled are marked and the job does only the rest
        assert_eq!(
            b.drill.scene.holes.iter().filter(|h| h.done).count(),
            saved.done
        );
        for _ in 0..20_000 {
            if b.running.is_none() {
                break;
            }
            b.tick();
        }
        assert!(b.running.is_none());
        assert!(b.drill.scene.holes.iter().all(|h| h.done));
        // a job that ended leaves nothing to go on from
        assert!(!file.exists());
        // and a job that is left can be cancelled
        let mut c = app();
        saved.save(&file);
        c.recovery_path = Some(file.clone());
        c.offer_resume();
        c.act(Act::ResumeCancel);
        assert!(c.modal.is_none() && !file.exists());
    }

    #[test]
    fn a_job_left_in_the_middle_does_not_go_on_when_the_files_or_the_settings_are_not_the_same() {
        let mut a = app();
        let file = std::env::temp_dir().join(format!("eh3d-bg-changed-{}.cfg", std::process::id()));
        a.recovery_path = Some(file.clone());
        a.auto_ok = true;
        a.fixed_dt = Some(2.0);
        a.act(Act::Start);
        let mut saved = crate::recovery::Recovery::load(&file).expect("the file of the job");
        assert!(!saved.fingerprint.is_empty());
        // the same files and settings: the fingerprint is the one of now
        let drill = saved.drill.clone();
        assert_eq!(
            saved.fingerprint,
            a.work_fingerprint(drill.as_ref(), saved.route.as_ref())
        );
        // something changed since the job stopped (a file edited, the sorting or the offset of the machine): it does not go on and says why
        saved.fingerprint = "0123456789abcdef".into();
        saved.save(&file);
        let mut b = app();
        b.recovery_path = Some(file.clone());
        b.offer_resume();
        b.act(Act::ResumeContinue);
        assert!(b.running.is_none() && b.modal.is_none());
        assert_eq!(b.message.as_deref(), Some(b.tr(Text::ResumeChanged)));
        assert!(!file.exists());
    }

    #[test]
    fn a_job_of_drilling_and_routing_left_in_the_middle_goes_on_with_the_rest_of_the_drilling_the_routing_and_the_tools_off(
    ) {
        let mut a = app_with_both();
        let file = std::env::temp_dir().join(format!("eh3d-bg-resume2-{}.cfg", std::process::id()));
        a.recovery_path = Some(file.clone());
        a.settings.holes_drilled.clear();
        a.auto_ok = true;
        a.fixed_dt = Some(2.0);
        let mut tools: Vec<u8> = a.drill.scene.holes.iter().map(|h| h.tool).collect();
        tools.sort_unstable();
        tools.dedup();
        assert!(tools.len() > 1);
        let off = tools[0];
        a.act(Act::ToolOff(Program::Drill, off));
        a.act(Act::Start);
        assert_eq!(
            (a.running, a.job_queue.clone()),
            (Some(Program::Drill), vec![Program::Route])
        );
        let mut done = 0;
        for _ in 0..40_000 {
            a.tick();
            done = a.drill.scene.holes.iter().filter(|h| h.done).count();
            if done >= 3 || a.running != Some(Program::Drill) {
                break;
            }
        }
        assert!(
            a.running == Some(Program::Drill) && done >= 3,
            "{done} holes"
        );
        let saved = crate::recovery::Recovery::load(&file).expect("the file of the job");
        assert_eq!(saved.queue, vec!["route".to_string()]);
        assert_eq!(saved.off, vec![("drill".to_string(), off)]);
        assert!(
            saved.drill.is_some() && saved.route.is_some(),
            "both files are kept"
        );
        // the window goes down: a new one goes on with the rest of the drilling, then the routing, and the tool stays off
        let mut b = app_with_both();
        b.recovery_path = Some(file.clone());
        b.settings.holes_drilled.clear();
        b.offer_resume();
        assert_eq!(b.modal, Some(Modal::Resume));
        b.auto_ok = true;
        b.fixed_dt = Some(2.0);
        b.act(Act::ResumeContinue);
        assert_eq!(
            (b.running, b.job_queue.clone()),
            (Some(Program::Drill), vec![Program::Route])
        );
        assert!(!b.tool_is_on(Program::Drill, off));
        let mut routed = false;
        for _ in 0..80_000 {
            if b.running.is_none() {
                break;
            }
            if b.running == Some(Program::Route) {
                routed = true;
            }
            b.tick();
        }
        assert!(routed && b.running.is_none());
        for h in &b.drill.scene.holes {
            assert_eq!(h.done, h.tool != off, "tool {}", h.tool);
        }
        assert!(b
            .route
            .scene
            .paths
            .iter()
            .all(|q| q.reached == q.points.len()));
        assert!(
            !file.exists(),
            "a job that ended leaves nothing to go on from"
        );
    }

    #[test]
    fn the_probe_measures_the_table_point_by_point_and_the_map_says_how_it_is_tilted() {
        let mut a = app();
        a.fixed_dt = Some(1.0);
        a.level_reset(5, 4);
        assert_eq!(a.level.map.measured(), 0);
        a.act(Act::LevelToggle);
        assert!(a.level.measuring);
        a.tick();
        a.tick();
        assert_eq!(a.level.map.measured(), 2);
        // it can be stopped and goes on from the point it was at
        a.act(Act::LevelToggle);
        a.tick();
        assert_eq!(a.level.map.measured(), 2);
        a.act(Act::LevelToggle);
        for _ in 0..40 {
            a.tick();
        }
        assert!(a.level.map.is_complete() && !a.level.measuring);
        let (tx, ty) = a.level.map.tilt_mm_per_m().unwrap();
        assert!(tx > 0.0 && ty > 0.0);
        // more points: a new empty map; the map can be cleared
        a.adjust(Field::LevCols, 2.0);
        assert_eq!(
            (a.level.cols, a.level.map.grid.cols, a.level.map.measured()),
            (7, 7, 0)
        );
        a.act(Act::LevelClear);
        assert_eq!(a.level.map.measured(), 0);
    }

    #[test]
    fn a_map_that_is_complete_corrects_the_jobs_only_when_the_person_asks() {
        let mut a = app();
        a.fixed_dt = Some(60.0);
        a.auto_ok = true;
        a.level_reset(4, 3);
        assert!(a.level_for_jobs().is_none());
        a.act(Act::LevelToggle);
        for _ in 0..30 {
            a.tick();
        }
        assert!(a.level.map.is_complete());
        // measured, but not asked for
        assert!(a.level_for_jobs().is_none());
        a.act(Act::LevelApply);
        assert!(a.level_for_jobs().is_some());
        // a job on the corrected table goes to its end like any other
        a.act(Act::Start);
        for _ in 0..20_000 {
            if a.running.is_none() {
                break;
            }
            a.tick();
        }
        assert!(a.running.is_none());
        assert!(a.drill.scene.holes.iter().all(|h| h.done));
        // a map that is not complete is never used
        a.level_clear();
        assert!(a.level_for_jobs().is_none());
    }

    #[test]
    fn the_files_opened_are_the_recent_ones_and_open_again_from_the_list() {
        let mut a = app();
        let ncd = a.settings.recent.first().cloned().unwrap();
        assert!(ncd.ends_with("05_small_board.ncd"));
        let plt = Path::new(&ncd).with_file_name("05_board_outline.plt");
        a.open_path(&plt);
        assert!(a.settings.recent[0].ends_with("05_board_outline.plt"));
        assert!(a.settings.recent[1].ends_with("05_small_board.ncd"));
        // opened again, it comes to the head and is not listed twice
        a.open_path(Path::new(&ncd));
        assert!(a.settings.recent[0].ends_with("05_small_board.ncd"));
        // the list is the user's and the tests share it: only these two files are counted, by their whole path
        let listed = |full: &str| {
            a.settings
                .recent
                .iter()
                .filter(|p| p.as_str() == full)
                .count()
        };
        assert_eq!((listed(&ncd), listed(&plt.to_string_lossy())), (1, 1));
        // a click on the second one opens it
        a.act(Act::OpenRecent(1));
        assert!(a.settings.recent[0].ends_with("05_board_outline.plt"));
    }

    #[test]
    fn a_job_that_does_not_fit_on_the_table_is_refused_before_it_starts() {
        let mut a = app();
        a.auto_ok = true;
        a.drill.scene.holes[0].x_mm = -50.0;
        a.act(Act::Start);
        assert!(a.running.is_none(), "a job outside the table was started");
        assert_eq!(
            a.message.as_deref(),
            Some(a.tr(Text::JobOutsideTable)),
            "the operator was not told"
        );
        a.drill.scene.holes[0].x_mm = 10.0;
        a.message = None;
        a.act(Act::Start);
        assert!(a.running.is_some());
    }

    #[test]
    fn the_emergency_button_is_dragged_and_stays_inside_the_window() {
        let mut a = app();
        let (x, y) = a.settings.estop_pos;
        a.drag(Zone::Estop, (30.0, -20.0), false);
        assert_eq!(a.settings.estop_pos, (x + 30.0, y - 20.0));
        a.drag(Zone::Estop, (50_000.0, 50_000.0), false);
        assert_eq!(a.settings.estop_pos, (1550.0, 850.0));
    }

    #[test]
    fn a_model_sets_the_area_and_the_tool_change_and_the_file_unit_reads_the_files_again() {
        let mut a = app();
        a.view3d.zoom = 3.0;
        a.view2d.zoom = 2.0;
        let atc = crate::machines::Model::from_name("CCD/2 ATC").unwrap();
        a.apply_model(atc);
        // both views start again for the new table
        assert_eq!(
            (a.view2d, a.view3d),
            (Default::default(), Default::default())
        );
        assert_eq!((a.table.width_mm, a.table.depth_mm), (325.0, 270.0));
        assert!(a.drill.engine.settings().unwrap().auto_change);
        let mtc = crate::machines::Model::from_name("CCD MTC").unwrap();
        a.apply_model(mtc);
        assert!(!a.drill.engine.settings().unwrap().auto_change);
        assert_eq!(a.model(), mtc);
        // the coordinates of the file are read in another unit: the board gets another size
        let before = a.drill.scene.bounds().unwrap();
        a.set_file_unit(3);
        let after = a.drill.scene.bounds().unwrap();
        assert!(after.2 < before.2);
        a.set_file_unit(1);
        assert_eq!(a.drill.scene.bounds().unwrap(), before);
    }

    #[test]
    fn lengths_follow_the_units_on_the_screen() {
        let mut a = app();
        assert_eq!(a.len_text(25.4), "25.4");
        a.settings.display_mil = true;
        assert_eq!(a.len_text(25.4), "1000");
        assert!((a.len_from_display(1000.0) - 25.4).abs() < 1e-9);
        a.start_edit(EditTarget::Field(Field::TableW), 0.0, 0.0, 100.0);
        a.edit.as_mut().unwrap().text = "20000".into();
        a.commit_edit();
        assert!((a.table.width_mm - 508.0).abs() < 0.01);
    }

    #[test]
    fn the_library_is_big_and_the_list_of_a_maker_scrolls() {
        assert!(
            crate::boards::MATERIALS.len() >= 100,
            "{} materials",
            crate::boards::MATERIALS.len()
        );
        assert!(crate::boards::makers().len() >= 18);
        let mut a = app();
        let rogers = crate::boards::makers()
            .iter()
            .position(|m| *m == "Rogers")
            .unwrap()
            + 1; // the list of makers starts with "all makers"
        a.board_pick(super::super::board::BoardPick::Maker(rogers));
        a.mat_scroll_by(1000);
        let n = crate::boards::of_maker("Rogers").len();
        assert_eq!(a.mat_scroll, n - super::super::board::MAT_ROWS);
        a.mat_scroll_by(-1000);
        assert_eq!(a.mat_scroll, 0);
    }

    #[test]
    fn the_list_of_makers_starts_with_all_of_them_and_a_board_is_chosen_the_same_from_it() {
        use super::super::board::BoardPick;
        let mut a = app();
        let items = a.combo_items(super::super::combo::ComboId::Maker);
        assert_eq!(items[0], a.tr(Text::AllMakers));
        assert_eq!(items.len(), crate::boards::makers().len() + 1);
        a.board_pick(BoardPick::Maker(0));
        assert!(a.settings.all_makers);
        let all = a.materials_shown();
        assert_eq!(all.len(), crate::boards::MATERIALS.len());
        // the same board comes out of the view of all the makers as out of the view of its maker
        let (place, m) = all[all.len() / 2];
        a.board_pick(BoardPick::Material(place));
        let from_all = a.board;
        let maker = crate::boards::makers()
            .iter()
            .position(|k| *k == m.maker)
            .unwrap();
        a.board_pick(BoardPick::Maker(maker + 1));
        assert!(!a.settings.all_makers);
        assert!(a.materials_shown().iter().any(|(i, _)| *i == place));
        a.board_pick(BoardPick::Material(place));
        assert_eq!(a.board, from_all);
    }

    #[test]
    fn a_board_bigger_than_the_table_is_warned_about_and_the_warning_follows_the_machine() {
        let mut a = app();
        a.board.size_mm = Some((200.0, 100.0));
        assert!(a.board_fit_warning().is_none());
        // bigger than the table in both ways, even turned
        a.board.size_mm = Some((a.table.width_mm + 30.0, a.table.depth_mm + 30.0));
        let (head, detail) = a.board_fit_warning().expect("no warning");
        assert_eq!(head, a.tr(Text::BoardTooBig));
        assert!(detail.contains("30.0"), "{detail}");
        // a bigger table takes the warning away
        a.table.width_mm += 100.0;
        a.table.depth_mm += 100.0;
        assert!(a.board_fit_warning().is_none());
    }

    #[test]
    fn the_sounds_are_switched_typed_and_written_to_the_machine_file() {
        use eh3d_drillpro::protocol::sound::{Cue, SoundSetting};
        let mut a = app();
        assert_eq!(a.conn.sound_finish, SoundSetting::Off);
        a.sound_choose(Cue::Finished, SoundSetting::Default);
        assert_eq!(a.conn.sound_finish, SoundSetting::Default);
        // typing a path while it is off does nothing; while it is on it becomes the file
        a.sound_set_path(Cue::ToolChange, "sounds/x.wav");
        assert_eq!(a.conn.sound_tool_change, SoundSetting::Off);
        a.sound_choose(Cue::ToolChange, SoundSetting::Default);
        a.sound_set_path(Cue::ToolChange, " sounds/x.wav ");
        assert_eq!(
            a.conn.sound_tool_change,
            SoundSetting::File(PathBuf::from("sounds/x.wav"))
        );
        a.sound_set_path(Cue::ToolChange, "");
        assert_eq!(a.conn.sound_tool_change, SoundSetting::Default);
        let text = crate::link::update("", &crate::link::pairs(&a.conn));
        assert!(
            text.contains("sound_finish = default") && text.contains("sound_tool_change = default")
        );
    }

    /// A server that answers one request with the fleet and goes away.
    fn fleet_server(body: &'static str) -> String {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap().to_string();
        std::thread::spawn(move || {
            if let Ok((mut s, _)) = listener.accept() {
                let mut buf = [0u8; 2048];
                let _ = s.read(&mut buf);
                let reply = format!(
                    "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",
                    body.len()
                );
                let _ = s.write_all(reply.as_bytes());
            }
        });
        address
    }

    #[test]
    fn the_fleet_window_asks_the_server_and_chooses_the_machine_of_the_jobs() {
        let mut a = app();
        // the machine file of the test is its own: choosing a machine writes it, and it must never be the file of the user
        a.conn_path =
            std::env::temp_dir().join(format!("eh3d-bg-fleet-{}.cfg", std::process::id()));
        // the window says why there is nothing to show when the connection is not the server
        a.conn.kind = LinkKind::Demo;
        a.show_modal(Modal::Fleet);
        a.fleet_tick();
        assert!(a.fleet.fleet.is_none() && a.fleet.rx.is_none());

        a.conn.kind = LinkKind::Server;
        a.conn.server_address = fleet_server(
            r#"{"stations":[{"id":"bench","machines":["m1","m2"]}],"machines":[
            {"id":"m1","name":"M1","station":"bench","connection":"demo","state":"busy","job":{"id":"j","name":"board","kind":"drill","items_done":1,"items_total":4,"percent":25},"ready":0},
            {"id":"m2","name":"M2","station":"bench","connection":"demo","state":"idle","job":null,"ready":2}]}"#,
        );
        a.fleet_tick();
        assert!(a.fleet.rx.is_some());
        // the answer comes from another thread (up to ten seconds: a busy machine can be slow to run it)
        for _ in 0..1000 {
            a.fleet_tick();
            if a.fleet.fleet.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let fleet = a
            .fleet
            .fleet
            .as_ref()
            .unwrap_or_else(|| panic!("the fleet was read: {:?}", a.fleet.error));
        assert_eq!(fleet.machines.len(), 2);
        assert!(a.fleet.error.is_none());

        // the second machine becomes the one the jobs go to
        a.fleet_use(1);
        assert_eq!(a.conn.server_machine, "m2");
        // an index that is not in the list changes nothing
        a.fleet_use(9);
        assert_eq!(a.conn.server_machine, "m2");
    }

    #[test]
    fn a_spoken_voice_is_in_the_list_and_is_chosen_for_each_event() {
        use super::super::combo::ComboId;
        use eh3d_drillpro::protocol::sound::{Cue, SoundSetting};
        let mut a = app();
        // the voices of the folder of the repository are found from the build folder
        let voices = a.voices();
        assert!(voices.len() >= 10, "{voices:?}");
        assert!(voices.contains(&("es", true)) && voices.contains(&("de", false)));
        let items = a.combo_items(ComboId::Sound(Cue::Finished));
        assert!(
            items[3].contains(" - ") && items[3].contains('('),
            "{}",
            items[3]
        );
        // the voice that is chosen is kept in the machine file as `voice:LANG:f|m`
        a.combo_open(ComboId::Sound(Cue::ToolChange));
        a.combo_pick(3);
        let (lang, female) = voices[0];
        assert_eq!(
            a.conn.sound_tool_change,
            SoundSetting::Voice {
                lang: lang.to_string(),
                female
            }
        );
        assert_eq!(a.combo_current(ComboId::Sound(Cue::ToolChange)), 3);
        assert!(a.conn.sound_tool_change.to_text().starts_with("voice:"));
        // the other event has its own voice
        a.combo_open(ComboId::Sound(Cue::Finished));
        a.combo_pick(4);
        assert_ne!(a.conn.sound_finish, a.conn.sound_tool_change);
        assert_eq!(a.combo_current(ComboId::Sound(Cue::Finished)), 4);
    }

    #[test]
    fn a_drop_down_list_opens_scrolls_and_chooses() {
        use super::super::combo::{ComboId, ROWS};
        use eh3d_drillpro::protocol::sound::{Cue, SoundSetting, TONES};
        let mut a = app();
        // the list of a sound has sixty sounds of the catalogue besides none, the standard tone and a file of the person's
        let items = a.combo_items(ComboId::Sound(Cue::Finished));
        // the spoken voices that are recorded come after the three fixed rows and before the sounds
        let v = a.voices().len();
        assert_eq!(items.len(), TONES + 3 + v);
        assert!(items.len() >= 50);
        assert!(
            items[3 + v].starts_with("01 ") && items[3 + v].contains("C7"),
            "{}",
            items[3 + v]
        );
        assert!(
            items[TONES + 2 + v].contains("C3"),
            "{}",
            items[TONES + 2 + v]
        );
        let tools = a.combo_items(ComboId::Sound(Cue::ToolChange));
        assert_ne!(items[5 + v], tools[5 + v]);
        // it opens with the chosen row in view, scrolls inside its limits and a second click on the box closes it
        a.combo_open(ComboId::Sound(Cue::Finished));
        assert_eq!(a.combo.unwrap().scroll, 0);
        a.combo_scroll(1000);
        assert_eq!(a.combo.unwrap().scroll, items.len() - ROWS);
        a.combo_scroll(-5000);
        assert_eq!(a.combo.unwrap().scroll, 0);
        a.combo_open(ComboId::Sound(Cue::Finished));
        assert!(a.combo.is_none());
        // a row is chosen: the list closes, the sound is the one of the catalogue and the choice is in the connection
        a.combo_open(ComboId::Sound(Cue::ToolChange));
        a.combo_pick(3 + v + 17);
        assert!(a.combo.is_none());
        assert_eq!(a.conn.sound_tool_change, SoundSetting::Tone(17));
        assert_eq!(a.combo_current(ComboId::Sound(Cue::ToolChange)), 20 + v);
        // the two events keep their own sound
        assert_eq!(a.conn.sound_finish, SoundSetting::Off);
        a.combo_open(ComboId::Sound(Cue::Finished));
        a.combo_pick(1);
        assert_eq!(a.conn.sound_finish, SoundSetting::Default);
        a.combo_open(ComboId::Sound(Cue::Finished));
        a.combo_pick(0);
        assert_eq!(a.conn.sound_finish, SoundSetting::Off);
        // the theme, the language and the model are chosen from their lists too
        a.combo_open(ComboId::Theme);
        a.combo_pick(2);
        assert_eq!(a.combo_current(ComboId::Theme), 2);
        a.combo_open(ComboId::Lang);
        a.combo_pick(1);
        assert_eq!(a.combo_current(ComboId::Lang), 1);
        let models = crate::machines::models().len();
        a.combo_open(ComboId::Model);
        a.combo_pick(models - 1);
        assert_eq!(a.model(), models - 1);
        // a row that is not in the list does nothing
        a.combo_open(ComboId::Model);
        a.combo_pick(999);
        assert_eq!(a.model(), models - 1);
    }
}
