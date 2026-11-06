//! What the window tells the log (see `applog`): the changes of its state (focus, minimised, maximised, full screen, size, screen), the changes of what the
//! program shows and does (the screen, the window on top, the job, the connection, the files, the red notices) and the heartbeat with the memory and the
//! times of the frames. Nothing is written for a frame that changes nothing.

use std::collections::BTreeMap;

use super::*;
use crate::applog;

/// The state of the window as the system reports it.
#[derive(Debug, Clone, Default, PartialEq)]
struct WinState {
    focused: Option<bool>,
    minimized: Option<bool>,
    maximized: Option<bool>,
    fullscreen: Option<bool>,
    occluded: Option<bool>,
    /// The inside of the window in points, the scale of the screen in hundredths and the size of the monitor in points.
    size: Option<(i32, i32)>,
    scale: Option<i32>,
    monitor: Option<(i32, i32)>,
}

pub(crate) struct Diag {
    beat: applog::Heartbeat,
    win: Option<WinState>,
    /// The last value logged for each thing that is watched.
    seen: BTreeMap<&'static str, String>,
    started: Instant,
}

impl Diag {
    pub(crate) fn new(heartbeat_s: u32) -> Diag {
        let now = Instant::now();
        Diag {
            beat: applog::Heartbeat::new(heartbeat_s, now),
            win: None,
            seen: BTreeMap::new(),
            started: now,
        }
    }

    /// Notes the value of something that is watched; returns the one before it when it changed (`Some("")` the first time).
    fn changed(&mut self, key: &'static str, value: String) -> Option<String> {
        let before = self.seen.get(key).cloned();
        match before {
            Some(old) if old == value => None,
            Some(old) => {
                self.seen.insert(key, value);
                Some(old)
            }
            None => {
                self.seen.insert(key, value);
                Some(String::new())
            }
        }
    }
}

fn flag(v: Option<bool>) -> &'static str {
    match v {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}

fn path_text(p: &Option<PathBuf>) -> String {
    p.as_ref()
        .map_or_else(|| "none".to_string(), |p| p.display().to_string())
}

impl App {
    /// Called when a frame begins: what the window reports and what the program shows are compared with the last look, and what changed is logged.
    pub(super) fn diag_begin(&mut self, ctx: &egui::Context) {
        if !applog::active() {
            return;
        }
        let (win, events) = ctx.input(|i| {
            let v = i.viewport();
            let round = |x: f32| x.round() as i32;
            (
                WinState {
                    focused: v.focused,
                    minimized: v.minimized,
                    maximized: v.maximized,
                    fullscreen: v.fullscreen,
                    occluded: v.occluded,
                    size: v.inner_rect.map(|r| (round(r.width()), round(r.height()))),
                    scale: v.native_pixels_per_point.map(|p| round(p * 100.0)),
                    monitor: v.monitor_size.map(|m| (round(m.x), round(m.y))),
                },
                v.events
                    .iter()
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<_>>(),
            )
        });
        for e in events {
            applog::info("window", format!("event of the window: {e}"));
        }
        match self.diag.win.clone() {
            None => applog::info(
                "window",
                format!(
                    "window at the start: focused {}, minimised {}, maximised {}, full screen {}, covered {}, size {:?}, scale {:?}/100, monitor {:?}",
                    flag(win.focused),
                    flag(win.minimized),
                    flag(win.maximized),
                    flag(win.fullscreen),
                    flag(win.occluded),
                    win.size,
                    win.scale,
                    win.monitor
                ),
            ),
            Some(old) => {
                if old.focused != win.focused {
                    applog::info(
                        "window",
                        if win.focused == Some(true) {
                            "the window gained the focus"
                        } else {
                            "the window lost the focus"
                        },
                    );
                    // the moment a program is most likely to be in trouble: the memory at each change of focus
                    applog::info("memory", applog::memory_line());
                }
                if old.minimized != win.minimized {
                    applog::info(
                        "window",
                        format!("minimised: {}", flag(win.minimized)),
                    );
                }
                if old.maximized != win.maximized {
                    applog::info(
                        "window",
                        format!("maximised: {}", flag(win.maximized)),
                    );
                }
                if old.fullscreen != win.fullscreen {
                    applog::info(
                        "window",
                        format!("full screen: {}", flag(win.fullscreen)),
                    );
                }
                if old.occluded != win.occluded {
                    applog::info(
                        "window",
                        format!("covered by other windows: {}", flag(win.occluded)),
                    );
                }
                if old.scale != win.scale || old.monitor != win.monitor {
                    applog::info(
                        "window",
                        format!(
                            "screen changed: scale {:?}/100, monitor {:?} (before {:?}, {:?})",
                            win.scale, win.monitor, old.scale, old.monitor
                        ),
                    );
                }
                if old.size != win.size {
                    applog::debug("window", format!("size {:?}", win.size));
                }
            }
        }
        self.diag.win = Some(win);
        self.diag_watch();
    }

