//! What the launcher offers and what it remembers. The modern interface is one choice; the classic programs (drilling or routing, on a
//! machine with manual or automatic tool change) are the others, and they are started as the programs they are.

use std::path::{Path, PathBuf};

use eh3d_view::texts::Lang;
use eh3d_view::theme::ThemeId;

use crate::spindle::{Spindle, SpindleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Program {
    Drill,
    Route,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Machine {
    /// Manual tool change.
    Mtc,
    /// Automatic tool change.
    Atc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Classic(Program, Machine),
    Modern,
}

impl Choice {
    pub const ALL: [Choice; 5] = [
        Choice::Modern,
        Choice::Classic(Program::Drill, Machine::Mtc),
        Choice::Classic(Program::Drill, Machine::Atc),
        Choice::Classic(Program::Route, Machine::Mtc),
        Choice::Classic(Program::Route, Machine::Atc),
    ];

    /// The word of the settings file.
    pub fn key(self) -> &'static str {
        match self {
            Choice::Modern => "modern",
            Choice::Classic(Program::Drill, Machine::Mtc) => "classic-drill-mtc",
            Choice::Classic(Program::Drill, Machine::Atc) => "classic-drill-atc",
            Choice::Classic(Program::Route, Machine::Mtc) => "classic-route-mtc",
            Choice::Classic(Program::Route, Machine::Atc) => "classic-route-atc",
        }
    }

    pub fn from_key(key: &str) -> Option<Choice> {
        let k = key.trim().to_ascii_lowercase();
        Choice::ALL.into_iter().find(|c| c.key() == k)
    }
}

/// How to start a classic program: the file and its arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub exe: PathBuf,
    pub args: Vec<String>,
}

/// The classic program of a choice, looked for next to the launcher and in a `classic` folder beside it. The machine decides which
/// configuration it is given: `config/mtc` or `config/atc` when the package has them. `None` for the modern interface and when the
/// program is not there.
pub fn classic_launch(choice: Choice, dir: &Path) -> Option<Launch> {
    let Choice::Classic(program, machine) = choice else {
        return None;
    };
    let name = match program {
        Program::Drill => "drillpro",
        Program::Route => "routepro",
    };
    let file = format!("{name}{}", std::env::consts::EXE_SUFFIX);
    let exe = [dir.join(&file), dir.join("classic").join(&file)]
        .into_iter()
        .find(|p| p.is_file())?;
    let sub = match machine {
        Machine::Mtc => "mtc",
        Machine::Atc => "atc",
    };
    let mut args = Vec::new();
    for base in [
        dir.join("config").join(sub),
        dir.join("classic").join("config").join(sub),
    ] {
        if base.is_dir() {
            args.push("--data".to_string());
            args.push(base.to_string_lossy().into_owned());
            break;
        }
    }
    Some(Launch { exe, args })
}

