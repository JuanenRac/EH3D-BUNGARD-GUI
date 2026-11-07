//! The report of a job that can be copied: how many pieces, the holes and the paths by tool, how long the job takes (an estimate from the figures of
//! `report_defaults.cfg`) and what looks wrong in it. It is worked out from the scene alone (the same for the window and for the command line, `--report`),
//! and written in any of the 37 languages from the files of `locales/report/` (`Name<TAB>text` per line; a name that is missing is taken from English).

use std::path::Path;

use eh3d_view::scene::Scene;
use eh3d_view::texts::Lang;

use crate::jobinfo::clock;

/// The starting values, in the same format as the file of the person (see `report_defaults.cfg`).
pub const DEFAULTS: &str = include_str!("report_defaults.cfg");

/// The figures of the estimate and of the checks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Config {
    /// Seconds a hole takes (the stroke down and up), without the move to it.
    pub drill_cycle_s: f64,
    /// Speed of the head between holes and to the start of a path, in mm per second.
    pub rapid_mm_s: f64,
    /// Speed of the cut along a path, in mm per second.
    pub route_feed_mm_s: f64,
    /// Seconds each tool takes to be taken and put back.
    pub tool_change_s: f64,
    /// A job that takes more minutes than this is warned about.
    pub long_job_min: f64,
    /// The gap between two pieces on the command line, in mm.
    pub panel_gap_mm: f64,
}

impl Config {
    /// The figures that come with the program (the same as `report_defaults.cfg`).
    pub fn defaults() -> Config {
        Config {
            drill_cycle_s: 0.8,
            rapid_mm_s: 100.0,
            route_feed_mm_s: 10.0,
            tool_change_s: 15.0,
            long_job_min: 240.0,
            panel_gap_mm: 5.0,
        }
    }

    /// The text of the file that comes with the program, to copy as the file of the person.
    pub fn default_file() -> &'static str {
        DEFAULTS
    }

    /// The defaults with the file of the person over them: a line that is wrong keeps the value it had.
    pub fn parse(text: &str) -> Config {
        let mut c = Config::defaults();
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let Some(v) = value
                .trim()
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v >= 0.0)
            else {
                continue;
            };
            match key.trim() {
                "drill_cycle_s" => c.drill_cycle_s = v,
                "rapid_mm_s" if v > 0.0 => c.rapid_mm_s = v,
                "route_feed_mm_s" if v > 0.0 => c.route_feed_mm_s = v,
                "tool_change_s" => c.tool_change_s = v,
                "long_job_min" => c.long_job_min = v,
                "panel_gap_mm" => c.panel_gap_mm = v,
                _ => {}
            }
        }
        c
    }

    /// Reads the file; a file that is not there gives the defaults.
    pub fn load(path: &Path) -> Config {
        std::fs::read_to_string(path)
            .map(|t| Config::parse(&t))
            .unwrap_or_else(|_| Config::defaults())
    }
}

/// One tool of the job: a drill with its holes, or a cutter with its paths.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolLine {
    pub tool: u8,
    pub drill: bool,
    /// The diameter of a drill (0 for a cutter: the file does not say it).
    pub diameter_mm: f64,
    /// Holes of a drill, paths of a cutter.
    pub count: usize,
}

/// What looks wrong in the job.
#[derive(Debug, Clone, PartialEq)]
pub enum Warn {
    /// No holes and no paths.
    Empty,
    /// Holes and paths that are not inside the table.
    Outside { holes: usize, paths: usize },
    /// The job takes more than the limit (the minutes of the limit).
    Long { limit_min: u64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub pieces: u32,
    pub holes: usize,
    pub paths: usize,
    pub tools: Vec<ToolLine>,
    /// The length of all the paths, in mm.
    pub route_mm: f64,
    /// The estimate of the time of the whole job, in seconds.
    pub seconds: u64,
    pub warnings: Vec<Warn>,
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).hypot(a.1 - b.1)
}

