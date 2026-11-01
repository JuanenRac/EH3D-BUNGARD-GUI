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
    /// A job packed in a ZIP archive (`--zip FILE`): its first drilling file and its first routing file are shown.
    pub zip: Option<PathBuf>,
    /// Files given without an option: each one is opened as what its extension says (a drilling file, a routing file or a ZIP archive).
    pub open: Vec<PathBuf>,
    /// A window to open at the start (`--modal about|open|panel|settings`), the panel (`--panel COLUMNS,ROWS`) and the turn of the 3D view (`--yaw DEGREES`): for the pictures.
    pub modal: Option<String>,
    /// A drop-down list to open at the start (`--combo theme|lang|model|finish|tool`): for the pictures.
    pub combo: Option<String>,
    /// A fleet written as the server answers it, shown in the window of the machines without a server (`--fleet FILE.json`): for the pictures and to look at the window offline.
    pub fleet_file: Option<PathBuf>,
    pub panel: Option<(u32, u32)>,
    /// Tabs on each side of the panel when the outside is routed (`--tabs COUNT`): for the pictures.
    pub tabs: Option<u32>,
    /// The table is measured at the start with the simulated probe (`--measure`): for the pictures.
    pub measure: bool,
    pub yaw: Option<f32>,
    /// The tab of the settings window to show (`--tab 0|1|2`).
    pub tab: Option<u8>,
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
    /// Runs the quality check through the camera as soon as the window is open (`--check`): for the pictures of the result.
    pub check: bool,
    /// The hole of the job in the centre of the picture of the quality check (`--hole N`, from 1): for the pictures.
    pub hole: Option<usize>,
    /// Answers Ok to every question of the machine to the operator (the spindle, the tool change): for the pictures and the tests of the simulator.
    pub auto_ok: bool,
    /// Turns on the choosing tool of the plan view and chooses what is in the left part of the board (`--select`): for the pictures.
    pub select: bool,
    /// The widths of the left and the right pane of the main screen (`--panes LEFT,RIGHT`, in the units of its drawing board): for the pictures.
    pub panes: Option<(f32, f32)>,
    /// The model of the machine to show (`--machine "CCD/2 ATC"`), without remembering it: for the pictures.
    pub machine: Option<String>,
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
                "--zip" => o.zip = Some(PathBuf::from(value("--zip")?)),
                "--data" => o.data = Some(PathBuf::from(value("--data")?)),
                "--machine" => o.machine = Some(value("--machine")?),
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
                "--modal" => o.modal = Some(value("--modal")?),
                "--combo" => o.combo = Some(value("--combo")?),
                "--fleet" => o.fleet_file = Some(PathBuf::from(value("--fleet")?)),
                "--panel" => {
                    let v = value("--panel")?;
                    let (c, r) = v.split_once(',').ok_or("--panel needs COLUMNS,ROWS")?;
                    o.panel = Some((
                        c.parse().map_err(|_| "--panel needs numbers")?,
                        r.parse().map_err(|_| "--panel needs numbers")?,
                    ));
                }
                "--measure" => o.measure = true,
                "--tabs" => {
                    let v = value("--tabs")?;
                    o.tabs = Some(v.parse().map_err(|_| "--tabs needs a number")?);
                }
                "--yaw" => {
                    let v = value("--yaw")?;
                    o.yaw = Some(v.parse().map_err(|_| "--yaw needs a number")?);
                }
                "--tab" => {
                    let v = value("--tab")?;
                    o.tab = Some(v.parse().map_err(|_| "--tab needs a number")?);
                }
                "--start" => o.start = true,
                "--check" => o.check = true,
                "--hole" => {
                    let v = value("--hole")?;
                    o.hole = Some(v.parse().map_err(|_| "--hole needs a number")?);
                }
                "--auto-ok" => o.auto_ok = true,
                "--select" => o.select = true,
                "--panes" => {
                    let v = value("--panes")?;
                    o.panes = Some(crate::panes::parse(&v).ok_or("--panes needs LEFT,RIGHT")?);
                }
                other if !other.starts_with("--") => o.open.push(PathBuf::from(other)),
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
        let o = parse("--lang es --theme windows-classic --drill a.ncd --route b.plt --zip job.zip --data cfg --connection m.cfg --screen viewer --shot s.png --wait 9 --start --auto-ok").unwrap();
        assert_eq!(o.lang, Some(Lang::Es));
        assert_eq!(o.theme, Some(ThemeId::WindowsClassic));
        assert_eq!(o.drill, Some(PathBuf::from("a.ncd")));
        assert_eq!(o.route, Some(PathBuf::from("b.plt")));
        assert_eq!(o.zip, Some(PathBuf::from("job.zip")));
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

    #[test]
    fn files_without_an_option_are_opened() {
        let o = parse("job.zip board.ncd").unwrap();
        assert_eq!(
            o.open,
            vec![PathBuf::from("job.zip"), PathBuf::from("board.ncd")]
        );
    }
}
