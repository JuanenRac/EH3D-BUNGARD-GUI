//! The panel that moves the machine by hand: a floating card with the cross of X and Y and the column of Z, the steps and the speeds, the position of the head
//! (of the machine and of the work), the zeros, home and four places to keep. What it does is in [`crate::jog`], which has no window and is tested on its own; here
//! it is only drawn and the presses are passed to it. The head it moves is the one the plan view and the 3D view draw, and on a real link the orders go to the
//! machine through the jog of the engines of the classic programs (the same the Offset window of DRILLPRO and ROUTEPRO uses).

use super::controls::Icon;
use super::extra::Zone;
use super::*;
use crate::jog::{Axis, Config, Dir, Gate, Jog, Limits, Order, Pos, Refusal, Speed};
use eh3d_drillpro::runner::TeachState;
use std::cell::{Cell, RefCell};

/// The size of the card, in units of the canvas, and where it sits when it is docked.
const W: f32 = 640.0;
const H: f32 = 504.0;
const DOCK: (f32, f32) = (24.0, 96.0);
/// How long a refusal stays on the card, and how long the ring of a press takes to fade, in seconds.
const REFUSAL_S: f32 = 3.0;
const PULSE_S: f32 = 0.35;
/// The width of each of the two columns of the card.
const COL_W: f32 = 296.0;
/// The side of an arrow button and the gap between two of them.
const CELL: f32 = 60.0;
const GAP: f32 = 8.0;

/// The keys that move the head: the arrows across the table (when nothing is chosen in the plan view, which they would move instead) and the page keys in Z.
const ARROW_KEYS: [(egui::Key, Dir); 4] = [
    (egui::Key::ArrowLeft, Dir::XMinus),
    (egui::Key::ArrowRight, Dir::XPlus),
    (egui::Key::ArrowDown, Dir::YMinus),
    (egui::Key::ArrowUp, Dir::YPlus),
];
const PAGE_KEYS: [(egui::Key, Dir); 2] = [
    (egui::Key::PageUp, Dir::ZUp),
    (egui::Key::PageDown, Dir::ZDown),
];

/// What a button of the panel asks for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum JogAct {
    /// Opens the panel or closes it.
    Toggle,
    /// Docks the panel at its own place or lets it be moved.
    Dock,
    /// Chooses a step of the list.
    Step(usize),
    /// Switches the continuous mode.
    Continuous,
    Speed(Speed),
    /// An axis goes to its zero of the work.
    Zero(Axis),
    /// The place of the head is the zero of the work.
    Origin,
    Home,
    /// Keeps the place of the head in a slot, or goes to the place kept.
    Save(usize),
    Go(usize),
    Stop,
}

/// The state of the window part of the panel.
pub(super) struct JogUi {
    pub(super) open: bool,
    pub(super) jog: Jog,
    /// The file the choices and the places are kept in.
    path: PathBuf,
    /// A session of the engine moves the real machine.
    live: bool,
    refused: Option<(Refusal, Instant)>,
    /// The top of the card was pressed, so the drag moves the card.
    grab: bool,
    /// The arrow held with the mouse or with the keys.
    mouse: Option<Dir>,
    keys: Option<Dir>,
    /// The last arrow pressed and when, for the ring that fades.
    flash: Option<(Dir, Instant)>,
    /// The arrows as drawn in this frame, and the top of the card.
    rects: RefCell<Vec<(Rect, Dir)>>,
    top: Cell<Option<Rect>>,
}

impl JogUi {
    pub(super) fn new(path: PathBuf) -> JogUi {
        let cfg = Config::load(&path);
        // the travel is set from the machine before the first move
        let jog = Jog::new(cfg, Limits::new(0.0, 0.0, 0.0, 0.0, 0.0));
        JogUi {
            open: false,
            jog,
            path,
            live: false,
            refused: None,
            grab: false,
            mouse: None,
            keys: None,
            flash: None,
            rects: RefCell::new(Vec::new()),
            top: Cell::new(None),
        }
    }
}

/// Keeps the card in the window, all of it.
fn clamp_panel(p: (f32, f32)) -> (f32, f32) {
    let fit = |v: f32, lo: f32, hi: f32| {
        if v.is_finite() {
            v.clamp(lo, hi.max(lo))
        } else {
            lo
        }
    };
    (fit(p.0, 0.0, design_w() - W), fit(p.1, 66.0, DESIGN_H - H))
}

/// An arrow drawn pointing up and turned to the way of `dir`.
fn arrow(cv: &Cv, cx: f32, cy: f32, s: f32, dir: Dir, c: Color32) {
    let turn = |p: (f32, f32)| -> (f32, f32) {
        let (x, y) = p;
        let (rx, ry) = match dir {
            Dir::YPlus | Dir::ZUp => (x, y),
            Dir::YMinus | Dir::ZDown => (x, -y),
            Dir::XPlus => (-y, x),
            Dir::XMinus => (y, -x),
        };
        (cx + rx * s, cy + ry * s)
    };
    let head = [(0.0, -1.0), (-0.85, 0.0), (0.85, 0.0)].map(turn);
    let stem = [(-0.3, 0.0), (0.3, 0.0), (0.3, 0.9), (-0.3, 0.9)].map(turn);
    cv.poly(&head, c, Stroke::NONE);
    cv.poly(&stem, c, Stroke::NONE);
}

fn axis_letter(i: usize) -> &'static str {
    ["X", "Y", "Z"][i.min(2)]
}

