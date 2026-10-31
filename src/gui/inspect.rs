//! The window of the quality check: after a job the camera of the drill head looks at the board, and the analyser of the shop says for each hole whether it is there, round, the
//! right size and in place, whether the milled channels are as wide as the tool and even, and gives one verdict that is big enough to read from the other side of the room: OK,
//! NOT OK, or "cannot judge" when the picture was too poor. The ring of LEDs around the camera says the same in colour. The tolerances live in a second window, and the whole
//! profile is kept in a file (and sent to the server of the machine, which keeps its own for the screen next to it).

use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;

use eh3d_aux::LightRequest;
use eh3d_drillpro::protocol::connection::HeadNode;
use eh3d_inspect::{Profile, Report, Verdict};

use super::*;
use crate::inspection::{self, Param, LIGHT_CHOICES};

pub(crate) struct InspectState {
    pub profile: Profile,
    loaded: bool,
    pub report: Option<Result<Report, String>>,
    rx: Option<Receiver<Result<Report, String>>>,
    pub running: bool,
    /// The hole of the job that is in the centre of the picture.
    pub hole: usize,
    /// Which of `LIGHT_CHOICES` is used for the picture.
    pub light: usize,
    /// When the profile was last saved, to say so for a moment.
    saved_at: Option<Instant>,
    /// The size of the last picture, to know which holes the camera sees.
    pub picture: (usize, usize),
    light_rx: Option<Receiver<Result<(), String>>>,
}

impl Default for InspectState {
    fn default() -> Self {
        InspectState {
            profile: Profile {
                px_per_mm: inspection::DEFAULT_PX_PER_MM,
                ..Profile::default()
            },
            loaded: false,
            report: None,
            rx: None,
            running: false,
            hole: 0,
            light: 2,
            saved_at: None,
            picture: (0, 0),
            light_rx: None,
        }
    }
}

const AMBER: Color32 = Color32::from_rgb(232, 164, 32);

impl App {
    fn insp_profile_path(&self) -> PathBuf {
        // beside the settings of the program (the folder of the program itself may not be writable)
        self.settings_path
            .parent()
            .unwrap_or(&self.base_dir)
            .join("inspect_profile.json")
    }

    /// The holes of the drill job, as (x, y, width) in the table's coordinates.
    fn insp_job_holes(&self) -> Vec<(f64, f64, f64)> {
        self.drill
            .scene
            .holes
            .iter()
            .map(|h| (h.x_mm, h.y_mm, h.diameter_mm))
            .collect()
    }

    pub(super) fn insp_open(&mut self) {
        if !self.insp.loaded {
            self.insp.profile = inspection::load_profile(&self.insp_profile_path());
            self.insp.loaded = true;
        }
        let n = self.insp_job_holes().len();
        if self.insp.hole >= n {
            self.insp.hole = 0;
        }
        self.show_modal(Modal::Inspect);
    }

    /// The size of the picture in mm, for the holes the camera sees.
    fn insp_fov(&self) -> (f64, f64) {
        let (w, h) = if self.insp.picture.0 > 0 {
            (self.insp.picture.0 as f64, self.insp.picture.1 as f64)
        } else {
            inspection::DEFAULT_PICTURE
        };
        let ppm = self.insp.profile.px_per_mm.max(1.0);
        (w / ppm, h / ppm)
    }

