//! The window of the updates: the version that is installed and the latest one on GitHub with its notes, the steps of the work with a bar each, the history, the
//! two switches (updates allowed, look when the program starts) and the buttons. The work itself is in [`crate::updates`], in a thread; here it is only shown and
//! asked for. Nothing is installed unless the person presses the button.

use super::extra::wrap;
use super::*;
use crate::updates as up;

/// The file the update would replace: the running program (a test never gets to touch it).
pub(super) fn program_to_replace() -> Option<PathBuf> {
    if cfg!(test) {
        return Some(
            std::env::temp_dir().join(format!("eh3d-bg-test-program-{}", std::process::id())),
        );
    }
    std::env::current_exe().ok()
}

/// A line cut to `max` letters, with an ellipsis when it was longer.
fn cut(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
        t.push('\u{2026}');
        t
    }
}

impl App {
    fn upd_view(&self) -> up::Shared {
        self.upd.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn upd_config(&self) -> up::Config {
        up::Config {
            api_url: self.settings.updates_api_url.clone(),
            timeout_s: self.settings.updates_timeout_s,
        }
    }

    fn upd_target(&self) -> Option<up::Target> {
        Some(up::Target {
            exe: self.upd_exe.clone()?,
            version: env!("CARGO_PKG_VERSION").to_string(),
            work: std::env::temp_dir()
                .join(format!("eh3d-bungard-gui-update-{}", std::process::id())),
            history: self.upd_history.clone(),
            label: up::platform_label()?,
        })
    }

    /// At start: look for a new version when the person asked for it and the time between two looks has passed.
    pub(super) fn upd_start(&mut self) {
        let s = &self.settings;
        let due = up::now().saturating_sub(s.updates_last_check)
            >= u64::from(s.updates_interval_h) * 3600;
        if s.updates_enabled && s.updates_at_start && due {
            self.upd_check();
        }
    }

    /// Every frame: when a look ends, the time is kept so that the next start does not look again before the interval.
    pub(super) fn upd_poll(&mut self) {
        if !self.upd_seen && self.upd_view().checked {
            self.upd_seen = true;
            self.settings.updates_last_check = up::now();
            self.remember_settings();
        }
    }

    pub(super) fn upd_check(&mut self) {
        if !self.settings.updates_enabled || self.upd_view().busy {
            return;
        }
        self.upd_seen = false;
        let _ = up::spawn_check(&self.upd, self.upd_config(), up::platform_label());
    }

    pub(super) fn upd_install(&mut self) {
        let v = self.upd_view();
        let (Some(rel), Some(target)) = (v.release.clone(), self.upd_target()) else {
            return;
        };
        if !self.settings.updates_enabled
            || v.busy
            || v.restart_needed
            || rel.package.is_none()
            || !up::is_newer(&rel.version, env!("CARGO_PKG_VERSION"))
        {
            return;
        }
        self.upd_cancel.store(false, Ordering::Relaxed);
        let _ = up::spawn_update(
            &self.upd,
            self.upd_config(),
            rel,
            target,
            self.upd_cancel.clone(),
        );
    }

    pub(super) fn upd_undo(&mut self) {
        let v = self.upd_view();
        let Some(target) = self.upd_target() else {
            return;
        };
        if v.busy || v.restart_needed || up::previous_version(&target.exe).is_none() {
            return;
        }
        let _ = up::run_undo(&self.upd, &target);
    }

    pub(super) fn upd_restart(&mut self) {
        if !self.upd_view().restart_needed {
            return;
        }
        if let Some(exe) = self.upd_exe.clone() {
            match up::restart(&exe) {
                Ok(()) => self.close = true,
                Err(e) => self.message = Some(format!("{}: {e}", self.tr(Text::UpdFailed))),
            }
        }
    }

    pub(super) fn upd_toggle_allow(&mut self) {
        self.settings.updates_enabled = !self.settings.updates_enabled;
        self.remember_settings();
    }

    pub(super) fn upd_toggle_start(&mut self) {
        self.settings.updates_at_start = !self.settings.updates_at_start;
        self.remember_settings();
    }

    fn step_text(&self, s: up::Step) -> &'static str {
        self.tr(match s {
            up::Step::Ask => Text::UpdStepAsk,
            up::Step::Download => Text::UpdStepDownload,
            up::Step::Verify => Text::UpdStepVerify,
            up::Step::Install => Text::UpdStepInstall,
        })
    }