impl App {
    // ------------------------------------------------------------------------------------------------------------------ what the views show

    /// Where the panel has put the head (X, Y, Z in mm, Z downwards) while it moves the machine by hand; the plan view and the 3D view draw this in place of the
    /// head of the job.
    pub(super) fn jog_head(&self) -> Option<(f64, f64, f64)> {
        let ui = &self.jog_ui;
        if !ui.open || self.running.is_some() || !ui.jog.touched() {
            return None;
        }
        let s = ui.jog.shown();
        Some((s.x, s.y, s.z))
    }

    fn jog_gate(&self) -> Gate {
        Gate {
            job_running: self.running.is_some(),
            estop: self.estop,
        }
    }

    /// How deep the surface of the board is below the home position, in mm (the same the 3D view draws the tool over).
    fn jog_board_z(&self) -> f64 {
        self.drill
            .engine
            .settings()
            .map_or(23.7, |m| m.z_max_depth_mm - m.base_mm - m.board_mm)
            .clamp(8.0, 60.0)
    }

    /// The travel of the machine as the table and the board are now.
    fn jog_sync(&mut self) {
        let cfg = self.jog_ui.jog.config();
        let limits = Limits::new(
            self.table.width_mm,
            self.table.depth_mm,
            self.jog_board_z(),
            cfg.clearance_mm,
            cfg.below_board_mm,
        );
        if limits != self.jog_ui.jog.limits() {
            self.jog_ui.jog.set_limits(limits);
        }
    }

    fn jog_save(&mut self) {
        let cfg = self.jog_ui.jog.config_to_save();
        if let Err(e) = cfg.save(&self.jog_ui.path) {
            self.message = Some(format!("{}: {e}", self.jog_ui.path.display()));
        }
    }

    fn jog_refuse(&mut self, r: Refusal) {
        self.jog_ui.refused = Some((r, Instant::now()));
    }