    pub(super) fn insp_run(&mut self) {
        if self.insp.running {
            return;
        }
        let Some(link) = self.head_link() else {
            self.insp.report = Some(Err(self.tr(Text::InspNeedLink).to_string()));
            return;
        };
        let profile = self.insp.profile.clone();
        let holes = if profile.check_holes {
            inspection::holes_in_view(&self.insp_job_holes(), self.insp.hole, self.insp_fov())
        } else {
            Vec::new()
        };
        let light = LIGHT_CHOICES[self.insp.light.min(LIGHT_CHOICES.len() - 1)];
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(inspection::run_check(&link, &profile, &holes, light));
        });
        self.insp.rx = Some(rx);
        self.insp.running = true;
        self.insp.report = None;
    }

    /// The ring of LEDs: off (`None`) or on in the white of the camera at a level.
    pub(super) fn head_light_set(&mut self, level: Option<u32>) {
        let Some(link) = self.head_link() else { return };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let r = match level {
                Some(l) => link.set_light(&LightRequest::camera(l)),
                None => link.set_light(&LightRequest::off()),
            };
            let _ = tx.send(r.map(|_| ()));
        });
        self.insp.light_rx = Some(rx);
        // the status says how it is now: ask again at once
        self.head.polled = None;
    }

    pub(super) fn insp_tick(&mut self) {
        if let Some(r) = self.insp.rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            if let Ok(rep) = &r {
                self.insp.picture = (rep.width_px, rep.height_px);
                // the verdict is said as the person set it in the settings (off, a beep, a file, or a voice: "the visual check of the board is correct / not correct")
                self.insp_say(rep.verdict);
            }
            self.insp.report = Some(r);
            self.insp.rx = None;
            self.insp.running = false;
        }
        if self
            .insp
            .light_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok())
            .is_some()
        {
            self.insp.light_rx = None;
            self.head.polled = None;
        }
    }

    /// The sound of a verdict: the one chosen for a check that is correct or not correct; "cannot judge" has none of its own, because it is neither.
    fn insp_say(&self, verdict: Verdict) {
        let (setting, cue) = match verdict {
            Verdict::Ok => (
                &self.conn.sound_check_ok,
                eh3d_drillpro::protocol::sound::Cue::CheckOk,
            ),
            Verdict::NotOk => (
                &self.conn.sound_check_bad,
                eh3d_drillpro::protocol::sound::Cue::CheckNotOk,
            ),
            Verdict::Unknown => return,
        };
        eh3d_drillpro::protocol::sound::play(setting, cue);
    }

    pub(super) fn insp_act(&mut self, a: Act) {
        match a {
            Act::HeadLight(level) => self.head_light_set(level),
            Act::InspOpen => self.insp_open(),
            Act::InspToggleHoles => {
                self.insp.profile.check_holes = !self.insp.profile.check_holes;
                self.insp.report = None;
            }
            Act::InspToggleMilling => {
                self.insp.profile.check_milling = !self.insp.profile.check_milling;
                self.insp.report = None;
            }
            Act::InspPreset(n) => {
                self.insp.profile = inspection::preset_for(n, &self.insp.profile);
                self.insp.report = None;
            }
            Act::InspLight(i) => self.insp.light = i.min(LIGHT_CHOICES.len() - 1),
            Act::InspHole(d) => {
                let n = self.insp_job_holes().len();
                if n > 0 {
                    self.insp.hole =
                        (self.insp.hole as i64 + d as i64).rem_euclid(n as i64) as usize;
                    self.insp.report = None;
                }
            }
            Act::InspRun => self.insp_run(),
            Act::InspCalibrate => {
                if let Some(Ok(r)) = &self.insp.report {
                    if let Some(s) = inspection::calibrated_scale(r, &self.insp.profile) {
                        self.insp.profile.px_per_mm = (s * 10.0).round() / 10.0;
                        self.insp.report = None;
                    }
                }
            }
            Act::InspScale(d) => {
                let p = &mut self.insp.profile;
                p.px_per_mm = (p.px_per_mm + d as f64).clamp(1.0, 2000.0);
                self.insp.report = None;
            }
            Act::InspTune(param, dir) => {
                param.nudge(&mut self.insp.profile, dir);
                self.insp.report = None;
            }
            Act::InspSave => self.insp_save(),
            _ => {}
        }
    }

    /// Keeps the profile in its file, and gives it to the server of the machine when the node is reached through it, so the screen next to the machine checks with the same one.
    fn insp_save(&mut self) {
        let _ = inspection::save_profile(&self.insp_profile_path(), &self.insp.profile);
        self.insp.saved_at = Some(Instant::now());
        if self.conn.head_node == HeadNode::Server {
            if let Some(link) = self.head_link() {
                let body = self.insp.profile.to_json().to_string();
                std::thread::spawn(move || {
                    let _ = link.put_inspect_profile(&body);
                });
            }
        }
    }

    // ------------------------------------------------------------------------------------------------------------------------------ painting

    /// The text of a finding with the number of the hole it is about.
    fn insp_finding_line(&self, code: &str, hole: Option<usize>) -> String {
        let words =
            inspection::finding_text(code).map_or(code.to_string(), |t| self.tr(t).to_string());
        match hole {
            Some(h) => format!("#{}: {words}", h + 1),
            None => words,
        }
    }

    pub(super) fn paint_inspect(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y) = self.modal_card(cv, self.tr(Text::InspTitle), 740.0);
        // the picture of the camera, with a mark on every hole that was looked at
        let (px, py, pw, ph) = (x + 30.0, y + 76.0, 420.0, 315.0);
        cv.p.rect_filled(
            cv.rect(px, py, pw, ph),
            4.0 * cv.u,
            alpha(col(t.panel_raised), 255),
        );
        match &self.head.texture {
            Some(tex) => {
                cv.p.image(
                    tex.id(),
                    cv.rect(px, py, pw, ph),
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                let s = if self.head_link().is_some() {
                    self.tr(Text::HeadWaiting)
                } else {
                    self.tr(Text::InspNeedLink)
                };
                cv.text(
                    px + pw / 2.0,
                    py + ph / 2.0,
                    Align2::CENTER_CENTER,
                    s,
                    16.0,
                    col(t.dim),
                );
            }
        }
        if let Some(Ok(rep)) = &self.insp.report {
            let k = pw / rep.width_px.max(1) as f32;
            let ppm = self.insp.profile.px_per_mm as f32;
            for h in &rep.holes {
                // a place the camera could not see is not a failure of the work: amber, without a cross
                let unseen = h.issues == [eh3d_inspect::HoleIssue::OutOfView];
                let colour = if h.ok() {
                    col(t.ok)
                } else if unseen {
                    AMBER
                } else {
                    col(t.danger)
                };
                let (cx, cy) = if h.found {
                    (px + h.cx_px as f32 * k, py + h.cy_px as f32 * k)
                } else {
                    (
                        px + (rep.width_px as f32 / 2.0 + h.expected.x_mm as f32 * ppm) * k,
                        py + (rep.height_px as f32 / 2.0 - h.expected.y_mm as f32 * ppm) * k,
                    )
                };
                let r = ((h.expected.dia_mm as f32 / 2.0 * ppm * k) + 5.0).max(7.0);
                if cx < px || cy < py || cx > px + pw || cy > py + ph {
                    continue;
                }
                cv.p.circle_stroke(cv.pt(cx, cy), r * cv.u, Stroke::new(2.5 * cv.u, colour));
                if !h.found && !unseen {
                    cv.line((cx - r, cy - r), (cx + r, cy + r), 2.5, colour);
                    cv.line((cx - r, cy + r), (cx + r, cy - r), 2.5, colour);
                }
                cv.text(
                    cx + r + 3.0,
                    cy - r,
                    Align2::LEFT_TOP,
                    &format!("{}", h.index + 1),
                    13.0,
                    colour,
                );
            }
        }
        // the verdict
        let (rx, mut ry) = (x + 470.0, y + 76.0);
        let (fill, word) = if self.insp.running {
            (col(t.panel_raised), self.tr(Text::InspRunning))
        } else {
            match &self.insp.report {
                Some(Ok(r)) => match r.verdict {
                    Verdict::Ok => (col(t.ok), self.tr(Text::InspVerdictOk)),
                    Verdict::NotOk => (col(t.danger), self.tr(Text::InspVerdictNotOk)),
                    Verdict::Unknown => (AMBER, self.tr(Text::InspVerdictUnknown)),
                },
                Some(Err(_)) => (col(t.danger), self.tr(Text::InspVerdictNotOk)),
                None => (col(t.panel_raised), self.tr(Text::InspIdle)),
            }
        };
        cv.p.rect_filled(cv.rect(rx, ry, 240.0, 80.0), 10.0 * cv.u, fill);
        let size = if word.chars().count() > 12 {
            20.0
        } else {
            30.0
        };
        cv.text(
            rx + 120.0,
            ry + 40.0,
            Align2::CENTER_CENTER,
            word,
            size,
            Color32::WHITE,
        );
        ry += 96.0;
        match &self.insp.report {
            Some(Ok(r)) => {
                if !r.holes.is_empty() {
                    let ok = r.holes_ok();
                    let c = if ok == r.holes.len() {
                        col(t.ok)
                    } else {
                        col(t.danger)
                    };
                    cv.text(
                        rx,
                        ry,
                        Align2::LEFT_TOP,
                        &format!("{ok}/{} {}", r.holes.len(), self.tr(Text::InspHolesOk)),
                        17.0,
                        c,
                    );
                    ry += 24.0;
                }
                if let Some(m) = &r.milling {
                    if m.samples > 0 {
                        let c = if m.ok() { col(t.ok) } else { col(t.danger) };
                        cv.text(
                            rx,
                            ry,
                            Align2::LEFT_TOP,
                            &format!("{} {:.2} mm", self.tr(Text::InspChannel), m.width_mm),
                            16.0,
                            c,
                        );
                        ry += 24.0;
                    }
                }
                for f in r.findings.iter().take(5) {
                    let line = self.insp_finding_line(f.code, f.hole);
                    // a long finding goes on a second line (the column is narrow), never off the window
                    for part in super::extra::wrap(&line, 31).into_iter().take(2) {
                        if ry > y + 314.0 {
                            break;
                        }
                        cv.text(
                            rx,
                            ry,
                            Align2::LEFT_TOP,
                            &part,
                            14.0,
                            if f.fails { col(t.danger) } else { AMBER },
                        );
                        ry += 19.0;
                    }
                }
            }
            Some(Err(e)) => {
                let e: String = e.chars().take(60).collect();
                cv.text(rx, ry, Align2::LEFT_TOP, &e, 14.0, col(t.danger));
            }
            None => {}
        }
        // the scale, and calibrating it from the hole in the centre
        cv.text(
            rx,
            y + 346.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::InspScale),
            14.0,
            col(t.dim),
        );
        for (i, (label, d)) in [("-5", -5.0f32), ("-1", -1.0), ("+1", 1.0), ("+5", 5.0)]
            .into_iter()
            .enumerate()
        {
            cv.chip(
                rx + i as f32 * 61.0,
                y + 352.0,
                55.0,
                32.0,
                label,
                false,
                Act::InspScale(d),
            );
        }
        cv.text(
            rx + 240.0,
            y + 346.0,
            Align2::RIGHT_BOTTOM,
            &format!("{:.1}", self.insp.profile.px_per_mm),
            17.0,
            col(t.accent),
        );
        cv.chip(
            rx,
            y + 392.0,
            240.0,
            32.0,
            self.tr(Text::InspCalibrate),
            false,
            Act::InspCalibrate,
        );

        // the checks
        cv.text(
            x + 30.0,
            y + 438.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::InspChecks),
            15.0,
            col(t.dim),
        );
        cv.chip(
            x + 30.0,
            y + 446.0,
            210.0,
            36.0,
            self.tr(Text::InspCheckHoles),
            self.insp.profile.check_holes,
            Act::InspToggleHoles,
        );
        cv.chip(
            x + 250.0,
            y + 446.0,
            210.0,
            36.0,
            self.tr(Text::InspCheckMilling),
            self.insp.profile.check_milling,
            Act::InspToggleMilling,
        );
        // the profile of the shop
        cv.text(
            x + 30.0,
            y + 508.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::InspProfile),
            15.0,
            col(t.dim),
        );
        for (i, label) in [
            Text::InspPresetStandard,
            Text::InspPresetFine,
            Text::InspPresetProto,
        ]
        .into_iter()
        .enumerate()
        {
            let on = self.insp.profile.name == Profile::presets()[i];
            cv.chip(
                x + 30.0 + i as f32 * 150.0,
                y + 516.0,
                142.0,
                36.0,
                self.tr(label),
                on,
                Act::InspPreset(i),
            );
        }
        cv.chip(
            x + 490.0,
            y + 516.0,
            220.0,
            36.0,
            self.tr(Text::InspTune),
            false,
            Act::Show(Modal::InspectTune),
        );
        // the light for the picture
        cv.text(
            x + 30.0,
            y + 578.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::InspLight),
            15.0,
            col(t.dim),
        );
        for (i, level) in LIGHT_CHOICES.into_iter().enumerate() {
            let label = match level {
                Some(l) => format!("{l} %"),
                None => self.tr(Text::InspLightKeep).to_string(),
            };
            cv.chip(
                x + 30.0 + i as f32 * 150.0,
                y + 586.0,
                142.0,
                36.0,
                &label,
                self.insp.light == i,
                Act::InspLight(i),
            );
        }
        // the hole of the job that is in the centre of the picture
        let holes = self.insp_job_holes();
        if holes.is_empty() {
            cv.text(
                x + 30.0,
                y + 664.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::InspNoJob),
                15.0,
                col(t.dim),
            );
        } else {
            let h = holes[self.insp.hole.min(holes.len() - 1)];
            let line = format!(
                "{} #{}/{}   X {:.2}  Y {:.2}  \u{d8} {:.2}",
                self.tr(Text::InspHolePick),
                self.insp.hole + 1,
                holes.len(),
                h.0,
                h.1,
                h.2
            );
            cv.text(
                x + 30.0,
                y + 664.0,
                Align2::LEFT_BOTTOM,
                &line,
                15.0,
                col(t.text),
            );
            cv.icon_chip(
                x + 600.0,
                y + 636.0,
                50.0,
                34.0,
                Icon::Back,
                false,
                "",
                Act::InspHole(-1),
            );
            cv.icon_chip(
                x + 660.0,
                y + 636.0,
                50.0,
                34.0,
                Icon::Next,
                false,
                "",
                Act::InspHole(1),
            );
        }
        // the buttons
        let busy = self.insp.running || self.head_link().is_none();
        if busy {
            cv.button_off(
                x + 30.0,
                y + 686.0,
                220.0,
                44.0,
                self.tr(if self.insp.running {
                    Text::InspRunning
                } else {
                    Text::InspRun
                }),
                col(t.dim),
            );
        } else {
            cv.chip(
                x + 30.0,
                y + 686.0,
                220.0,
                44.0,
                self.tr(Text::InspRun),
                true,
                Act::InspRun,
            );
        }
        let saved = self
            .insp
            .saved_at
            .is_some_and(|s| s.elapsed().as_secs() < 2);
        cv.chip(
            x + 270.0,
            y + 686.0,
            240.0,
            44.0,
            if saved {
                "\u{2713}"
            } else {
                self.tr(Text::InspSave)
            },
            saved,
            Act::InspSave,
        );
        cv.chip(
            x + 530.0,
            y + 686.0,
            180.0,
            44.0,
            self.tr(Text::Close),
            false,
            Act::CloseModal,
        );
    }

    pub(super) fn paint_inspect_tune(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y) = self.modal_card(cv, self.tr(Text::InspTune), 640.0);
        for (i, param) in Param::ALL.into_iter().enumerate() {
            let ry = y + 80.0 + i as f32 * 50.0;
            cv.text(
                x + 30.0,
                ry + 28.0,
                Align2::LEFT_BOTTOM,
                self.tr(param.label()),
                16.0,
                col(t.text),
            );
            cv.chip(
                x + 480.0,
                ry,
                54.0,
                36.0,
                "-",
                false,
                Act::InspTune(param, -1),
            );
            cv.text(
                x + 600.0,
                ry + 28.0,
                Align2::CENTER_BOTTOM,
                &param.show(&self.insp.profile),
                17.0,
                col(t.accent),
            );
            cv.chip(
                x + 666.0,
                ry,
                44.0,
                36.0,
                "+",
                false,
                Act::InspTune(param, 1),
            );
        }
        cv.chip(
            x + 530.0,
            y + 584.0,
            180.0,
            44.0,
            self.tr(Text::Close),
            true,
            Act::Show(Modal::Inspect),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(&crate::cli::Options {
            data: Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")),
            screen: Some(crate::cli::Screen::Viewer),
            ..Default::default()
        })
    }

    #[test]
    fn the_window_changes_the_profile_and_the_hole_in_the_centre() {
        let mut a = app();
        a.insp.loaded = true;
        a.act(Act::InspToggleMilling);
        assert!(!a.insp.profile.check_milling);
        a.act(Act::InspPreset(1));
        assert_eq!(a.insp.profile.name, "fine_pitch");
        assert!(
            !a.insp.profile.check_milling,
            "a preset keeps which checks are on"
        );
        let before = a.insp.profile.px_per_mm;
        a.act(Act::InspScale(5.0));
        assert_eq!(a.insp.profile.px_per_mm, before + 5.0);
        a.act(Act::InspScale(-5000.0));
        assert_eq!(a.insp.profile.px_per_mm, 1.0);
        a.act(Act::InspTune(Param::HolePos, 1));
        assert!(
            a.insp.profile.hole_pos_tol_mm > Profile::preset("fine_pitch").unwrap().hole_pos_tol_mm
        );
        a.act(Act::InspLight(3));
        assert_eq!(a.insp.light, 3);
        a.act(Act::InspLight(99));
        assert_eq!(a.insp.light, LIGHT_CHOICES.len() - 1);
        // with no drill job there is no hole to choose
        a.act(Act::InspHole(1));
        assert_eq!(a.insp.hole, 0);
    }

    #[test]
    fn the_sounds_of_the_visual_check_are_chosen_like_the_other_sounds() {
        use super::super::combo::ComboId;
        use eh3d_drillpro::protocol::sound::{Cue, SoundSetting};
        let mut a = app();
        // off until the person chooses (a file is chosen here so that nothing is played while testing)
        assert_eq!(a.conn.sound_check_ok, SoundSetting::Off);
        assert_eq!(a.conn.sound_check_bad, SoundSetting::Off);
        a.sound_choose(Cue::CheckOk, SoundSetting::File("ok.wav".into()));
        a.sound_choose(Cue::CheckNotOk, SoundSetting::File("bad.wav".into()));
        assert_eq!(a.conn.sound_check_ok, SoundSetting::File("ok.wav".into()));
        assert_eq!(a.conn.sound_check_bad, SoundSetting::File("bad.wav".into()));
        // the lists are the same as the ones of the other two sounds, and the machine file keeps both
        assert_eq!(
            a.combo_items(ComboId::Sound(Cue::CheckOk)).len(),
            a.combo_items(ComboId::Sound(Cue::Finished)).len()
        );
        assert_eq!(a.combo_current(ComboId::Sound(Cue::CheckNotOk)), 2);
        let pairs = crate::link::pairs(&a.conn);
        assert!(pairs
            .iter()
            .any(|(k, v)| *k == "sound_check_ok" && v.ends_with("ok.wav")));
        assert!(pairs
            .iter()
            .any(|(k, v)| *k == "sound_check_bad" && v.ends_with("bad.wav")));
    }

    #[test]
    fn a_check_without_a_node_says_so_and_does_not_hang() {
        let mut a = app();
        a.insp.loaded = true;
        a.act(Act::InspRun);
        assert!(!a.insp.running);
        assert!(matches!(a.insp.report, Some(Err(_))));
    }
}