/// The report of a scene that holds `pieces` copies of the board (the holes and the paths of all of them).
pub fn build(scene: &Scene, pieces: u32, cfg: &Config) -> Report {
    let mut tools: Vec<ToolLine> = Vec::new();
    for h in &scene.holes {
        match tools.iter_mut().find(|t| t.drill && t.tool == h.tool) {
            Some(t) => t.count += 1,
            None => tools.push(ToolLine {
                tool: h.tool,
                drill: true,
                diameter_mm: h.diameter_mm,
                count: 1,
            }),
        }
    }
    for p in &scene.paths {
        match tools.iter_mut().find(|t| !t.drill && t.tool == p.tool) {
            Some(t) => t.count += 1,
            None => tools.push(ToolLine {
                tool: p.tool,
                drill: false,
                diameter_mm: 0.0,
                count: 1,
            }),
        }
    }
    // the head starts at the corner of the table, drills in the order of the file and then follows the paths in their order
    let mut at = (0.0, 0.0);
    let mut seconds = tools.len() as f64 * cfg.tool_change_s;
    for h in &scene.holes {
        let to = (h.x_mm, h.y_mm);
        seconds += dist(at, to) / cfg.rapid_mm_s + cfg.drill_cycle_s;
        at = to;
    }
    let mut route_mm = 0.0;
    for p in scene.paths.iter().filter(|p| !p.points.is_empty()) {
        seconds += dist(at, p.points[0]) / cfg.rapid_mm_s;
        let len: f64 = p.points.windows(2).map(|w| dist(w[0], w[1])).sum();
        route_mm += len;
        seconds += len / cfg.route_feed_mm_s;
        at = p.points[p.points.len() - 1];
    }
    let seconds = seconds.round() as u64;
    let mut warnings = Vec::new();
    if scene.holes.is_empty() && scene.paths.is_empty() {
        warnings.push(Warn::Empty);
    }
    let (out_holes, out_paths) = scene.outside_table();
    if !out_holes.is_empty() || !out_paths.is_empty() {
        warnings.push(Warn::Outside {
            holes: out_holes.len(),
            paths: out_paths.len(),
        });
    }
    if cfg.long_job_min > 0.0 && seconds as f64 > cfg.long_job_min * 60.0 {
        warnings.push(Warn::Long {
            limit_min: cfg.long_job_min.round() as u64,
        });
    }
    Report {
        pieces,
        holes: scene.holes.len(),
        paths: scene.paths.len(),
        tools,
        route_mm,
        seconds,
        warnings,
    }
}

/// The scene repeated in `columns` x `rows` pieces, each one the size of the work and the gap apart (the panel of the command line).
pub fn panel(scene: &Scene, columns: u32, rows: u32, gap_mm: f64) -> Scene {
    match scene.bounds() {
        Some((x0, y0, x1, y1)) => {
            scene.panelized(columns, rows, (x1 - x0 + gap_mm, y1 - y0 + gap_mm))
        }
        None => scene.clone(),
    }
}

/// The names of the words of the report, in the order of the files of the languages.
pub const KEYS: [&str; 15] = [
    "Title",
    "Pieces",
    "Holes",
    "PerPiece",
    "Tools",
    "Paths",
    "Length",
    "Time",
    "Warnings",
    "None",
    "Copy",
    "Copied",
    "WarnEmpty",
    "WarnOutside",
    "WarnLong",
];

/// The files of the languages, one for each language of the program.
const FILES: [(&str, &str); 37] = [
    ("en", include_str!("../locales/report/en.tsv")),
    ("es", include_str!("../locales/report/es.tsv")),
    ("it", include_str!("../locales/report/it.tsv")),
    ("de", include_str!("../locales/report/de.tsv")),
    ("fr", include_str!("../locales/report/fr.tsv")),
    ("pt", include_str!("../locales/report/pt.tsv")),
    ("zh", include_str!("../locales/report/zh.tsv")),
    ("ja", include_str!("../locales/report/ja.tsv")),
    ("bg", include_str!("../locales/report/bg.tsv")),
    ("hr", include_str!("../locales/report/hr.tsv")),
    ("cs", include_str!("../locales/report/cs.tsv")),
    ("da", include_str!("../locales/report/da.tsv")),
    ("nl", include_str!("../locales/report/nl.tsv")),
    ("et", include_str!("../locales/report/et.tsv")),
    ("fi", include_str!("../locales/report/fi.tsv")),
    ("el", include_str!("../locales/report/el.tsv")),
    ("hu", include_str!("../locales/report/hu.tsv")),
    ("ga", include_str!("../locales/report/ga.tsv")),
    ("lv", include_str!("../locales/report/lv.tsv")),
    ("lt", include_str!("../locales/report/lt.tsv")),
    ("mt", include_str!("../locales/report/mt.tsv")),
    ("pl", include_str!("../locales/report/pl.tsv")),
    ("ro", include_str!("../locales/report/ro.tsv")),
    ("sk", include_str!("../locales/report/sk.tsv")),
    ("sl", include_str!("../locales/report/sl.tsv")),
    ("sv", include_str!("../locales/report/sv.tsv")),
    ("no", include_str!("../locales/report/no.tsv")),
    ("is", include_str!("../locales/report/is.tsv")),
    ("uk", include_str!("../locales/report/uk.tsv")),
    ("ru", include_str!("../locales/report/ru.tsv")),
    ("sr", include_str!("../locales/report/sr.tsv")),
    ("sq", include_str!("../locales/report/sq.tsv")),
    ("ca", include_str!("../locales/report/ca.tsv")),
    ("gl", include_str!("../locales/report/gl.tsv")),
    ("eu", include_str!("../locales/report/eu.tsv")),
    ("pt-br", include_str!("../locales/report/pt-br.tsv")),
    ("fr-ca", include_str!("../locales/report/fr-ca.tsv")),
];

