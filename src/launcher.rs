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
}

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
            estop_pos: (1020.0, 770.0),
            display_mil: false,
            spindle: SpindleId::Original,
            spindles: [
                Spindle::default_for(SpindleId::Original),
                Spindle::default_for(SpindleId::Vfd),
            ],
            holes_drilled: std::collections::BTreeMap::new(),
            recent: Vec::new(),
            fullscreen: false,
            all_makers: false,
            pane_left: crate::panes::DEFAULT_LEFT,
            pane_right: crate::panes::DEFAULT_RIGHT,
        }
    }
}

impl Settings {
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
                            s.estop_pos = (x.clamp(0.0, 1600.0), y.clamp(0.0, 900.0));
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
                "all_makers" => s.all_makers = matches!(v.trim(), "yes" | "on" | "true" | "1"),
                "panes" => {
                    if let Some((l, r)) = crate::panes::parse(v) {
                        let (l, _, r) = crate::panes::split(l, r);
                        s.pane_left = l;
                        s.pane_right = r;
                    }
                }
                "spindle" => s.spindle = SpindleId::from_code(v).unwrap_or(s.spindle),
                k if k.starts_with("spindle_") => {
                    if let Some(id) = SpindleId::from_code(&k["spindle_".len()..]) {
                        s.spindles[id as usize] = Spindle::from_text(id, v);
                    }
                }
                "board" => s.board = crate::boards::from_text(v).unwrap_or(s.board),
                _ => {}
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        format!(
            "choice = {}\nlang = {}\ntheme = {}\nboard = {}\nask_operator = {}\ntools_on_open = {}\nopen_preview = {}\nsim_speed = {}\nmodel = {}\nemergency = {}\nestop_pos = {},{}\ndisplay_units = {}\nspindle = {}\nspindle_original = {}\nspindle_vfd = {}\nholes_drilled = {}\nrecent = {}\nfullscreen = {}\nall_makers = {}\npanes = {},{}\n",
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
            self.holes_drilled
                .iter()
                .map(|(t, n)| format!("{t}:{n}"))
                .collect::<Vec<_>>()
                .join(","),
            self.recent.join("|"),
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

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(path, self.to_text())
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
            holes_drilled: [(1u8, 9172u64), (3, 40)].into_iter().collect(),
            recent: vec!["C:/jobs/a.ncd".to_string(), "C:/jobs/b.zip".to_string()],
            fullscreen: true,
            all_makers: true,
            pane_left: 300.0,
            pane_right: 500.0,
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path), s);
        let ask = Settings::default();
        assert!(ask.to_text().contains("choice = ask"));
        assert_eq!(Settings::parse(&ask.to_text()), ask);
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
    fn a_wrong_setting_keeps_the_default() {
        let s =
            Settings::parse("choice = fast\nlang = xx\ntheme = pink\nsomething else\nlang = pt\n");
        assert_eq!(s.choice, None);
        assert_eq!(s.theme, ThemeId::Forge);
        assert_eq!(s.lang, Lang::Pt);
    }
}
