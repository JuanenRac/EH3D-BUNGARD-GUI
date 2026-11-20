//! The command line of the modern interface.

use std::path::PathBuf;

use eh3d_view::texts::Lang;
use eh3d_view::theme::ThemeId;

/// The windows `--modal` can open at the start: the words `gui::run` knows (a test keeps the two lists equal).
pub const MODALS: [&str; 17] = [
    "about",
    "update",
    "open",
    "panel",
    "settings",
    "fleet",
    "connection",
    "board",
    "tools",
    "report",
    "bar",
    "slots",
    "jog",
    "catalogue",
    "head",
    "inspect",
    "inspect-tune",
];

/// The drop-down lists `--combo` can open at the start: the words `gui::run` knows (a test keeps the two lists equal).
pub const COMBOS: [&str; 10] = [
    "theme",
    "lang",
    "model",
    "finish",
    "tool",
    "maker",
    "type",
    "thickness",
    "size",
    "tabs",
];

/// The most frames `--wait` can ask for before the picture is taken.
pub const MAX_WAIT: u32 = 3600;
/// The most tabs `--tabs` can ask for on each side (the window holds it between 1 and this).
pub const MAX_TABS: u32 = 20;
/// The last tab of the settings window `--tab` can ask for (the first is 0).
pub const MAX_TAB: u8 = 3;
/// The longest side, in pixels, of the window of a picture (`--size`).
pub const MAX_SIDE: f32 = 8192.0;

/// What `--help` prints: every option the command line reads.
pub fn usage() -> String {
    let languages: Vec<&str> = Lang::ALL.into_iter().map(|l| l.code()).collect();
    let themes: Vec<&str> = ThemeId::ALL.into_iter().map(|t| t.key()).collect();
    format!(
        "bungard-gui: the modern interface for the Bungard machines

  bungard-gui [FILE...] [--drill FILE.NCD] [--route FILE.PLT] [--zip JOB.ZIP]
              [--lang CODE] [--theme NAME] [--data DIR] [--connection FILE]
              [--screen launcher|viewer] [--machine MODEL]

  FILE        opened as what its extension says (a drilling file, a routing file or a ZIP archive)
  --report    write the report of the job (pieces, holes, tools, time, warnings) to the terminal and leave, with no window
  --server-status   ask the EH3D server how it is (the address, token and limits of the link are in the machine file) and print the metrics of the link, with no window
  --machines  list the machines and the stations of the EH3D server and leave, with no window
  --help, -h  this text
  --version, -V

For the pictures of the window (none of them is remembered):
  --shot FILE.png [--wait FRAMES (0 to {MAX_WAIT}, 6 by default)] [--size WIDTHxHEIGHT (400x300 to {MAX_SIDE}, 1600x900 by default)]
  --modal WINDOW ({modals})
  --combo LIST ({combos})
  --tab N (0 to {MAX_TAB}) of the settings window   --panel COLUMNS,ROWS (1 or more)   --tabs COUNT (1 to {MAX_TABS})
  --yaw DEGREES   --zoom FACTOR (above 0)   --panes LEFT,RIGHT   --hole N (from 1)   --fleet FILE.json
  --start   --check   --measure   --select   --auto-ok

--lang takes one of: {languages}
--theme takes one of: {themes}
An option that is written wrong, given twice or given a value that makes no sense is an error, never something to ignore.",
        modals = MODALS.join(", "),
        combos = COMBOS.join(", "),
        languages = languages.join(", "),
        themes = themes.join(", "),
    )
}

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
    /// A window to open at the start (`--modal about|open|panel|settings|jog|report`), the panel (`--panel COLUMNS,ROWS`) and the turn of the 3D view (`--yaw DEGREES`): for the pictures.
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
    /// The zoom of the plan view (`--zoom FACTOR`, 1 is the fit): below 1 the aluminium profiles of the axes, outside the table, come into view; for the pictures.
    pub zoom: Option<f32>,
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
    /// The size of the window of a picture (`--size WIDTHxHEIGHT`, 1600x900 when not given): for the pictures.
    pub size: Option<(f32, f32)>,
    /// The model of the machine to show (`--machine "CCD/2 ATC"`), without remembering it: for the pictures.
    pub machine: Option<String>,
    /// Writes the report of the job (pieces, holes, tools, time, warnings) to the terminal and leaves, with no window (`--report`).
    pub report: bool,
    /// Asks the EH3D server how it is and prints the metrics of the link, with no window (`--server-status`).
    pub server_status: bool,
    /// Lists the machines and the stations of the EH3D server and leaves, with no window (`--machines`).
    pub machines: bool,
    /// Prints the options and leaves (`--help` or `-h`).
    pub help: bool,
    /// Prints the version and leaves (`--version` or `-V`).
    pub version: bool,
}

