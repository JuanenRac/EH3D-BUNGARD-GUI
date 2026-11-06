//! The panel that moves the machine by hand, without its window: the step and the speed chosen, where the head is, the limits of its travel, the orders that a
//! press of a button sends and what stops them. The window (`gui::jogpanel`) only draws this and passes the presses; the orders go to the machine through the
//! engines of the classic programs (the same jog the Offset window of DRILLPRO and ROUTEPRO uses), or only to the picture when there is no link.
//!
//! The head is at `pos` in millimetres of the table (X, Y, as the views draw them) and Z is the depth below the home position, positive downwards, as the
//! steps of the engines count it. Every move is an [`Order`]: a distance relative to where the head is.

use std::path::Path;

/// The starting values, in the same format as the file of the person (see `jog_defaults.cfg`).
pub const DEFAULTS: &str = include_str!("jog_defaults.cfg");

/// How many positions the person can keep.
pub const SLOTS: usize = 4;

/// The most steps the list may have.
const MAX_STEPS: usize = 8;
/// The longest step a list may name, in mm.
const MAX_STEP_MM: f64 = 500.0;
/// The most stretches of a held arrow that one frame may send (a window that was dragged stops the clock for a while).
const MAX_CHUNKS_PER_TICK: usize = 8;
/// One mil, the unit of the jog of the engines, in mm.
const MIL_MM: f64 = 0.0254;
/// Distances below this are no distance.
const EPS: f64 = 1e-9;

/// A point of the machine: X and Y along the table, Z downwards from home.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Pos {
    pub const HOME: Pos = Pos {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    fn minus(self, o: Pos) -> Pos {
        Pos {
            x: self.x - o.x,
            y: self.y - o.y,
            z: self.z - o.z,
        }
    }
}

/// The six arrows of the panel: the cross of X and Y, and Z up and down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    XMinus,
    XPlus,
    YMinus,
    YPlus,
    ZUp,
    ZDown,
}

impl Dir {
    pub const ALL: [Dir; 6] = [
        Dir::XMinus,
        Dir::XPlus,
        Dir::YMinus,
        Dir::YPlus,
        Dir::ZUp,
        Dir::ZDown,
    ];

    /// The direction as a vector of length one (Z up is less depth).
    pub fn unit(self) -> (f64, f64, f64) {
        match self {
            Dir::XMinus => (-1.0, 0.0, 0.0),
            Dir::XPlus => (1.0, 0.0, 0.0),
            Dir::YMinus => (0.0, -1.0, 0.0),
            Dir::YPlus => (0.0, 1.0, 0.0),
            Dir::ZUp => (0.0, 0.0, -1.0),
            Dir::ZDown => (0.0, 0.0, 1.0),
        }
    }

    fn across(self) -> bool {
        !matches!(self, Dir::ZUp | Dir::ZDown)
    }
}

/// One of the axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
    Z,
}

/// The three speeds of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Speed {
    Slow,
    Medium,
    Fast,
}

impl Speed {
    pub const ALL: [Speed; 3] = [Speed::Slow, Speed::Medium, Speed::Fast];

    fn index(self) -> usize {
        match self {
            Speed::Slow => 0,
            Speed::Medium => 1,
            Speed::Fast => 2,
        }
    }

    fn from_word(w: &str) -> Option<Speed> {
        match w.trim().to_ascii_lowercase().as_str() {
            "slow" => Some(Speed::Slow),
            "medium" => Some(Speed::Medium),
            "fast" => Some(Speed::Fast),
            _ => None,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Speed::Slow => "slow",
            Speed::Medium => "medium",
            Speed::Fast => "fast",
        }
    }
}

/// Why a press did not move the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// A job is running.
    Job,
    /// The emergency stop is on.
    Estop,
    /// The head is already at the end of the travel that way.
    Limit,
    /// The head is too low to move across the board: Z goes up first.
    RaiseZ,
}

/// What forbids a move at this moment, told by the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Gate {
    pub job_running: bool,
    pub estop: bool,
}

impl Gate {
    fn check(self) -> Result<(), Refusal> {
        if self.estop {
            Err(Refusal::Estop)
        } else if self.job_running {
            Err(Refusal::Job)
        } else {
            Ok(())
        }
    }
}

/// A move relative to where the head is, in mm (Z positive downwards), with the feed it is asked at, in mm per minute.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Order {
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
    pub feed: f64,
}

/// The travel the head may use: the table along X and Y, and Z from home down to the surface of the board (and a little below when the person allows it).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub x_max: f64,
    pub y_max: f64,
    /// The deepest Z, in mm below home.
    pub z_floor: f64,
    /// Across the board the head must be at this depth or higher.
    pub safe_z: f64,
}

impl Limits {
    /// The table `width` x `depth`, the surface of the board `board_z` below home, the clearance the head keeps over it when it moves across and how far below
    /// the surface the tool may go.
    pub fn new(width: f64, depth: f64, board_z: f64, clearance: f64, below: f64) -> Limits {
        let z_floor = (board_z + below.max(0.0)).max(0.0);
        Limits {
            x_max: width.max(0.0),
            y_max: depth.max(0.0),
            z_floor,
            safe_z: (board_z - clearance.max(0.0)).clamp(0.0, z_floor),
        }
    }

    fn clamp(&self, p: Pos) -> Pos {
        Pos {
            x: p.x.clamp(0.0, self.x_max),
            y: p.y.clamp(0.0, self.y_max),
            z: p.z.clamp(0.0, self.z_floor),
        }
    }
}

/// What is read from the file and written back: the lists, the defaults, the saved places and the place of the panel.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub steps: Vec<f64>,
    pub step: f64,
    pub continuous: bool,
    pub feeds: [f64; 3],
    pub speed: Speed,
    pub chunk_mm: f64,
    pub clearance_mm: f64,
    pub below_board_mm: f64,
    pub panel: (f32, f32),
    pub dock: bool,
    pub origin: Pos,
    pub slots: [Option<Pos>; SLOTS],
}

fn numbers(text: &str) -> Option<Vec<f64>> {
    text.split(',')
        .map(|w| w.trim().parse::<f64>().ok().filter(|v| v.is_finite()))
        .collect()
}