/// What is kept between runs.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The choice to take without asking (the launcher is skipped).
    pub choice: Option<Choice>,
    pub lang: Lang,
    pub theme: ThemeId,
    /// The board of the job: material, thickness, copper and size.
    pub board: crate::boards::Board,
    /// The job stops at the questions of the machine (the tool change, the spindle start).
    pub ask_operator: bool,
    /// The window of the tools opens when the person opens a file, as the classic programs show theirs after loading one.
    pub tools_on_open: bool,
    /// The Open window shows the plan of the file chosen, at its right.
    pub open_preview: bool,
    /// How many times faster than the machine the simulation of a job runs (1 is the speed of the machine).
    pub sim_speed: f32,
    /// The model of the machine (`CCD MTC`, `CCD/2 ATC`...).
    pub model: String,
    /// The emergency button is on the window, and where (in the units of the canvas).
    pub emergency: bool,
    pub estop_pos: (f32, f32),
    /// Lengths are shown in mils (0.001 inch) instead of millimetres.
    pub display_mil: bool,
    /// The spindle the machine carries now, and the limits of each one: the catalogue and the tools warn when a bit asks for more than it can give.
    pub spindle: SpindleId,
    pub spindles: [Spindle; 2],
    /// The name the operator gives to the machine and its make and model, as free text (empty: the model chosen in the list is the name).
    pub machine_name: String,
    /// The measures of the aluminium profile of the X and Y axes (the plan view and the 3D view are drawn from them).
    pub profile: crate::profile::ProfileDims,
    pub machine_make: String,
    /// The spindle limits of the models that are not the one in use. Each model has its own copy (a copy of the CCD MTC until there are official data for it), so changing one
    /// never touches another; the ones of the model in use are in `spindles`.
    pub machine_spindles: std::collections::BTreeMap<String, [Spindle; 2]>,
    /// The thickness (mm) and the copper (um) the person typed for each material whose maker gives none, by `maker|name`, so the next time the material is chosen they come back.
    pub material_values: std::collections::BTreeMap<String, (f64, f64)>,
    /// The places the person gave to the tools in the bar of the tool changer, kept for each machine (by the key of its model) so that the next job with the same tools finds
    /// them where they were put.
    pub slot_books: std::collections::BTreeMap<String, crate::slots::Book>,
    /// The sizes the 3D view draws the original spindle with (from its data sheet unless the settings file says otherwise).
    pub head_dims: crate::spindle::HeadDims,
    /// The updates from GitHub: the master switch (off, the program never asks the network), whether it looks for a new version when it starts, the hours
    /// between two such looks, the address of the API that answers with the latest release, and when it last looked (seconds since 1970).
    pub updates_enabled: bool,
    pub updates_at_start: bool,
    pub updates_interval_h: u32,
    pub updates_api_url: String,
    pub updates_last_check: u64,
    /// Seconds a connection to GitHub may wait before the update gives up.
    pub updates_timeout_s: u64,
    /// How many holes each drill (by its tool number) has drilled in all the jobs, has drilled in all the jobs of this window, kept between runs.
    pub holes_drilled: std::collections::BTreeMap<u8, u64>,
    /// The files opened last, the newest first (at most [`RECENT_MAX`]).
    pub recent: Vec<String>,
    /// The window fills the screen (off: it opens maximized).
    pub fullscreen: bool,
    /// The list of boards shows the boards of all the makers mixed (off: those of the maker of the board in use).
    pub all_makers: bool,
    /// The width of the left pane and of the right pane of the main screen, in the units of its drawing board (the middle one is what is left).
    pub pane_left: f32,
    pub pane_right: f32,
    /// Minutes the clock is from universal time, for the hour the job is expected to end at. `None` (the default) is the time zone of the system;
    /// a value in the machine file (`clock_utc_offset_minutes`) overrides it.
    pub clock_utc_offset_min: Option<i32>,
    /// How near the head of a routing job has to pass to a drawn path to mark it as cut, in mm, and the longest stretch of machine time the simulation goes on
    /// before the head is looked at again, in seconds.
    pub route_mark_tolerance_mm: f64,
    pub route_mark_step_s: f64,
    /// How far into the board (in mm) the tool has to be for the head to count as cutting; above that the head is travelling and marks nothing.
    pub route_mark_min_depth_mm: f64,
    /// The log of the program (`bungard-gui.log`): whether it is written, how detailed, how big, how many files, how often the heartbeat beats.
    pub log: crate::applog::Config,
}

/// Where the latest release of this program is asked for (the API of GitHub); the settings file can point it elsewhere.
pub const DEFAULT_UPDATES_API: &str =
    "https://api.github.com/repos/JuanenRac/EH3D-BUNGARD-GUI/releases/latest";

/// Where the emergency stop starts: at the top right of the 3D view, clear of the bar of numbers and of the buttons of the job under the views.
pub const DEFAULT_ESTOP: (f32, f32) = (1474.0, 204.0);

/// How many recent files are kept.
pub const RECENT_MAX: usize = 6;

impl Default for Settings {
    fn default() -> Self {
        Settings {
            choice: None,
            lang: Lang::En,
            theme: ThemeId::Forge,
            board: crate::boards::Board::standard(),
            ask_operator: true,
            tools_on_open: true,
            open_preview: true,
            sim_speed: 1.0,
            model: "CCD MTC".to_string(),
            emergency: true,
            estop_pos: DEFAULT_ESTOP,
            display_mil: false,
            spindle: SpindleId::Original,
            spindles: [
                Spindle::default_for(SpindleId::Original),
                Spindle::default_for(SpindleId::Vfd),
            ],
            machine_name: String::new(),
            profile: crate::profile::ProfileDims::default(),
            machine_make: String::new(),
            machine_spindles: std::collections::BTreeMap::new(),
            material_values: std::collections::BTreeMap::new(),
            slot_books: std::collections::BTreeMap::new(),
            head_dims: crate::spindle::HeadDims::default(),
            updates_enabled: true,
            updates_at_start: false,
            updates_interval_h: 24,
            updates_api_url: DEFAULT_UPDATES_API.to_string(),
            updates_last_check: 0,
            updates_timeout_s: 30,
            holes_drilled: std::collections::BTreeMap::new(),
            recent: Vec::new(),
            fullscreen: false,
            all_makers: false,
            pane_left: crate::panes::DEFAULT_LEFT,
            pane_right: crate::panes::DEFAULT_RIGHT,
            clock_utc_offset_min: None,
            route_mark_tolerance_mm: 0.35,
            route_mark_step_s: 0.05,
            route_mark_min_depth_mm: 0.02,
            log: crate::applog::Config::default(),
        }
    }
}