impl Options {
    pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
        let mut o = Options {
            wait: 6,
            ..Default::default()
        };
        let mut it = args.into_iter();
        let mut seen: Vec<String> = Vec::new();
        while let Some(a) = it.next() {
            // an option is given once: the second would silently replace the first
            if a.starts_with("--") {
                if seen.contains(&a) {
                    return Err(format!("{a} is given twice"));
                }
                seen.push(a.clone());
            }
            let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
            match a.as_str() {
                "--help" | "-h" => o.help = true,
                "--version" | "-V" => o.version = true,
                "--lang" => {
                    let v = value("--lang")?;
                    o.lang = Some(Lang::from_code(&v).ok_or_else(|| {
                        let codes: Vec<&str> = Lang::ALL.into_iter().map(|l| l.code()).collect();
                        format!("unknown language '{v}' (use one of: {})", codes.join(", "))
                    })?);
                }
                "--theme" => {
                    let v = value("--theme")?;
                    o.theme = Some(ThemeId::from_key(&v).ok_or_else(|| {
                        let keys: Vec<&str> = ThemeId::ALL.into_iter().map(|t| t.key()).collect();
                        format!("unknown theme '{v}' (use one of: {})", keys.join(", "))
                    })?);
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
                        .parse::<u32>()
                        .ok()
                        .filter(|n| *n <= MAX_WAIT)
                        .ok_or_else(|| {
                            format!("--wait needs a number from 0 to {MAX_WAIT}, not '{v}'")
                        })?;
                }
                "--modal" => {
                    let v = value("--modal")?;
                    if !MODALS.contains(&v.as_str()) {
                        return Err(format!(
                            "--modal must be one of {}, not '{v}'",
                            MODALS.join(", ")
                        ));
                    }
                    o.modal = Some(v);
                }
                "--combo" => {
                    let v = value("--combo")?;
                    if !COMBOS.contains(&v.as_str()) {
                        return Err(format!(
                            "--combo must be one of {}, not '{v}'",
                            COMBOS.join(", ")
                        ));
                    }
                    o.combo = Some(v);
                }
                "--fleet" => o.fleet_file = Some(PathBuf::from(value("--fleet")?)),
                "--panel" => {
                    let v = value("--panel")?;
                    let (c, r) = v.split_once(',').ok_or("--panel needs COLUMNS,ROWS")?;
                    let (c, r): (u32, u32) = (
                        c.trim().parse().map_err(|_| "--panel needs numbers")?,
                        r.trim().parse().map_err(|_| "--panel needs numbers")?,
                    );
                    if c == 0 || r == 0 {
                        return Err("--panel needs COLUMNS,ROWS of 1 or more".to_string());
                    }
                    o.panel = Some((c, r));
                }
                "--measure" => o.measure = true,
                "--tabs" => {
                    let v = value("--tabs")?;
                    o.tabs = Some(
                        v.parse::<u32>()
                            .ok()
                            .filter(|n| (1..=MAX_TABS).contains(n))
                            .ok_or_else(|| {
                                format!("--tabs needs a number from 1 to {MAX_TABS}, not '{v}'")
                            })?,
                    );
                }
                "--zoom" => {
                    let v = value("--zoom")?;
                    o.zoom = Some(
                        v.parse::<f32>()
                            .ok()
                            .filter(|z| z.is_finite() && *z > 0.0)
                            .ok_or_else(|| format!("--zoom needs a number above 0, not '{v}'"))?,
                    );
                }
                "--yaw" => {
                    let v = value("--yaw")?;
                    o.yaw = Some(
                        v.parse::<f32>()
                            .ok()
                            .filter(|y| y.is_finite())
                            .ok_or_else(|| format!("--yaw needs a number, not '{v}'"))?,
                    );
                }
                "--tab" => {
                    let v = value("--tab")?;
                    o.tab = Some(v.parse::<u8>().ok().filter(|n| *n <= MAX_TAB).ok_or_else(
                        || format!("--tab needs a number from 0 to {MAX_TAB}, not '{v}'"),
                    )?);
                }
                "--start" => o.start = true,
                "--check" => o.check = true,
                "--hole" => {
                    let v = value("--hole")?;
                    o.hole = Some(
                        v.parse::<usize>()
                            .ok()
                            .filter(|n| *n >= 1)
                            .ok_or_else(|| format!("--hole needs a number from 1, not '{v}'"))?,
                    );
                }
                "--auto-ok" => o.auto_ok = true,
                "--report" => o.report = true,
                "--server-status" => o.server_status = true,
                "--machines" => o.machines = true,
                "--select" => o.select = true,
                "--panes" => {
                    let v = value("--panes")?;
                    o.panes = Some(crate::panes::parse(&v).ok_or("--panes needs LEFT,RIGHT")?);
                }
                "--size" => {
                    let v = value("--size")?;
                    let size = v.split_once('x').and_then(|(w, h)| {
                        Some((w.trim().parse::<f32>().ok()?, h.trim().parse::<f32>().ok()?))
                    });
                    o.size = Some(
                        size.filter(|(w, h)| {
                            (400.0..=MAX_SIDE).contains(w) && (300.0..=MAX_SIDE).contains(h)
                        })
                        .ok_or_else(|| {
                            format!(
                                "--size needs WIDTHxHEIGHT, from 400x300 to {MAX_SIDE}x{MAX_SIDE}"
                            )
                        })?,
                    );
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
    fn the_zoom_of_the_plan_is_read_and_a_word_that_is_not_a_number_is_refused() {
        assert_eq!(parse("--zoom 0.5").unwrap().zoom, Some(0.5));
        assert_eq!(parse("--zoom 2 --yaw 10").unwrap().zoom, Some(2.0));
        assert_eq!(parse("--yaw 10").unwrap().zoom, None);
        assert!(parse("--zoom big").unwrap_err().contains("--zoom"));
        assert!(parse("--zoom").is_err());
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
    fn the_report_is_asked_for_with_its_files_and_its_panel() {
        let o = parse("--report --drill a.ncd --panel 2,3 --lang de b.plt").unwrap();
        assert!(o.report);
        assert_eq!(o.panel, Some((2, 3)));
        assert_eq!(o.lang, Some(Lang::De));
        assert_eq!(o.open, vec![PathBuf::from("b.plt")]);
        assert!(!parse("--drill a.ncd").unwrap().report);
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
    fn numbers_that_make_no_sense_are_refused_and_the_ones_that_do_are_read() {
        for bad in [
            "--wait 3601",
            "--wait -1",
            "--hole 0",
            "--hole -3",
            "--tabs 0",
            "--tabs 21",
            "--tab 4",
            "--tab -1",
            "--panel 0,2",
            "--panel 2,0",
            "--panel 2",
            "--zoom 0",
            "--zoom -1",
            "--zoom NaN",
            "--zoom inf",
            "--yaw NaN",
            "--yaw inf",
            "--size 9000x900",
            "--size 1600x9000",
            "--size 100x100",
            "--size inf x900",
        ] {
            let e = parse(bad).unwrap_err();
            assert!(e.starts_with(&bad[..bad.find(' ').unwrap()]), "{bad}: {e}");
        }
        let o = parse("--wait 3600 --hole 1 --tabs 20 --tab 3 --panel 1,1 --zoom 0.01 --yaw -90 --size 8192x300").unwrap();
        assert_eq!(
            (o.wait, o.hole, o.tabs, o.tab),
            (3600, Some(1), Some(20), Some(3))
        );
        assert_eq!(
            (o.panel, o.zoom, o.yaw),
            (Some((1, 1)), Some(0.01), Some(-90.0))
        );
        assert_eq!(o.size, Some((8192.0, 300.0)));
        // the turn used for the pictures of the 3D view is beyond what the view allows and is still read
        assert_eq!(parse("--yaw 75").unwrap().yaw, Some(75.0));
        assert_eq!(parse("--wait 0").unwrap().wait, 0);
    }

    #[test]
    fn the_words_of_modal_and_combo_are_the_ones_the_window_knows() {
        for w in MODALS {
            assert_eq!(
                parse(&format!("--modal {w}")).unwrap().modal.as_deref(),
                Some(w)
            );
        }
        for w in COMBOS {
            assert_eq!(
                parse(&format!("--combo {w}")).unwrap().combo.as_deref(),
                Some(w)
            );
        }
        let e = parse("--modal abuot").unwrap_err();
        assert!(
            e.contains("abuot") && e.contains("about") && e.contains("inspect-tune"),
            "{e}"
        );
        let e = parse("--combo colour").unwrap_err();
        assert!(e.contains("colour") && e.contains("thickness"), "{e}");
        assert!(parse("--modal").unwrap_err().contains("needs a value"));
        // the lists and the code of the window say the same words
        let gui = include_str!("gui.rs");
        let modal_at = gui
            .find("match o.modal.as_deref() {")
            .expect("the match of the window");
        let modal_code = &gui[modal_at..modal_at + gui[modal_at..].find("_ => {}").unwrap()];
        let mut in_code: Vec<&str> = modal_code
            .split("Some(\"")
            .skip(1)
            .filter_map(|s| s.split('"').next())
            .collect();
        in_code.sort_unstable();
        let mut listed = MODALS.to_vec();
        listed.sort_unstable();
        assert_eq!(in_code, listed, "--modal and gui::run disagree");
        let combo_at = gui.find("match c {").expect("the match of the lists");
        let combo_code = &gui[combo_at..combo_at + gui[combo_at..].find("_ => None").unwrap()];
        let mut in_code: Vec<&str> = combo_code.split('"').skip(1).step_by(2).collect();
        in_code.sort_unstable();
        let mut listed = COMBOS.to_vec();
        listed.sort_unstable();
        assert_eq!(in_code, listed, "--combo and gui::run disagree");
    }

    #[test]
    fn help_and_version_are_read_and_the_help_names_every_option() {
        let o = parse("--help").unwrap();
        assert!(o.help && !o.version);
        assert!(parse("-h").unwrap().help);
        assert!(parse("-V").unwrap().version);
        assert!(parse("--version").unwrap().version);
        assert!(!parse("--drill a.ncd").unwrap().help);
        // the word -h is not a file to open
        assert!(parse("-h").unwrap().open.is_empty());
        let text = usage();
        let source = include_str!("cli.rs");
        let code = &source[..source.find("#[cfg(test)]").unwrap()];
        let mut options: Vec<&str> = code
            .split("\"--")
            .skip(1)
            .filter_map(|s| {
                let end = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))?;
                (s[end..].starts_with('"')).then_some(&s[..end])
            })
            .collect();
        options.sort_unstable();
        options.dedup();
        assert!(options.len() >= 25, "{options:?}");
        for name in options {
            assert!(
                text.contains(&format!("--{name}")),
                "the help does not name --{name}"
            );
        }
        assert!(text.contains("-h") && text.contains("-V"));
        for l in ["en", "pt-br", "fr-ca"] {
            assert!(text.contains(l), "{l}");
        }
        assert!(text.contains("windows-classic") && text.contains("inspect-tune"));
    }

    #[test]
    fn the_two_commands_of_the_server_are_options_that_are_off_unless_they_are_given() {
        let o = parse("").unwrap();
        assert!(!o.server_status && !o.machines);
        let o = parse("--server-status").unwrap();
        assert!(o.server_status && !o.machines);
        let o = parse("--machines --connection m.cfg").unwrap();
        assert!(o.machines && !o.server_status);
        assert_eq!(o.connection, Some(PathBuf::from("m.cfg")));
        // --machines is not the model of the machine (--machine)
        let o = parse("--machine CCD").unwrap();
        assert!(!o.machines && o.machine.as_deref() == Some("CCD"));
        assert!(parse("--machines --machines")
            .unwrap_err()
            .contains("twice"));
        let help = usage();
        assert!(help.contains("--server-status") && help.contains("--machines"));
    }

    #[test]
    fn an_unknown_language_or_theme_names_the_choices() {
        let e = parse("--lang xx").unwrap_err();
        assert!(
            e.contains("unknown language 'xx'") && e.contains("en") && e.contains("pt-br"),
            "{e}"
        );
        let e = parse("--theme pink").unwrap_err();
        assert!(
            e.contains("unknown theme 'pink'")
                && e.contains("forge")
                && e.contains("windows-classic"),
            "{e}"
        );
        // a region that is not a variant still gives its main language
        assert_eq!(parse("--lang es-MX").unwrap().lang, Some(Lang::Es));
    }

    #[test]
    fn an_option_given_twice_is_an_error_and_files_may_be_many() {
        for twice in [
            "--lang en --lang es",
            "--drill a.ncd --drill b.ncd",
            "--start --start",
            "--wait 3 --shot s.png --wait 4",
            "--help --help",
        ] {
            let e = parse(twice).unwrap_err();
            assert!(e.contains("is given twice"), "{twice}: {e}");
        }
        assert!(parse("--drill a.ncd --route a.plt").is_ok());
        // the value of an option that looks like an option is the value, not a second option
        assert_eq!(
            parse("--data --lang").unwrap().data,
            Some(PathBuf::from("--lang"))
        );
        assert_eq!(parse("a.ncd b.ncd c.zip").unwrap().open.len(), 3);
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