    pub(super) fn jog_panel_pos(&self) -> (f32, f32) {
        if self.jog_ui.jog.docked() {
            DOCK
        } else {
            clamp_panel(self.jog_ui.jog.panel())
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ the real machine

    /// The machine is about to be moved: on a real link the engine starts its session (the machine goes home, where the panel starts counting).
    fn jog_arm(&mut self) -> bool {
        let live = self.conn.is_real();
        if live && !self.jog_ui.live {
            self.jog_ui.jog.park();
            self.drill.engine.teach_begin();
            self.jog_ui.live = true;
        }
        live
    }

    /// The orders go to the engine in mils, which is the unit of its jog (what a mil leaves over goes on to the next order).
    fn jog_send(&mut self, orders: Vec<Order>, live: bool) {
        if !live {
            return;
        }
        for o in orders {
            let (x, y, z) = self.jog_ui.jog.mils_of(o);
            if (x, y, z) != (0, 0, 0) {
                self.drill.engine.jog(x, y, z);
            }
        }
    }

    /// Lets go of the machine: the session of a real link ends (the head goes home) and the views take their own head back.
    pub(super) fn jog_yield(&mut self) {
        if self.jog_ui.live {
            self.drill.engine.teach_end();
            self.jog_ui.live = false;
        }
        self.jog_ui.mouse = None;
        self.jog_ui.keys = None;
        self.jog_ui.jog.park();
    }

    // ------------------------------------------------------------------------------------------------------------------ the presses

    /// Runs a move of the panel: refused with a word when a job runs or the emergency stop is on, or when the travel says no; otherwise sent.
    fn jog_do(&mut self, f: impl FnOnce(&mut Jog, Gate) -> Result<Vec<Order>, Refusal>) {
        self.jog_sync();
        let gate = self.jog_gate();
        if gate.estop || gate.job_running {
            self.jog_ui.jog.release();
            self.jog_refuse(if gate.estop {
                Refusal::Estop
            } else {
                Refusal::Job
            });
            return;
        }
        let live = self.jog_arm();
        match f(&mut self.jog_ui.jog, gate) {
            Ok(orders) => self.jog_send(orders, live),
            Err(r) => self.jog_refuse(r),
        }
    }

    pub(super) fn jog_press(&mut self, dir: Dir) {
        self.jog_ui.flash = Some((dir, Instant::now()));
        self.jog_do(|j, g| j.press(dir, g));
    }

    fn jog_stop(&mut self) {
        let live = self.jog_ui.live;
        self.jog_ui.jog.stop(live);
        self.jog_ui.mouse = None;
        self.jog_ui.keys = None;
    }

    pub(super) fn jog_act(&mut self, a: JogAct) {
        self.jog_sync();
        match a {
            JogAct::Toggle => {
                if self.jog_ui.open {
                    self.jog_ui.open = false;
                    self.jog_yield();
                } else {
                    self.jog_ui.open = true;
                }
            }
            JogAct::Dock => {
                let on = !self.jog_ui.jog.docked();
                self.jog_ui.jog.set_docked(on);
                self.jog_save();
            }
            JogAct::Step(i) => {
                self.jog_ui.jog.set_step(i);
                self.jog_save();
            }
            JogAct::Continuous => {
                let on = !self.jog_ui.jog.continuous();
                self.jog_ui.jog.set_continuous(on);
                self.jog_save();
            }
            JogAct::Speed(s) => {
                self.jog_ui.jog.set_speed(s);
                self.jog_save();
            }
            JogAct::Zero(axis) => self.jog_do(|j, g| j.go_zero(axis, g)),
            JogAct::Home => self.jog_do(|j, g| j.home(g)),
            JogAct::Go(i) => self.jog_do(|j, g| j.goto_slot(i, g)),
            JogAct::Origin | JogAct::Save(_) => {
                // a place is only taken where the head is: not while a job moves it
                if self.running.is_some() {
                    self.jog_refuse(Refusal::Job);
                    return;
                }
                match a {
                    JogAct::Save(i) => self.jog_ui.jog.save_slot(i),
                    _ => self.jog_ui.jog.set_origin_here(),
                }
                self.jog_save();
            }
            JogAct::Stop => self.jog_stop(),
        }
    }

    /// The card is dragged by its top (a press anywhere else on it moves nothing).
    pub(super) fn jog_drag(&mut self, d: (f32, f32)) {
        if !self.jog_ui.grab || self.jog_ui.jog.docked() {
            return;
        }
        let (x, y) = self.jog_panel_pos();
        self.jog_ui.jog.set_panel(clamp_panel((x + d.0, y + d.1)));
    }

    // ------------------------------------------------------------------------------------------------------------------ the pointer and the keys

    /// A press on an arrow moves at once (and, in the continuous mode, holds until the button is let go); a press on the top of the card takes the card.
    fn jog_pointer(&mut self, ctx: &egui::Context) {
        let (pressed, down, pos) = ctx.input(|i| {
            (
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.pointer.hover_pos(),
            )
        });
        if self.modal.is_some() {
            self.jog_ui.grab = false;
            if self.jog_ui.mouse.take().is_some() {
                self.jog_ui.jog.release();
            }
            return;
        }
        if let (true, Some(p)) = (pressed, pos) {
            if self.jog_ui.top.get().is_some_and(|r| r.contains(p)) {
                self.jog_ui.grab = true;
            } else {
                let hit = self
                    .jog_ui
                    .rects
                    .borrow()
                    .iter()
                    .find(|(r, _)| r.contains(p))
                    .map(|(_, d)| *d);
                if let Some(d) = hit {
                    self.jog_ui.mouse = Some(d);
                    self.jog_press(d);
                }
            }
        }
        if !down {
            if self.jog_ui.grab {
                self.jog_ui.grab = false;
                self.jog_save();
            }
            if self.jog_ui.mouse.take().is_some() {
                self.jog_ui.jog.release();
            }
        }
    }

    /// The keys, while the panel is open and nothing else wants them: Esc stops (before it closes anything), the arrows and the page keys move, + and - change the step.
    pub(super) fn jog_keys(&mut self, ctx: &egui::Context) {
        let free = self.jog_ui.open
            && self.screen == Screen::Viewer
            && self.modal.is_none()
            && self.edit.is_none()
            && !ctx.egui_wants_keyboard_input();
        if !free {
            if self.jog_ui.keys.take().is_some() {
                self.jog_ui.jog.release();
            }
            return;
        }
        let busy = self.jog_ui.jog.busy();
        if busy && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            self.jog_stop();
            return;
        }
        // the arrows move what is chosen in the plan view when something is chosen
        let arrows = self.selection_count() == 0;
        let (pressed, down, plus, minus) = ctx.input(|i| {
            let mut pressed: Vec<Dir> = Vec::new();
            let mut down: Vec<Dir> = Vec::new();
            if !i.modifiers.command && !i.modifiers.alt {
                let arrow_keys: &[(egui::Key, Dir)] = if arrows { &ARROW_KEYS } else { &[] };
                for &(key, dir) in arrow_keys.iter().chain(PAGE_KEYS.iter()) {
                    if i.key_pressed(key) {
                        pressed.push(dir);
                    }
                    if i.key_down(key) {
                        down.push(dir);
                    }
                }
            }
            (
                pressed,
                down,
                i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals),
                i.key_pressed(egui::Key::Minus),
            )
        });
        for dir in pressed {
            if !self.jog_ui.jog.continuous() {
                // a key that repeats repeats the step
                self.jog_press(dir);
            } else if self.jog_ui.jog.held().is_none() {
                self.jog_ui.keys = Some(dir);
                self.jog_press(dir);
            }
        }
        if let Some(k) = self.jog_ui.keys {
            if !down.contains(&k) {
                self.jog_ui.keys = None;
                self.jog_ui.jog.release();
            }
        }
        if plus {
            self.jog_ui.jog.step_up();
            self.jog_save();
        }
        if minus {
            self.jog_ui.jog.step_down();
            self.jog_save();
        }
    }

    /// Once a frame: the stretches of a held arrow, the picture of the head on its way, the state of the link.
    pub(super) fn jog_tick(&mut self, ctx: &egui::Context) {
        if self.running.is_some() || (self.jog_ui.open && self.screen != Screen::Viewer) {
            // a job moves the machine now, or the viewer was left: the panel lets go of it
            if self.jog_ui.open && self.screen != Screen::Viewer {
                self.jog_ui.open = false;
            }
            if self.jog_ui.live || self.jog_ui.jog.touched() {
                self.jog_yield();
            }
            return;
        }
        if !self.jog_ui.open {
            return;
        }
        self.jog_sync();
        self.jog_pointer(ctx);
        let dt = f64::from(ctx.input(|i| i.stable_dt)).min(0.25);
        let (gate, live) = (self.jog_gate(), self.jog_ui.live);
        match self.jog_ui.jog.tick(dt, gate) {
            Ok(orders) => self.jog_send(orders, live),
            Err(r) => self.jog_refuse(r),
        }
        self.jog_ui.jog.advance(dt);
        if self.jog_ui.live {
            if !self.conn.is_real() {
                self.jog_ui.live = false;
                self.drill.engine.teach_end();
            } else if let Some(TeachState::Failed(e)) = self.drill.engine.teach_state() {
                // the link did not hold: the person is told and the panel starts again from home with the next press
                self.message = Some(e);
                self.jog_ui.live = false;
                self.drill.engine.teach_end();
                self.jog_ui.jog.park();
            }
        }
        if self.jog_ui.jog.busy() || self.jog_ui.flash.is_some() {
            ctx.request_repaint();
        }
    }