/// A value on one line: a line break in it could add keys to the file (the address the updates come from among them).
fn one_line(v: &str) -> String {
    v.replace(['\r', '\n'], " ")
}

/// The key of a model in the settings file: its name in lower case with every other character turned into an underscore (`CCD/2 ATC` is `ccd_2_atc`).
pub fn model_key(model: &str) -> String {
    let mut k = String::new();
    for c in model.trim().chars() {
        k.push(if c.is_ascii_alphanumeric() {
            c.to_ascii_lowercase()
        } else if k.ends_with('_') {
            continue;
        } else {
            '_'
        });
    }
    k.trim_matches('_').to_string()
}

impl Settings {
    /// The model in use changes: the spindle limits of the one that goes are kept apart under its name and the ones of the new model are taken (a model that never had any gets
    /// a copy of the starting values of the CCD MTC, which it keeps on its own from then on).
    pub fn switch_model(&mut self, new_model: &str) {
        if model_key(&self.model) == model_key(new_model) {
            self.model = new_model.to_string();
            return;
        }
        self.machine_spindles
            .insert(model_key(&self.model), self.spindles);
        self.spindles = self
            .machine_spindles
            .remove(&model_key(new_model))
            .unwrap_or([
                Spindle::default_for(SpindleId::Original),
                Spindle::default_for(SpindleId::Vfd),
            ]);
        self.model = new_model.to_string();
    }

    /// Whether the spindle limits of the model in use are the provisional copy of the CCD MTC (every model but the CCD MTC itself, until official data come).
    pub fn spindle_is_provisional(&self) -> bool {
        model_key(&self.model) != model_key("CCD MTC")
    }

    /// The spindle the machine carries now.
    pub fn current_spindle(&self) -> Spindle {
        self.spindles[self.spindle as usize]
    }

