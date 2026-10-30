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
        self.text(
            x + w / 2.0,
            y + h / 2.0 + 6.0,
            Align2::CENTER_BOTTOM,
            &label.to_uppercase(),
            17.0,
            alpha(col(self.t.text), 90),
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
        self.p.rect_filled(boxr, r, fill);
        self.p.rect_stroke(
            boxr,
            r,
            Stroke::new(
                (1.2 + 0.8 * h) * self.u,
                alpha(accent, (170.0 + 85.0 * h) as u8),
            ),
            StrokeKind::Inside,
        );
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
        self.text(
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
        }
    }

    /// Switches a sound on (the default tone) or off.
    pub(crate) fn sound_toggle(&mut self, cue: Cue) {
        let s = self.sound_mut(cue);
        *s = if s.is_on() {
            SoundSetting::Off
        } else {
            SoundSetting::Default
        };
        self.saved = false;
        self.apply_conn();
    }

    /// The path typed for a sound: empty is the default tone.
    pub(crate) fn sound_set_path(&mut self, cue: Cue, text: &str) {
        let t = text.trim();
        let s = self.sound_mut(cue);
        if s.is_on() {
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
        };
        eh3d_drillpro::protocol::sound::play(s, cue);
    }

    /// The boxes of the paths of the sounds, over the Job tab of the settings.
    pub(super) fn sound_fields(&mut self, ctx: &egui::Context) {
        let u = self.ui_scale;
        let (x, y) = (380.0, 30.0 + 120.0);
        for (i, cue) in [Cue::Finished, Cue::ToolChange].into_iter().enumerate() {
            let s = match cue {
                Cue::Finished => self.conn.sound_finish.clone(),
                Cue::ToolChange => self.conn.sound_tool_change.clone(),
            };
            if !s.is_on() {
                continue;
            }
            let mut text = match &s {
                SoundSetting::File(p) => p.display().to_string(),
                _ => String::new(),
            };
            let before = text.clone();
            let pos = Pos2::new(
                self.ui_origin.x + (x + 60.0) * u,
                self.ui_origin.y + (y + 206.0 + i as f32 * 96.0) * u,
            );
            egui::Area::new(egui::Id::unique(("sound-path", i)))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    ui.add(
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
        cv.text(
            ex,
            ey + 6.0,
            Align2::CENTER_BOTTOM,
            "STOP",
            15.0,
            Color32::WHITE,
        );
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
        let atc = crate::machines::Model::from_name("CCD/2 ATC").unwrap();
        a.apply_model(atc);
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
            .unwrap();
        a.board_pick(super::super::board::BoardPick::Maker(rogers));
        a.mat_scroll_by(1000);
        let n = crate::boards::of_maker("Rogers").len();
        assert_eq!(a.mat_scroll, n - super::super::board::MAT_ROWS);
        a.mat_scroll_by(-1000);
        assert_eq!(a.mat_scroll, 0);
    }

    #[test]
    fn the_sounds_are_switched_typed_and_written_to_the_machine_file() {
        use eh3d_drillpro::protocol::sound::{Cue, SoundSetting};
        let mut a = app();
        assert_eq!(a.conn.sound_finish, SoundSetting::Off);
        a.sound_toggle(Cue::Finished);
        assert_eq!(a.conn.sound_finish, SoundSetting::Default);
        // typing a path while it is off does nothing; while it is on it becomes the file
        a.sound_set_path(Cue::ToolChange, "sounds/x.wav");
        assert_eq!(a.conn.sound_tool_change, SoundSetting::Off);
        a.sound_toggle(Cue::ToolChange);
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
}