fn triple(text: &str) -> Option<Pos> {
    match numbers(text)?.as_slice() {
        &[x, y, z] => Some(Pos { x, y, z }),
        _ => None,
    }
}

impl Config {
    /// The starting values, from `jog_defaults.cfg`.
    pub fn defaults() -> Config {
        let mut c = Config {
            steps: vec![1.0],
            step: 1.0,
            continuous: false,
            feeds: [300.0, 1200.0, 3000.0],
            speed: Speed::Medium,
            chunk_mm: 2.0,
            clearance_mm: 5.0,
            below_board_mm: 0.0,
            panel: (24.0, 96.0),
            dock: false,
            origin: Pos::HOME,
            slots: [None; SLOTS],
        };
        c.apply(DEFAULTS);
        c
    }

    /// The defaults with the file of the person over them: a line that is wrong keeps the value it had.
    pub fn parse(text: &str) -> Config {
        let mut c = Config::defaults();
        c.apply(text);
        c
    }

    /// Reads the file; a file that is not there gives the defaults.
    pub fn load(path: &Path) -> Config {
        std::fs::read_to_string(path)
            .map(|t| Config::parse(&t))
            .unwrap_or_else(|_| Config::defaults())
    }

    fn apply(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            let positive = |v: &str| v.parse::<f64>().ok().filter(|n| n.is_finite() && *n > 0.0);
            match key {
                "steps" => {
                    if let Some(list) = numbers(value) {
                        let ok = !list.is_empty()
                            && list.len() <= MAX_STEPS
                            && list.iter().all(|s| *s > 0.0 && *s <= MAX_STEP_MM);
                        if ok {
                            self.steps = list;
                        }
                    }
                }
                "step" => {
                    if let Some(v) = positive(value) {
                        self.step = v;
                    }
                }
                "continuous" => match value {
                    "true" => self.continuous = true,
                    "false" => self.continuous = false,
                    _ => {}
                },
                "speed_slow" | "speed_medium" | "speed_fast" => {
                    if let Some(v) = positive(value) {
                        let which = match key {
                            "speed_slow" => Speed::Slow,
                            "speed_medium" => Speed::Medium,
                            _ => Speed::Fast,
                        };
                        self.feeds[which.index()] = v;
                    }
                }
                "speed" => {
                    if let Some(s) = Speed::from_word(value) {
                        self.speed = s;
                    }
                }
                "chunk_mm" => {
                    if let Some(v) = positive(value) {
                        self.chunk_mm = v.min(50.0);
                    }
                }
                "clearance_mm" => {
                    if let Some(v) = value
                        .parse::<f64>()
                        .ok()
                        .filter(|n| *n >= 0.0 && n.is_finite())
                    {
                        self.clearance_mm = v;
                    }
                }
                "below_board_mm" => {
                    if let Some(v) = value
                        .parse::<f64>()
                        .ok()
                        .filter(|n| *n >= 0.0 && n.is_finite())
                    {
                        self.below_board_mm = v;
                    }
                }
                "panel" => {
                    if let Some(&[x, y]) = numbers(value).as_deref() {
                        self.panel = (x as f32, y as f32);
                    }
                }
                "dock" => match value {
                    "true" => self.dock = true,
                    "false" => self.dock = false,
                    _ => {}
                },
                "origin" => {
                    if let Some(p) = triple(value) {
                        self.origin = p;
                    }
                }
                _ => {
                    let slot = key
                        .strip_prefix("slot")
                        .and_then(|n| n.parse::<usize>().ok())
                        .filter(|n| (1..=SLOTS).contains(n));
                    if let Some(n) = slot {
                        if value.is_empty() {
                            self.slots[n - 1] = None;
                        } else if let Some(p) = triple(value) {
                            self.slots[n - 1] = Some(p);
                        }
                    }
                }
            }
        }
    }

    /// The file as it is written back: every key, the lists and the places the person keeps.
    pub fn to_text(&self) -> String {
        let list: Vec<String> = self.steps.iter().map(|s| number(*s)).collect();
        let mut t = String::new();
        t.push_str(&format!("steps = {}\n", list.join(", ")));
        t.push_str(&format!("step = {}\n", number(self.step)));
        t.push_str(&format!("continuous = {}\n", self.continuous));
        for s in Speed::ALL {
            t.push_str(&format!(
                "speed_{} = {}\n",
                s.word(),
                number(self.feeds[s.index()])
            ));
        }
        t.push_str(&format!("speed = {}\n", self.speed.word()));
        t.push_str(&format!("chunk_mm = {}\n", number(self.chunk_mm)));
        t.push_str(&format!("clearance_mm = {}\n", number(self.clearance_mm)));
        t.push_str(&format!(
            "below_board_mm = {}\n",
            number(self.below_board_mm)
        ));
        t.push_str(&format!("panel = {}, {}\n", self.panel.0, self.panel.1));
        t.push_str(&format!("dock = {}\n", self.dock));
        t.push_str(&format!(
            "origin = {}, {}, {}\n",
            number(self.origin.x),
            number(self.origin.y),
            number(self.origin.z)
        ));
        for (i, s) in self.slots.iter().enumerate() {
            match s {
                Some(p) => t.push_str(&format!(
                    "slot{} = {}, {}, {}\n",
                    i + 1,
                    number(p.x),
                    number(p.y),
                    number(p.z)
                )),
                None => t.push_str(&format!("slot{} =\n", i + 1)),
            }
        }
        t
    }

    /// Writes the file whole to a side file and renames it over the other.
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("tmp");
        if let Err(e) = std::fs::write(&tmp, self.to_text()) {
            let _ = std::fs::remove_file(&tmp);
            return Err(e);
        }
        std::fs::rename(&tmp, path).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}