    /// The text of the file: `key = value` lines; what is not known or not valid keeps its default.
    pub fn parse(text: &str) -> Settings {
        let mut s = Settings::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match k.trim().to_ascii_lowercase().as_str() {
                "choice" => s.choice = Choice::from_key(v),
                "lang" => s.lang = Lang::from_code(v).unwrap_or(s.lang),
                "theme" => s.theme = ThemeId::from_key(v).unwrap_or(s.theme),
                "sim_speed" => {
                    s.sim_speed = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|x: &f32| (0.1..=1000.0).contains(x))
                        .unwrap_or(s.sim_speed)
                }
                "model" => {
                    if crate::machines::Model::from_name(v).is_some() {
                        s.model = v.trim().to_string();
                    }
                }
                "emergency" => s.emergency = !matches!(v.trim(), "no" | "off" | "false" | "0"),
                "estop_pos" => {
                    if let Some((x, y)) = v.split_once(',') {
                        if let (Ok(x), Ok(y)) = (x.trim().parse::<f32>(), y.trim().parse::<f32>()) {
                            s.estop_pos = (
                                x.clamp(0.0, crate::panes::BOARD_W_MAX),
                                y.clamp(0.0, crate::panes::BOARD_H),
                            );
                        }
                    }
                }
                "display_units" => s.display_mil = v.trim().eq_ignore_ascii_case("mil"),
                "recent" => {
                    s.recent = v
                        .split('|')
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                        .take(RECENT_MAX)
                        .map(str::to_string)
                        .collect()
                }
                "holes_drilled" => {
                    s.holes_drilled = v
                        .split(',')
                        .filter_map(|p| {
                            let (t, n) = p.split_once(':')?;
                            Some((t.trim().parse::<u8>().ok()?, n.trim().parse::<u64>().ok()?))
                        })
                        .collect()
                }
                "ask_operator" => {
                    s.ask_operator = !matches!(v.trim(), "no" | "off" | "false" | "0")
                }
                "tools_on_open" => {
                    s.tools_on_open = !matches!(v.trim(), "no" | "off" | "false" | "0")
                }
                "open_preview" => {
                    s.open_preview = !matches!(v.trim(), "no" | "off" | "false" | "0")
                }
                "fullscreen" => s.fullscreen = matches!(v.trim(), "yes" | "on" | "true" | "1"),
                "clock_utc_offset_minutes" => {
                    s.clock_utc_offset_min = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|m: &i32| (-14 * 60..=14 * 60).contains(m))
                        .or(s.clock_utc_offset_min)
                }
                "route_mark_tolerance_mm" => {
                    s.route_mark_tolerance_mm = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|x: &f64| (0.01..=5.0).contains(x))
                        .unwrap_or(s.route_mark_tolerance_mm)
                }
                "route_mark_min_depth_mm" => {
                    s.route_mark_min_depth_mm = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|x: &f64| (0.0..=5.0).contains(x))
                        .unwrap_or(s.route_mark_min_depth_mm)
                }
                "route_mark_step_s" => {
                    s.route_mark_step_s = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|x: &f64| (0.005..=1.0).contains(x))
                        .unwrap_or(s.route_mark_step_s)
                }
                "all_makers" => s.all_makers = matches!(v.trim(), "yes" | "on" | "true" | "1"),
                "panes" => {
                    if let Some((l, r)) = crate::panes::parse(v) {
                        let (l, _, r) = crate::panes::split(l, r);
                        s.pane_left = l;
                        s.pane_right = r;
                    }
                }
                "updates" => s.updates_enabled = !matches!(v.trim(), "no" | "off" | "false" | "0"),
                "updates_at_start" => {
                    s.updates_at_start = matches!(v.trim(), "yes" | "on" | "true" | "1")
                }
                "updates_interval_hours" => {
                    s.updates_interval_h = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|h: &u32| (1..=24 * 90).contains(h))
                        .unwrap_or(s.updates_interval_h)
                }
                "updates_api_url" => {
                    let u = v.trim();
                    if !u.is_empty() {
                        s.updates_api_url = u.to_string();
                    }
                }
                "updates_timeout_seconds" => {
                    s.updates_timeout_s = v
                        .trim()
                        .parse()
                        .ok()
                        .filter(|n: &u64| (2..=300).contains(n))
                        .unwrap_or(s.updates_timeout_s)
                }
                "updates_last_check" => s.updates_last_check = v.trim().parse().unwrap_or(0),
                "spindle_drawing" => s.head_dims = crate::spindle::HeadDims::from_text(v),
                "spindle" => s.spindle = SpindleId::from_code(v).unwrap_or(s.spindle),
                "profile_aluminium" => s.profile = crate::profile::ProfileDims::from_text(v),
                "machine_name" => s.machine_name = one_line(v.trim()),
                "machine_make" => s.machine_make = one_line(v.trim()),
                k if k.starts_with("bar_slots_") => {
                    // `bar_slots_<model>`: the places of the tools in the bar of that machine (see `slots::Book`)
                    let model = &k["bar_slots_".len()..];
                    let book = crate::slots::Book::parse(v);
                    if !model.is_empty() && !book.is_empty() {
                        s.slot_books.insert(model.to_string(), book);
                    }
                }
                k if k.starts_with("machine_spindle_") => {
                    // `machine_spindle_<model>_<original|vfd>`: the limits of a model that is not in use
                    if let Some((model, code)) = k["machine_spindle_".len()..].rsplit_once('_') {
                        if let Some(id) = SpindleId::from_code(code) {
                            let e = s.machine_spindles.entry(model.to_string()).or_insert([
                                Spindle::default_for(SpindleId::Original),
                                Spindle::default_for(SpindleId::Vfd),
                            ]);
                            e[id as usize] = Spindle::from_text(id, v);
                        }
                    }
                }
                k if k.starts_with("spindle_") => {
                    if let Some(id) = SpindleId::from_code(&k["spindle_".len()..]) {
                        s.spindles[id as usize] = Spindle::from_text(id, v);
                    }
                }
                "board" => s.board = crate::boards::from_text(v).unwrap_or(s.board),
                "material_values" => {
                    // `maker|name|thickness|copper`, one line for each material
                    let p: Vec<&str> = v.trim().split('|').collect();
                    if let [maker, name, t, c] = p[..] {
                        if let (Ok(t), Ok(c)) = (t.parse::<f64>(), c.parse::<f64>()) {
                            if t.is_finite() && c.is_finite() && !maker.is_empty() {
                                s.remember_material_values(maker, name, t, c);
                            }
                        }
                    }
                }
                other => {
                    // the keys of the log; any other key is not known and is left
                    s.log.set(other, v);
                }
            }
        }
        // the model in use keeps its limits in `spindles`
        let own = model_key(&s.model);
        s.machine_spindles.remove(&own);
        s
    }

    pub fn to_text(&self) -> String {
        let mut out = self.main_text();
        // the name of the machine, and the limits of the spindles of each model that is not in use, one line each
        out.push_str(&format!(
            "machine_name = {}\nmachine_make = {}\nprofile_aluminium = {}\n",
            one_line(&self.machine_name),
            one_line(&self.machine_make),
            self.profile.to_text()
        ));
        // the clock offset is written only when one was chosen; otherwise the system clock is used
        let clock = self
            .clock_utc_offset_min
            .map(|m| format!("clock_utc_offset_minutes = {m}\n"))
            .unwrap_or_default();
        out.push_str(&format!(
            "{clock}route_mark_tolerance_mm = {}\nroute_mark_step_s = {}\nroute_mark_min_depth_mm = {}\n",
            self.route_mark_tolerance_mm,
            self.route_mark_step_s,
            self.route_mark_min_depth_mm
        ));
        out.push_str(&self.log.to_text());
        for (key, (t, c)) in &self.material_values {
            out.push_str(&format!("material_values = {}|{t}|{c}\n", one_line(key)));
        }
        for (model, book) in &self.slot_books {
            if !model.is_empty() && !book.is_empty() {
                out.push_str(&format!("bar_slots_{model} = {}\n", book.to_text()));
            }
        }
        for (model, pair) in &self.machine_spindles {
            if *model == model_key(&self.model) || model.is_empty() {
                continue;
            }
            for id in SpindleId::ALL {
                out.push_str(&format!(
                    "machine_spindle_{model}_{} = {}\n",
                    id.code(),
                    pair[id as usize].to_text()
                ));
            }
        }
        out
    }

    fn main_text(&self) -> String {
        format!(
            "choice = {}\nlang = {}\ntheme = {}\nboard = {}\nask_operator = {}\ntools_on_open = {}\nopen_preview = {}\nsim_speed = {}\nmodel = {}\nemergency = {}\nestop_pos = {},{}\ndisplay_units = {}\nspindle = {}\nspindle_original = {}\nspindle_vfd = {}\nspindle_drawing = {}\nupdates = {}\nupdates_at_start = {}\nupdates_interval_hours = {}\nupdates_api_url = {}\nupdates_last_check = {}\nupdates_timeout_seconds = {}\nholes_drilled = {}\nrecent = {}\nfullscreen = {}\nall_makers = {}\npanes = {},{}\n",
            self.choice.map_or("ask", |c| c.key()),
            self.lang.code(),
            self.theme.key(),
            crate::boards::to_text(&self.board),
            if self.ask_operator { "yes" } else { "no" },
            if self.tools_on_open { "yes" } else { "no" },
            if self.open_preview { "yes" } else { "no" },
            self.sim_speed,
            self.model,
            if self.emergency { "yes" } else { "no" },
            self.estop_pos.0,
            self.estop_pos.1,
            if self.display_mil { "mil" } else { "mm" },
            self.spindle.code(),
            self.spindles[0].to_text(),
            self.spindles[1].to_text(),
            self.head_dims.to_text(),
            if self.updates_enabled { "yes" } else { "no" },
            if self.updates_at_start { "yes" } else { "no" },
            self.updates_interval_h,
            one_line(&self.updates_api_url),
            self.updates_last_check,
            self.updates_timeout_s,
            self.holes_drilled
                .iter()
                .map(|(t, n)| format!("{t}:{n}"))
                .collect::<Vec<_>>()
                .join(","),
            self.recent.iter().map(|p| one_line(p)).collect::<Vec<_>>().join("|"),
            if self.fullscreen { "yes" } else { "no" },
            if self.all_makers { "yes" } else { "no" },
            self.pane_left,
            self.pane_right
        )
    }

    /// Puts a file at the head of the recent ones (it is not listed twice).
    pub fn remember_recent(&mut self, path: &str) {
        self.recent.retain(|p| p != path);
        self.recent.insert(0, path.to_string());
        self.recent.truncate(RECENT_MAX);
    }

    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path)
            .map(|t| Settings::parse(&t))
            .unwrap_or_default()
    }

    /// Keeps the thickness and the copper the person typed for a material whose maker gives none.
    pub fn remember_material_values(
        &mut self,
        maker: &str,
        name: &str,
        thickness_mm: f64,
        copper_um: f64,
    ) {
        self.material_values
            .insert(format!("{maker}|{name}"), (thickness_mm, copper_um));
    }

    /// What the person typed the last time for this material, if anything.
    pub fn material_values_of(&self, maker: &str, name: &str) -> Option<(f64, f64)> {
        self.material_values
            .get(&format!("{maker}|{name}"))
            .copied()
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // written whole to a side file and renamed over the other, so a crash or a full disk in the middle never leaves half a file
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

/// Where the settings file is: the settings folder of the user, with the other EH3D programs.
pub fn settings_path() -> PathBuf {
    // the tests never touch the settings of the user
    if cfg!(test) {
        return std::env::temp_dir()
            .join(format!("eh3d-bungard-gui-test-{}.cfg", std::process::id()));
    }
    if cfg!(windows) {
        let base = std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("EH3D").join("bungard-gui.cfg")
    } else {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".config").join("eh3d").join("bungard-gui.cfg")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("eh3d-launcher-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn every_choice_has_a_word_and_comes_back() {
        for c in Choice::ALL {
            assert_eq!(Choice::from_key(c.key()), Some(c));
        }
        assert_eq!(Choice::from_key(" Modern "), Some(Choice::Modern));
        assert_eq!(Choice::from_key("ask"), None);
        let mut keys: Vec<_> = Choice::ALL.iter().map(|c| c.key()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), 5);
    }

    #[test]
    fn the_classic_program_is_found_beside_the_launcher_or_in_classic() {
        let d = scratch("find");
        let exe = format!("drillpro{}", std::env::consts::EXE_SUFFIX);
        assert!(classic_launch(Choice::Classic(Program::Drill, Machine::Mtc), &d).is_none());
        std::fs::create_dir_all(d.join("classic")).unwrap();
        std::fs::write(d.join("classic").join(&exe), b"x").unwrap();
        let l = classic_launch(Choice::Classic(Program::Drill, Machine::Mtc), &d).unwrap();
        assert_eq!(l.exe, d.join("classic").join(&exe));
        assert!(l.args.is_empty());
        assert!(classic_launch(Choice::Classic(Program::Route, Machine::Atc), &d).is_none());
        assert!(classic_launch(Choice::Modern, &d).is_none());
    }

    #[test]
    fn the_machine_picks_its_configuration() {
        let d = scratch("config");
        let exe = format!("routepro{}", std::env::consts::EXE_SUFFIX);
        std::fs::write(d.join(&exe), b"x").unwrap();
        std::fs::create_dir_all(d.join("config").join("atc")).unwrap();
        let atc = classic_launch(Choice::Classic(Program::Route, Machine::Atc), &d).unwrap();
        assert_eq!(
            atc.args,
            vec![
                "--data".to_string(),
                d.join("config").join("atc").to_string_lossy().into_owned()
            ]
        );
        // there is none for the manual one: the program uses its own
        let mtc = classic_launch(Choice::Classic(Program::Route, Machine::Mtc), &d).unwrap();
        assert!(mtc.args.is_empty());
    }

    #[test]
    fn what_is_remembered_comes_back() {
        let d = scratch("settings");
        let path = d.join("sub").join("bungard-gui.cfg");
        assert_eq!(Settings::load(&path), Settings::default());
        let s = Settings {
            choice: Some(Choice::Classic(Program::Drill, Machine::Atc)),
            lang: Lang::Ja,
            theme: ThemeId::WindowsClassic,
            board: crate::boards::Board::standard().with_material(3),
            ask_operator: false,
            tools_on_open: false,
            open_preview: false,
            sim_speed: 5.0,
            model: "CCD/2 ATC".to_string(),
            emergency: false,
            estop_pos: (300.0, 400.5),
            display_mil: true,
            spindle: SpindleId::Vfd,
            spindles: [
                Spindle::default_for(SpindleId::Original),
                Spindle {
                    max_xy_mm_min: 900,
                    ..Spindle::default_for(SpindleId::Vfd)
                },
            ],
            profile: crate::profile::ProfileDims {
                width_mm: 90.0,
                ..crate::profile::ProfileDims::default()
            },
            machine_name: "Workshop 1".to_string(),
            machine_make: "Bungard CCD/2 ATC".to_string(),
            machine_spindles: [(
                "ccd_mtc".to_string(),
                [
                    Spindle {
                        max_rpm: 40_000,
                        ..Spindle::default_for(SpindleId::Original)
                    },
                    Spindle::default_for(SpindleId::Vfd),
                ],
            )]
            .into_iter()
            .collect(),
            material_values: [("Acme|Plain laminate".to_string(), (1.6, 35.0))]
                .into_iter()
                .collect(),
            slot_books: [(
                "ccd_2_atc".to_string(),
                crate::slots::Book::parse("v1;d800#1=5,r0#7=14"),
            )]
            .into_iter()
            .collect(),
            holes_drilled: [(1u8, 9172u64), (3, 40)].into_iter().collect(),
            recent: vec!["C:/jobs/a.ncd".to_string(), "C:/jobs/b.zip".to_string()],
            fullscreen: true,
            all_makers: true,
            pane_left: 300.0,
            pane_right: 500.0,
            head_dims: crate::spindle::HeadDims {
                body_mm: 90.0,
                ..crate::spindle::HeadDims::default()
            },
            updates_enabled: false,
            updates_at_start: true,
            updates_interval_h: 12,
            updates_api_url: "http://127.0.0.1:9/latest".to_string(),
            updates_last_check: 99,
            updates_timeout_s: 8,
            clock_utc_offset_min: Some(120),
            route_mark_tolerance_mm: 0.5,
            route_mark_step_s: 0.1,
            route_mark_min_depth_mm: 0.05,
            log: crate::applog::Config {
                enabled: false,
                level: crate::applog::Level::Debug,
                max_kb: 256,
                files: 5,
                heartbeat_s: 10,
            },
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path), s);
        let ask = Settings::default();
        assert!(ask.to_text().contains("choice = ask"));
        assert_eq!(Settings::parse(&ask.to_text()), ask);
    }

    #[test]
    fn every_model_keeps_its_own_copy_of_the_spindle_of_the_mtc_and_changing_one_leaves_the_others()
    {
        let start = Spindle::default_for(SpindleId::Original);
        let mut s = Settings::default();
        assert!(!s.spindle_is_provisional());
        s.spindles[0].max_rpm = 50_000;
        s.switch_model("CCD/2 ATC");
        assert!(s.spindle_is_provisional());
        // a model that never had its own limits starts from the values of the MTC, not from what the MTC was tuned to
        assert_eq!(s.spindles[0], start);
        s.spindles[0].max_rpm = 30_000;
        s.switch_model("CCD XL MTC");
        assert_eq!(s.spindles[0], start);
        s.switch_model("CCD MTC");
        assert_eq!(s.spindles[0].max_rpm, 50_000);
        s.switch_model("CCD/2 ATC");
        assert_eq!(s.spindles[0].max_rpm, 30_000);
        // the same name again changes nothing
        s.switch_model("CCD/2 ATC");
        assert_eq!(s.spindles[0].max_rpm, 30_000);
        // each model has its own lines in the file and they come back
        let text = s.to_text();
        assert!(
            text.contains("machine_spindle_ccd_mtc_original = "),
            "{text}"
        );
        assert!(text.contains("machine_spindle_ccd_xl_mtc_vfd = "), "{text}");
        assert!(
            !text.contains("machine_spindle_ccd_2_atc"),
            "the model in use is in spindle_original"
        );
        let back = Settings::parse(&text);
        assert_eq!(back, s);
        let mut back = back;
        back.switch_model("CCD MTC");
        assert_eq!(back.spindles[0].max_rpm, 50_000);
    }

    #[test]
    fn the_name_and_the_model_of_the_machine_are_kept_on_one_line_each() {
        let s = Settings {
            machine_name: "Workshop\nmodel = CCD XL ATC".to_string(),
            machine_make: "Bungard\r\nspindle = vfd".to_string(),
            ..Settings::default()
        };
        let back = Settings::parse(&s.to_text());
        assert_eq!(back.model, Settings::default().model);
        assert_eq!(back.spindle, Settings::default().spindle);
        assert_eq!(back.machine_name, "Workshop model = CCD XL ATC");
        assert_eq!(back.machine_make, "Bungard  spindle = vfd");
        assert_eq!(model_key("CCD/2 ATC"), "ccd_2_atc");
        assert_eq!(model_key(" CCD  Premium MTC "), "ccd_premium_mtc");
    }

    #[test]
    fn the_updates_and_the_drawing_of_the_spindle_are_kept_and_start_careful() {
        let d = Settings::default();
        assert!(d.updates_enabled && !d.updates_at_start);
        assert_eq!(d.updates_api_url, DEFAULT_UPDATES_API);
        assert_eq!(d.head_dims, crate::spindle::HeadDims::default());
        let s = Settings {
            updates_enabled: false,
            updates_at_start: true,
            updates_interval_h: 48,
            updates_api_url: "http://127.0.0.1:1/latest".to_string(),
            updates_last_check: 1234,
            head_dims: crate::spindle::HeadDims {
                total_mm: 180.0,
                ..crate::spindle::HeadDims::default()
            },
            ..Settings::default()
        };
        let back = Settings::parse(&s.to_text());
        assert!(!back.updates_enabled);
        assert!(back.updates_at_start);
        assert_eq!(back.updates_interval_h, 48);
        assert_eq!(back.updates_api_url, "http://127.0.0.1:1/latest");
        assert_eq!(back.updates_last_check, 1234);
        assert_eq!(back.updates_timeout_s, 30);
        assert_eq!(back.head_dims.total_mm, 180.0);
        // an interval out of range keeps the default
        assert_eq!(
            Settings::parse("updates_interval_hours = 0").updates_interval_h,
            24
        );
    }

    #[test]
    fn the_clock_offset_and_the_marking_of_the_routing_are_settings_with_limits() {
        let d = Settings::default();
        assert_eq!(d.clock_utc_offset_min, None);
        let s = Settings::parse(
            "clock_utc_offset_minutes = 60
route_mark_tolerance_mm = 0.8
route_mark_step_s = 0.2
",
        );
        assert_eq!(s.clock_utc_offset_min, Some(60));
        assert_eq!(s.route_mark_tolerance_mm, 0.8);
        assert_eq!(s.route_mark_step_s, 0.2);
        let back = Settings::parse(&s.to_text());
        assert_eq!(back.clock_utc_offset_min, Some(60));
        // the system's own zone is not written to the file
        assert!(!Settings::default()
            .to_text()
            .contains("clock_utc_offset_minutes"));
        assert_eq!(back.route_mark_step_s, 0.2);
        // values out of range keep the default
        let wild = Settings::parse(
            "clock_utc_offset_minutes = 5000
route_mark_tolerance_mm = 0
route_mark_step_s = 99
",
        );
        assert_eq!(wild.clock_utc_offset_min, None);
        assert_eq!(wild.route_mark_tolerance_mm, d.route_mark_tolerance_mm);
        assert_eq!(wild.route_mark_step_s, d.route_mark_step_s);
    }

    #[test]
    fn the_keys_of_the_log_are_read_with_limits_and_written_with_the_others() {
        let s = Settings::parse(
            "log = no
log_level = debug
log_max_kb = 512
log_files = 2
log_heartbeat_s = 15
",
        );
        assert!(!s.log.enabled);
        assert_eq!(s.log.level, crate::applog::Level::Debug);
        assert_eq!((s.log.max_kb, s.log.files, s.log.heartbeat_s), (512, 2, 15));
        assert_eq!(Settings::parse(&s.to_text()).log, s.log);
        let d = Settings::default();
        assert!(d.log.enabled);
        assert!(d.to_text().contains("log_heartbeat_s = 30"));
        // out of range or not a level: the defaults stay
        let wild = Settings::parse(
            "log_max_kb = 1
log_level = loud
",
        );
        assert_eq!(wild.log, d.log);
    }

    #[test]
    fn the_window_and_the_pane_widths_are_remembered_and_have_defaults() {
        let d = Settings::default();
        assert!(!d.fullscreen && !d.all_makers);
        assert_eq!((d.pane_left, d.pane_right), (280.0, 608.0));
        // an older file without these lines gives the defaults
        let old = Settings::parse("lang = es\n");
        assert_eq!(old.pane_left, 280.0);
        assert!(!old.fullscreen);
        let s = Settings::parse("fullscreen = yes\nall_makers = yes\npanes = 300, 500\n");
        assert!(s.fullscreen && s.all_makers);
        assert_eq!((s.pane_left, s.pane_right), (300.0, 500.0));
        assert_eq!(Settings::parse(&s.to_text()), s);
        // widths outside the limits are brought back inside them
        let wild = Settings::parse("panes = 5, 5000\n");
        let (l, m, r) = crate::panes::split(wild.pane_left, wild.pane_right);
        assert_eq!((l, r), (wild.pane_left, wild.pane_right));
        assert!(m >= crate::panes::MIN[1] - 0.01);
        // a line that is not numbers keeps the defaults
        assert_eq!(Settings::parse("panes = x\n").pane_left, 280.0);
    }

    #[test]
    fn only_the_last_files_are_remembered_and_none_twice() {
        let mut s = Settings::default();
        for i in 0..10 {
            s.remember_recent(&format!("C:/jobs/{i}.ncd"));
        }
        assert_eq!(s.recent.len(), RECENT_MAX);
        assert_eq!(s.recent[0], "C:/jobs/9.ncd");
        s.remember_recent("C:/jobs/7.ncd");
        assert_eq!(s.recent[0], "C:/jobs/7.ncd");
        assert_eq!(s.recent.iter().filter(|p| p.ends_with("/7.ncd")).count(), 1);
        assert_eq!(Settings::parse(&s.to_text()).recent, s.recent);
    }

    #[test]
    fn saving_the_settings_leaves_no_side_file_and_reports_a_failure() {
        let d = scratch("save-side");
        let path = d.join("bungard-gui.cfg");
        let mut s = Settings::default();
        s.save(&path).unwrap();
        s.remember_recent("C:/jobs/a.ncd");
        s.save(&path).unwrap();
        assert!(!path.with_extension("tmp").exists());
        assert_eq!(Settings::load(&path).recent, s.recent);
        // a path whose folder is a file cannot be written: the error comes back
        let blocked = path.join("inside.cfg");
        assert!(s.save(&blocked).is_err());
    }

    #[test]
    fn the_values_typed_for_a_material_come_back_by_its_name() {
        let mut s = Settings::default();
        assert_eq!(s.material_values_of("Acme", "Plain laminate"), None);
        s.remember_material_values("Acme", "Plain laminate", 1.2, 18.0);
        s.remember_material_values("Acme", "Plain laminate", 1.6, 35.0);
        s.remember_material_values("Acme", "Other", 0.8, 70.0);
        assert_eq!(
            s.material_values_of("Acme", "Plain laminate"),
            Some((1.6, 35.0))
        );
        assert_eq!(s.material_values.len(), 2);
        let back = Settings::parse(&s.to_text());
        assert_eq!(back.material_values, s.material_values);
        // a line that is not four parts or not numbers is left out
        let bad = Settings::parse("material_values = a|b|x|y\nmaterial_values = a|b|1\n");
        assert!(bad.material_values.is_empty());
    }

    #[test]
    fn a_wrong_setting_keeps_the_default() {
        let s =
            Settings::parse("choice = fast\nlang = xx\ntheme = pink\nsomething else\nlang = pt\n");
        assert_eq!(s.choice, None);
        assert_eq!(s.theme, ThemeId::Forge);
        assert_eq!(s.lang, Lang::Pt);
    }
}
