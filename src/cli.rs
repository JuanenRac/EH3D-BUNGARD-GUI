//! The command line of the modern interface.

use std::path::PathBuf;

use eh3d_view::texts::Lang;
use eh3d_view::theme::ThemeId;

/// Which screen the window opens on (`--screen`); without it the launcher, or the modern interface when a choice is remembered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Launcher,
    Viewer,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Options {
    pub lang: Option<Lang>,
    pub theme: Option<ThemeId>,
    /// The drilling file (`--drill FILE`) and the routing file (`--route FILE`) to show.
    pub drill: Option<PathBuf>,
    pub route: Option<PathBuf>,
    /// The folder of the machine and tool files (`--data DIR`).
    pub data: Option<PathBuf>,
    /// The machine file with the connection (`--connection FILE`), the same one the classic programs read.
    pub connection: Option<PathBuf>,
    pub screen: Option<Screen>,
    /// A picture of the window to save and leave (`--shot FILE`, after `--wait` frames).
    pub shot: Option<PathBuf>,
    pub wait: u32,
    /// Starts the job as soon as the window is open (`--start`): for the pictures of a job in progress.
    pub start: bool,
    /// Answers Ok to every question of the machine to the operator (the spindle, the tool change): for the pictures and the tests of the simulator.
    pub auto_ok: bool,
}

impl Options {
    pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
        let mut o = Options {
            wait: 6,
            ..Default::default()
        };
        let mut it = args.into_iter();
        while let Some(a) = it.next() {
            let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
            match a.as_str() {
                "--lang" => {
                    let v = value("--lang")?;
                    o.lang = Some(Lang::from_code(&v).ok_or(format!("unknown language '{v}'"))?);
                }
                "--theme" => {
                    let v = value("--theme")?;
                    o.theme = Some(ThemeId::from_key(&v).ok_or(format!("unknown theme '{v}'"))?);
                }
                "--drill" => o.drill = Some(PathBuf::from(value("--drill")?)),
                "--route" => o.route = Some(PathBuf::from(value("--route")?)),
                "--data" => o.data = Some(PathBuf::from(value("--data")?)),
                "--connection" => o.connection = Some(PathBuf::from(value("--connection")?)),
                "--screen" => {
                    let v = value("--screen")?;
                    o.screen = Some(match v.as_str() {
                        "launcher" => Screen::Launcher,
                        "viewer" => Screen::Viewer,
                        _ => return Err(format!("--screen must be launcher or viewer, not '{v}'")),
                    });
                }
                "--shot" => o.shot = Some(PathBuf::from(value("--shot")?)),
                "--wait" => {
                    let v = value("--wait")?;
                    o.wait = v
                        .parse()
                        .map_err(|_| format!("--wait needs a number, not '{v}'"))?;
                }
                "--start" => o.start = true,
                "--auto-ok" => o.auto_ok = true,
                other => return Err(format!("unknown option '{other}'")),
            }
        }
        Ok(o)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(s: &str) -> Result<Options, String> {
        Options::parse(s.split_whitespace().map(String::from))
    }

    #[test]
    fn the_options_are_read() {
        let o = parse("--lang es --theme windows-classic --drill a.ncd --route b.plt --data cfg --connection m.cfg --screen viewer --shot s.png --wait 9 --start --auto-ok").unwrap();
        assert_eq!(o.lang, Some(Lang::Es));
        assert_eq!(o.theme, Some(ThemeId::WindowsClassic));
        assert_eq!(o.drill, Some(PathBuf::from("a.ncd")));
        assert_eq!(o.route, Some(PathBuf::from("b.plt")));
        assert_eq!(o.data, Some(PathBuf::from("cfg")));
        assert_eq!(o.connection, Some(PathBuf::from("m.cfg")));
        assert_eq!(o.screen, Some(Screen::Viewer));
        assert_eq!(
            (o.shot, o.wait, o.start, o.auto_ok),
            (Some(PathBuf::from("s.png")), 9, true, true)
        );
        assert_eq!(
            parse("").unwrap(),
            Options {
                wait: 6,
                ..Default::default()
            }
        );
    }

    #[test]
    fn a_wrong_option_is_said() {
        assert!(parse("--lang xx").unwrap_err().contains("unknown language"));
        assert!(parse("--theme pink").unwrap_err().contains("unknown theme"));
        assert!(parse("--screen map")
            .unwrap_err()
            .contains("launcher or viewer"));
        assert!(parse("--wait many").unwrap_err().contains("number"));
        assert!(parse("--drill").unwrap_err().contains("needs a value"));
        assert!(parse("--fast").unwrap_err().contains("unknown option"));
    }
}
