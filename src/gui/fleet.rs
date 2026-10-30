//! The window of the machines and stations: the central screen that follows every machine of the EH3D server at once. It asks the server
//! for the whole fleet every few seconds (in another thread, so that a server that does not answer never stops the window), lists the
//! machines under their stations with what each one is doing and how far its job has got, and chooses the machine the jobs of this
//! window are for.

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eh3d_drillpro::server_link::Client;
use eh3d_fleet::{Fleet, MachineState};

use super::extra::Modal;
use super::*;

/// How often the server is asked while the window is open.
const EVERY: Duration = Duration::from_secs(3);
/// The machines the window shows at once.
const ROWS: usize = 11;

/// What the window knows of the fleet.
#[derive(Default)]
pub(super) struct FleetView {
    pub fleet: Option<Fleet>,
    pub error: Option<String>,
    pub rx: Option<Receiver<Result<Fleet, String>>>,
    pub asked: Option<Instant>,
}

impl App {
    /// Asks the server for the fleet when the window is open, the connection is the server and it is time.
    pub(super) fn fleet_tick(&mut self) {
        if let Some(r) = self.fleet.rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.fleet.rx = None;
            match r {
                Ok(f) => {
                    self.fleet.fleet = Some(f);
                    self.fleet.error = None;
                }
                Err(e) => self.fleet.error = Some(e),
            }
        }
        if self.modal != Some(Modal::Fleet) || self.fleet.rx.is_some() {
            return;
        }
        if self.conn.kind != LinkKind::Server {
            return;
        }
        if self.fleet.asked.is_some_and(|t| t.elapsed() < EVERY) {
            return;
        }
        self.fleet_ask();
    }

    /// Asks now.
    pub(super) fn fleet_ask(&mut self) {
        let client = Client::new(&self.conn);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(client.fleet());
        });
        self.fleet.rx = Some(rx);
        self.fleet.asked = Some(Instant::now());
    }

    /// The machine of the list that the jobs are sent to from now on (it goes to the machine file too).
    pub(super) fn fleet_use(&mut self, index: usize) {
        let id = self
            .fleet
            .fleet
            .as_ref()
            .and_then(|f| f.machines.get(index))
            .map(|m| m.id.clone());
        if let Some(id) = id {
            self.conn.server_machine = id;
            self.conn_save();
        }
    }

    pub(super) fn paint_fleet_modal(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y, w, h) = (110.0, 90.0, 1380.0, 720.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 44.0, self.tr(Text::FleetTitle));
        let Some(fleet) = self.fleet.fleet.as_ref() else {
            // nothing to show yet: why
            let why = if self.conn.kind != LinkKind::Server {
                self.tr(Text::FleetNoServer).to_string()
            } else if let Some(e) = &self.fleet.error {
                e.clone()
            } else {
                "...".to_string()
            };
            cv.text(
                x + 30.0,
                y + 130.0,
                Align2::LEFT_BOTTOM,
                &why,
                20.0,
                col(t.dim),
            );
            self.fleet_buttons(cv, x, y, w, h);
            return;
        };
        let s = fleet.summary();
        let line = format!(
            "{}: {}   {}: {}   {}: {}   {}: {}   {}: {}",
            self.tr(Text::FleetTitle),
            s.machines,
            self.tr(Text::McStIdle),
            s.idle,
            self.tr(Text::McStRun),
            s.busy,
            self.tr(Text::FleetWaiting),
            s.waiting,
            "Stations",
            s.stations
        );
        cv.text(
            x + 30.0,
            y + 84.0,
            Align2::LEFT_BOTTOM,
            &line,
            15.0,
            col(t.dim),
        );
        // the headings of the columns
        let cols = [
            (30.0, Text::ColName),
            (330.0, Text::McState),
            (570.0, Text::McProgram),
        ];
        for (dx, n) in cols {
            cv.text(
                x + dx + 30.0,
                y + 118.0,
                Align2::LEFT_BOTTOM,
                self.tr(n),
                12.0,
                col(t.accent),
            );
        }
        let mut ry = y + 128.0;
        let mut shown = 0;
        let index_of = |id: &str| fleet.machines.iter().position(|m| m.id == id);
        'stations: for st in &fleet.stations {
            let name = if st.id.is_empty() {
                self.tr(Text::FleetNoStation).to_string()
            } else {
                st.id.to_uppercase()
            };
            if shown >= ROWS {
                break;
            }
            cv.text(
                x + 30.0,
                ry + 20.0,
                Align2::LEFT_BOTTOM,
                &name,
                14.0,
                col(t.accent),
            );
            ry += 26.0;
            for id in &st.machines {
                if shown >= ROWS {
                    break 'stations;
                }
                let Some(i) = index_of(id) else { continue };
                let m = &fleet.machines[i];
                let (dot, state) = match m.state {
                    MachineState::Idle => (col(t.ok), self.tr(Text::McStIdle)),
                    MachineState::Busy => (col(t.accent), self.tr(Text::McStRun)),
                    MachineState::Waiting => (col(t.warn), self.tr(Text::FleetWaiting)),
                };
                cv.p.circle_filled(cv.pt(x + 44.0, ry + 20.0), 7.0 * cv.u, dot);
                cv.text(
                    x + 64.0,
                    ry + 28.0,
                    Align2::LEFT_BOTTOM,
                    &m.name,
                    18.0,
                    col(t.text),
                );
                cv.text(
                    x + 64.0 + 250.0,
                    ry + 28.0,
                    Align2::LEFT_BOTTOM,
                    state,
                    16.0,
                    col(t.text),
                );
                if let Some(j) = &m.job {
                    cv.text(
                        x + 600.0,
                        ry + 20.0,
                        Align2::LEFT_BOTTOM,
                        &j.name,
                        15.0,
                        col(t.text),
                    );
                    // the progress of the job
                    let bar = cv.rect(x + 600.0, ry + 26.0, 260.0, 8.0);
                    cv.p.rect_filled(bar, 4.0 * cv.u, alpha(col(t.line), 60));
                    let done = Rect::from_min_size(
                        bar.min,
                        Vec2::new(bar.width() * j.percent as f32 / 100.0, bar.height()),
                    );
                    cv.p.rect_filled(done, 4.0 * cv.u, dot);
                    cv.text(
                        x + 880.0,
                        ry + 36.0,
                        Align2::LEFT_BOTTOM,
                        &format!("{}%", j.percent),
                        15.0,
                        col(t.text),
                    );
                } else if m.ready > 0 {
                    cv.text(
                        x + 600.0,
                        ry + 28.0,
                        Align2::LEFT_BOTTOM,
                        &format!("{} {}", m.ready, self.tr(Text::FleetReady)),
                        15.0,
                        col(t.dim),
                    );
                }
                cv.chip(
                    x + w - 330.0,
                    ry + 2.0,
                    300.0,
                    36.0,
                    &if self.conn.server_machine == m.id {
                        format!("* {}", self.tr(Text::FleetTarget))
                    } else {
                        self.tr(Text::FleetUse).to_string()
                    },
                    self.conn.server_machine == m.id,
                    Act::FleetUse(i),
                );
                ry += 44.0;
                shown += 1;
            }
            ry += 6.0;
        }
        if let Some(e) = &self.fleet.error {
            cv.text(
                x + 30.0,
                y + h - 76.0,
                Align2::LEFT_BOTTOM,
                e,
                15.0,
                col(t.danger),
            );
        }
        self.fleet_buttons(cv, x, y, w, h);
    }

    fn fleet_buttons(&self, cv: &mut Cv, x: f32, y: f32, w: f32, h: f32) {
        cv.chip(
            x + w - 400.0,
            y + h - 60.0,
            170.0,
            42.0,
            self.tr(Text::FleetRefresh),
            false,
            Act::FleetRefresh,
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