/// The text of a word in a language: its own, or the English one when the file of the language lacks it.
pub fn word(lang: Lang, key: &str) -> String {
    let find = |code: &str| {
        FILES
            .iter()
            .find(|(c, _)| *c == code)
            .and_then(|(_, text)| read(text, key))
    };
    find(lang.code())
        .or_else(|| find("en"))
        .unwrap_or_else(|| key.to_string())
}

fn read(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let (k, v) = line.trim_end_matches('\r').split_once('\t')?;
        (k.trim() == key && !v.trim().is_empty()).then(|| v.trim().replace("\\n", "\n"))
    })
}

/// The report as text, line by line, in the language. `notes` are lines the window adds to the warnings (a board that does not fit the table).
pub fn lines(r: &Report, lang: Lang, notes: &[String]) -> Vec<String> {
    let w = |k: &str| word(lang, k);
    let mut out = vec![w("Title")];
    out.push(format!("{}: {}", w("Pieces"), r.pieces));
    let per = if r.pieces > 0 {
        r.holes / r.pieces as usize
    } else {
        r.holes
    };
    out.push(format!(
        "{}: {} ({} {})",
        w("Holes"),
        r.holes,
        per,
        w("PerPiece")
    ));
    out.push(format!("{}: {}", w("Paths"), r.paths));
    out.push(format!("{}: {}", w("Tools"), r.tools.len()));
    for t in &r.tools {
        if t.drill {
            out.push(format!(
                "  T{:02}  {:.2} mm  x{}",
                t.tool, t.diameter_mm, t.count
            ));
        } else {
            out.push(format!("  T{:02}  x{}", t.tool, t.count));
        }
    }
    out.push(format!("{}: {:.1} mm", w("Length"), r.route_mm));
    out.push(format!("{}: {}", w("Time"), clock(r.seconds)));
    let mut warn: Vec<String> = Vec::new();
    for x in &r.warnings {
        warn.push(match x {
            Warn::Empty => w("WarnEmpty"),
            Warn::Outside { holes, paths } => format!(
                "{} ({}: {}, {}: {})",
                w("WarnOutside"),
                w("Holes"),
                holes,
                w("Paths"),
                paths
            ),
            Warn::Long { limit_min } => {
                format!("{} {}", w("WarnLong"), clock(limit_min * 60))
            }
        });
    }
    warn.extend(notes.iter().cloned());
    if warn.is_empty() {
        out.push(format!("{}: {}", w("Warnings"), w("None")));
    } else {
        out.push(format!("{}:", w("Warnings")));
        out.extend(warn.into_iter().map(|l| format!("  - {l}")));
    }
    out
}