    // ------------------------------------------------------------------------------------------------------------------ drawing

    /// One arrow of the panel: a raised button that sinks while it is pressed or held, with a ring that spreads and fades when it fires.
    fn paint_dir(&self, cv: &mut Cv, x: f32, y: f32, dir: Dir, blocked: bool) {
        let t = cv.t.clone();
        let rect = cv.rect(x, y, CELL, CELL);
        let (hover, down) = cv.touch(rect);
        let held = self.jog_ui.jog.held() == Some(dir);
        let pressed = !blocked && (down || held);
        let hover = hover && !blocked;
        let r = if t.corner == Corner::Square {
            0.0
        } else {
            12.0 * cv.u
        };
        let base = if pressed {
            shade(col(t.accent), 0.85)
        } else {
            col(t.panel_raised)
        };
        cv.raised(rect, r, base, hover, pressed);
        let ink = if blocked {
            alpha(col(t.text), 90)
        } else if pressed {
            col(t.background.0)
        } else if hover {
            Color32::WHITE
        } else {
            col(t.accent)
        };
        let dy = if pressed { 1.8 } else { 0.0 };
        arrow(cv, x + CELL / 2.0, y + CELL / 2.0 + dy, 17.0, dir, ink);
        if let (Some((d, at)), true) = (self.jog_ui.flash, self.animate) {
            let age = at.elapsed().as_secs_f32();
            if d == dir && age < PULSE_S {
                let k = age / PULSE_S;
                let grow = (2.0 + 14.0 * k) * cv.u;
                cv.p.rect_stroke(
                    rect.expand(grow),
                    r + grow,
                    Stroke::new(2.0 * cv.u, alpha(col(t.accent), ((1.0 - k) * 180.0) as u8)),
                    StrokeKind::Outside,
                );
            }
        }
        self.jog_ui.rects.borrow_mut().push((rect, dir));
    }

    /// A readout of the three axes: a label above and the three numbers in a sunk box.
    fn paint_readout(&self, cv: &mut Cv, x: f32, y: f32, label: Text, p: Pos) {
        let t = cv.t.clone();
        cv.text(
            x,
            y + 12.0,
            Align2::LEFT_BOTTOM,
            &self.tr(label).to_uppercase(),
            11.0,
            col(t.dim),
        );
        cv.sunk(
            cv.rect(x, y + 16.0, COL_W, 38.0),
            8.0 * cv.u,
            col(t.panel_raised),
            false,
        );
        for (i, v) in [p.x, p.y, p.z].into_iter().enumerate() {
            let cx = x + 4.0 + i as f32 * 98.0;
            cv.text(
                cx,
                y + 44.0,
                Align2::LEFT_BOTTOM,
                axis_letter(i),
                12.0,
                col(t.accent),
            );
            cv.text_fit(
                cx + 14.0,
                y + 45.0,
                Align2::LEFT_BOTTOM,
                &format!("{v:.2}"),
                20.0,
                col(t.text),
                76.0,
            );
        }
    }

