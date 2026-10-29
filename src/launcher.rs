//! What the launcher offers and what it remembers. The modern interface is one choice; the classic programs (drilling or routing, on a
//! machine with manual or automatic tool change) are the others, and they are started as the programs they are.

use std::path::{Path, PathBuf};

use eh3d_motioncnc::texts::Lang;
use eh3d_motioncnc::theme::ThemeId;

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// The choice to take without asking (the launcher is skipped).
    pub choice: Option<Choice>,
    pub lang: Lang,
    pub theme: ThemeId,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            choice: None,
            lang: Lang::En,
            theme: ThemeId::Forge,
        }
    }
}

impl Settings {
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
                _ => {}
            }
        }
        s
    }

    pub fn to_text(&self) -> String {
        format!(
            "choice = {}\nlang = {}\ntheme = {}\n",
            self.choice.map_or("ask", |c| c.key()),
            self.lang.code(),
            self.theme.key()
        )
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
        };
        s.save(&path).unwrap();
        assert_eq!(Settings::load(&path), s);
        let ask = Settings::default();
        assert!(ask.to_text().contains("choice = ask"));
        assert_eq!(Settings::parse(&ask.to_text()), ask);
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