    /// The things of the program whose changes are logged.
    fn diag_watch(&mut self) {
        let job = format!(
            "running {:?}, paused {}, waiting for the next stage {}, queue {:?}",
            self.running, self.paused, self.stage_wait, self.job_queue
        );
        let question = match self.running {
            Some(Program::Drill) => self.drill.engine.question(),
            Some(Program::Route) => self.route.engine.question(),
            None => None,
        };
        let items: [(&'static str, String); 12] = [
            ("screen", format!("{:?}", self.screen)),
            ("modal", format!("{:?}", self.modal)),
            ("job", job),
            ("estop", self.estop.to_string()),
            ("connection", self.connection.clone()),
            ("model", self.settings.model.clone()),
            ("drill file", path_text(&self.drill_path)),
            ("route file", path_text(&self.route_path)),
            ("language", format!("{:?}", self.lang)),
            ("theme", format!("{:?}", self.theme.id)),
            ("recovery offered", self.resume.is_some().to_string()),
            ("machine asks", format!("{question:?}")),
        ];
        for (key, value) in items {
            let Some(old) = self.diag.changed(key, value.clone()) else {
                continue;
            };
            if key == "job" {
                let extra = format!(
                    "; holes done {}, paths done {}, elapsed {}",
                    self.drill.last_done,
                    self.route.last_done,
                    self.job_started.map_or_else(
                        || "-".to_string(),
                        |t| format!("{} s", t.elapsed().as_secs())
                    )
                );
                applog::info("job", format!("{value}{extra}"));
            } else if old.is_empty() {
                applog::info("state", format!("{key}: {value}"));
            } else {
                applog::info("state", format!("{key}: {old} -> {value}"));
            }
        }
        // the red notices: every new one is logged as it is shown
        let notice = self.message.clone().unwrap_or_default();
        if self.diag.changed("notice", notice.clone()).is_some() && !notice.is_empty() {
            applog::warn("notice", notice);
        }
    }

    /// Called when a frame ends, with the time it took to draw: the heartbeat, and a pause that was too long.
    pub(super) fn diag_end(&mut self, began: Instant) {
        if !applog::active() {
            return;
        }
        let now = Instant::now();
        let (beat, gap) = self
            .diag
            .beat
            .frame(now, now.saturating_duration_since(began));
        if let Some(gap) = gap {
            applog::warn(
                "frame",
                format!(
                    "no frame was drawn for {:.1} s (the window was behind another one, or the program stopped); focused {}",
                    gap.as_secs_f32(),
                    flag(self.diag.win.as_ref().and_then(|w| w.focused))
                ),
            );
        }
        let Some(b) = beat else {
            return;
        };
        let win = self.diag.win.clone().unwrap_or_default();
        applog::info(
            "heartbeat",
            format!(
                "up {} s; frames {} (+{}), drawing mean {:.1} ms max {:.1} ms; background threads {}; focused {}, minimised {}, covered {}; screen {:?}, window {:?}, job {:?}; holes done {}",
                self.diag.started.elapsed().as_secs(),
                b.total_frames,
                b.frames,
                b.mean_ms,
                b.max_ms,
                applog::live_threads(),
                flag(win.focused),
                flag(win.minimized),
                flag(win.occluded),
                self.screen,
                self.modal,
                self.running,
                self.drill.last_done
            ),
        );
        applog::info("memory", applog::memory_line());
    }
}