/// The report as one text to copy.
pub fn text(r: &Report, lang: Lang, notes: &[String]) -> String {
    lines(r, lang, notes).join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use eh3d_view::scene::{Hole, Path as ScenePath, Table};

    fn hole(x: f64, y: f64, d: f64, tool: u8) -> Hole {
        Hole {
            x_mm: x,
            y_mm: y,
            diameter_mm: d,
            tool,
            done: false,
        }
    }

    /// Three holes of two drills and one path, on a table of 495 x 325 mm.
    fn job() -> Scene {
        let mut s = Scene::new(Table::new(495.0, 325.0));
        s.holes = vec![
            hole(30.0, 40.0, 1.0, 10),
            hole(60.0, 80.0, 1.0, 10),
            hole(60.0, 130.0, 0.8, 8),
        ];
        s.paths = vec![ScenePath {
            tool: 1,
            points: vec![(0.0, 0.0), (100.0, 0.0), (100.0, 50.0)],
            reached: 0,
        }];
        s
    }

    #[test]
    fn a_panel_puts_the_pieces_the_size_of_the_work_and_the_gap_apart() {
        let mut s = Scene::new(Table::new(495.0, 325.0));
        s.holes = vec![hole(10.0, 20.0, 1.0, 10), hole(110.0, 70.0, 1.0, 10)];
        // the work is 100 x 50 mm, so the pitch is 105 x 55 mm
        let p = panel(&s, 2, 2, 5.0);
        assert_eq!(p.holes.len(), 8);
        let last = &p.holes[7];
        assert_eq!((last.x_mm, last.y_mm), (215.0, 125.0));
        assert_eq!(panel(&s, 1, 1, 5.0), s);
        let empty = Scene::new(Table::new(495.0, 325.0));
        assert_eq!(panel(&empty, 3, 3, 5.0), empty);
    }

    #[test]
    fn the_pieces_the_holes_and_the_tools_are_counted() {
        let r = build(&job(), 1, &Config::defaults());
        assert_eq!((r.pieces, r.holes, r.paths), (1, 3, 1));
        assert_eq!(
            r.tools,
            vec![
                ToolLine {
                    tool: 10,
                    drill: true,
                    diameter_mm: 1.0,
                    count: 2
                },
                ToolLine {
                    tool: 8,
                    drill: true,
                    diameter_mm: 0.8,
                    count: 1
                },
                ToolLine {
                    tool: 1,
                    drill: false,
                    diameter_mm: 0.0,
                    count: 1
                },
            ]
        );
        assert_eq!(r.route_mm, 150.0);
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn the_time_is_the_sum_of_the_moves_the_strokes_the_cuts_and_the_tool_changes() {
        // worked by hand with the default figures (100 mm/s, 0.8 s a hole, 10 mm/s, 15 s a tool):
        // tools 3 x 15 = 45; moves 50 + 50 + 50 = 150 mm = 1.5 s; strokes 3 x 0.8 = 2.4 s;
        // to the path from (60,130): hypot(60,130) = 143.178211 mm = 1.43178211 s; cut 150 mm = 15 s
        // total 45 + 1.5 + 2.4 + 1.43178211 + 15 = 65.33178211 s, which rounds to 65
        let r = build(&job(), 1, &Config::defaults());
        assert_eq!(r.seconds, 65);
    }

    #[test]
    fn the_figures_of_the_file_change_the_time() {
        let cfg = Config::parse(
            "tool_change_s = 0\ndrill_cycle_s = 1\nrapid_mm_s = 50\nroute_feed_mm_s = 5",
        );
        // 0 + (50 + 50 + 50) / 50 = 3 s + 3 strokes of 1 s + hypot(60,130) / 50 = 2.86356421 s + 150 / 5 = 30 s = 38.86356421 s
        assert_eq!(build(&job(), 1, &cfg).seconds, 39);
    }

    #[test]
    fn a_job_with_nothing_in_it_is_warned_about() {
        let r = build(
            &Scene::new(Table::new(495.0, 325.0)),
            1,
            &Config::defaults(),
        );
        assert_eq!(r.warnings, vec![Warn::Empty]);
        assert_eq!(r.seconds, 0);
    }

    #[test]
    fn what_is_outside_the_table_is_counted() {
        let mut s = job();
        s.holes.push(hole(500.0, 10.0, 1.0, 10));
        s.paths.push(ScenePath {
            tool: 1,
            points: vec![(10.0, 10.0), (10.0, 330.0)],
            reached: 0,
        });
        let r = build(&s, 1, &Config::defaults());
        assert_eq!(r.warnings, vec![Warn::Outside { holes: 1, paths: 1 }]);
    }

    #[test]
    fn a_long_job_is_warned_about_with_the_limit_of_the_file() {
        let cfg = Config::parse("long_job_min = 1");
        // a minute is 60 s and the job takes 65
        assert_eq!(
            build(&job(), 1, &cfg).warnings,
            vec![Warn::Long { limit_min: 1 }]
        );
        // a limit of 0 turns the warning off
        let off = Config::parse("long_job_min = 0");
        assert!(build(&job(), 1, &off).warnings.is_empty());
    }

    #[test]
    fn the_text_in_english_is_the_one_to_copy() {
        let r = build(&job(), 1, &Config::defaults());
        let t = text(&r, Lang::En, &[]);
        let expected = [
            "Work report",
            "Pieces: 1",
            "Holes: 3 (3 per piece)",
            "Paths: 1",
            "Tools: 3",
            "  T10  1.00 mm  x2",
            "  T08  0.80 mm  x1",
            "  T01  x1",
            "Routed length: 150.0 mm",
            "Time (estimate): 1:05",
            "Warnings: none",
        ]
        .join("\n");
        assert_eq!(t, expected);
    }

    #[test]
    fn the_pieces_divide_the_holes_and_the_warnings_and_the_notes_are_listed() {
        let mut s = Scene::new(Table::new(495.0, 325.0));
        for i in 0..4 {
            s.holes.push(hole(10.0 + 20.0 * i as f64, 10.0, 1.0, 10));
            s.holes.push(hole(10.0 + 20.0 * i as f64, 30.0, 1.0, 10));
        }
        let r = build(&s, 4, &Config::defaults());
        let l = lines(&r, Lang::En, &["The board is too big for the table".into()]);
        assert_eq!(l[2], "Holes: 8 (2 per piece)");
        assert_eq!(l[l.len() - 2], "Warnings:");
        assert_eq!(l[l.len() - 1], "  - The board is too big for the table");
        let es = text(&r, Lang::Es, &[]);
        assert!(es.starts_with("Informe de trabajo"), "{es}");
        assert!(es.contains("Piezas: 4"), "{es}");
    }

    #[test]
    fn the_file_that_comes_with_the_program_is_the_defaults() {
        assert_eq!(Config::parse(Config::default_file()), Config::defaults());
        // and every key of the defaults is written in it
        for k in [
            "drill_cycle_s",
            "rapid_mm_s",
            "route_feed_mm_s",
            "tool_change_s",
            "long_job_min",
            "panel_gap_mm",
        ] {
            assert!(Config::default_file().contains(&format!("\n{k} =")), "{k}");
        }
    }

    #[test]
    fn a_line_that_is_wrong_keeps_the_value_it_had() {
        for bad in [
            "rapid_mm_s = 0",
            "rapid_mm_s = fast",
            "route_feed_mm_s = -3",
            "tool_change_s = nan",
            "unknown = 4",
            "no equals sign",
        ] {
            assert_eq!(Config::parse(bad), Config::defaults(), "{bad}");
        }
        assert_eq!(Config::parse("tool_change_s = 20").tool_change_s, 20.0);
        let missing = std::env::temp_dir().join("eh3d-report-missing.cfg");
        assert_eq!(Config::load(&missing), Config::defaults());
    }

    #[test]
    fn every_language_has_every_word_and_only_those() {
        assert_eq!(FILES.len(), Lang::ALL.len());
        for l in Lang::ALL {
            let (code, text) = FILES
                .iter()
                .find(|(c, _)| *c == l.code())
                .unwrap_or_else(|| panic!("no file for {}", l.code()));
            assert_eq!(*code, l.code());
            let mut seen = Vec::new();
            for line in text.lines() {
                if line.starts_with('#') || line.trim().is_empty() {
                    continue;
                }
                let (k, v) = line
                    .split_once('\t')
                    .unwrap_or_else(|| panic!("{code}: {line}"));
                assert!(
                    KEYS.contains(&k),
                    "{code}: the name {k} is not a word of the report"
                );
                assert!(!v.trim().is_empty(), "{code}: {k} is empty");
                assert!(!seen.contains(&k), "{code}: {k} twice");
                seen.push(k);
            }
            for k in KEYS {
                assert!(seen.contains(&k), "{code} lacks {k}");
            }
        }
    }

    #[test]
    fn no_language_but_english_keeps_the_english_sentences() {
        let en: Vec<String> = KEYS.iter().map(|k| word(Lang::En, k)).collect();
        for l in Lang::ALL.into_iter().filter(|l| *l != Lang::En) {
            let same = KEYS
                .iter()
                .zip(&en)
                .filter(|(k, e)| word(l, k) == **e)
                .count();
            // only names that read the same in many languages may match (a short word such as the title of a unit)
            assert!(same <= 3, "{} keeps {same} English words", l.code());
        }
    }

    #[test]
    fn the_report_is_written_in_every_language_with_its_numbers() {
        let r = build(&job(), 1, &Config::defaults());
        for l in Lang::ALL {
            let t = text(&r, l, &[]);
            assert!(
                t.contains("150.0 mm") && t.contains("1:05"),
                "{}: {t}",
                l.code()
            );
            assert!(t.lines().count() == 11, "{}: {t}", l.code());
        }
    }
}