/// A number without the zeros that mean nothing (0.1, 10, 0.01).
pub fn number(v: f64) -> String {
    let t = format!("{v:.4}");
    t.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// The state of the panel.
#[derive(Debug, Clone)]
pub struct Jog {
    cfg: Config,
    limits: Limits,
    step: usize,
    continuous: bool,
    speed: Speed,
    /// Where the head has been sent.
    pos: Pos,
    /// Where the picture of the head is, on its way to `pos` at the speed of the move.
    shown: Pos,
    held: Option<Dir>,
    /// Seconds that have passed towards the next stretch of a held arrow.
    owed_s: f64,
    /// The panel has moved the head, so the views show it.
    touched: bool,
    /// The part of a mil that the last orders did not reach, per axis.
    carry: (f64, f64, f64),
}

impl Jog {
    pub fn new(cfg: Config, limits: Limits) -> Jog {
        let step = nearest(&cfg.steps, cfg.step);
        let (continuous, speed) = (cfg.continuous, cfg.speed);
        Jog {
            cfg,
            limits,
            step,
            continuous,
            speed,
            pos: Pos::HOME,
            shown: Pos::HOME,
            held: None,
            owed_s: 0.0,
            touched: false,
            carry: (0.0, 0.0, 0.0),
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// The config with what the person chose, to be written.
    pub fn config_to_save(&self) -> Config {
        let mut c = self.cfg.clone();
        c.step = self.step_mm();
        c.continuous = self.continuous;
        c.speed = self.speed;
        c
    }

    pub fn limits(&self) -> Limits {
        self.limits
    }

    /// The machine changed (another table, another board): the head stays where it is inside the new travel.
    pub fn set_limits(&mut self, limits: Limits) {
        self.limits = limits;
        self.pos = limits.clamp(self.pos);
        self.shown = limits.clamp(self.shown);
    }

    // ------------------------------------------------------------------------------------------------------------- the choices

    pub fn steps(&self) -> &[f64] {
        &self.cfg.steps
    }

    pub fn step_index(&self) -> usize {
        self.step
    }

    pub fn step_mm(&self) -> f64 {
        self.cfg.steps.get(self.step).copied().unwrap_or(1.0)
    }

    pub fn set_step(&mut self, i: usize) {
        if i < self.cfg.steps.len() {
            self.step = i;
        }
    }

    /// The next longer step (the key +), or the same at the end of the list.
    pub fn step_up(&mut self) {
        self.set_step(self.step + 1);
    }

    /// The next shorter step (the key -).
    pub fn step_down(&mut self) {
        self.set_step(self.step.saturating_sub(1));
    }

    pub fn continuous(&self) -> bool {
        self.continuous
    }

    pub fn set_continuous(&mut self, on: bool) {
        self.continuous = on;
        if !on {
            self.held = None;
        }
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }

    pub fn set_speed(&mut self, s: Speed) {
        self.speed = s;
    }

    /// The feed of the speed chosen, in mm per minute.
    pub fn feed(&self) -> f64 {
        self.cfg.feeds[self.speed.index()]
    }

    pub fn feed_of(&self, s: Speed) -> f64 {
        self.cfg.feeds[s.index()]
    }

    // ------------------------------------------------------------------------------------------------------------- where it is

    pub fn pos(&self) -> Pos {
        self.pos
    }

    /// The position in the coordinates of the work: from the work origin.
    pub fn work_pos(&self) -> Pos {
        self.pos.minus(self.cfg.origin)
    }

    pub fn origin(&self) -> Pos {
        self.cfg.origin
    }

    /// Where the head is drawn.
    pub fn shown(&self) -> Pos {
        self.shown
    }

    /// The panel has moved the head since it was parked.
    pub fn touched(&self) -> bool {
        self.touched
    }

    /// The head is on its way, or an arrow is held.
    pub fn busy(&self) -> bool {
        self.held.is_some() || self.shown != self.pos
    }

    pub fn held(&self) -> Option<Dir> {
        self.held
    }

    /// The head is back at home and the views take over again (a job ran, the machine went home).
    pub fn park(&mut self) {
        self.pos = Pos::HOME;
        self.shown = Pos::HOME;
        self.held = None;
        self.touched = false;
        self.carry = (0.0, 0.0, 0.0);
    }

    // ------------------------------------------------------------------------------------------------------------- moves

    fn order(&self, d: (f64, f64, f64)) -> Order {
        Order {
            dx: d.0,
            dy: d.1,
            dz: d.2,
            feed: self.feed(),
        }
    }

    /// Moves the head `dist` mm along `dir`, as far as the travel allows; the distance left over is not moved.
    fn go(&mut self, dir: Dir, dist: f64) -> Result<Order, Refusal> {
        let u = dir.unit();
        if dir.across() && self.pos.z > self.limits.safe_z + EPS {
            return Err(Refusal::RaiseZ);
        }
        let want = Pos {
            x: self.pos.x + u.0 * dist,
            y: self.pos.y + u.1 * dist,
            z: self.pos.z + u.2 * dist,
        };
        let to = self.limits.clamp(want);
        // a move that the travel does not cut is the distance asked for, not a difference of two sums
        let d = if to == want {
            (u.0 * dist, u.1 * dist, u.2 * dist)
        } else {
            (to.x - self.pos.x, to.y - self.pos.y, to.z - self.pos.z)
        };
        if d.0.abs() + d.1.abs() + d.2.abs() < EPS {
            return Err(Refusal::Limit);
        }
        self.pos = to;
        self.touched = true;
        Ok(self.order(d))
    }

    /// An arrow is pressed. In the step mode the head moves one step; in the continuous mode the arrow is held and [`Jog::tick`] sends the stretches.
    pub fn press(&mut self, dir: Dir, gate: Gate) -> Result<Vec<Order>, Refusal> {
        gate.check()?;
        if self.continuous {
            // the first stretch goes at once; an arrow that cannot move the head is not held
            if dir.across() && self.pos.z > self.limits.safe_z + EPS {
                return Err(Refusal::RaiseZ);
            }
            self.held = Some(dir);
            self.owed_s = self.chunk_s();
            return Ok(Vec::new());
        }
        self.go(dir, self.step_mm()).map(|o| vec![o])
    }

    /// The arrow is let go.
    pub fn release(&mut self) {
        self.held = None;
    }

    fn chunk_s(&self) -> f64 {
        self.cfg.chunk_mm / (self.feed() / 60.0).max(EPS)
    }

    /// `dt` seconds have passed: the stretches of a held arrow that are due go out, one every `chunk_mm / speed`.
    pub fn tick(&mut self, dt: f64, gate: Gate) -> Result<Vec<Order>, Refusal> {
        let Some(dir) = self.held else {
            return Ok(Vec::new());
        };
        if let Err(r) = gate.check() {
            self.held = None;
            return Err(r);
        }
        self.owed_s += dt.max(0.0);
        let every = self.chunk_s();
        let mut orders = Vec::new();
        while self.owed_s + EPS >= every && orders.len() < MAX_CHUNKS_PER_TICK {
            self.owed_s -= every;
            match self.go(dir, self.cfg.chunk_mm) {
                Ok(o) => orders.push(o),
                Err(r) => {
                    // the end of the travel ends the hold; what was sent before it still goes
                    self.held = None;
                    return if orders.is_empty() {
                        Err(r)
                    } else {
                        Ok(orders)
                    };
                }
            }
        }
        if orders.len() >= MAX_CHUNKS_PER_TICK {
            self.owed_s = 0.0;
        }
        Ok(orders)
    }

    /// Moves the picture of the head towards where it was sent, at the feed of the move.
    pub fn advance(&mut self, dt: f64) {
        let to = self.pos.minus(self.shown);
        let dist = (to.x * to.x + to.y * to.y + to.z * to.z).sqrt();
        if dist < EPS {
            self.shown = self.pos;
            return;
        }
        let step = self.feed() / 60.0 * dt.max(0.0);
        if step >= dist {
            self.shown = self.pos;
        } else {
            let k = step / dist;
            self.shown = Pos {
                x: self.shown.x + to.x * k,
                y: self.shown.y + to.y * k,
                z: self.shown.z + to.z * k,
            };
        }
    }

    /// Esc: the arrow is let go and, with no machine behind the picture (`live` false), the head stops where it is drawn. A real machine has been sent its orders
    /// and cannot be called back.
    pub fn stop(&mut self, live: bool) {
        self.held = None;
        if !live {
            self.pos = self.shown;
        }
    }

    /// Goes to `target` (inside the travel): Z goes up to the safe height first if the head is lower and has to move across, then across, then Z to its depth.
    pub fn goto(&mut self, target: Pos, gate: Gate) -> Result<Vec<Order>, Refusal> {
        gate.check()?;
        self.held = None;
        let to = self.limits.clamp(target);
        let mut orders = Vec::new();
        let mut z = self.pos.z;
        let across = (to.x - self.pos.x).abs() > EPS || (to.y - self.pos.y).abs() > EPS;
        if across && z > self.limits.safe_z + EPS {
            let up = self.limits.safe_z - z;
            orders.push(self.order((0.0, 0.0, up)));
            z = self.limits.safe_z;
        }
        if across {
            orders.push(self.order((to.x - self.pos.x, to.y - self.pos.y, 0.0)));
        }
        if (to.z - z).abs() > EPS {
            orders.push(self.order((0.0, 0.0, to.z - z)));
        }
        if orders.is_empty() {
            return Ok(orders);
        }
        self.pos = to;
        self.touched = true;
        Ok(orders)
    }

    /// The axis goes to its zero of the work.
    pub fn go_zero(&mut self, axis: Axis, gate: Gate) -> Result<Vec<Order>, Refusal> {
        let o = self.cfg.origin;
        let mut t = self.pos;
        match axis {
            Axis::X => t.x = o.x,
            Axis::Y => t.y = o.y,
            Axis::Z => t.z = o.z,
        }
        self.goto(t, gate)
    }

    /// All the axes go to the machine zero: across (after Z goes up, if the head is too low for that), then Z up.
    pub fn home(&mut self, gate: Gate) -> Result<Vec<Order>, Refusal> {
        self.goto(Pos::HOME, gate)
    }

    /// The place where the head is now is the zero of the work.
    pub fn set_origin_here(&mut self) {
        self.cfg.origin = self.pos;
    }

    pub fn slot(&self, i: usize) -> Option<Pos> {
        self.cfg.slots.get(i).copied().flatten()
    }

    /// Keeps the place of the head in slot `i`.
    pub fn save_slot(&mut self, i: usize) {
        let p = self.pos;
        if let Some(s) = self.cfg.slots.get_mut(i) {
            *s = Some(p);
        }
    }

    /// Goes to the place kept in slot `i` (nothing happens for an empty slot).
    pub fn goto_slot(&mut self, i: usize, gate: Gate) -> Result<Vec<Order>, Refusal> {
        match self.slot(i) {
            Some(p) => self.goto(p, gate),
            None => {
                gate.check()?;
                Ok(Vec::new())
            }
        }
    }

    pub fn panel(&self) -> (f32, f32) {
        self.cfg.panel
    }

    pub fn set_panel(&mut self, p: (f32, f32)) {
        self.cfg.panel = p;
    }

    pub fn docked(&self) -> bool {
        self.cfg.dock
    }

    pub fn set_docked(&mut self, on: bool) {
        self.cfg.dock = on;
    }

    /// The order in mils (the unit of the jog of the engines), keeping what rounding left over for the next order: a step of 0.01 mm is less than a mil, and
    /// ten of them make up the mils they are worth.
    pub fn mils_of(&mut self, o: Order) -> (i32, i32, i32) {
        let take = |d: f64, carry: f64| {
            let v = d / MIL_MM + carry;
            let r = v.round();
            (r as i32, v - r)
        };
        let (mx, cx) = take(o.dx, self.carry.0);
        let (my, cy) = take(o.dy, self.carry.1);
        let (mz, cz) = take(o.dz, self.carry.2);
        self.carry = (cx, cy, cz);
        (mx, my, mz)
    }
}

/// The index of the step nearest to `v`.
fn nearest(list: &[f64], v: f64) -> usize {
    list.iter()
        .enumerate()
        .min_by(|a, b| (a.1 - v).abs().total_cmp(&(b.1 - v).abs()))
        .map_or(0, |(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const OPEN: Gate = Gate {
        job_running: false,
        estop: false,
    };

    /// A table of 500 x 300, the board 24 below home, 4 of clearance (so across only at 20 or higher) and nothing below the board.
    fn limits() -> Limits {
        Limits::new(500.0, 300.0, 24.0, 4.0, 0.0)
    }

    /// A config whose numbers are exact in binary, so the pace of the held arrow can be checked with `==`.
    fn cfg() -> Config {
        Config::parse("speed_medium = 600\nchunk_mm = 2.5\nspeed = medium\nstep = 1\n")
    }

    fn jog() -> Jog {
        Jog::new(cfg(), limits())
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn only(r: Result<Vec<Order>, Refusal>) -> Order {
        let v = r.expect("the move was refused");
        assert_eq!(v.len(), 1, "{v:?}");
        v[0]
    }

    #[test]
    fn the_defaults_are_the_file_that_comes_with_the_program() {
        let c = Config::defaults();
        assert_eq!(c.steps, vec![0.01, 0.1, 1.0, 10.0, 50.0]);
        assert_eq!(c.step, 1.0);
        assert!(!c.continuous);
        assert_eq!(c.feeds, [300.0, 1200.0, 3000.0]);
        assert_eq!(c.speed, Speed::Medium);
        assert_eq!(c.chunk_mm, 2.0);
        assert_eq!(c.clearance_mm, 5.0);
        assert_eq!(c.below_board_mm, 0.0);
        assert_eq!(c.panel, (24.0, 96.0));
        assert!(!c.dock);
        assert_eq!(c.origin, Pos::HOME);
        assert_eq!(c.slots, [None; SLOTS]);
        let j = Jog::new(c, limits());
        assert_eq!(j.step_mm(), 1.0);
        assert_eq!(j.step_index(), 2);
    }

    #[test]
    fn the_file_of_the_person_changes_the_steps_and_a_wrong_line_keeps_the_value() {
        let c = Config::parse(
            "steps = 0.05, 0.5, 5, 25 # mine\nstep = 5\nspeed = fast\ncontinuous = true\nspeed_fast = 4000\nchunk_mm = banana\n",
        );
        assert_eq!(c.steps, vec![0.05, 0.5, 5.0, 25.0]);
        assert_eq!(c.step, 5.0);
        assert_eq!(c.speed, Speed::Fast);
        assert!(c.continuous);
        assert_eq!(c.feeds[2], 4000.0);
        assert_eq!(c.chunk_mm, 2.0);
        // a list that is empty, has a zero, a negative, a number that is not one, too many or one that is too long is not taken
        for bad in [
            "steps =",
            "steps = 0, 1",
            "steps = -1, 2",
            "steps = 1, x",
            "steps = 1, 2, 3, 4, 5, 6, 7, 8, 9",
            "steps = 1000",
            "speed = warp",
            "continuous = maybe",
            "speed_slow = -3",
            "panel = 1",
            "origin = 1, 2",
            "slot9 = 1, 2, 3",
        ] {
            assert_eq!(Config::parse(bad), Config::defaults(), "{bad}");
        }
        // the step of the file is chosen in the list, or the nearest
        let j = Jog::new(Config::parse("step = 7"), limits());
        assert_eq!(j.step_mm(), 10.0);
    }

    #[test]
    fn what_the_person_keeps_is_written_and_read_back() {
        let mut j = jog();
        j.set_step(4);
        j.set_continuous(true);
        j.set_speed(Speed::Fast);
        j.set_panel((300.5, 80.0));
        j.set_docked(true);
        j.set_continuous(false);
        j.go(Dir::XPlus, 40.0).unwrap();
        j.go(Dir::YPlus, 12.5).unwrap();
        j.go(Dir::ZDown, 7.25).unwrap();
        j.set_origin_here();
        j.save_slot(1);
        let text = j.config_to_save().to_text();
        let back = Config::parse(&text);
        assert_eq!(back, j.config_to_save());
        assert_eq!(back.step, 50.0);
        assert_eq!(back.speed, Speed::Fast);
        assert_eq!(back.panel, (300.5, 80.0));
        assert!(back.dock);
        assert_eq!(
            back.origin,
            Pos {
                x: 40.0,
                y: 12.5,
                z: 7.25
            }
        );
        assert_eq!(back.slots[0], None);
        assert_eq!(back.slots[1], Some(back.origin));
        // and through the file
        let path = std::env::temp_dir().join(format!("eh3d-jog-test-{}.cfg", std::process::id()));
        back.save(&path).unwrap();
        assert_eq!(Config::load(&path), back);
        let _ = std::fs::remove_file(&path);
        assert_eq!(
            Config::load(&path.with_extension("missing")),
            Config::defaults()
        );
    }

    #[test]
    fn each_arrow_sends_a_move_of_the_step_that_way_and_the_head_goes_there() {
        // the head starts at the middle of the table and 10 above the board, so every way is open
        let mut j = jog();
        j.go(Dir::XPlus, 100.0).unwrap();
        j.go(Dir::YPlus, 100.0).unwrap();
        j.go(Dir::ZDown, 10.0).unwrap();
        j.set_step(1);
        let expect = [
            (Dir::XMinus, (-0.1, 0.0, 0.0)),
            (Dir::XPlus, (0.1, 0.0, 0.0)),
            (Dir::YMinus, (0.0, -0.1, 0.0)),
            (Dir::YPlus, (0.0, 0.1, 0.0)),
            (Dir::ZUp, (0.0, 0.0, -0.1)),
            (Dir::ZDown, (0.0, 0.0, 0.1)),
        ];
        for (dir, (dx, dy, dz)) in expect {
            let before = j.pos();
            let o = only(j.press(dir, OPEN));
            assert_eq!((o.dx, o.dy, o.dz), (dx, dy, dz), "{dir:?}");
            assert_eq!(o.feed, 600.0);
            let after = j.pos();
            assert!(near(after.x - before.x, dx), "{dir:?}");
            assert!(near(after.y - before.y, dy), "{dir:?}");
            assert!(near(after.z - before.z, dz), "{dir:?}");
        }
        // the steps of the list, one by one
        for (i, s) in [0.01, 0.1, 1.0, 10.0, 50.0].into_iter().enumerate() {
            j.set_step(i);
            assert_eq!(j.step_mm(), s);
            assert_eq!(only(j.press(Dir::XPlus, OPEN)).dx, s);
        }
        assert!(j.touched());
    }

    #[test]
    fn the_keys_of_the_step_walk_the_list_and_stop_at_its_ends() {
        let mut j = jog();
        assert_eq!(j.step_mm(), 1.0);
        j.step_up();
        j.step_up();
        assert_eq!(j.step_mm(), 50.0);
        j.step_up();
        assert_eq!(j.step_mm(), 50.0);
        for _ in 0..9 {
            j.step_down();
        }
        assert_eq!(j.step_mm(), 0.01);
        j.set_step(99);
        assert_eq!(j.step_index(), 0);
    }

    #[test]
    fn the_head_stops_at_the_ends_of_the_travel_and_the_last_part_is_moved() {
        let mut j = jog();
        j.set_step(4);
        // X: 500 mm of table in steps of 50 ends with a move of 0 and then the limit; 495 first leaves only 5
        j.go(Dir::XPlus, 495.0).unwrap();
        let o = only(j.press(Dir::XPlus, OPEN));
        assert_eq!(o.dx, 5.0);
        assert_eq!(j.pos().x, 500.0);
        assert_eq!(j.press(Dir::XPlus, OPEN), Err(Refusal::Limit));
        assert_eq!(j.pos().x, 500.0);
        // the other ends
        assert_eq!(j.press(Dir::YMinus, OPEN), Err(Refusal::Limit));
        assert_eq!(j.press(Dir::ZUp, OPEN), Err(Refusal::Limit));
        let o = only(j.press(Dir::YPlus, OPEN));
        assert_eq!(o.dy, 50.0);
        j.go(Dir::YPlus, 250.0).unwrap();
        assert_eq!(j.press(Dir::YPlus, OPEN), Err(Refusal::Limit));
        // Z stops at the surface of the board (24), and only at 20 or higher may the head go across
        j.set_step(3);
        j.go(Dir::ZDown, 10.0).unwrap();
        let o = only(j.press(Dir::ZDown, OPEN));
        assert_eq!(o.dz, 10.0);
        let o = only(j.press(Dir::ZDown, OPEN));
        assert_eq!(o.dz, 4.0);
        assert_eq!(j.pos().z, 24.0);
        assert_eq!(j.press(Dir::ZDown, OPEN), Err(Refusal::Limit));
    }

    #[test]
    fn the_head_does_not_go_across_when_it_is_too_low_but_z_still_moves() {
        let mut j = jog();
        j.set_step(3);
        j.press(Dir::ZDown, OPEN).unwrap();
        j.press(Dir::ZDown, OPEN).unwrap();
        assert_eq!(j.pos().z, 20.0);
        // exactly at the safe height is still allowed
        only(j.press(Dir::XPlus, OPEN));
        j.set_step(2);
        only(j.press(Dir::ZDown, OPEN));
        assert_eq!(j.pos().z, 21.0);
        let before = j.pos();
        for dir in [Dir::XMinus, Dir::XPlus, Dir::YMinus, Dir::YPlus] {
            assert_eq!(j.press(dir, OPEN), Err(Refusal::RaiseZ), "{dir:?}");
        }
        assert_eq!(j.pos(), before);
        only(j.press(Dir::ZUp, OPEN));
        assert_eq!(j.pos().z, 20.0);
        only(j.press(Dir::XMinus, OPEN));
    }

    #[test]
    fn a_job_or_the_emergency_stop_blocks_every_order() {
        let mut j = jog();
        j.go(Dir::XPlus, 10.0).unwrap();
        let before = j.pos();
        let job = Gate {
            job_running: true,
            estop: false,
        };
        let stop = Gate {
            job_running: false,
            estop: true,
        };
        let both = Gate {
            job_running: true,
            estop: true,
        };
        for dir in Dir::ALL {
            assert_eq!(j.press(dir, job), Err(Refusal::Job));
            assert_eq!(j.press(dir, stop), Err(Refusal::Estop));
        }
        // the emergency stop is the one that is told when both are on
        assert_eq!(j.press(Dir::XPlus, both), Err(Refusal::Estop));
        assert_eq!(j.goto(Pos::HOME, job), Err(Refusal::Job));
        assert_eq!(j.home(stop), Err(Refusal::Estop));
        assert_eq!(j.go_zero(Axis::X, job), Err(Refusal::Job));
        j.save_slot(0);
        assert_eq!(j.goto_slot(0, stop), Err(Refusal::Estop));
        assert_eq!(j.goto_slot(3, job), Err(Refusal::Job));
        assert_eq!(j.pos(), before);
        // and a held arrow is let go when it comes on
        j.set_continuous(true);
        j.press(Dir::XPlus, OPEN).unwrap();
        assert_eq!(j.held(), Some(Dir::XPlus));
        assert_eq!(j.tick(0.25, stop), Err(Refusal::Estop));
        assert_eq!(j.held(), None);
        assert_eq!(j.pos(), before);
    }

    #[test]
    fn a_held_arrow_sends_a_stretch_at_once_and_then_one_at_the_pace_of_the_speed() {
        let mut j = jog();
        j.set_continuous(true);
        // 600 mm/min is 10 mm/s: a stretch of 2.5 mm every quarter of a second
        assert_eq!(j.press(Dir::XPlus, OPEN), Ok(Vec::new()));
        assert_eq!(j.held(), Some(Dir::XPlus));
        let first = j.tick(0.0, OPEN).unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!((first[0].dx, first[0].feed), (2.5, 600.0));
        assert!(j.tick(0.125, OPEN).unwrap().is_empty());
        assert_eq!(j.tick(0.125, OPEN).unwrap().len(), 1);
        assert_eq!(j.pos().x, 5.0);
        // a long frame sends the stretches that are due, a few at most, and does not owe the rest
        let many = j.tick(1.0, OPEN).unwrap();
        assert_eq!(many.len(), 4);
        assert_eq!(j.pos().x, 15.0);
        let capped = j.tick(10.0, OPEN).unwrap();
        assert_eq!(capped.len(), MAX_CHUNKS_PER_TICK);
        assert_eq!(j.pos().x, 35.0);
        assert!(j.tick(0.0, OPEN).unwrap().is_empty());
        // letting go ends it
        j.release();
        assert_eq!(j.held(), None);
        assert!(j.tick(5.0, OPEN).unwrap().is_empty());
        assert_eq!(j.pos().x, 35.0);
        // the other speeds change the pace: 3000 mm/min is 50 mm/s, a stretch every 0.05 s
        let mut j = Jog::new(
            Config::parse("speed_fast = 3000\nchunk_mm = 2.5\nspeed = fast\ncontinuous = true"),
            limits(),
        );
        j.press(Dir::YPlus, OPEN).unwrap();
        assert_eq!(j.tick(0.125, OPEN).unwrap().len(), 3);
    }

    #[test]
    fn a_held_arrow_ends_at_the_limit_and_does_not_start_when_the_head_is_too_low() {
        let mut j = jog();
        j.set_continuous(true);
        j.go(Dir::XPlus, 497.5).unwrap();
        j.press(Dir::XPlus, OPEN).unwrap();
        let sent = j.tick(1.0, OPEN).unwrap();
        // 2.5 mm left to the end: one stretch goes and the hold ends
        assert_eq!(sent.len(), 1);
        assert_eq!(j.pos().x, 500.0);
        assert_eq!(j.held(), None);
        j.press(Dir::XPlus, OPEN).unwrap();
        assert_eq!(j.tick(1.0, OPEN), Err(Refusal::Limit));
        assert_eq!(j.held(), None);
        // too low to go across: the arrow is not held
        let mut j = jog();
        j.go(Dir::ZDown, 22.0).unwrap();
        j.set_continuous(true);
        assert_eq!(j.press(Dir::XPlus, OPEN), Err(Refusal::RaiseZ));
        assert_eq!(j.held(), None);
        // Z goes on being held
        j.press(Dir::ZDown, OPEN).unwrap();
        assert_eq!(j.tick(0.0, OPEN).unwrap().len(), 1);
        // leaving the continuous mode lets go
        j.set_continuous(false);
        assert_eq!(j.held(), None);
    }

    #[test]
    fn going_to_a_place_raises_z_first_then_goes_across_then_goes_down() {
        let mut j = jog();
        j.go(Dir::XPlus, 10.0).unwrap();
        j.go(Dir::YPlus, 10.0).unwrap();
        // the head is low: 22 is below the safe height of 20
        j.go(Dir::ZDown, 22.0).unwrap();
        let target = Pos {
            x: 50.0,
            y: 60.0,
            z: 0.0,
        };
        let orders = j.goto(target, OPEN).unwrap();
        let moves: Vec<(f64, f64, f64)> = orders.iter().map(|o| (o.dx, o.dy, o.dz)).collect();
        assert_eq!(
            moves,
            vec![(0.0, 0.0, -2.0), (40.0, 50.0, 0.0), (0.0, 0.0, -20.0)]
        );
        assert_eq!(j.pos(), target);
        // to a deeper place: across first at the height the head has, then down
        let deep = Pos {
            x: 55.0,
            y: 60.0,
            z: 24.0,
        };
        let orders = j.goto(deep, OPEN).unwrap();
        let moves: Vec<(f64, f64, f64)> = orders.iter().map(|o| (o.dx, o.dy, o.dz)).collect();
        assert_eq!(moves, vec![(5.0, 0.0, 0.0), (0.0, 0.0, 24.0)]);
        // only Z: one move; no change: none
        let up = Pos { z: 3.0, ..deep };
        assert_eq!(j.goto(up, OPEN).unwrap().len(), 1);
        assert!(j.goto(up, OPEN).unwrap().is_empty());
        // a place outside the travel is taken to its edge
        let out = Pos {
            x: 900.0,
            y: -5.0,
            z: 99.0,
        };
        j.goto(out, OPEN).unwrap();
        assert_eq!(
            j.pos(),
            Pos {
                x: 500.0,
                y: 0.0,
                z: 24.0
            }
        );
    }

    #[test]
    fn home_goes_to_the_machine_zero_with_z_up_first() {
        let mut j = jog();
        j.set_step(4);
        j.go(Dir::XPlus, 120.0).unwrap();
        j.go(Dir::YPlus, 80.0).unwrap();
        j.go(Dir::ZDown, 15.0).unwrap();
        let orders = j.home(OPEN).unwrap();
        let moves: Vec<(f64, f64, f64)> = orders.iter().map(|o| (o.dx, o.dy, o.dz)).collect();
        assert_eq!(moves, vec![(-120.0, -80.0, 0.0), (0.0, 0.0, -15.0)]);
        assert_eq!(j.pos(), Pos::HOME);
        assert!(j.home(OPEN).unwrap().is_empty());
    }

    #[test]
    fn the_work_origin_is_where_the_head_was_set_and_each_axis_goes_back_to_its_zero() {
        let mut j = jog();
        j.go(Dir::XPlus, 100.0).unwrap();
        j.go(Dir::YPlus, 40.0).unwrap();
        j.go(Dir::ZDown, 5.0).unwrap();
        // before the origin is set the work is the machine
        assert_eq!(j.work_pos(), j.pos());
        j.set_origin_here();
        assert_eq!(j.work_pos(), Pos::HOME);
        j.go(Dir::XPlus, 25.0).unwrap();
        j.go(Dir::YMinus, 10.0).unwrap();
        j.go(Dir::ZUp, 2.0).unwrap();
        assert_eq!(
            j.work_pos(),
            Pos {
                x: 25.0,
                y: -10.0,
                z: -2.0
            }
        );
        // X goes back to its zero of the work and nothing else moves
        let o = only(j.go_zero(Axis::X, OPEN));
        assert_eq!((o.dx, o.dy, o.dz), (-25.0, 0.0, 0.0));
        let o = only(j.go_zero(Axis::Y, OPEN));
        assert_eq!((o.dx, o.dy, o.dz), (0.0, 10.0, 0.0));
        let o = only(j.go_zero(Axis::Z, OPEN));
        assert_eq!((o.dx, o.dy, o.dz), (0.0, 0.0, 2.0));
        assert_eq!(j.work_pos(), Pos::HOME);
        assert!(j.go_zero(Axis::X, OPEN).unwrap().is_empty());
        // the machine position did not change meaning
        assert_eq!(
            j.pos(),
            Pos {
                x: 100.0,
                y: 40.0,
                z: 5.0
            }
        );
    }

    #[test]
    fn four_places_are_kept_and_gone_back_to_and_an_empty_one_does_nothing() {
        let mut j = jog();
        j.set_step(4);
        for i in 0..SLOTS {
            assert_eq!(j.slot(i), None);
        }
        j.go(Dir::XPlus, 50.0).unwrap();
        j.save_slot(0);
        j.go(Dir::YPlus, 100.0).unwrap();
        j.go(Dir::XPlus, 100.0).unwrap();
        j.save_slot(3);
        assert_eq!(
            j.slot(0),
            Some(Pos {
                x: 50.0,
                y: 0.0,
                z: 0.0
            })
        );
        assert_eq!(
            j.slot(3),
            Some(Pos {
                x: 150.0,
                y: 100.0,
                z: 0.0
            })
        );
        let o = only(j.goto_slot(0, OPEN));
        assert_eq!((o.dx, o.dy), (-100.0, -100.0));
        assert_eq!(j.pos().x, 50.0);
        assert!(j.goto_slot(1, OPEN).unwrap().is_empty());
        assert!(j.goto_slot(7, OPEN).unwrap().is_empty());
        j.save_slot(7);
        let o = only(j.goto_slot(3, OPEN));
        assert_eq!((o.dx, o.dy), (100.0, 100.0));
        // a table that is smaller takes the kept place to its edge
        j.set_limits(Limits::new(120.0, 300.0, 24.0, 4.0, 0.0));
        assert_eq!(j.pos().x, 120.0);
        j.goto_slot(0, OPEN).unwrap();
        j.goto_slot(3, OPEN).unwrap();
        assert_eq!(j.pos().x, 120.0);
        assert_eq!(j.pos().y, 100.0);
    }

    #[test]
    fn the_picture_follows_the_order_at_the_feed_and_stop_cuts_it_short_without_a_link() {
        let mut j = jog();
        j.set_step(4);
        j.go(Dir::XPlus, 50.0).unwrap();
        assert!(j.busy());
        assert_eq!(j.shown().x, 0.0);
        j.advance(2.0);
        // 600 mm/min = 10 mm/s
        assert!(near(j.shown().x, 20.0));
        j.advance(1.0);
        assert!(near(j.shown().x, 30.0));
        j.stop(false);
        assert!(near(j.pos().x, 30.0));
        j.advance(1.0);
        assert_eq!(j.shown(), j.pos());
        assert!(!j.busy());
        // with a machine behind it the orders have gone and the picture goes on to the end
        j.go(Dir::XPlus, 50.0).unwrap();
        j.advance(1.0);
        j.stop(true);
        assert!(near(j.pos().x, 80.0));
        j.advance(100.0);
        assert!(near(j.shown().x, 80.0));
        // parking sends the picture home and gives the views back
        j.park();
        assert!(!j.touched());
        assert_eq!((j.pos(), j.shown()), (Pos::HOME, Pos::HOME));
    }

    #[test]
    fn a_step_of_less_than_a_mil_is_not_lost_in_the_mils_sent_to_the_engine() {
        let mut j = jog();
        j.set_step(0);
        let mut total = (0, 0, 0);
        for _ in 0..127 {
            let o = only(j.press(Dir::XPlus, OPEN));
            let m = j.mils_of(o);
            total.0 += m.0;
            total.1 += m.1;
            total.2 += m.2;
        }
        // 127 steps of 0.01 mm are 1.27 mm, which is 50 mils
        assert_eq!(total, (50, 0, 0));
        // a move of a mil is a mil, in both ways and in Z, and a step of 1 mm is 39.37 mils and the 0.37 goes on to the next
        let mut j = jog();
        let m = j.mils_of(Order {
            dx: 0.0254,
            dy: -0.0254,
            dz: 0.0508,
            feed: 1.0,
        });
        assert_eq!(m, (1, -1, 2));
        let mut j = jog();
        let a = j.mils_of(Order {
            dx: 1.0,
            dy: 0.0,
            dz: 0.0,
            feed: 1.0,
        });
        let b = j.mils_of(Order {
            dx: 1.0,
            dy: 0.0,
            dz: 0.0,
            feed: 1.0,
        });
        let c = j.mils_of(Order {
            dx: 1.0,
            dy: 0.0,
            dz: 0.0,
            feed: 1.0,
        });
        // 39.37 mils each: 39, then 40 with the 0.37 that was left over, then 39 with the -0.26 left over; three millimetres are 118.1 mils
        assert_eq!((a.0, b.0, c.0), (39, 40, 39));
        // a head that is parked starts counting again
        j.park();
        assert_eq!(j.carry, (0.0, 0.0, 0.0));
    }

    #[test]
    fn the_travel_of_z_follows_the_board_and_what_is_allowed_below_it() {
        let l = Limits::new(500.0, 300.0, 23.7, 5.0, 0.0);
        assert_eq!(l.z_floor, 23.7);
        assert!((l.safe_z - 18.7).abs() < 1e-9);
        let l = Limits::new(500.0, 300.0, 23.7, 5.0, 1.5);
        assert!((l.z_floor - 25.2).abs() < 1e-9);
        // a clearance bigger than the depth of the board leaves the head at home to move across
        let l = Limits::new(500.0, 300.0, 3.0, 5.0, 0.0);
        assert_eq!(l.safe_z, 0.0);
        // nothing negative or odd gives a negative travel
        let l = Limits::new(-1.0, -1.0, -3.0, -1.0, -1.0);
        assert_eq!(
            (l.x_max, l.y_max, l.z_floor, l.safe_z),
            (0.0, 0.0, 0.0, 0.0)
        );
    }

    #[test]
    fn the_numbers_of_the_file_have_no_zeros_that_mean_nothing() {
        assert_eq!(number(0.01), "0.01");
        assert_eq!(number(0.1), "0.1");
        assert_eq!(number(10.0), "10");
        assert_eq!(number(50.0), "50");
        assert_eq!(number(-2.5), "-2.5");
        assert_eq!(number(0.0), "0");
    }
}