    pub(super) fn paint_jog(&self, cv: &mut Cv) {
        let ui = &self.jog_ui;
        ui.rects.borrow_mut().clear();
        ui.top.set(None);
        if !ui.open || self.screen != Screen::Viewer {
            return;
        }
        let t = cv.t.clone();
        let j = &ui.jog;
        let (px, py) = self.jog_panel_pos();
        let gate = self.jog_gate();
        let blocked = gate.job_running || gate.estop;
        cv.card(px, py, W, H, true, None);
        ui.top.set(Some(cv.rect(px, py, W - 100.0, 44.0)));
        cv.zones.push((cv.rect(px, py, W, H), Zone::Jog));

        // ---- the top: the picture, the name, the dock and the close buttons
        cv.draw_icon(px + 26.0, py + 24.0, 22.0, Icon::Jog, col(t.accent));
        cv.title(px + 46.0, py + 30.0, self.tr(Text::JogTitle));
        let docked = j.docked();
        cv.chip(
            px + W - 84.0,
            py + 8.0,
            34.0,
            28.0,
            "",
            docked,
            Act::Jog(JogAct::Dock),
        );
        let pin = if docked {
            col(t.background.0)
        } else {
            col(t.text)
        };
        let (pcx, pcy) = (px + W - 67.0, py + 22.0);
        cv.p.circle_filled(cv.pt(pcx, pcy - 4.0), 4.0 * cv.u, pin);
        cv.line((pcx - 5.0, pcy + 1.0), (pcx + 5.0, pcy + 1.0), 2.0, pin);
        cv.line((pcx, pcy + 1.0), (pcx, pcy + 9.0), 2.0, pin);
        cv.chip(
            px + W - 44.0,
            py + 8.0,
            34.0,
            28.0,
            "",
            false,
            Act::Jog(JogAct::Toggle),
        );
        let (ccx, ccy) = (px + W - 27.0, py + 22.0);
        cv.line(
            (ccx - 6.0, ccy - 6.0),
            (ccx + 6.0, ccy + 6.0),
            2.2,
            col(t.text),
        );
        cv.line(
            (ccx - 6.0, ccy + 6.0),
            (ccx + 6.0, ccy - 6.0),
            2.2,
            col(t.text),
        );

        // ---- the left column: where the head is, the zeros, home, the cross and Z
        let x0 = px + 16.0;
        let s = j.shown();
        let o = j.origin();
        self.paint_readout(cv, x0, py + 54.0, Text::JogMachine, s);
        let work = Pos {
            x: s.x - o.x,
            y: s.y - o.y,
            z: s.z - o.z,
        };
        self.paint_readout(cv, x0, py + 114.0, Text::JogWork, work);
        for (i, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            cv.chip(
                x0 + i as f32 * 102.0,
                py + 178.0,
                92.0,
                28.0,
                &format!("{} {}0", self.tr(Text::JogGo), axis_letter(i)),
                false,
                Act::Jog(JogAct::Zero(axis)),
            );
        }
        cv.chip(
            x0,
            py + 214.0,
            COL_W,
            30.0,
            self.tr(Text::JogHome),
            false,
            Act::Jog(JogAct::Home),
        );
        let (gx, gy) = (x0 + 8.0, py + 254.0);
        let step = CELL + GAP;
        for (r, c, dir) in [
            (0.0, 1.0, Dir::YPlus),
            (1.0, 0.0, Dir::XMinus),
            (1.0, 2.0, Dir::XPlus),
            (2.0, 1.0, Dir::YMinus),
        ] {
            self.paint_dir(cv, gx + c * step, gy + r * step, dir, blocked);
        }
        // the stop at the middle of the cross
        {
            let (sx, sy) = (gx + step, gy + step);
            let rect = cv.rect(sx, sy, CELL, CELL);
            let (hover, down) = cv.touch(rect);
            cv.raised(
                rect,
                CELL / 2.0 * cv.u,
                shade(col(t.danger), if down { 0.7 } else { 0.9 }),
                hover,
                down,
            );
            cv.draw_icon(
                sx + CELL / 2.0,
                sy + CELL / 2.0 + if down { 1.8 } else { 0.0 },
                22.0,
                Icon::Stop,
                Color32::WHITE,
            );
            cv.hits.push((rect, Act::Jog(JogAct::Stop)));
        }
        // the column of Z: up, the name of the axis, down
        let zx = gx + 3.0 * step + 8.0;
        self.paint_dir(cv, zx, gy, Dir::ZUp, blocked);
        cv.sunk(
            cv.rect(zx, gy + step, CELL, CELL),
            10.0 * cv.u,
            col(t.panel_raised),
            false,
        );
        cv.text(
            zx + CELL / 2.0,
            gy + step + CELL / 2.0,
            Align2::CENTER_CENTER,
            axis_letter(2),
            26.0,
            col(t.accent),
        );
        self.paint_dir(cv, zx, gy + 2.0 * step, Dir::ZDown, blocked);

        // ---- the right column: the step, the mode, the speed, the zero of the work, the places kept
        let x1 = px + 16.0 + COL_W + 16.0;
        cv.text(
            x1,
            py + 66.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::JogStep).to_uppercase(),
            11.0,
            col(t.dim),
        );
        cv.text(
            x1 + COL_W,
            py + 66.0,
            Align2::RIGHT_BOTTOM,
            &format!("{} mm", crate::jog::number(j.step_mm())),
            12.0,
            col(t.dim),
        );
        let n = j.steps().len().max(1) as f32;
        let chip_w = (COL_W - 6.0 * (n - 1.0)) / n;
        for (i, s) in j.steps().iter().enumerate() {
            cv.chip(
                x1 + i as f32 * (chip_w + 6.0),
                py + 72.0,
                chip_w,
                32.0,
                &crate::jog::number(*s),
                i == j.step_index(),
                Act::Jog(JogAct::Step(i)),
            );
        }
        cv.chip(
            x1,
            py + 112.0,
            COL_W,
            30.0,
            self.tr(Text::JogContinuous),
            j.continuous(),
            Act::Jog(JogAct::Continuous),
        );
        cv.text(
            x1,
            py + 160.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::JogSpeed).to_uppercase(),
            11.0,
            col(t.dim),
        );
        cv.text(
            x1 + COL_W,
            py + 160.0,
            Align2::RIGHT_BOTTOM,
            &format!("{} mm/min", crate::jog::number(j.feed())),
            12.0,
            col(t.dim),
        );
        let speed_w = (COL_W - 12.0) / 3.0;
        for (i, (sp, word)) in [
            (Speed::Slow, Text::JogSlow),
            (Speed::Medium, Text::JogMedium),
            (Speed::Fast, Text::JogFast),
        ]
        .into_iter()
        .enumerate()
        {
            cv.chip(
                x1 + i as f32 * (speed_w + 6.0),
                py + 166.0,
                speed_w,
                32.0,
                self.tr(word),
                j.speed() == sp,
                Act::Jog(JogAct::Speed(sp)),
            );
        }
        cv.chip(
            x1,
            py + 208.0,
            COL_W,
            32.0,
            self.tr(Text::JogSetOrigin),
            false,
            Act::Jog(JogAct::Origin),
        );
        cv.text(
            x1,
            py + 262.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::JogSaved).to_uppercase(),
            11.0,
            col(t.dim),
        );
        for i in 0..crate::jog::SLOTS {
            let y = py + 270.0 + i as f32 * 38.0;
            cv.p.circle_stroke(
                cv.pt(x1 + 12.0, y + 17.0),
                11.0 * cv.u,
                Stroke::new(1.4 * cv.u, col(t.accent)),
            );
            cv.text(
                x1 + 12.0,
                y + 17.0,
                Align2::CENTER_CENTER,
                &(i + 1).to_string(),
                13.0,
                col(t.text),
            );
            match j.slot(i) {
                Some(p) => {
                    cv.text_fit(
                        x1 + 30.0,
                        y + 16.0,
                        Align2::LEFT_BOTTOM,
                        &format!("X {:.1}  Y {:.1}", p.x, p.y),
                        12.0,
                        col(t.text),
                        104.0,
                    );
                    cv.text_fit(
                        x1 + 30.0,
                        y + 31.0,
                        Align2::LEFT_BOTTOM,
                        &format!("Z {:.1}", p.z),
                        12.0,
                        col(t.dim),
                        104.0,
                    );
                }
                None => {
                    cv.text(
                        x1 + 30.0,
                        y + 22.0,
                        Align2::LEFT_BOTTOM,
                        "-",
                        14.0,
                        col(t.dim),
                    );
                }
            }
            cv.chip(
                x1 + 140.0,
                y,
                70.0,
                34.0,
                self.tr(Text::JogGo),
                false,
                Act::Jog(JogAct::Go(i)),
            );
            cv.chip(
                x1 + 214.0,
                y,
                82.0,
                34.0,
                self.tr(Text::JogSave),
                false,
                Act::Jog(JogAct::Save(i)),
            );
        }

        // ---- the foot: why nothing moves, whether the machine is real, the keys
        let why = if gate.estop {
            Some(Text::JogBusyStop)
        } else if gate.job_running {
            Some(Text::JogBusyJob)
        } else {
            ui.refused
                .filter(|(_, at)| at.elapsed().as_secs_f32() < REFUSAL_S)
                .map(|(r, _)| match r {
                    Refusal::Job => Text::JogBusyJob,
                    Refusal::Estop => Text::JogBusyStop,
                    Refusal::Limit => Text::JogLimit,
                    Refusal::RaiseZ => Text::JogRaiseZ,
                })
        };
        if let Some(w) = why {
            cv.text_fit(
                px + 20.0,
                py + H - 36.0,
                Align2::LEFT_BOTTOM,
                self.tr(w),
                14.0,
                if blocked { col(t.danger) } else { col(t.warn) },
                W - 40.0,
            );
        }
        let real = self.conn.is_real();
        cv.p.circle_filled(
            cv.pt(px + 24.0, py + H - 19.0),
            4.0 * cv.u,
            if real { col(t.ok) } else { col(t.dim) },
        );
        cv.text_fit(
            px + 36.0,
            py + H - 14.0,
            Align2::LEFT_BOTTOM,
            self.tr(if real { Text::JogLive } else { Text::JogDemo }),
            12.0,
            col(t.dim),
            220.0,
        );
        cv.text(
            px + W - 20.0,
            py + H - 14.0,
            Align2::RIGHT_BOTTOM,
            &format!("{}   {}   {}", "PgUp/PgDn", "+ / -", "Esc"),
            12.0,
            col(t.dim),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;
    use std::path::Path;

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir: PathBuf =
            std::env::temp_dir().join(format!("eh3d-bg-jog-{}-{n}", std::process::id()));
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
        a.animate = false;
        a.open_path(&dir.join("05_small_board.ncd"));
        a
    }

    fn press(a: &mut App, dir: Dir) {
        a.jog_press(dir);
    }

    /// Lets the picture of the head arrive where the head was sent.
    fn settle(a: &mut App) {
        a.jog_ui.jog.advance(600.0);
    }

    #[test]
    fn the_panel_opens_from_its_button_and_the_head_of_both_views_follows_the_arrows() {
        let mut a = app();
        assert!(!a.jog_ui.open);
        assert!(a.jog_head().is_none());
        a.act(Act::Jog(JogAct::Toggle));
        assert!(a.jog_ui.open);
        // nothing moved yet: the views keep their own head
        assert!(a.jog_head().is_none());
        // the step that comes with the program is 1 mm
        press(&mut a, Dir::XPlus);
        press(&mut a, Dir::XPlus);
        press(&mut a, Dir::YPlus);
        settle(&mut a);
        assert_eq!(a.head(), Some((2.0, 1.0)));
        assert_eq!(a.head3d(), Some((2.0, 1.0)));
        // Z down 3 steps of 1 mm: the tip is 3 mm nearer the board in the 3D view
        let board = a.jog_board_z();
        let before = a.head_lift_mm();
        assert!((before - board).abs() < 1e-9);
        for _ in 0..3 {
            press(&mut a, Dir::ZDown);
        }
        settle(&mut a);
        assert!((a.head_lift_mm() - (board - 3.0)).abs() < 1e-9);
        // closing the panel gives the views their own head back
        a.act(Act::Jog(JogAct::Toggle));
        assert!(!a.jog_ui.open);
        assert!(a.jog_head().is_none());
        assert_eq!(a.head3d(), Some(a.m3d.pos()));
        assert!((a.head_lift_mm() - board).abs() < 1e-9);
        assert_eq!(a.jog_ui.jog.pos(), Pos::HOME);
    }

    #[test]
    fn a_step_chosen_moves_that_far_and_the_orders_are_the_ones_of_the_step() {
        let mut a = app();
        a.act(Act::Jog(JogAct::Toggle));
        for (i, s) in [0.01, 0.1, 1.0, 10.0, 50.0].into_iter().enumerate() {
            a.act(Act::Jog(JogAct::Step(i)));
            assert_eq!(a.jog_ui.jog.step_mm(), s);
            let before = a.jog_ui.jog.pos().x;
            press(&mut a, Dir::XPlus);
            assert!((a.jog_ui.jog.pos().x - before - s).abs() < 1e-9, "{s}");
        }
        a.act(Act::Jog(JogAct::Speed(Speed::Fast)));
        assert_eq!(a.jog_ui.jog.feed(), 3000.0);
    }

    #[test]
    fn a_job_or_the_emergency_stop_keeps_the_head_where_it_is_and_says_why() {
        let mut a = app();
        a.act(Act::Jog(JogAct::Toggle));
        press(&mut a, Dir::XPlus);
        let at = a.jog_ui.jog.pos();
        a.estop = true;
        press(&mut a, Dir::XPlus);
        a.act(Act::Jog(JogAct::Home));
        a.act(Act::Jog(JogAct::Go(0)));
        assert_eq!(a.jog_ui.jog.pos(), at);
        assert_eq!(a.jog_ui.refused.map(|r| r.0), Some(Refusal::Estop));
        a.estop = false;
        a.running = Some(Program::Drill);
        press(&mut a, Dir::XPlus);
        a.act(Act::Jog(JogAct::Zero(Axis::X)));
        assert_eq!(a.jog_ui.jog.pos(), at);
        assert_eq!(a.jog_ui.refused.map(|r| r.0), Some(Refusal::Job));
        // the views take their own head while a job runs, and no place is taken
        assert!(a.jog_head().is_none());
        a.act(Act::Jog(JogAct::Save(2)));
        a.act(Act::Jog(JogAct::Origin));
        assert_eq!(a.jog_ui.jog.slot(2), None);
        assert_eq!(a.jog_ui.jog.origin(), Pos::HOME);
        a.running = None;
        press(&mut a, Dir::XPlus);
        assert!(a.jog_ui.jog.pos().x > at.x);
    }

    #[test]
    fn the_travel_is_the_table_and_the_board_and_the_head_stays_inside_it() {
        let mut a = app();
        a.act(Act::Jog(JogAct::Toggle));
        a.act(Act::Jog(JogAct::Step(4)));
        for _ in 0..100 {
            press(&mut a, Dir::XPlus);
        }
        assert_eq!(a.jog_ui.jog.pos().x, a.table.width_mm);
        assert_eq!(a.jog_ui.refused.map(|r| r.0), Some(Refusal::Limit));
        for _ in 0..100 {
            press(&mut a, Dir::YPlus);
        }
        assert_eq!(a.jog_ui.jog.pos().y, a.table.depth_mm);
        // Z stops at the surface of the board
        for _ in 0..10 {
            press(&mut a, Dir::ZDown);
        }
        assert!((a.jog_ui.jog.pos().z - a.jog_board_z()).abs() < 1e-9);
        // and too low to go across: Z goes up first
        press(&mut a, Dir::XMinus);
        assert_eq!(a.jog_ui.refused.map(|r| r.0), Some(Refusal::RaiseZ));
        let x = a.jog_ui.jog.pos().x;
        a.act(Act::Jog(JogAct::Home));
        assert_eq!(a.jog_ui.jog.pos(), Pos::HOME);
        assert!(x > 0.0);
    }

    #[test]
    fn the_places_the_origin_and_the_choices_are_kept_in_the_file_and_read_back() {
        let mut a = app();
        a.act(Act::Jog(JogAct::Toggle));
        a.act(Act::Jog(JogAct::Step(3)));
        press(&mut a, Dir::XPlus);
        press(&mut a, Dir::YPlus);
        a.act(Act::Jog(JogAct::Save(3)));
        a.act(Act::Jog(JogAct::Origin));
        a.act(Act::Jog(JogAct::Speed(Speed::Fast)));
        a.act(Act::Jog(JogAct::Dock));
        a.act(Act::Jog(JogAct::Continuous));
        let c = Config::load(&a.jog_ui.path);
        assert_eq!(c.step, 10.0);
        assert!(c.continuous);
        assert_eq!(c.speed, Speed::Fast);
        assert!(c.dock);
        let kept = Pos {
            x: 10.0,
            y: 10.0,
            z: 0.0,
        };
        assert_eq!(c.slots[3], Some(kept));
        assert_eq!(c.slots[0], None);
        assert_eq!(c.origin, kept);
        // a new window starts from the file
        let again = JogUi::new(a.jog_ui.path.clone());
        assert_eq!(again.jog.step_mm(), 10.0);
        assert!(again.jog.continuous());
        assert_eq!(again.jog.speed(), Speed::Fast);
        assert!(again.jog.docked());
        assert_eq!(again.jog.slot(3), Some(kept));
        assert_eq!(again.jog.origin(), kept);
        // going home and back to the place kept
        a.act(Act::Jog(JogAct::Continuous));
        assert!(!a.jog_ui.jog.continuous());
        a.act(Act::Jog(JogAct::Home));
        assert_eq!(a.jog_ui.jog.pos(), Pos::HOME);
        a.act(Act::Jog(JogAct::Go(3)));
        assert_eq!(a.jog_ui.jog.pos(), kept);
        // the zero of the work is the place kept: each axis goes there
        a.act(Act::Jog(JogAct::Home));
        a.act(Act::Jog(JogAct::Zero(Axis::X)));
        assert_eq!(a.jog_ui.jog.pos().x, 10.0);
        a.act(Act::Jog(JogAct::Zero(Axis::Y)));
        assert_eq!(a.jog_ui.jog.pos().y, 10.0);
    }

    #[test]
    fn the_card_is_dragged_by_its_top_stays_in_the_window_and_stays_put_when_docked() {
        let mut a = app();
        a.act(Act::Jog(JogAct::Toggle));
        let (x, y) = a.jog_panel_pos();
        // a press that did not start on the top moves nothing
        a.drag(Zone::Jog, (40.0, 30.0), false);
        assert_eq!(a.jog_panel_pos(), (x, y));
        a.jog_ui.grab = true;
        a.drag(Zone::Jog, (40.0, 30.0), false);
        assert_eq!(a.jog_panel_pos(), (x + 40.0, y + 30.0));
        a.drag(Zone::Jog, (50_000.0, 50_000.0), false);
        assert_eq!(a.jog_panel_pos(), (design_w() - W, DESIGN_H - H));
        a.drag(Zone::Jog, (-90_000.0, -90_000.0), false);
        assert_eq!(a.jog_panel_pos(), (0.0, 66.0));
        // the place that is not a number is brought in
        a.jog_ui.jog.set_panel((f32::NAN, f32::INFINITY));
        assert_eq!(a.jog_panel_pos(), (0.0, 66.0));
        // docked: it is at its own place and does not move
        a.act(Act::Jog(JogAct::Dock));
        assert_eq!(a.jog_panel_pos(), DOCK);
        a.drag(Zone::Jog, (40.0, 30.0), false);
        assert_eq!(a.jog_panel_pos(), DOCK);
        // a double click on it does not send it anywhere
        a.reset_view(Zone::Jog);
        assert_eq!(a.jog_panel_pos(), DOCK);
    }

    #[test]
    fn the_panel_paints_in_every_language_and_every_theme_without_a_cut_text() {
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        let mut a = app();
        a.screen = Screen::Viewer;
        a.act(Act::Jog(JogAct::Toggle));
        a.act(Act::Jog(JogAct::Save(0)));
        press(&mut a, Dir::XPlus);
        press(&mut a, Dir::ZDown);
        a.act(Act::Jog(JogAct::Save(1)));
        a.act(Act::Jog(JogAct::Origin));
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(design_w(), DESIGN_H));
        let paint = |a: &mut App| {
            let input = egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| {
                let p = ui.ctx().layer_painter(egui::LayerId::background());
                a.paint(&p, rect);
            });
            out.textures_delta.clear();
        };
        // every look of the buttons paints
        for id in ThemeId::ALL {
            a.set_theme(id);
            paint(&mut a);
            paint(&mut a);
        }
        let mut found: Vec<String> = Vec::new();
        for lang in Lang::ALL {
            a.set_lang(lang);
            // the foot says each thing in turn: nothing, the emergency stop, a job, a limit and a head that is too low
            for why in 0..5 {
                a.estop = why == 1;
                a.jog_ui.refused = match why {
                    2 => Some((Refusal::Job, Instant::now())),
                    3 => Some((Refusal::Limit, Instant::now())),
                    4 => Some((Refusal::RaiseZ, Instant::now())),
                    _ => None,
                };
                paint(&mut a);
                CLIPPED.with(|c| c.borrow_mut().clear());
                paint(&mut a);
                for t in CLIPPED.with(|c| std::mem::take(&mut *c.borrow_mut())) {
                    found.push(format!("{}: {t}", lang.code()));
                }
            }
        }
        a.estop = false;
        found.sort();
        found.dedup();
        assert!(
            found.is_empty(),
            "{} texts are cut:\n{}",
            found.len(),
            found.join("\n")
        );
        // the four arrows of the cross and the two of Z were drawn for the pointer to find
        assert_eq!(a.jog_ui.rects.borrow().len(), 6);
    }

    #[test]
    fn the_panel_is_closed_when_the_viewer_is_left_and_a_job_takes_the_head_back() {
        let ctx = egui::Context::default();
        let mut a = app();
        a.act(Act::Jog(JogAct::Toggle));
        press(&mut a, Dir::XPlus);
        assert!(a.jog_ui.jog.touched());
        a.running = Some(Program::Drill);
        a.jog_tick(&ctx);
        assert!(!a.jog_ui.jog.touched());
        assert_eq!(a.jog_ui.jog.pos(), Pos::HOME);
        a.running = None;
        a.screen = Screen::Launcher;
        a.jog_tick(&ctx);
        assert!(!a.jog_ui.open);
    }

    #[test]
    fn the_words_of_the_panel_exist_in_all_the_languages() {
        for lang in Lang::ALL {
            for t in [
                Text::JogTitle,
                Text::JogStep,
                Text::JogContinuous,
                Text::JogSpeed,
                Text::JogSlow,
                Text::JogMedium,
                Text::JogFast,
                Text::JogMachine,
                Text::JogWork,
                Text::JogSetOrigin,
                Text::JogHome,
                Text::JogSaved,
                Text::JogSave,
                Text::JogGo,
                Text::JogBusyJob,
                Text::JogBusyStop,
                Text::JogLimit,
                Text::JogRaiseZ,
                Text::JogLive,
                Text::JogDemo,
            ] {
                assert!(!tr(lang, t).is_empty(), "{lang:?} {t:?}");
            }
        }
    }
}