    pub(super) fn paint_update(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y) = self.modal_card(cv, self.tr(Text::UpdTitle), 740.0);
        let v = self.upd_view();
        let mine = env!("CARGO_PKG_VERSION");
        let enabled = self.settings.updates_enabled;
        // the versions
        cv.text(
            x + 30.0,
            y + 98.0,
            Align2::LEFT_BOTTOM,
            &format!("{}: {mine}", self.tr(Text::UpdInstalled)),
            18.0,
            col(t.text),
        );
        let latest = v.release.as_ref().map_or("-", |r| r.version.as_str());
        cv.text(
            x + 380.0,
            y + 98.0,
            Align2::LEFT_BOTTOM,
            &format!("{}: {latest}", self.tr(Text::UpdLatest)),
            18.0,
            col(t.text),
        );
        // what is the case, in a line
        let newer = v
            .release
            .as_ref()
            .is_some_and(|r| up::is_newer(&r.version, mine));
        let no_package = newer && v.release.as_ref().is_some_and(|r| r.package.is_none());
        let (line, ink) = if !enabled {
            (self.tr(Text::UpdOff).to_string(), col(t.dim))
        } else if v.restart_needed {
            (self.tr(Text::UpdDone).to_string(), col(t.accent))
        } else if let Some(e) = &v.error {
            (
                format!("{}: {}", self.tr(Text::UpdFailed), cut(e, 70)),
                col(t.danger),
            )
        } else if no_package {
            (self.tr(Text::UpdNoPackage).to_string(), col(t.danger))
        } else if newer {
            (self.tr(Text::UpdAvailable).to_string(), col(t.accent))
        } else if v.release.is_some() {
            (self.tr(Text::UpdUpToDate).to_string(), col(t.text))
        } else {
            (String::new(), col(t.dim))
        };
        cv.text(x + 30.0, y + 130.0, Align2::LEFT_BOTTOM, &line, 17.0, ink);
        // the steps, each with its bar
        for (i, (step, state, f)) in v.steps.iter().enumerate() {
            let ry = y + 152.0 + i as f32 * 26.0;
            cv.text(
                x + 30.0,
                ry + 18.0,
                Align2::LEFT_BOTTOM,
                self.step_text(*step),
                15.0,
                if *state == up::StepState::Waiting {
                    col(t.dim)
                } else {
                    col(t.text)
                },
            );
            let (bx, bw) = (x + 330.0, 340.0);
            cv.p.rect_filled(
                cv.rect(bx, ry + 5.0, bw, 10.0),
                5.0 * cv.u,
                alpha(Color32::WHITE, 26),
            );
            let (fill, shown) = match state {
                up::StepState::Done => (col(t.accent), 1.0),
                up::StepState::Running => (col(t.accent), f.clamp(0.04, 1.0)),
                up::StepState::Failed => (col(t.danger), 1.0),
                up::StepState::Waiting => (col(t.accent), 0.0),
            };
            if shown > 0.0 {
                cv.p.rect_filled(cv.rect(bx, ry + 5.0, bw * shown, 10.0), 5.0 * cv.u, fill);
            }
        }
        // the notes of the latest release, as plain text
        cv.text(
            x + 30.0,
            y + 292.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::UpdNotes).to_uppercase(),
            13.0,
            col(t.dim),
        );
        let mut rows: Vec<String> = Vec::new();
        if let Some(r) = &v.release {
            for n in &r.notes {
                rows.extend(wrap(n, 74));
            }
        }
        let room = 6;
        let more = rows.len() > room;
        rows.truncate(room);
        if more {
            if let Some(last) = rows.last_mut() {
                *last = format!("{} \u{2026}", cut(last, 72));
            }
        }
        for (i, l) in rows.iter().enumerate() {
            cv.text(
                x + 30.0,
                y + 318.0 + i as f32 * 21.0,
                Align2::LEFT_BOTTOM,
                l,
                15.0,
                col(t.text),
            );
        }
        // the history, the last three
        cv.text(
            x + 30.0,
            y + 462.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::UpdHistory).to_uppercase(),
            13.0,
            col(t.dim),
        );
        let hist = up::read_history(&self.upd_history);
        for (i, e) in hist.iter().rev().take(3).enumerate() {
            let what = match e.what.as_str() {
                "update" => self.tr(Text::UpdHistUpdated),
                "undo" => self.tr(Text::UpdHistUndone),
                _ => self.tr(Text::UpdFailed),
            };
            let mut s = format!(
                "{}   {}   {} \u{2192} {}",
                up::format_time(e.time),
                what,
                e.from,
                e.to
            );
            if e.what == "failed" && !e.note.is_empty() {
                s.push_str(&format!("   {}", cut(&e.note, 40)));
            }
            cv.text(
                x + 30.0,
                y + 486.0 + i as f32 * 21.0,
                Align2::LEFT_BOTTOM,
                &s,
                15.0,
                col(if e.what == "failed" { t.danger } else { t.text }),
            );
        }
        // what is sent and what is not
        for (i, l) in wrap(self.tr(Text::UpdPrivacy), 92)
            .iter()
            .take(3)
            .enumerate()
        {
            cv.text(
                x + 30.0,
                y + 566.0 + i as f32 * 18.0,
                Align2::LEFT_BOTTOM,
                l,
                13.0,
                col(t.dim),
            );
        }
        cv.checkbox(
            x + 24.0,
            y + 590.0,
            690.0,
            self.tr(Text::UpdAllow),
            enabled,
            Act::UpdAllow,
        );
        cv.checkbox(
            x + 24.0,
            y + 628.0,
            690.0,
            self.tr(Text::UpdAtStart),
            self.settings.updates_at_start,
            Act::UpdAtStart,
        );
        // the buttons: look, install (after an install or an undo this one is the restart) and undo
        let kept = self
            .upd_exe
            .as_ref()
            .is_some_and(|e| up::previous_version(e).is_some());
        let restart = v.restart_needed;
        let buttons: [(&str, bool, Act, bool); 3] = [
            (
                self.tr(Text::UpdCheck),
                false,
                Act::UpdCheck,
                enabled && !v.busy,
            ),
            if restart {
                (self.tr(Text::UpdRestart), true, Act::UpdRestart, !v.busy)
            } else {
                (
                    self.tr(Text::UpdInstall),
                    true,
                    Act::UpdInstall,
                    enabled && !v.busy && newer && !no_package,
                )
            },
            (
                self.tr(Text::UpdUndo),
                false,
                Act::UpdUndo,
                !v.busy && !restart && kept,
            ),
        ];
        for (i, (label, primary, act, can)) in buttons.into_iter().enumerate() {
            let bx = x + 30.0 + i as f32 * 232.0;
            if can {
                cv.chip(bx, y + 676.0, 216.0, 44.0, label, primary, act);
            } else {
                cv.button_off(bx, y + 676.0, 216.0, 44.0, label, col(t.dim));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn long_lines_are_cut_with_an_ellipsis() {
        assert_eq!(cut("short", 10), "short");
        assert_eq!(cut("0123456789abc", 10).chars().count(), 10);
        assert!(cut("0123456789abc", 10).ends_with('\u{2026}'));
    }
}
