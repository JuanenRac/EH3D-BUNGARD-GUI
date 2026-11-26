//! Behaviour tests of the log, the job report, the server commands, the command line, the settings file and the update helpers.
//! They read only the public API of the crate and work in folders of their own under the target directory.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A folder of its own for one test.
fn scratch(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("behaviour-files-{n}-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

/// A small generator with a fixed seed.
struct Lcg(u64);

impl Lcg {
    fn step(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// applog
// ---------------------------------------------------------------------------------------------------------------------------------

mod applog_tests {
    use super::{read, scratch, Lcg};
    use eh3d_bungard_gui::applog::{
        self, format_line, proc_kb, scrub, settings_line, unclean_tail, Config, Heartbeat, Level,
        Logger, CLEAN_EXIT, SESSION_START,
    };
    use std::path::Path;
    use std::time::{Duration, Instant};

    #[test]
    fn the_numbers_of_the_log_settings_take_exactly_their_range() {
        let set = |key: &str, text: &str| {
            let mut c = Config::default();
            assert!(c.set(key, text), "{key}");
            c
        };
        for (text, expect) in [
            ("63", 1024),
            ("64", 64),
            ("65536", 65_536),
            ("65537", 1024),
            ("-1", 1024),
            ("64.0", 1024),
            (" 128 ", 128),
            ("", 1024),
        ] {
            assert_eq!(set("log_max_kb", text).max_kb, expect, "max_kb {text}");
        }
        for (text, expect) in [("0", 3), ("1", 1), ("20", 20), ("21", 3), ("x", 3)] {
            assert_eq!(set("log_files", text).files, expect, "files {text}");
        }
        for (text, expect) in [("4", 30), ("5", 5), ("3600", 3600), ("3601", 30)] {
            assert_eq!(
                set("log_heartbeat_s", text).heartbeat_s,
                expect,
                "heartbeat {text}"
            );
        }
    }

    #[test]
    fn the_switch_and_the_level_words_are_read_loosely_and_the_text_comes_back() {
        for l in [Level::Error, Level::Warn, Level::Info, Level::Debug] {
            assert_eq!(Level::parse(l.key()), Some(l));
            assert_eq!(Level::parse(&l.key().to_ascii_uppercase()), Some(l));
            assert_eq!(l.label().len(), 5);
            assert_eq!(l.label().trim_end().to_ascii_lowercase(), l.key());
        }
        assert_eq!(Level::parse("  Warning\t"), Some(Level::Warn));
        assert_eq!(Level::parse(""), None);
        assert_eq!(Level::parse("trace"), None);
        assert!(
            Level::Error < Level::Warn && Level::Warn < Level::Info && Level::Info < Level::Debug
        );
        for off in ["no", "NO", " Off ", "false", "0"] {
            let mut c = Config::default();
            assert!(c.set("log", off));
            assert!(!c.enabled, "{off}");
        }
        for on in ["yes", "on", "1", "true", "", "whatever"] {
            let mut c = Config {
                enabled: false,
                ..Config::default()
            };
            assert!(c.set("log", on));
            assert!(c.enabled, "{on}");
        }
        // the caller gives the key in lower case; another key is not one of the log
        let mut c = Config::default();
        assert!(!c.set("LOG_LEVEL", "debug"));
        assert!(!c.set("logs", "no"));
        assert!(!c.set("", "x"));
        assert_eq!(c, Config::default());
        assert_eq!(
            Config::default().to_text(),
            "log = yes\nlog_level = info\nlog_max_kb = 1024\nlog_files = 3\nlog_heartbeat_s = 30\n"
        );
        let mut g = Lcg(42);
        for _ in 0..60 {
            let levels = [Level::Error, Level::Warn, Level::Info, Level::Debug];
            let c = Config {
                enabled: (g.step() & 1) == 0,
                level: levels[(g.step() % 4) as usize],
                max_kb: 64 + (g.step() % 65_473) as u32,
                files: 1 + (g.step() % 20) as u32,
                heartbeat_s: 5 + (g.step() % 3_596) as u32,
            };
            let mut back = Config::default();
            for line in c.to_text().lines() {
                let (k, v) = line.split_once('=').unwrap();
                assert!(back.set(k.trim(), v));
            }
            assert_eq!(back, c);
        }
    }

    #[test]
    fn a_line_of_the_log_keeps_one_time_at_its_head_whatever_the_message_holds() {
        let l = format_line("T", 5, "1.0", 9, Level::Info, "x", "a\r\nb\n\nc");
        assert_eq!(l, "T 5 v1.0 p9 INFO  [x] a\n    b\n    \n    c\n");
        assert_eq!(
            format_line("T", 5, "1.0", 9, Level::Debug, "x", ""),
            "T 5 v1.0 p9 DEBUG [x] \n"
        );
        let many = format_line("T", 0, "1", 1, Level::Error, "t", &"w\n".repeat(50));
        assert_eq!(many.lines().count(), 51);
        assert!(many.lines().skip(1).all(|l| l.starts_with("    ")));
        assert!(many.starts_with("T 0 v1 p1 ERROR [t] w"));
    }

    #[test]
    fn an_address_loses_its_user_its_password_and_its_query() {
        assert_eq!(scrub("ftp://bob@host/dir"), "ftp://host/dir");
        assert_eq!(scrub("https://u:p@h:8/x?y=z@w"), "https://h:8/x");
        assert_eq!(
            scrub("see http://a/b?c=d and https://user:pw@x.y/z?q=1 end"),
            "see http://a/b and https://x.y/z end"
        );
        assert_eq!(scrub("a://"), "a://");
        assert_eq!(scrub("go:https://u:p@h/x"), "go:https://h/x");
        assert_eq!(scrub("no scheme user@host?x=1"), "no scheme user@host?x=1");
        assert_eq!(scrub(""), "");
        assert_eq!(scrub("a  b"), "a  b");
        assert_eq!(scrub(" x "), " x ");
    }

    #[test]
    fn a_secret_is_hidden_in_every_shape_it_is_written_and_ordinary_words_stay() {
        assert_eq!(
            scrub("Authorization: Basic dXNlcg== end"),
            "Authorization: Basic <hidden> end"
        );
        assert_eq!(scrub("send Bearer"), "send Bearer");
        assert_eq!(scrub("Bearer  tok"), "Bearer  <hidden>");
        assert_eq!(scrub(r#"{"token":"abc"}"#), r#"{"token":<hidden>"#);
        assert_eq!(scrub("token= abc"), "token= <hidden>");
        assert_eq!(scrub("secret : x"), "secret : <hidden>");
        assert_eq!(scrub("set token : abc"), "set token : <hidden>");
        assert_eq!(scrub("Token:abc"), "Token:<hidden>");
        assert_eq!(scrub("the token"), "the token");
        assert_eq!(scrub("keyboard=us layout"), "keyboard=us layout");
        assert_eq!(scrub("auth =x"), "auth =x");
        assert_eq!(scrub("pass=é ünï=ok"), "pass=<hidden> ünï=ok");
        assert_eq!(scrub("ключ=секрет token=абв"), "ключ=секрет token=<hidden>");
    }

    #[test]
    fn scrubbing_never_panics_on_any_cut_of_a_text_and_does_not_change_what_it_already_cleaned() {
        let sample =
            "https://u:p@h/é?x=1 password=ñ token : ü Bearer ß \"key\":\"v\" auth=1 ✓ secret = x";
        for at in 0..=sample.len() {
            if sample.is_char_boundary(at) {
                let _ = scrub(&sample[..at]);
            }
        }
        for text in [
            sample,
            "password=<hidden>",
            "Bearer <hidden>",
            "token : abc and more",
            r#"{"password":"x","y":1}"#,
            "plain words only",
        ] {
            let once = scrub(text);
            assert_eq!(scrub(&once), once, "{text}");
        }
    }

    #[test]
    fn a_line_of_the_settings_hides_secret_keys_and_cuts_a_long_value_at_its_160th_letter() {
        for key in [
            "api_token",
            "API_KEY",
            "user_pass",
            "secret",
            "auth_header",
            "credential",
            "PWD",
        ] {
            assert_eq!(
                settings_line(&format!("{key} = abc")).unwrap(),
                format!("{key} = <hidden>")
            );
        }
        assert_eq!(settings_line("k = a=b").unwrap(), "k = a=b");
        assert_eq!(settings_line("= x").unwrap(), " = x");
        assert_eq!(
            settings_line("server = https://u:p@h/x").unwrap(),
            "server = https://h/x"
        );
        let exact = format!("k = {}", "x".repeat(160));
        assert_eq!(settings_line(&exact).unwrap(), exact);
        let over = settings_line(&format!("k = {}", "x".repeat(161))).unwrap();
        assert_eq!(over, format!("k = {}...", "x".repeat(160)));
        // 160 letters of two bytes each are 160 letters
        let wide = format!("k = {}", "é".repeat(160));
        assert_eq!(settings_line(&wide).unwrap(), wide);
        assert!(settings_line(&format!("k = {}", "é".repeat(161)))
            .unwrap()
            .ends_with("..."));
        assert!(settings_line("").is_none());
    }

    #[test]
    fn only_the_last_forty_lines_of_a_run_that_did_not_end_are_kept() {
        let start = format!("t INFO  [run] {SESSION_START} version 1");
        for extra in [0usize, 1, 38, 39, 40, 41, 100] {
            let mut text = format!("{start}\n");
            for i in 0..extra {
                text.push_str(&format!("t INFO  [x] line {i}\n"));
            }
            let tail = unclean_tail(&text).unwrap();
            assert_eq!(tail.len(), (extra + 1).min(40), "{extra}");
            if extra < 40 {
                assert!(tail[0].contains("--- session start"), "{extra}");
            } else {
                assert!(
                    tail[0].ends_with(&format!("line {}", extra - 40)),
                    "{extra}"
                );
            }
            if extra > 0 {
                assert!(tail
                    .last()
                    .unwrap()
                    .ends_with(&format!("line {}", extra - 1)));
            }
            assert!(tail.iter().all(|l| !l.contains("===")));
        }
        // the lines of a file written with CRLF are the same lines
        let crlf = format!("{start}\r\nt INFO  [x] work\r\n");
        let tail = unclean_tail(&crlf).unwrap();
        assert_eq!(tail.len(), 2);
        assert!(tail.iter().all(|l| !l.contains('\r')));
        let ended = format!("{crlf}t INFO  [run] {CLEAN_EXIT}\r\n");
        assert!(unclean_tail(&ended).is_none());
    }

    #[test]
    fn the_level_the_secrets_and_the_line_breaks_are_dealt_with_before_a_line_reaches_the_file() {
        let d = scratch("filters");
        let mut l = Logger::new(&d, "f.log", Config::default(), "1");
        l.log(Level::Debug, "a", "hidden detail");
        assert!(!d.join("f.log").exists());
        l.log_forced(Level::Debug, "a", "forced detail");
        l.log(
            Level::Info,
            "net",
            "login password=hunter2 now\nsecond line",
        );
        let text = read(&d.join("f.log"));
        assert!(text.contains("DEBUG [a] forced detail\n"));
        assert!(!text.contains("hidden detail") && !text.contains("hunter2"));
        assert!(text.contains("INFO  [net] login password=<hidden> now\n    second line\n"));
        let mut quiet = Logger::new(
            &d,
            "q.log",
            Config {
                level: Level::Error,
                ..Config::default()
            },
            "1",
        );
        quiet.log(Level::Warn, "a", "w");
        assert!(!d.join("q.log").exists());
        quiet.log(Level::Error, "a", "e");
        assert_eq!(read(&d.join("q.log")).lines().count(), 1);
        let mut off = Logger::new(
            &d,
            "o.log",
            Config {
                enabled: false,
                ..Config::default()
            },
            "1",
        );
        off.log_forced(Level::Error, "a", "x");
        assert!(!d.join("o.log").exists());
    }

    #[test]
    fn a_line_that_repeats_is_written_three_times_and_the_rest_are_counted_once() {
        let d = scratch("repeats");
        let mut three = Logger::new(&d, "three.log", Config::default(), "1");
        for _ in 0..3 {
            three.log(Level::Warn, "t", "same");
        }
        three.finish();
        let text = read(&d.join("three.log"));
        assert_eq!(text.matches("[t] same").count(), 3);
        assert!(!text.contains("came"));
        let mut four = Logger::new(&d, "four.log", Config::default(), "1");
        for _ in 0..4 {
            four.log(Level::Warn, "t", "same");
        }
        four.finish();
        let text = read(&d.join("four.log"));
        assert_eq!(text.matches("[t] same").count(), 3);
        assert!(text.contains("WARN  [t] the last line came 1 more times\n"));
        // another level or another part of the program is another line
        let mut mixed = Logger::new(&d, "mixed.log", Config::default(), "1");
        for _ in 0..3 {
            mixed.log(Level::Warn, "t", "same");
        }
        for _ in 0..3 {
            mixed.log(Level::Error, "t", "same");
        }
        for _ in 0..3 {
            mixed.log(Level::Warn, "u", "same");
        }
        mixed.finish();
        let text = read(&d.join("mixed.log"));
        assert_eq!(text.matches("same").count(), 9);
        assert!(!text.contains("came"));
        // the count is written when something else comes, and the first line may come again afterwards
        let mut again = Logger::new(&d, "again.log", Config::default(), "1");
        for _ in 0..5 {
            again.log(Level::Warn, "t", "x");
        }
        again.log(Level::Warn, "t", "y");
        again.log(Level::Warn, "t", "x");
        again.finish();
        let text = read(&d.join("again.log"));
        assert_eq!(text.matches("[t] x").count(), 4);
        assert!(text.contains("the last line came 2 more times"));
    }

    #[test]
    fn errors_are_not_limited_and_the_files_of_the_log_are_numbered() {
        let l = Logger::new(Path::new("logs"), "x.log", Config::default(), "1");
        assert_eq!(l.path(0), Path::new("logs").join("x.log"));
        assert_eq!(l.path(3), Path::new("logs").join("x.log.3"));
        assert_eq!(l.config(), &Config::default());
        let d = scratch("errors");
        let mut l = Logger::new(&d, "e.log", Config::default(), "1");
        for i in 0..150 {
            l.log(Level::Error, "e", &format!("distinct {i}"));
        }
        let text = read(&d.join("e.log"));
        assert_eq!(text.lines().count(), 150);
        assert!(text.lines().all(|l| l.contains(" ERROR [e] distinct ")));
        assert!(!text.contains("left out"));
    }

    #[test]
    fn rotating_keeps_the_newest_lines_in_order_and_no_file_goes_over_its_size() {
        let d = scratch("rotate");
        let c = Config {
            max_kb: 64,
            files: 2,
            ..Config::default()
        };
        let mut l = Logger::new(&d, "r.log", c, "1");
        let pad = "y".repeat(500);
        for i in 0..600 {
            l.log(Level::Error, "fill", &format!("n{i:04} {pad}"));
        }
        l.finish();
        assert!(d.join("r.log").exists() && d.join("r.log.1").exists());
        assert!(!d.join("r.log.2").exists());
        let numbers = |p: &Path| -> Vec<usize> {
            read(p)
                .lines()
                .filter_map(|l| {
                    let at = l.find("] n")?;
                    l[at + 3..at + 7].parse().ok()
                })
                .collect()
        };
        let (old, new) = (numbers(&d.join("r.log.1")), numbers(&d.join("r.log")));
        assert!(!old.is_empty() && !new.is_empty());
        assert!(old.windows(2).all(|w| w[1] == w[0] + 1));
        assert!(new.windows(2).all(|w| w[1] == w[0] + 1));
        assert_eq!(old[old.len() - 1] + 1, new[0]);
        assert_eq!(new[new.len() - 1], 599);
        for p in [d.join("r.log"), d.join("r.log.1")] {
            assert!(std::fs::metadata(p).unwrap().len() <= 64 * 1024);
        }
    }

    #[test]
    fn a_file_that_was_already_big_is_closed_at_the_second_line() {
        let d = scratch("prefill");
        std::fs::write(d.join("p.log"), "x".repeat(70_000)).unwrap();
        let c = Config {
            max_kb: 64,
            files: 2,
            ..Config::default()
        };
        let mut l = Logger::new(&d, "p.log", c, "1");
        l.log(Level::Error, "a", "first");
        l.log(Level::Error, "a", "second");
        let old = read(&d.join("p.log.1"));
        assert!(old.starts_with("xxxx") && old.contains("[a] first") && !old.contains("second"));
        let new = read(&d.join("p.log"));
        assert!(new.contains("[a] second") && !new.contains("first") && !new.contains("xxxx"));
    }

    #[test]
    fn the_heartbeat_is_due_at_its_interval_and_a_pause_is_told_from_one_and_a_half_seconds() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut h = Heartbeat::new(5, t0);
        let (b, g) = h.frame(t0 + ms(4_999), ms(2));
        assert!(b.is_none() && g.is_none());
        let (b, g) = h.frame(t0 + ms(5_000), ms(4));
        assert!(g.is_none());
        let b = b.unwrap();
        assert_eq!((b.total_frames, b.frames), (2, 2));
        assert!((b.mean_ms - 3.0).abs() < 1e-9 && (b.max_ms - 4.0).abs() < 1e-9);
        let (b, g) = h.frame(t0 + ms(6_499), ms(1));
        assert!(b.is_none() && g.is_none());
        let (b, g) = h.frame(t0 + ms(7_999), ms(1));
        assert!(b.is_none());
        assert_eq!(g, Some(ms(1_500)));
        // the first frame has no pause before it, and a beat is due at once when the interval has passed
        let mut h = Heartbeat::new(30, t0);
        let (b, g) = h.frame(t0 + Duration::from_secs(3600), Duration::from_micros(1_500));
        assert!(g.is_none());
        let b = b.unwrap();
        assert_eq!((b.total_frames, b.frames), (1, 1));
        assert!((b.mean_ms - 1.5).abs() < 1e-9);
        // a clock that goes back is no pause and no beat
        let (b, g) = h.frame(t0 + Duration::from_secs(3590), ms(1));
        assert!(b.is_none() && g.is_none());
        // an interval of zero seconds is one second
        let mut z = Heartbeat::new(0, t0);
        assert!(z.frame(t0 + ms(999), ms(1)).0.is_none());
        assert!(z.frame(t0 + ms(1_000), ms(1)).0.is_some());
    }

    #[test]
    fn a_line_of_a_proc_file_is_found_by_its_whole_key() {
        let text = "RssAnon:\t 5 kB\nRss:\t 7 kB\nVmRSS: 12\nVmBad:\tabc kB\nVmNeg:\t-3 kB\nVmBig:\t18446744073709551615 kB\n";
        assert_eq!(proc_kb(text, "RssAnon"), Some(5));
        assert_eq!(proc_kb(text, "Rss"), Some(7));
        assert_eq!(proc_kb(text, "VmRSS"), Some(12));
        assert_eq!(proc_kb(text, "VmBad"), None);
        assert_eq!(proc_kb(text, "VmNeg"), None);
        assert_eq!(proc_kb(text, "VmBig"), Some(u64::MAX));
        assert_eq!(proc_kb(text, "Vm"), None);
        assert_eq!(proc_kb(text, ""), None);
        assert_eq!(proc_kb("", "Rss"), None);
    }

    #[test]
    fn before_the_log_is_started_every_function_of_the_program_log_does_nothing() {
        assert!(!applog::active());
        applog::error("t", "x");
        applog::warn("t", "x");
        applog::info("t", 5);
        applog::debug("t", "x");
        applog::clean_exit();
        applog::emergency(Level::Error, "t", "password=x");
        applog::log_settings("a = b\nkey = secret\nno equals sign\n");
        let line = applog::memory_line();
        assert!(!line.is_empty() && !line.contains('\n'));
        let dir = applog::default_dir();
        assert!(
            dir == Path::new(".")
                || dir
                    .file_name()
                    .is_some_and(|n| n.to_str() == Some("EH3D") || n.to_str() == Some("eh3d")),
            "{dir:?}"
        );
        let guard = applog::thread_guard("test");
        assert!(applog::live_threads() >= 1);
        drop(guard);
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// report
// ---------------------------------------------------------------------------------------------------------------------------------

mod report_tests {
    use super::{scratch, Lcg};
    use eh3d_bungard_gui::report::{
        build, lines, panel, text, word, Config, Report, ToolLine, Warn, KEYS,
    };
    use eh3d_view::scene::{Hole, Path as ScenePath, Scene, Table};
    use eh3d_view::texts::Lang;

    fn hole(x: f64, y: f64, d: f64, tool: u8) -> Hole {
        Hole {
            x_mm: x,
            y_mm: y,
            diameter_mm: d,
            tool,
            done: false,
        }
    }

    fn path(tool: u8, points: &[(f64, f64)]) -> ScenePath {
        ScenePath {
            tool,
            points: points.to_vec(),
            reached: 0,
        }
    }

    fn scene_of(holes: Vec<Hole>, paths: Vec<ScenePath>) -> Scene {
        let mut s = Scene::new(Table::new(495.0, 325.0));
        s.holes = holes;
        s.paths = paths;
        s
    }

    #[test]
    fn the_figures_file_is_read_line_by_line_with_comments_and_limits() {
        let d = Config::defaults();
        assert_eq!(Config::parse(""), d);
        assert_eq!(Config::parse("   \n# only a comment\n\n"), d);
        let c = Config::parse(
            "  drill_cycle_s =  2.5   # slower\n# long_job_min = 1\r\nrapid_mm_s=250\r\n",
        );
        assert_eq!(c.drill_cycle_s, 2.5);
        assert_eq!(c.rapid_mm_s, 250.0);
        assert_eq!(c.long_job_min, d.long_job_min);
        // zero is a value for the figures that may be zero and no value for a speed
        let zero = Config::parse(
            "drill_cycle_s = 0\ntool_change_s = 0\nlong_job_min = 0\npanel_gap_mm = 0\nrapid_mm_s = 0\nroute_feed_mm_s = 0\n",
        );
        assert_eq!(
            (
                zero.drill_cycle_s,
                zero.tool_change_s,
                zero.long_job_min,
                zero.panel_gap_mm
            ),
            (0.0, 0.0, 0.0, 0.0)
        );
        assert_eq!(
            (zero.rapid_mm_s, zero.route_feed_mm_s),
            (d.rapid_mm_s, d.route_feed_mm_s)
        );
        for key in [
            "drill_cycle_s",
            "rapid_mm_s",
            "route_feed_mm_s",
            "tool_change_s",
            "long_job_min",
            "panel_gap_mm",
        ] {
            for bad in ["-0.5", "inf", "-inf", "NaN", "1 2", "= 3", ""] {
                assert_eq!(Config::parse(&format!("{key} = {bad}")), d, "{key} {bad}");
            }
        }
        assert_eq!(Config::parse("RAPID_MM_S = 5"), d);
        // a line that is wrong after a good one keeps the good one
        assert_eq!(
            Config::parse("rapid_mm_s = 5\nrapid_mm_s = 0").rapid_mm_s,
            5.0
        );
        assert_eq!(
            Config::parse("rapid_mm_s = 5\nrapid_mm_s = 7").rapid_mm_s,
            7.0
        );
        assert_eq!(Config::parse("route_feed_mm_s = 1e1").route_feed_mm_s, 10.0);
        assert_eq!(Config::parse("tool_change_s = .5").tool_change_s, 0.5);
        let dir = scratch("figures");
        let file = dir.join("report.cfg");
        std::fs::write(&file, "tool_change_s = 3\n").unwrap();
        assert_eq!(Config::load(&file).tool_change_s, 3.0);
        assert_eq!(Config::load(&dir), d);
        assert_eq!(Config::load(&dir.join("missing.cfg")), d);
    }

    #[test]
    fn the_time_of_one_hole_is_rounded_to_the_nearest_second_and_halves_go_up() {
        let cfg = |cycle: f64| Config {
            tool_change_s: 0.0,
            drill_cycle_s: cycle,
            ..Config::defaults()
        };
        let at_origin = scene_of(vec![hole(0.0, 0.0, 1.0, 3)], vec![]);
        for (cycle, expect) in [(0.49, 0u64), (0.5, 1), (1.49, 1), (1.5, 2), (2.5, 3)] {
            assert_eq!(build(&at_origin, 1, &cfg(cycle)).seconds, expect, "{cycle}");
        }
        let one = scene_of(vec![hole(30.0, 40.0, 1.0, 3)], vec![]);
        // 15 s for the tool, 50 mm at 100 mm/s and 0.8 s for the stroke
        assert_eq!(build(&one, 1, &Config::defaults()).seconds, 16);
        // the same tool is taken once, and the second hole is not a move away
        let two = scene_of(
            vec![hole(30.0, 40.0, 1.0, 3), hole(30.0, 40.0, 1.0, 3)],
            vec![],
        );
        assert_eq!(build(&two, 1, &Config::defaults()).seconds, 17);
        let other = scene_of(
            vec![hole(30.0, 40.0, 1.0, 3), hole(30.0, 40.0, 1.0, 4)],
            vec![],
        );
        assert_eq!(build(&other, 1, &Config::defaults()).seconds, 32);
        let r = build(&one, 7, &Config::defaults());
        assert_eq!((r.pieces, r.holes, r.paths), (7, 1, 0));
        let empty = build(&scene_of(vec![], vec![]), 7, &Config::defaults());
        assert_eq!((empty.pieces, empty.seconds, empty.route_mm), (7, 0, 0.0));
        assert!(empty.tools.is_empty());
        assert_eq!(empty.warnings, vec![Warn::Empty]);
    }

    #[test]
    fn the_tools_are_listed_drills_first_each_in_the_order_it_first_appears() {
        let s = scene_of(
            vec![
                hole(10.0, 10.0, 1.0, 5),
                hole(20.0, 10.0, 0.5, 3),
                hole(30.0, 10.0, 0.25, 5),
                hole(40.0, 10.0, 0.5, 3),
                hole(50.0, 10.0, 0.8, 9),
            ],
            vec![
                path(3, &[(0.0, 0.0), (1.0, 0.0)]),
                path(5, &[(0.0, 0.0), (1.0, 0.0)]),
                path(3, &[(0.0, 0.0), (1.0, 0.0)]),
            ],
        );
        let cfg = Config {
            tool_change_s: 10.0,
            drill_cycle_s: 0.0,
            rapid_mm_s: 1e12,
            route_feed_mm_s: 1e12,
            ..Config::defaults()
        };
        let r = build(&s, 1, &cfg);
        let tools: Vec<(u8, bool, usize)> =
            r.tools.iter().map(|t| (t.tool, t.drill, t.count)).collect();
        assert_eq!(
            tools,
            vec![
                (5, true, 2),
                (3, true, 2),
                (9, true, 1),
                (3, false, 2),
                (5, false, 1)
            ]
        );
        // a drill keeps the diameter of its first hole and a cutter has none
        assert_eq!(r.tools[0].diameter_mm, 1.0);
        assert_eq!(r.tools[3].diameter_mm, 0.0);
        // five tools, each taken once
        assert_eq!(r.seconds, 50);
    }

    #[test]
    fn a_path_counts_by_its_points_and_a_path_without_points_only_takes_its_tool() {
        let cfg = Config::defaults();
        let s = scene_of(vec![], vec![path(1, &[(0.0, 0.0), (3.0, 4.0)])]);
        let r = build(&s, 1, &cfg);
        assert_eq!(r.route_mm, 5.0);
        // 15 s for the tool and 5 mm at 10 mm/s
        assert_eq!(r.seconds, 16);
        let three = scene_of(
            vec![],
            vec![
                path(1, &[(0.0, 0.0), (3.0, 4.0)]),
                path(1, &[]),
                path(1, &[(10.0, 0.0), (10.0, 0.0)]),
            ],
        );
        let r = build(&three, 1, &cfg);
        assert_eq!((r.paths, r.route_mm), (3, 5.0));
        assert_eq!(r.tools.len(), 1);
        assert_eq!(r.tools[0].count, 3);
        let single = build(&scene_of(vec![], vec![path(1, &[(30.0, 40.0)])]), 1, &cfg);
        assert_eq!(single.route_mm, 0.0);
        assert_eq!(single.seconds, 16);
        let nothing = build(&scene_of(vec![], vec![path(1, &[])]), 1, &cfg);
        assert_eq!((nothing.seconds, nothing.route_mm), (15, 0.0));
        assert!(!nothing.warnings.contains(&Warn::Empty));
        // the points the machine has reached do not change the report
        let mut done = three.clone();
        for p in &mut done.paths {
            p.reached = 99;
        }
        assert_eq!(build(&done, 1, &cfg), build(&three, 1, &cfg));
    }

    #[test]
    fn the_table_includes_its_edge_and_the_warnings_come_in_a_fixed_order() {
        let s = scene_of(
            vec![
                hole(495.0, 325.0, 1.0, 1),
                hole(495.001, 325.0, 1.0, 1),
                hole(-0.001, 0.0, 1.0, 1),
                hole(0.0, 0.0, 1.0, 1),
            ],
            vec![
                path(1, &[(0.0, 0.0), (496.0, 0.0), (497.0, 0.0)]),
                path(1, &[(495.0, 325.0)]),
            ],
        );
        let r = build(&s, 1, &Config::defaults());
        assert_eq!(r.warnings, vec![Warn::Outside { holes: 2, paths: 1 }]);
        let tiny = Config {
            long_job_min: 0.01,
            ..Config::defaults()
        };
        assert_eq!(
            build(&s, 1, &tiny).warnings,
            vec![
                Warn::Outside { holes: 2, paths: 1 },
                Warn::Long { limit_min: 0 }
            ]
        );
        // a job that takes exactly the limit is not a long one; one second more is
        let inside = scene_of(vec![hole(10.0, 10.0, 1.0, 1)], vec![]);
        let limit = |tool_change_s: f64, long_job_min: f64| Config {
            tool_change_s,
            drill_cycle_s: 1.0,
            rapid_mm_s: 1e9,
            long_job_min,
            ..Config::defaults()
        };
        let at = build(&inside, 1, &limit(59.0, 1.0));
        assert_eq!(at.seconds, 60);
        assert!(at.warnings.is_empty());
        let over = build(&inside, 1, &limit(60.0, 1.0));
        assert_eq!(over.seconds, 61);
        assert_eq!(over.warnings, vec![Warn::Long { limit_min: 1 }]);
        // the minutes of the warning are rounded
        let rounded = build(&inside, 1, &limit(100.0, 1.6));
        assert_eq!(rounded.warnings, vec![Warn::Long { limit_min: 2 }]);
        // an empty job is empty and never long
        let empty = build(&scene_of(vec![], vec![]), 1, &tiny);
        assert_eq!(empty.warnings, vec![Warn::Empty]);
    }

    #[test]
    fn a_panel_is_the_work_and_the_gap_apart_and_a_count_of_zero_is_one() {
        let work = scene_of(
            vec![],
            vec![path(1, &[(0.0, 0.0), (40.0, 0.0), (40.0, 10.0)])],
        );
        let two = panel(&work, 2, 1, 5.0);
        assert_eq!(two.paths.len(), 2);
        assert_eq!(two.paths[1].points[0], (45.0, 0.0));
        assert_eq!(two.paths[1].points[2], (85.0, 10.0));
        let high = panel(&work, 1, 2, 5.0);
        assert_eq!(high.paths[1].points[0], (0.0, 15.0));
        assert_eq!(panel(&work, 0, 0, 5.0), panel(&work, 1, 1, 5.0));
        assert_eq!(panel(&work, 0, 3, 5.0).paths.len(), 3);
        // without a gap the next piece starts where the work ends
        let pair = scene_of(
            vec![hole(10.0, 20.0, 1.0, 1), hole(110.0, 20.0, 1.0, 1)],
            vec![],
        );
        let mut xs: Vec<f64> = panel(&pair, 2, 1, 0.0)
            .holes
            .iter()
            .map(|h| h.x_mm)
            .collect();
        xs.sort_by(f64::total_cmp);
        assert_eq!(xs, vec![10.0, 110.0, 110.0, 210.0]);
        // one hole has no size: the pieces are the gap apart
        let single = scene_of(vec![hole(1.0, 1.0, 1.0, 1)], vec![]);
        let xs: Vec<f64> = panel(&single, 3, 1, 5.0)
            .holes
            .iter()
            .map(|h| h.x_mm)
            .collect();
        assert_eq!(xs, vec![1.0, 6.0, 11.0]);
        assert_eq!(panel(&pair, 7, 9, 1.0).holes.len(), 126);
    }

    #[test]
    fn the_lines_of_a_report_have_one_form_for_each_part() {
        let w = |k: &str| word(Lang::En, k);
        let r = Report {
            pieces: 2,
            holes: 7,
            paths: 3,
            tools: vec![
                ToolLine {
                    tool: 5,
                    drill: true,
                    diameter_mm: 0.35,
                    count: 7,
                },
                ToolLine {
                    tool: 255,
                    drill: false,
                    diameter_mm: 0.0,
                    count: 3,
                },
            ],
            route_mm: 12.34,
            seconds: 3661,
            warnings: vec![
                Warn::Empty,
                Warn::Outside { holes: 2, paths: 1 },
                Warn::Long { limit_min: 90 },
            ],
        };
        let l = lines(&r, Lang::En, &["a note".to_string()]);
        let expected = vec![
            w("Title"),
            format!("{}: 2", w("Pieces")),
            format!("{}: 7 (3 {})", w("Holes"), w("PerPiece")),
            format!("{}: 3", w("Paths")),
            format!("{}: 2", w("Tools")),
            "  T05  0.35 mm  x7".to_string(),
            "  T255  x3".to_string(),
            format!("{}: 12.3 mm", w("Length")),
            format!("{}: 1:01:01", w("Time")),
            format!("{}:", w("Warnings")),
            format!("  - {}", w("WarnEmpty")),
            format!(
                "  - {} ({}: 2, {}: 1)",
                w("WarnOutside"),
                w("Holes"),
                w("Paths")
            ),
            format!("  - {} 1:30:00", w("WarnLong")),
            "  - a note".to_string(),
        ];
        assert_eq!(l, expected);
        assert_eq!(
            text(&r, Lang::En, &["a note".to_string()]),
            expected.join("\n")
        );
        // no pieces: the holes are not divided; a part is not a whole piece
        let none = Report {
            pieces: 0,
            holes: 6,
            tools: vec![],
            warnings: vec![],
            ..r.clone()
        };
        assert_eq!(
            lines(&none, Lang::En, &[])[2],
            format!("{}: 6 (6 {})", w("Holes"), w("PerPiece"))
        );
        let odd = Report {
            pieces: 2,
            holes: 7,
            tools: vec![],
            warnings: vec![],
            ..r.clone()
        };
        assert_eq!(
            lines(&odd, Lang::En, &[])[2],
            format!("{}: 7 (3 {})", w("Holes"), w("PerPiece"))
        );
        // a job with no warning says so on one line, a note alone is a warning
        let quiet = lines(&odd, Lang::En, &[]);
        assert_eq!(
            quiet[quiet.len() - 1],
            format!("{}: {}", w("Warnings"), w("None"))
        );
        let noted = lines(&odd, Lang::En, &["x".to_string()]);
        assert_eq!(noted[noted.len() - 2], format!("{}:", w("Warnings")));
        for (seconds, clock) in [
            (0u64, "0:00"),
            (59, "0:59"),
            (60, "1:00"),
            (3599, "59:59"),
            (3600, "1:00:00"),
            (86_399, "23:59:59"),
            (360_000, "100:00:00"),
        ] {
            let timed = Report {
                seconds,
                ..odd.clone()
            };
            assert_eq!(
                lines(&timed, Lang::En, &[])[6],
                format!("{}: {clock}", w("Time")),
                "{seconds}"
            );
        }
    }

    #[test]
    fn the_words_of_english_are_fixed_and_a_word_that_is_not_there_is_its_own_name() {
        let expected = [
            ("Title", "Work report"),
            ("Pieces", "Pieces"),
            ("Holes", "Holes"),
            ("PerPiece", "per piece"),
            ("Tools", "Tools"),
            ("Paths", "Paths"),
            ("Length", "Routed length"),
            ("Time", "Time (estimate)"),
            ("Warnings", "Warnings"),
            ("None", "none"),
        ];
        for (key, value) in expected {
            assert_eq!(word(Lang::En, key), value);
        }
        for l in Lang::ALL {
            assert_eq!(word(l, "NoSuchWord"), "NoSuchWord");
            assert_eq!(word(l, ""), "");
            for key in KEYS {
                let w = word(l, key);
                assert!(!w.is_empty() && !w.contains('\t'), "{} {key}", l.code());
            }
        }
        let mut keys = KEYS.to_vec();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), 15);
        let r = build(
            &scene_of(vec![hole(30.0, 40.0, 1.0, 1)], vec![]),
            1,
            &Config::defaults(),
        );
        let t = text(&r, Lang::En, &[]);
        assert_eq!(t, text(&r, Lang::En, &[]));
        assert!(!t.ends_with('\n') && !t.contains('\r'));
        assert_eq!(t.lines().count(), lines(&r, Lang::En, &[]).len());
    }

    #[test]
    fn a_big_random_job_gives_a_report_that_agrees_with_its_scene() {
        let mut g = Lcg(2024);
        let mut holes = Vec::new();
        for _ in 0..2000 {
            let (x, y) = (
                (g.step() % 6000) as f64 / 10.0,
                (g.step() % 4000) as f64 / 10.0,
            );
            holes.push(hole(
                x,
                y,
                0.5 + (g.step() % 10) as f64 / 10.0,
                1 + (g.step() % 8) as u8,
            ));
        }
        let mut paths = Vec::new();
        for _ in 0..50 {
            let n = 1 + (g.step() % 6) as usize;
            let points: Vec<(f64, f64)> = (0..n)
                .map(|_| {
                    (
                        (g.step() % 5000) as f64 / 10.0,
                        (g.step() % 3000) as f64 / 10.0,
                    )
                })
                .collect();
            paths.push(path(1 + (g.step() % 4) as u8, &points));
        }
        let s = scene_of(holes, paths);
        let cfg = Config::defaults();
        let r = build(&s, 1, &cfg);
        assert_eq!(r, build(&s, 1, &cfg));
        let drilled: usize = r.tools.iter().filter(|t| t.drill).map(|t| t.count).sum();
        let cut: usize = r.tools.iter().filter(|t| !t.drill).map(|t| t.count).sum();
        assert_eq!((drilled, cut), (2000, 50));
        let mut seen: Vec<(u8, bool)> = r.tools.iter().map(|t| (t.tool, t.drill)).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), r.tools.len());
        let length: f64 = s
            .paths
            .iter()
            .map(|p| {
                p.points
                    .windows(2)
                    .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
                    .sum::<f64>()
            })
            .sum();
        assert!((r.route_mm - length).abs() < 1e-6 * length.max(1.0));
        let floor = r.tools.len() as f64 * cfg.tool_change_s
            + 2000.0 * cfg.drill_cycle_s
            + length / cfg.route_feed_mm_s;
        assert!(r.seconds as f64 + 0.5 >= floor, "{} {floor}", r.seconds);
        let (out_holes, out_paths) = s.outside_table();
        let outside = r.warnings.iter().find_map(|w| match w {
            Warn::Outside { holes, paths } => Some((*holes, *paths)),
            _ => None,
        });
        assert_eq!(
            outside.unwrap_or((0, 0)),
            (out_holes.len(), out_paths.len())
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// reportcmd
// ---------------------------------------------------------------------------------------------------------------------------------

mod reportcmd_tests {
    use super::scratch;
    use eh3d_bungard_gui::cli::Options;
    use eh3d_bungard_gui::report::{word, Config};
    use eh3d_bungard_gui::reportcmd::run;
    use eh3d_view::texts::Lang;
    use std::path::Path;

    fn ncd(holes: usize) -> String {
        let mut t = String::from("%\nT10\n");
        for i in 0..holes {
            t.push_str(&format!("X{}Y1000\n", 1000 + (i % 200) * 10));
        }
        t.push_str("M30\n");
        t
    }

    fn put(dir: &Path, name: &str, text: &str) -> std::path::PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }

    #[test]
    fn the_files_given_without_an_option_are_told_apart_by_their_extension_and_the_first_one_counts(
    ) {
        let d = scratch("sort");
        let a = put(&d, "a.NCD", &ncd(1));
        let b = put(&d, "b.drl", &ncd(5));
        let c = put(&d, "c.ncd", &ncd(2));
        let cfg = Config::defaults();
        let o = Options {
            open: vec![a.clone()],
            ..Default::default()
        };
        assert!(run(&o, &cfg).unwrap().contains("Holes: 1 (1 per piece)"));
        let o = Options {
            open: vec![a.clone(), b.clone()],
            ..Default::default()
        };
        assert!(run(&o, &cfg).unwrap().contains("Holes: 1 (1 per piece)"));
        let o = Options {
            open: vec![b, a.clone()],
            ..Default::default()
        };
        assert!(run(&o, &cfg).unwrap().contains("Holes: 5 (5 per piece)"));
        // the file of the option wins over the one without
        let o = Options {
            drill: Some(c),
            open: vec![a],
            ..Default::default()
        };
        assert!(run(&o, &cfg).unwrap().contains("Holes: 2 (2 per piece)"));
        // a routing file by any of its three extensions
        let plt = "IN;SP1;PU0,0;PD400,0;";
        for name in ["r.plt", "r.HPGL", "r.hpg"] {
            let o = Options {
                open: vec![put(&d, name, plt)],
                ..Default::default()
            };
            let t = run(&o, &cfg).unwrap();
            assert!(t.contains("Holes: 0 (0 per piece)"), "{t}");
            assert!(t.contains("Paths: 1"), "{t}");
            assert!(t.contains("Routed length: 10.0 mm"), "{t}");
        }
    }

    #[test]
    fn what_cannot_be_reported_is_said_with_the_name_of_the_file() {
        let d = scratch("errors");
        let cfg = Config::defaults();
        let nothing = run(&Options::default(), &cfg).unwrap_err();
        assert_eq!(
            nothing,
            "--report needs a drilling file (--drill FILE) or a routing file (--route FILE)"
        );
        let zip = put(&d, "job.zip", "not read");
        let o = Options {
            open: vec![zip],
            ..Default::default()
        };
        assert_eq!(run(&o, &cfg).unwrap_err(), nothing);
        let missing = d.join("none.plt");
        let o = Options {
            route: Some(missing.clone()),
            ..Default::default()
        };
        let e = run(&o, &cfg).unwrap_err();
        assert!(e.starts_with(&missing.display().to_string()), "{e}");
        let bad = put(&d, "bad.ncd", "%\nX1000Y1000\n");
        let o = Options {
            drill: Some(bad.clone()),
            ..Default::default()
        };
        assert_eq!(
            run(&o, &cfg).unwrap_err(),
            format!("{}: not a drilling file", bad.display())
        );
    }

    #[test]
    fn a_panel_multiplies_the_pieces_and_a_count_of_zero_is_one_piece() {
        let d = scratch("panel");
        let cfg = Config::defaults();
        let one = Options {
            drill: Some(put(&d, "a.ncd", &ncd(1))),
            panel: Some((0, 0)),
            ..Default::default()
        };
        assert!(run(&one, &cfg).unwrap().contains("Pieces: 1"));
        let six = Options {
            panel: Some((3, 2)),
            ..one
        };
        let t = run(&six, &cfg).unwrap();
        assert!(t.contains("Pieces: 6"), "{t}");
        assert!(t.contains("Holes: 6 (1 per piece)"), "{t}");
        assert!(t.ends_with("Warnings: none"), "{t}");
        let routed = Options {
            route: Some(put(&d, "r.plt", "IN;SP1;PU0,0;PD400,0;")),
            panel: Some((3, 2)),
            ..Default::default()
        };
        let t = run(&routed, &cfg).unwrap();
        assert!(t.contains("Routed length: 60.0 mm"), "{t}");
        assert!(t.contains("Paths: 6"), "{t}");
    }

    #[test]
    fn the_language_and_the_figures_of_the_estimate_reach_the_text() {
        let d = scratch("lang");
        let drill = put(&d, "a.ncd", &ncd(1));
        let german = Options {
            drill: Some(drill.clone()),
            lang: Some(Lang::De),
            ..Default::default()
        };
        let t = run(&german, &Config::defaults()).unwrap();
        assert_eq!(t.lines().next().unwrap(), word(Lang::De, "Title"));
        assert!(
            t.contains(&format!("{}: 1", word(Lang::De, "Pieces"))),
            "{t}"
        );
        let plain = Options {
            drill: Some(drill),
            ..Default::default()
        };
        let slow = Config {
            tool_change_s: 10_000.0,
            long_job_min: 1.0,
            ..Config::defaults()
        };
        let t = run(&plain, &slow).unwrap();
        assert!(t.contains(&word(Lang::En, "WarnLong")), "{t}");
        // a wide gap puts the second piece off the table
        let wide = Config {
            panel_gap_mm: 1000.0,
            ..Config::defaults()
        };
        let two = Options {
            panel: Some((2, 1)),
            ..plain
        };
        let t = run(&two, &wide).unwrap();
        assert!(t.contains(&word(Lang::En, "WarnOutside")), "{t}");
    }

    #[test]
    fn a_file_with_windows_line_ends_and_a_big_one_are_read_whole() {
        let d = scratch("big");
        let cfg = Config::defaults();
        let crlf = Options {
            drill: Some(put(&d, "crlf.ncd", &ncd(3).replace('\n', "\r\n"))),
            ..Default::default()
        };
        assert!(run(&crlf, &cfg).unwrap().contains("Holes: 3 (3 per piece)"));
        let big = Options {
            drill: Some(put(&d, "big.ncd", &ncd(5000))),
            ..Default::default()
        };
        assert!(run(&big, &cfg)
            .unwrap()
            .contains("Holes: 5000 (5000 per piece)"));
        let bare = Options {
            drill: Some(put(&d, "bare.ncd", ncd(2).trim_end())),
            ..Default::default()
        };
        assert!(run(&bare, &cfg).unwrap().contains("Holes: 2 (2 per piece)"));
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// servercmd
// ---------------------------------------------------------------------------------------------------------------------------------

mod servercmd_tests {
    use super::scratch;
    use eh3d_bungard_gui::servercmd::{
        machine_file, machines, machines_from, settings_from_text, settings_of, status, Failure,
    };
    use eh3d_serverclient::{Client, Lang, Reply, Request, Settings, Transport, TransportError};
    use std::path::Path;

    /// A server that gives one answer to everything, or none.
    struct Answer {
        status: u16,
        body: String,
        down: bool,
    }

    impl Transport for Answer {
        fn send(&self, _request: &Request) -> Result<Reply, TransportError> {
            if self.down {
                Err(TransportError::Connect("nope".into()))
            } else {
                Ok(Reply {
                    status: self.status,
                    body: self.body.clone(),
                })
            }
        }
    }

    fn fleet_of(status: u16, body: &str) -> Result<String, String> {
        let settings = Settings::from_text(
            "server_address = 127.0.0.1:1\nserver_token = t\nserver_retries = 0\n",
        )
        .unwrap();
        let answer = Answer {
            status,
            body: body.to_string(),
            down: false,
        };
        let mut client = Client::new(settings, Box::new(answer));
        machines_from(&mut client, Lang::En)
    }

    #[test]
    fn the_machine_file_is_read_with_comments_in_any_case_and_a_bad_value_names_its_line() {
        let s = settings_from_text(
            "; old style comment\n  SERVER_ADDRESS = 10.0.0.5:18200   # the server\nserver_machine =  bench \nserver_token = tok\nconnection = server\n",
        )
        .unwrap();
        assert_eq!(s.server_address, "10.0.0.5:18200");
        assert_eq!(s.server_machine, "bench");
        assert!(s.to_text(true).contains("server_token = tok"));
        assert!(!s.to_text(false).contains("tok\n"));
        // a # without a space before it is part of the value
        assert_eq!(
            settings_from_text("server_address = a:1#x\n")
                .unwrap()
                .server_address,
            "a:1#x"
        );
        assert_eq!(
            settings_from_text("\n\n# c\nserver_address = a:1\nserver_retries = 6\n").unwrap_err(),
            "line 5: server_retries: must be a whole number from 0 to 5"
        );
        assert_eq!(
            settings_from_text("server_address = a b\n").unwrap_err(),
            "line 1: server_address: must not hold a space or a control character"
        );
        let no_server = "the machine file has no server_address: there is no server to ask";
        assert_eq!(settings_from_text("").unwrap_err(), no_server);
        assert_eq!(
            settings_from_text("server_address =   \n").unwrap_err(),
            no_server
        );
        assert_eq!(
            settings_from_text("server_timeout_s = 120\nserver_address = a:1\n")
                .unwrap()
                .server_timeout_s,
            120
        );
        assert!(settings_from_text("server_address = a:1\nserver_timeout_s = 121\n").is_err());
    }

    #[test]
    fn the_machine_file_is_the_one_named_and_a_file_that_cannot_be_used_is_named_in_the_error() {
        let d = scratch("file");
        let file = d.join("machine.cfg");
        assert_eq!(machine_file(Some(&file)), file);
        std::fs::write(&file, "server_address = h:1\n").unwrap();
        assert_eq!(settings_of(Some(&file)).unwrap().server_address, "h:1");
        let missing = d.join("none.cfg");
        let e = settings_of(Some(&missing)).unwrap_err();
        assert!(e.starts_with(&missing.display().to_string()), "{e}");
        std::fs::write(&file, "server_address = h:1\nserver_mdns = maybe\n").unwrap();
        let e = settings_of(Some(&file)).unwrap_err();
        assert!(e.starts_with(&file.display().to_string()), "{e}");
        assert!(e.contains("line 2") && e.contains("server_mdns"), "{e}");
        std::fs::write(&file, "").unwrap();
        assert!(settings_of(Some(&file))
            .unwrap_err()
            .contains("no server_address"));
    }

    #[test]
    fn a_command_without_a_server_to_ask_leaves_with_the_code_two_before_touching_the_network() {
        let d = scratch("code");
        let none = d.join("no-server.cfg");
        std::fs::write(&none, "connection = demo\n").unwrap();
        let missing = d.join("missing.cfg");
        type Command = fn(Option<&Path>) -> Result<String, Failure>;
        let commands: [Command; 2] = [status, machines];
        for command in commands {
            let f = command(Some(&none)).unwrap_err();
            assert_eq!(f.code, 2);
            assert!(f.text.contains("no server_address"), "{}", f.text);
            assert!(
                f.text.starts_with(&none.display().to_string()),
                "{}",
                f.text
            );
            let f = command(Some(&missing)).unwrap_err();
            assert_eq!(f.code, 2);
            assert!(
                f.text.starts_with(&missing.display().to_string()),
                "{}",
                f.text
            );
        }
    }

    #[test]
    fn the_machines_are_counted_by_their_state_and_a_word_that_is_not_known_is_free() {
        let empty = fleet_of(200, r#"{"machines":[]}"#).unwrap();
        assert!(
            empty.ends_with("0 machines: 0 free, 0 busy, 0 waiting for the operator; 0 stations\n")
        );
        let four = fleet_of(
            200,
            r#"{"stations":[{"id":"bench","machines":["a","b","c"]}],"machines":[
                {"id":"a","station":"bench","state":"idle"},
                {"id":"b","station":"bench","state":"busy"},
                {"id":"c","station":"bench","state":"waiting"},
                {"id":"d","station":"bench","state":"paused"}],"extra":[1,2]}"#,
        )
        .unwrap();
        assert!(
            four.ends_with("4 machines: 2 free, 1 busy, 1 waiting for the operator; 1 stations\n"),
            "{four}"
        );
        for id in ["a", "b", "c", "d"] {
            assert!(four.contains(id), "{four}");
        }
        // a station without a name is not counted
        let named = fleet_of(
            200,
            r#"{"stations":[{"id":"","machines":[]},{"id":"x","machines":[]}],"machines":[]}"#,
        )
        .unwrap();
        assert!(named.ends_with("; 1 stations\n"), "{named}");
    }

    #[test]
    fn an_answer_that_is_a_refusal_or_not_a_fleet_is_one_line_that_says_why() {
        assert_eq!(fleet_of(404, "{}").unwrap_err(), "the server answered 404");
        assert_eq!(fleet_of(401, "{}").unwrap_err(), "the server answered 401");
        assert_eq!(
            fleet_of(409, r#"{"error":"busy","detail":"a job is running"}"#).unwrap_err(),
            "busy: a job is running"
        );
        assert_eq!(fleet_of(409, r#"{"error":"busy"}"#).unwrap_err(), "busy");
        assert!(fleet_of(200, "not json").unwrap_err().contains("not JSON"));
        for body in ["{}", "[]", r#"{"machines":3}"#] {
            assert_eq!(
                fleet_of(200, body).unwrap_err(),
                "the answer has no list of machines",
                "{body}"
            );
        }
        assert_eq!(
            fleet_of(200, r#"{"machines":[{"name":"x"}]}"#).unwrap_err(),
            "a machine has no id"
        );
        let settings = Settings::from_text(
            "server_address = 127.0.0.1:1\nserver_token = t\nserver_retries = 0\n",
        )
        .unwrap();
        let down = Answer {
            status: 0,
            body: String::new(),
            down: true,
        };
        let mut client = Client::new(settings, Box::new(down));
        let e = machines_from(&mut client, Lang::En).unwrap_err();
        assert!(e.contains("cannot connect: nope"), "{e}");
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// server_wire
// ---------------------------------------------------------------------------------------------------------------------------------

mod server_wire_tests {
    use eh3d_bungard_gui::server_wire::{exchange, explain, refusal, status_text, Link};
    use eh3d_serverclient::sim::{Fault, SimServer};
    use eh3d_serverclient::{Client, ClientError, Lang, TransportError};

    fn link() -> Link {
        Link {
            address: "127.0.0.1:18200".into(),
            token: "program-token".into(),
            machine: "bench".into(),
            timeout_s: 15,
            retries: 2,
            mdns: false,
            max_answer_mib: 8,
        }
    }

    fn client(sim: &SimServer, l: &Link) -> Client {
        Client::new(l.settings().unwrap(), Box::new(sim.clone()))
            .with_sleeper(Box::new(sim.sleeper()))
    }

    #[test]
    fn each_limit_of_the_link_takes_exactly_its_range_and_names_itself_when_it_does_not() {
        for t in [1u64, 15, 120] {
            let l = Link {
                timeout_s: t,
                ..link()
            };
            assert_eq!(l.settings().unwrap().server_timeout_s, t);
        }
        for t in [0u64, 121] {
            let l = Link {
                timeout_s: t,
                ..link()
            };
            assert_eq!(
                l.settings().unwrap_err(),
                "server_timeout_s: must be a whole number from 1 to 120"
            );
        }
        for r in [0u32, 5] {
            let l = Link {
                retries: r,
                ..link()
            };
            assert_eq!(l.settings().unwrap().server_retries, r);
        }
        let l = Link {
            retries: 6,
            ..link()
        };
        assert_eq!(
            l.settings().unwrap_err(),
            "server_retries: must be a whole number from 0 to 5"
        );
        for m in [1u64, 256] {
            let l = Link {
                max_answer_mib: m,
                ..link()
            };
            assert_eq!(l.settings().unwrap().server_max_answer_mib, m);
        }
        for m in [0u64, 257] {
            let l = Link {
                max_answer_mib: m,
                ..link()
            };
            assert_eq!(
                l.settings().unwrap_err(),
                "server_max_answer_mib: must be a whole number from 1 to 256"
            );
        }
        let on = Link {
            mdns: true,
            machine: "  bench  ".into(),
            ..link()
        };
        let s = on.settings().unwrap();
        assert!(s.server_mdns);
        assert_eq!(s.server_machine, "bench");
        let blank = Link {
            address: String::new(),
            ..link()
        };
        assert!(!blank.settings().unwrap().is_configured());
        for bad in ["a\tb", "a\nb", "a b"] {
            let l = Link {
                address: bad.into(),
                ..link()
            };
            assert_eq!(
                l.settings().unwrap_err(),
                "server_address: must not hold a space or a control character"
            );
        }
        let l = Link {
            token: "a b".into(),
            ..link()
        };
        assert_eq!(
            l.settings().unwrap_err(),
            "server_token: must not hold a space or a control character"
        );
        assert_eq!(link(), link());
    }

    #[test]
    fn a_refusal_is_given_back_as_the_status_and_a_json_body_that_reads_the_same_way() {
        assert_eq!(
            refusal(&ClientError::Unauthorized { code: None }),
            Some((401, r#"{"detail":""}"#.to_string()))
        );
        assert_eq!(
            refusal(&ClientError::Forbidden {
                code: Some("forbidden".into())
            }),
            Some((403, r#"{"error":"forbidden","detail":""}"#.to_string()))
        );
        assert_eq!(
            refusal(&ClientError::Server {
                status: 500,
                code: None,
                detail: "x".into()
            }),
            Some((500, r#"{"detail":"x"}"#.to_string()))
        );
        let detail = "tab\there\nnew \"q\" \\ é 日本";
        let (status, body) = refusal(&ClientError::Server {
            status: 422,
            code: Some("bad \"code\"".into()),
            detail: detail.into(),
        })
        .unwrap();
        assert_eq!(status, 422);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(v["error"], "bad \"code\"");
        assert_eq!(v["detail"], detail);
        for e in [
            ClientError::Config("c".into()),
            ClientError::BadReply("b".into()),
            ClientError::Unreachable {
                error: TransportError::Timeout,
                attempts: 3,
            },
        ] {
            assert!(refusal(&e).is_none(), "{e:?}");
        }
    }

    #[test]
    fn what_a_failure_says_is_its_text_and_the_detail_when_the_text_does_not_hold_it() {
        let known = |detail: &str| ClientError::Server {
            status: 422,
            code: Some("bad_gcode".into()),
            detail: detail.into(),
        };
        for lang in Lang::ALL {
            let with = known("line 3: no");
            assert_eq!(
                explain(&with, lang),
                format!("{}: line 3: no", with.text(lang))
            );
            let without = known("");
            assert_eq!(explain(&without, lang), without.text(lang));
            let unknown = ClientError::Server {
                status: 409,
                code: Some("zz_new_code".into()),
                detail: "because".into(),
            };
            assert_eq!(explain(&unknown, lang), "zz_new_code: because");
            let plain = ClientError::Server {
                status: 500,
                code: None,
                detail: "boom".into(),
            };
            assert_eq!(explain(&plain, lang), "500 boom");
            assert_eq!(explain(&ClientError::Config("x".into()), lang), "x");
            assert_eq!(explain(&ClientError::BadReply("odd".into()), lang), "odd");
        }
        let down = ClientError::Unreachable {
            error: TransportError::Timeout,
            attempts: 3,
        };
        assert!(explain(&down, Lang::En).contains("timed out"));
        assert_ne!(explain(&down, Lang::En), explain(&down, Lang::Es));
        let no_login = ClientError::Unauthorized { code: None };
        assert_ne!(explain(&no_login, Lang::En), explain(&no_login, Lang::Ja));
    }

    #[test]
    fn a_request_is_sent_again_exactly_as_many_times_as_the_retries_allow() {
        let with_retries = |retries: u32| Link { retries, ..link() };
        // one retry: the second try gets the answer
        let sim = SimServer::new();
        sim.script(vec![Fault::NoConnection]);
        let mut c = client(&sim, &with_retries(1));
        let (status, _) = exchange(&mut c, "GET", "/api/v1/echo", None, Lang::En).unwrap();
        assert_eq!(status, 200);
        assert_eq!(sim.pauses().len(), 1);
        // none: the first failure is the answer
        let sim = SimServer::new();
        sim.script(vec![Fault::NoConnection]);
        let mut c = client(&sim, &with_retries(0));
        assert!(exchange(&mut c, "GET", "/api/v1/echo", None, Lang::En).is_err());
        assert!(sim.pauses().is_empty());
        assert_eq!(sim.request_count(), 0);
        // one retry that fails too
        let sim = SimServer::new();
        sim.script(vec![Fault::NoConnection; 2]);
        let mut c = client(&sim, &with_retries(1));
        assert!(exchange(&mut c, "GET", "/api/v1/echo", None, Lang::En).is_err());
        assert_eq!(sim.pauses().len(), 1);
        // a refusal of a write is an answer and is not sent again
        let sim = SimServer::new();
        sim.script(vec![Fault::Status(409, "busy")]);
        let mut c = client(&sim, &with_retries(2));
        let (status, body) =
            exchange(&mut c, "POST", "/api/v1/echo", Some("{}"), Lang::En).unwrap();
        assert_eq!(status, 409);
        assert!(body.contains("\"error\":\"busy\""), "{body}");
        assert!(sim.pauses().is_empty());
    }

    #[test]
    fn the_status_is_one_line_about_the_server_and_then_the_lines_of_the_metrics() {
        let sim = SimServer::new();
        let mut c = client(&sim, &link());
        let text = status_text(&mut c, Lang::En).unwrap();
        let mut it = text.lines();
        let first = it.next().unwrap();
        assert!(
            first.starts_with("eh3d-server ") && first.ends_with(')'),
            "{first}"
        );
        assert!(first.contains(", api "), "{first}");
        assert!(it.next().unwrap().starts_with("requests = "));
        assert_eq!(text.lines().count(), 14, "{text}");
        assert!(text.ends_with('\n'));
        // a server that cannot be reached is told in the language asked for
        let sim = SimServer::new();
        sim.script(vec![Fault::NoConnection; 3]);
        let mut c = client(&sim, &link());
        let en = status_text(&mut c, Lang::En).unwrap_err();
        let sim = SimServer::new();
        sim.script(vec![Fault::NoConnection; 3]);
        let mut c = client(&sim, &link());
        let es = status_text(&mut c, Lang::Es).unwrap_err();
        assert!(!en.is_empty() && !es.is_empty());
        assert_ne!(en, es);
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// cli
// ---------------------------------------------------------------------------------------------------------------------------------

mod cli_tests {
    use eh3d_bungard_gui::cli::{
        usage, Options, COMBOS, MAX_SIDE, MAX_TAB, MAX_TABS, MAX_WAIT, MODALS,
    };
    use eh3d_view::texts::Lang;
    use eh3d_view::theme::ThemeId;
    use std::path::PathBuf;

    fn parse(args: &[&str]) -> Result<Options, String> {
        Options::parse(args.iter().map(|s| String::from(*s)))
    }

    #[test]
    fn the_switches_are_off_until_they_are_given_and_each_one_sets_only_itself() {
        assert_eq!(Options::default().wait, 0);
        assert_eq!(parse(&[]).unwrap().wait, 6);
        let all = parse(&[
            "--start",
            "--check",
            "--measure",
            "--select",
            "--auto-ok",
            "--report",
            "--server-status",
            "--machines",
            "--help",
            "--version",
        ])
        .unwrap();
        assert_eq!(
            all,
            Options {
                wait: 6,
                start: true,
                check: true,
                measure: true,
                select: true,
                auto_ok: true,
                report: true,
                server_status: true,
                machines: true,
                help: true,
                version: true,
                ..Default::default()
            }
        );
        for arg in [
            "--start",
            "--check",
            "--measure",
            "--select",
            "--auto-ok",
            "--report",
            "--server-status",
            "--machines",
            "--help",
            "-h",
            "--version",
            "-V",
        ] {
            let o = parse(&[arg]).unwrap();
            let on = [
                o.start,
                o.check,
                o.measure,
                o.select,
                o.auto_ok,
                o.report,
                o.server_status,
                o.machines,
                o.help,
                o.version,
            ];
            assert_eq!(on.iter().filter(|b| **b).count(), 1, "{arg}");
            assert!(o.open.is_empty(), "{arg}");
        }
        // the two short forms are not "given twice" with the long ones
        assert!(parse(&["--help", "-h"]).unwrap().help);
    }

    #[test]
    fn every_option_that_takes_a_value_says_so_when_the_value_is_missing() {
        for name in [
            "--lang",
            "--theme",
            "--drill",
            "--route",
            "--zip",
            "--data",
            "--machine",
            "--connection",
            "--screen",
            "--shot",
            "--wait",
            "--modal",
            "--combo",
            "--fleet",
            "--panel",
            "--tabs",
            "--zoom",
            "--yaw",
            "--tab",
            "--hole",
            "--panes",
            "--size",
        ] {
            assert_eq!(parse(&[name]).unwrap_err(), format!("{name} needs a value"));
            assert_eq!(
                parse(&["--start", "a.ncd", name]).unwrap_err(),
                format!("{name} needs a value")
            );
        }
    }

    #[test]
    fn the_numbers_are_read_in_their_plain_forms_and_nothing_else() {
        assert_eq!(parse(&["--wait", "+5"]).unwrap().wait, 5);
        assert_eq!(parse(&["--wait", "007"]).unwrap().wait, 7);
        for bad in ["5 ", " 5", "", "5.0", "1e2"] {
            assert!(
                parse(&["--wait", bad])
                    .unwrap_err()
                    .starts_with("--wait needs a number from 0 to 3600"),
                "{bad}"
            );
        }
        assert_eq!(parse(&["--panel", "2, 3"]).unwrap().panel, Some((2, 3)));
        let widest = format!("{},1", u32::MAX);
        assert_eq!(
            parse(&["--panel", &widest]).unwrap().panel,
            Some((u32::MAX, 1))
        );
        assert_eq!(
            parse(&["--panel", "4294967296,1"]).unwrap_err(),
            "--panel needs numbers"
        );
        assert_eq!(
            parse(&["--panel", "2,3,4"]).unwrap_err(),
            "--panel needs numbers"
        );
        assert_eq!(
            parse(&["--panel", "x,2"]).unwrap_err(),
            "--panel needs numbers"
        );
        assert_eq!(
            parse(&["--panel", "2"]).unwrap_err(),
            "--panel needs COLUMNS,ROWS"
        );
        assert_eq!(
            parse(&["--panel", "0,5"]).unwrap_err(),
            "--panel needs COLUMNS,ROWS of 1 or more"
        );
        for (text, expect) in [
            ("400x300", Some((400.0, 300.0))),
            ("400 x 300", Some((400.0, 300.0))),
            ("8192x8192", Some((8192.0, 8192.0))),
            ("1600.5x900.25", Some((1600.5, 900.25))),
            ("399.9x300", None),
            ("400x299.9", None),
            ("8192.5x300", None),
            ("400X300", None),
            ("400", None),
            ("400x300x2", None),
            ("x", None),
        ] {
            assert_eq!(
                parse(&["--size", text]).ok().and_then(|o| o.size),
                expect,
                "{text}"
            );
        }
        for (text, ok) in [
            ("1e-30", true),
            (".5", true),
            ("+2", true),
            ("1e40", false),
            ("0.0", false),
            ("-0", false),
            ("-0.0", false),
        ] {
            assert_eq!(parse(&["--zoom", text]).is_ok(), ok, "{text}");
        }
        assert_eq!(parse(&["--zoom", "+2"]).unwrap().zoom, Some(2.0));
        assert_eq!(parse(&["--yaw", "360000"]).unwrap().yaw, Some(360_000.0));
        assert!(parse(&["--yaw", "-0"]).is_ok());
        assert!(parse(&["--yaw", "1e40"]).is_err());
        assert_eq!(parse(&["--hole", "00001"]).unwrap().hole, Some(1));
        assert!(parse(&["--hole", "99999999999999999999999"]).is_err());
        assert!(parse(&["--tab", "255"]).is_err());
        assert!(parse(&["--tab", "256"]).is_err());
        assert_eq!(parse(&["--tabs", "1"]).unwrap().tabs, Some(1));
    }

    #[test]
    fn a_refusal_of_the_command_line_has_one_exact_wording_for_each_option() {
        let wrong = |args: &[&str]| parse(args).unwrap_err();
        assert_eq!(
            wrong(&["--screen", "map"]),
            "--screen must be launcher or viewer, not 'map'"
        );
        assert_eq!(
            wrong(&["--wait", "many"]),
            format!("--wait needs a number from 0 to {MAX_WAIT}, not 'many'")
        );
        assert_eq!(
            wrong(&["--tabs", "21"]),
            format!("--tabs needs a number from 1 to {MAX_TABS}, not '21'")
        );
        assert_eq!(
            wrong(&["--tab", "4"]),
            format!("--tab needs a number from 0 to {MAX_TAB}, not '4'")
        );
        assert_eq!(
            wrong(&["--hole", "0"]),
            "--hole needs a number from 1, not '0'"
        );
        assert_eq!(
            wrong(&["--zoom", "big"]),
            "--zoom needs a number above 0, not 'big'"
        );
        assert_eq!(wrong(&["--yaw", "NaN"]), "--yaw needs a number, not 'NaN'");
        assert_eq!(wrong(&["--panes", "x"]), "--panes needs LEFT,RIGHT");
        assert_eq!(
            wrong(&["--size", "1x1"]),
            format!("--size needs WIDTHxHEIGHT, from 400x300 to {MAX_SIDE}x{MAX_SIDE}")
        );
        assert_eq!(
            wrong(&["--modal", "About"]),
            format!("--modal must be one of {}, not 'About'", MODALS.join(", "))
        );
        assert_eq!(
            wrong(&["--combo", "Theme"]),
            format!("--combo must be one of {}, not 'Theme'", COMBOS.join(", "))
        );
        let codes: Vec<&str> = Lang::ALL.into_iter().map(|l| l.code()).collect();
        assert_eq!(
            wrong(&["--lang", "zz"]),
            format!("unknown language 'zz' (use one of: {})", codes.join(", "))
        );
        let keys: Vec<&str> = ThemeId::ALL.into_iter().map(|t| t.key()).collect();
        assert_eq!(
            wrong(&["--theme", "zz"]),
            format!("unknown theme 'zz' (use one of: {})", keys.join(", "))
        );
        assert_eq!(wrong(&["--fast"]), "unknown option '--fast'");
        assert_eq!(wrong(&["--"]), "unknown option '--'");
        assert_eq!(wrong(&["--start", "--start"]), "--start is given twice");
        // the first mistake is the one that is told
        assert!(wrong(&["--wait", "x", "--lang", "yy"]).starts_with("--wait needs"));
        assert!(wrong(&["--lang", "yy", "--wait", "x"]).starts_with("unknown language"));
        assert_eq!(
            wrong(&["--start", "--start", "--wait", "x"]),
            "--start is given twice"
        );
        // the value of an option is taken even when it is an option too
        assert!(wrong(&["--lang", "--lang"]).starts_with("unknown language '--lang'"));
    }

    #[test]
    fn what_is_not_an_option_is_a_file_in_the_order_it_came_and_values_are_kept_as_given() {
        assert_eq!(parse(&[""]).unwrap().open, vec![PathBuf::from("")]);
        assert_eq!(
            parse(&["-x", "-hh", "-"]).unwrap().open,
            vec![
                PathBuf::from("-x"),
                PathBuf::from("-hh"),
                PathBuf::from("-")
            ]
        );
        let o = parse(&["a b.ncd", "--start", "C:\\dir with space\\x.zip"]).unwrap();
        assert_eq!(
            o.open,
            vec![
                PathBuf::from("a b.ncd"),
                PathBuf::from("C:\\dir with space\\x.zip")
            ]
        );
        let o = parse(&[
            "--data",
            "C:\\a b\\c",
            "--machine",
            "CCD/2 ATC",
            "--shot",
            "x y.png",
            "--fleet",
            "f.json",
        ])
        .unwrap();
        assert_eq!(o.data, Some(PathBuf::from("C:\\a b\\c")));
        assert_eq!(o.machine.as_deref(), Some("CCD/2 ATC"));
        assert_eq!(o.shot, Some(PathBuf::from("x y.png")));
        assert_eq!(o.fleet_file, Some(PathBuf::from("f.json")));
        assert_eq!(
            parse(&["--machine", ""]).unwrap().machine.as_deref(),
            Some("")
        );
        let names: Vec<String> = (0..500).map(|i| format!("job{i}.ncd")).collect();
        let refs: Vec<&str> = names.iter().map(String::as_str).collect();
        let o = parse(&refs).unwrap();
        assert_eq!(o.open.len(), 500);
        assert_eq!(o.open[499], PathBuf::from("job499.ncd"));
        assert_eq!(o.open[0], PathBuf::from("job0.ncd"));
        assert!(parse(&["--panes", "300,500"]).unwrap().panes.is_some());
    }

    #[test]
    fn the_help_text_has_its_fixed_first_and_last_lines_and_names_every_choice() {
        let text = usage();
        assert_eq!(text, usage());
        assert_eq!(
            text.lines().next().unwrap(),
            "bungard-gui: the modern interface for the Bungard machines"
        );
        assert!(text.ends_with(
            "An option that is written wrong, given twice or given a value that makes no sense is an error, never something to ignore."
        ));
        assert!(text.contains(&format!("(0 to {MAX_WAIT}, 6 by default)")));
        assert!(text.contains(&format!("--tabs COUNT (1 to {MAX_TABS})")));
        assert!(text.contains(&format!("--tab N (0 to {MAX_TAB})")));
        assert!(text.contains(&format!("400x300 to {MAX_SIDE}")));
        for w in MODALS.iter().chain(COMBOS.iter()) {
            assert!(text.contains(w), "{w}");
        }
        let codes: Vec<&str> = Lang::ALL.into_iter().map(|l| l.code()).collect();
        let keys: Vec<&str> = ThemeId::ALL.into_iter().map(|t| t.key()).collect();
        let lang_line = text
            .lines()
            .find(|l| l.starts_with("--lang takes"))
            .unwrap();
        assert_eq!(
            lang_line,
            format!("--lang takes one of: {}", codes.join(", "))
        );
        let theme_line = text
            .lines()
            .find(|l| l.starts_with("--theme takes"))
            .unwrap();
        assert_eq!(
            theme_line,
            format!("--theme takes one of: {}", keys.join(", "))
        );
    }

    #[test]
    fn the_limits_and_the_lists_of_the_window_words_are_what_the_help_promises() {
        assert_eq!(MAX_WAIT, 3600);
        assert_eq!(MAX_TABS, 20);
        assert_eq!(MAX_TAB, 3);
        assert_eq!(MAX_SIDE, 8192.0);
        assert_eq!(MODALS.len(), 17);
        assert_eq!(COMBOS.len(), 10);
        for list in [&MODALS[..], &COMBOS[..]] {
            let mut sorted = list.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), list.len());
            for w in list {
                assert!(
                    !w.is_empty() && w.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                    "{w}"
                );
            }
        }
        // the limits are accepted and the next step is refused
        assert!(parse(&["--wait", &MAX_WAIT.to_string()]).is_ok());
        assert!(parse(&["--wait", &(MAX_WAIT + 1).to_string()]).is_err());
        assert!(parse(&["--tabs", &MAX_TABS.to_string()]).is_ok());
        assert!(parse(&["--tabs", &(MAX_TABS + 1).to_string()]).is_err());
        assert!(parse(&["--tab", &MAX_TAB.to_string()]).is_ok());
        assert!(parse(&["--tab", &(u32::from(MAX_TAB) + 1).to_string()]).is_err());
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// launcher
// ---------------------------------------------------------------------------------------------------------------------------------

mod launcher_tests {
    use super::scratch;
    use eh3d_bungard_gui::launcher::{
        classic_launch, model_key, Choice, Machine, Program, Settings, DEFAULT_ESTOP,
        DEFAULT_UPDATES_API, RECENT_MAX,
    };
    use eh3d_bungard_gui::spindle::SpindleId;
    use eh3d_view::texts::Lang;
    use eh3d_view::theme::ThemeId;

    #[test]
    fn the_words_of_the_choices_are_fixed_and_read_without_regard_to_case_or_spaces() {
        assert_eq!(
            Choice::ALL.map(Choice::key),
            [
                "modern",
                "classic-drill-mtc",
                "classic-drill-atc",
                "classic-route-mtc",
                "classic-route-atc"
            ]
        );
        assert_eq!(
            Choice::from_key("  CLASSIC-DRILL-ATC\t"),
            Some(Choice::Classic(Program::Drill, Machine::Atc))
        );
        assert_eq!(Choice::from_key("Modern\n"), Some(Choice::Modern));
        for bad in [
            "",
            "classic-drill",
            "classic-drill-mtc2",
            "ask",
            "modern modern",
        ] {
            assert_eq!(Choice::from_key(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_program_beside_the_launcher_wins_over_the_classic_folder_and_a_folder_is_not_a_program()
    {
        let d = scratch("classic");
        let suffix = std::env::consts::EXE_SUFFIX;
        let route = Choice::Classic(Program::Route, Machine::Mtc);
        std::fs::create_dir_all(d.join(format!("routepro{suffix}"))).unwrap();
        assert!(classic_launch(route, &d).is_none());
        std::fs::create_dir_all(d.join("classic")).unwrap();
        std::fs::write(d.join("classic").join(format!("routepro{suffix}")), b"x").unwrap();
        let found = classic_launch(route, &d).unwrap();
        assert_eq!(
            found.exe,
            d.join("classic").join(format!("routepro{suffix}"))
        );
        let drill = Choice::Classic(Program::Drill, Machine::Atc);
        std::fs::write(d.join(format!("drillpro{suffix}")), b"x").unwrap();
        std::fs::write(d.join("classic").join(format!("drillpro{suffix}")), b"x").unwrap();
        assert_eq!(
            classic_launch(drill, &d).unwrap().exe,
            d.join(format!("drillpro{suffix}"))
        );
        // the configuration of the machine: beside the launcher first, then in the classic folder
        let inner = d.join("classic").join("config").join("atc");
        std::fs::create_dir_all(&inner).unwrap();
        let args =
            |p: &std::path::Path| vec!["--data".to_string(), p.to_string_lossy().into_owned()];
        assert_eq!(classic_launch(drill, &d).unwrap().args, args(&inner));
        let outer = d.join("config").join("atc");
        std::fs::create_dir_all(&outer).unwrap();
        assert_eq!(classic_launch(drill, &d).unwrap().args, args(&outer));
        // a file with the name of the folder is no configuration
        std::fs::write(d.join("config").join("mtc"), b"x").unwrap();
        let mtc = Choice::Classic(Program::Drill, Machine::Mtc);
        assert!(classic_launch(mtc, &d).unwrap().args.is_empty());
        assert!(classic_launch(Choice::Modern, &d).is_none());
    }

    #[test]
    fn the_numbers_of_the_settings_file_take_exactly_their_range() {
        let d = Settings::default();
        for (text, expect) in [("0.1", 0.1f32), ("1000", 1000.0), ("2.5", 2.5)] {
            assert_eq!(
                Settings::parse(&format!("sim_speed = {text}")).sim_speed,
                expect
            );
        }
        for bad in ["0.09", "1000.5", "nan", "inf", "fast", "-1", ""] {
            assert_eq!(
                Settings::parse(&format!("sim_speed = {bad}")).sim_speed,
                d.sim_speed,
                "{bad}"
            );
        }
        let mark = |key: &str, text: &str| Settings::parse(&format!("{key} = {text}"));
        assert_eq!(
            mark("route_mark_tolerance_mm", "0.01").route_mark_tolerance_mm,
            0.01
        );
        assert_eq!(
            mark("route_mark_tolerance_mm", "5").route_mark_tolerance_mm,
            5.0
        );
        assert_eq!(
            mark("route_mark_tolerance_mm", "0.0099").route_mark_tolerance_mm,
            0.35
        );
        assert_eq!(
            mark("route_mark_tolerance_mm", "5.01").route_mark_tolerance_mm,
            0.35
        );
        assert_eq!(
            mark("route_mark_min_depth_mm", "0").route_mark_min_depth_mm,
            0.0
        );
        assert_eq!(
            mark("route_mark_min_depth_mm", "5").route_mark_min_depth_mm,
            5.0
        );
        assert_eq!(
            mark("route_mark_min_depth_mm", "-0.01").route_mark_min_depth_mm,
            0.02
        );
        assert_eq!(
            mark("route_mark_min_depth_mm", "5.01").route_mark_min_depth_mm,
            0.02
        );
        assert_eq!(mark("route_mark_step_s", "0.005").route_mark_step_s, 0.005);
        assert_eq!(mark("route_mark_step_s", "1").route_mark_step_s, 1.0);
        assert_eq!(mark("route_mark_step_s", "0.0049").route_mark_step_s, 0.05);
        assert_eq!(mark("route_mark_step_s", "1.01").route_mark_step_s, 0.05);
        for (text, expect) in [
            ("840", Some(840)),
            ("-840", Some(-840)),
            ("841", None),
            ("-841", None),
            ("abc", None),
        ] {
            assert_eq!(
                mark("clock_utc_offset_minutes", text).clock_utc_offset_min,
                expect,
                "{text}"
            );
        }
        // a bad value after a good one leaves the good one
        let kept =
            Settings::parse("clock_utc_offset_minutes = 60\nclock_utc_offset_minutes = 9999\n");
        assert_eq!(kept.clock_utc_offset_min, Some(60));
        assert_eq!(
            mark("updates_interval_hours", "2160").updates_interval_h,
            2160
        );
        assert_eq!(
            mark("updates_interval_hours", "2161").updates_interval_h,
            24
        );
        // the time of the last look is the one value that goes to zero when it is wrong
        assert_eq!(mark("updates_last_check", "77").updates_last_check, 77);
        let reset = Settings::parse("updates_last_check = 5\nupdates_last_check = x\n");
        assert_eq!(reset.updates_last_check, 0);
        assert_eq!(mark("updates_last_check", "-1").updates_last_check, 0);
        assert_eq!(
            mark("updates_last_check", &u64::MAX.to_string()).updates_last_check,
            u64::MAX
        );
        // the address of the updates: an empty one keeps the default and the rest is trimmed
        assert_eq!(
            mark("updates_api_url", "   ").updates_api_url,
            DEFAULT_UPDATES_API
        );
        assert_eq!(
            mark("updates_api_url", "  http://h/x  ").updates_api_url,
            "http://h/x"
        );
        assert_eq!(mark("updates_api_url", "a = b").updates_api_url, "a = b");
    }

    #[test]
    fn the_lists_of_the_settings_file_drop_what_they_cannot_read_and_keep_the_rest() {
        let s = Settings::parse("recent = a| |b||c|d|e|f|g|h\n");
        assert_eq!(s.recent, ["a", "b", "c", "d", "e", "f"]);
        assert_eq!(s.recent.len(), RECENT_MAX);
        let mut s = s;
        s.remember_recent("b");
        s.remember_recent("z");
        s.remember_recent("Z");
        assert_eq!(s.recent, ["Z", "z", "b", "a", "c", "d"]);
        let text = format!(
            "holes_drilled = 1:5,x:2,3:y,300:1, 7:9 ,2:{},1:6\n",
            u64::MAX
        );
        let s = Settings::parse(&text);
        let held: Vec<(u8, u64)> = s.holes_drilled.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(held, vec![(1, 6), (2, u64::MAX), (7, 9)]);
        assert_eq!(Settings::parse(&s.to_text()).holes_drilled, s.holes_drilled);
        assert!(Settings::parse("holes_drilled =\n")
            .holes_drilled
            .is_empty());
        let m = Settings::parse(
            "material_values = |n|1|2\nmaterial_values = a|n|NaN|1\nmaterial_values = a|n|1|inf\nmaterial_values = a|n|1|2|3\nmaterial_values = a||1.5|20\nmaterial_values = b|n| 1 |2\n",
        );
        assert_eq!(m.material_values.len(), 1);
        assert_eq!(m.material_values_of("a", ""), Some((1.5, 20.0)));
        assert!(m.to_text().contains("material_values = a||1.5|20\n"));
        assert_eq!(
            Settings::parse(&m.to_text()).material_values,
            m.material_values
        );
    }

    #[test]
    fn keys_are_read_in_any_case_and_the_switch_words_have_one_meaning_each() {
        let s = Settings::parse(
            "LANG = de\n  Theme=windows-classic\nMachine_Name =  a=b \nunknown = 1\nno equals\n\n= x\n",
        );
        let expect = Settings {
            lang: Lang::De,
            theme: ThemeId::WindowsClassic,
            machine_name: "a=b".to_string(),
            ..Settings::default()
        };
        assert_eq!(s, expect);
        // a file with Windows line ends is the same file
        let crlf = Settings::parse("lang = es\r\nsim_speed = 2\r\nchoice = Classic-Drill-MTC\r\n");
        assert_eq!(crlf.lang, Lang::Es);
        assert_eq!(crlf.sim_speed, 2.0);
        assert_eq!(
            crlf.choice,
            Some(Choice::Classic(Program::Drill, Machine::Mtc))
        );
        for off in ["no", "off", "false", "0"] {
            let t = format!(
                "emergency = {off}\nask_operator = {off}\ntools_on_open = {off}\nopen_preview = {off}\nupdates = {off}\n"
            );
            let s = Settings::parse(&t);
            assert!(
                !s.emergency
                    && !s.ask_operator
                    && !s.tools_on_open
                    && !s.open_preview
                    && !s.updates_enabled,
                "{off}"
            );
        }
        // anything else leaves these on
        let s = Settings::parse(
            "emergency = maybe\nask_operator = maybe\ntools_on_open = maybe\nopen_preview = maybe\nupdates = maybe\n",
        );
        assert!(
            s.emergency && s.ask_operator && s.tools_on_open && s.open_preview && s.updates_enabled
        );
        for on in ["yes", "on", "true", "1"] {
            let t = format!(
                "fullscreen = {on}\nall_makers = {on}\nupdates_at_start = {on}\nupdates_require_signature = {on}\n"
            );
            let s = Settings::parse(&t);
            assert!(
                s.fullscreen && s.all_makers && s.updates_at_start && s.updates_require_signature,
                "{on}"
            );
        }
        // and for these anything else, the last word wins, turns them off
        let s = Settings::parse(
            "fullscreen = yes\nfullscreen = maybe\nupdates_require_signature = on\nupdates_require_signature = \n",
        );
        assert!(!s.fullscreen && !s.updates_require_signature);
    }

    #[test]
    fn the_file_of_the_settings_has_its_keys_once_and_in_one_order_and_writing_it_again_changes_nothing(
    ) {
        let text = Settings::default().to_text();
        assert!(text.ends_with('\n') && !text.contains('\r'));
        assert!(text.lines().all(|l| l.contains('=')));
        let keys: Vec<&str> = text
            .lines()
            .filter_map(|l| l.split_once('='))
            .map(|(k, _)| k.trim())
            .collect();
        let mut sorted = keys.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len());
        let expected = [
            "choice",
            "lang",
            "theme",
            "board",
            "ask_operator",
            "tools_on_open",
            "open_preview",
            "sim_speed",
            "model",
            "emergency",
            "estop_pos",
            "display_units",
            "spindle",
            "spindle_original",
            "spindle_vfd",
            "spindle_drawing",
            "updates",
            "updates_at_start",
            "updates_interval_hours",
            "updates_api_url",
            "updates_last_check",
            "updates_timeout_seconds",
            "updates_require_signature",
            "holes_drilled",
            "recent",
            "fullscreen",
            "all_makers",
            "panes",
            "machine_name",
            "machine_make",
            "profile_aluminium",
            "route_mark_tolerance_mm",
            "route_mark_step_s",
            "route_mark_min_depth_mm",
            "log",
            "log_level",
            "log_max_kb",
            "log_files",
            "log_heartbeat_s",
        ];
        let mut at = 0;
        for k in expected {
            let pos = keys[at..]
                .iter()
                .position(|x| *x == k)
                .unwrap_or_else(|| panic!("{k} is missing or out of its place"));
            at += pos + 1;
        }
        let custom = Settings {
            lang: Lang::De,
            sim_speed: 2.5,
            estop_pos: (10.5, 20.25),
            recent: vec!["a.ncd".into(), "b.zip".into()],
            fullscreen: true,
            ..Settings::default()
        };
        let once = custom.to_text();
        assert_eq!(Settings::parse(&once).to_text(), once);
        assert_eq!(DEFAULT_ESTOP, (1474.0, 204.0));
        assert_eq!(RECENT_MAX, 6);
        assert!(DEFAULT_UPDATES_API.starts_with("https://"));
        assert_eq!(Settings::default().updates_api_url, DEFAULT_UPDATES_API);
    }

    #[test]
    fn saving_makes_the_folders_replaces_the_file_whole_and_a_file_that_cannot_be_read_is_the_defaults(
    ) {
        let d = scratch("save");
        let deep = d.join("a").join("b").join("c.cfg");
        let first = Settings {
            fullscreen: true,
            ..Settings::default()
        };
        first.save(&deep).unwrap();
        assert_eq!(Settings::load(&deep), first);
        let second = Settings {
            recent: vec!["x".to_string()],
            ..Settings::default()
        };
        second.save(&deep).unwrap();
        assert_eq!(Settings::load(&deep), second);
        assert!(!deep.with_extension("tmp").exists());
        // a folder where the file should be: the error comes back and nothing is left behind
        let blocked = d.join("folder.cfg");
        std::fs::create_dir_all(&blocked).unwrap();
        assert!(first.save(&blocked).is_err());
        assert!(!blocked.with_extension("tmp").exists());
        assert!(blocked.is_dir());
        assert_eq!(Settings::load(&blocked), Settings::default());
        let junk = d.join("junk.cfg");
        std::fs::write(&junk, [0xffu8, 0xfe, 0x00, 0x41]).unwrap();
        assert_eq!(Settings::load(&junk), Settings::default());
        let empty = d.join("empty.cfg");
        std::fs::write(&empty, "").unwrap();
        assert_eq!(Settings::load(&empty), Settings::default());
        std::fs::write(&empty, "# nothing\n\n").unwrap();
        assert_eq!(Settings::load(&empty), Settings::default());
    }

    #[test]
    fn the_key_of_a_model_and_the_change_of_model_keep_the_limits_of_each_one_apart() {
        for (model, key) in [
            ("", ""),
            ("   ", ""),
            ("---", ""),
            ("A-B_C", "a_b_c"),
            ("CCD/2  ATC", "ccd_2_atc"),
            ("Ünï", "n"),
            ("  X  ", "x"),
            ("a__b", "a_b"),
        ] {
            assert_eq!(model_key(model), key, "{model:?}");
            assert_eq!(model_key(&model_key(model)), key, "{model:?}");
        }
        let mut s = Settings::default();
        s.spindles[0].max_rpm = 11_111;
        // a name with the same key is the same model: only the name changes
        s.switch_model("ccd-mtc");
        assert_eq!(s.model, "ccd-mtc");
        assert_eq!(s.spindles[0].max_rpm, 11_111);
        assert!(s.machine_spindles.is_empty());
        assert!(!s.spindle_is_provisional());
        s.switch_model("CCD XL ATC");
        assert_ne!(s.spindles[0].max_rpm, 11_111);
        assert!(s.machine_spindles.contains_key("ccd_mtc"));
        assert!(s.spindle_is_provisional());
        s.switch_model("CCD MTC");
        assert_eq!(s.spindles[0].max_rpm, 11_111);
        assert!(!s.machine_spindles.contains_key("ccd_mtc"));
        assert!(s.machine_spindles.contains_key("ccd_xl_atc"));
        // the spindle in use is the one of its code
        s.spindles[1].max_rpm = 4_321;
        s.spindle = SpindleId::Vfd;
        assert_eq!(s.current_spindle().max_rpm, 4_321);
        s.spindle = SpindleId::Original;
        assert_eq!(s.current_spindle().max_rpm, 11_111);
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// updates (needs the feature of the same name)
// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "updates")]
mod updates_tests {
    use super::{scratch, Lcg};
    use eh3d_bungard_gui::updates::{
        add_history, address_allowed, check_signature, extract_program, format_time, install,
        is_newer, package_name, parse_release, parse_sum_file, plain_notes, platform_label,
        previous_path, previous_version, read_history, run_undo, sha256_file, spawn_check,
        spawn_update, staging_path, undo, verify, version_parts, Config, Entry, Failure, Package,
        Release, State, Step, StepState, Target, PROGRAM,
    };
    use serde_json::json;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;

    const API: &str = "https://api.example.test/latest";
    const LABEL: &str = "linux-x86_64";
    const EMPTY_SUM: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const ABC_SUM: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const MILLION_A_SUM: &str = "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0";

    fn package(name: &str) -> Package {
        Package {
            name: name.into(),
            url: "https://example.test/p".into(),
            sum_url: "https://example.test/s".into(),
            sig_url: None,
            record_url: None,
            record_sig_url: None,
            size: 0,
            digest: None,
        }
    }

    fn asset(name: &str, url: &str) -> serde_json::Value {
        json!({"name": name, "browser_download_url": url, "size": 5})
    }

    fn release_text(tag: &str, assets: Vec<serde_json::Value>) -> String {
        json!({"tag_name": tag, "body": "- **a**", "html_url": "https://example.test/r", "assets": assets})
            .to_string()
    }

    fn entry(time: u64) -> Entry {
        Entry {
            time,
            what: "update".into(),
            from: "0.1.0".into(),
            to: "0.2.0".into(),
            note: String::new(),
        }
    }

    fn tar_gz(entries: &[(&str, &str)]) -> Vec<u8> {
        let owned: Vec<(&str, &[u8])> = entries.iter().map(|&(n, d)| (n, d.as_bytes())).collect();
        tar_gz_bytes(&owned)
    }

    fn tar_gz_bytes(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        {
            let gz = flate2::write::GzEncoder::new(&mut out, flate2::Compression::fast());
            let mut b = tar::Builder::new(gz);
            for &(name, data) in entries {
                let mut h = tar::Header::new_gnu();
                h.set_size(data.len() as u64);
                h.set_mode(0o644);
                h.set_cksum();
                b.append_data(&mut h, name, data).unwrap();
            }
            b.into_inner().unwrap().finish().unwrap();
        }
        out
    }

    fn zip_of(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let o = zip::write::SimpleFileOptions::default();
            for &(name, data) in entries {
                z.start_file(name, o).unwrap();
                z.write_all(data.as_bytes()).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn a_version_is_one_to_four_numbers_and_the_later_one_is_the_one_that_is_greater_part_by_part()
    {
        assert_eq!(version_parts("v1.2.3"), Some(vec![1, 2, 3]));
        assert_eq!(version_parts(" V10 "), Some(vec![10]));
        assert_eq!(version_parts("0.8.0.1"), Some(vec![0, 8, 0, 1]));
        let widest = u64::MAX.to_string();
        assert_eq!(version_parts(&widest), Some(vec![u64::MAX]));
        assert_eq!(version_parts("18446744073709551616"), None);
        for bad in [
            "",
            "v",
            "1.",
            ".1",
            "1..2",
            "1.2.3.4.5",
            "1.2.x",
            "-1",
            "1.2.3-beta",
            "1 2",
        ] {
            assert_eq!(version_parts(bad), None, "{bad:?}");
        }
        let order = [
            "0.0.1", "0.0.9", "0.1", "0.1.0.1", "0.1.1", "0.2.0", "0.10.0", "1", "1.0.0.1", "2.0",
        ];
        for (i, low) in order.iter().enumerate() {
            assert!(!is_newer(low, low), "{low}");
            for high in &order[i + 1..] {
                assert!(is_newer(high, low), "{high} over {low}");
                assert!(!is_newer(low, high), "{low} over {high}");
            }
        }
        // a part that is missing is a zero, and a version that cannot be read is never later or earlier
        assert!(!is_newer("1.0.0", "1") && !is_newer("1", "1.0.0"));
        assert!(!is_newer("1.0", "bad") && !is_newer("bad", "1.0"));
    }

    #[test]
    fn the_packages_are_named_by_their_system_and_an_address_is_https_or_this_computer() {
        assert_eq!(PROGRAM, "bungard-gui");
        for (label, name) in [
            ("windows-x86_64", "bungard-gui-1.0.0-windows-x86_64.zip"),
            ("windows-x86", "bungard-gui-1.0.0-windows-x86.zip"),
            ("windows-arm", "bungard-gui-1.0.0-windows-arm.zip"),
            ("linux-x86_64", "bungard-gui-1.0.0-linux-x86_64.tar.gz"),
            ("macos-x86_64", "bungard-gui-1.0.0-macos-x86_64.tar.gz"),
            ("freebsd", "bungard-gui-1.0.0-freebsd.tar.gz"),
            ("", "bungard-gui-1.0.0-.tar.gz"),
        ] {
            assert_eq!(package_name("1.0.0", label), name);
        }
        if let Some(label) = platform_label() {
            let zip = package_name("1.0.0", label).ends_with(".zip");
            assert_eq!(zip, label.starts_with("windows"));
        }
        let local = "http://127.0.0.1:9/latest";
        let secure = "https://api.github.com/x";
        for (url, api, allowed) in [
            ("https://x.test/p", local, true),
            ("https://", local, true),
            ("https://x", "http://example.test/latest", true),
            ("", local, false),
            ("http://localhost", local, true),
            ("http://localhost:8080/x?y", local, true),
            ("http://[::1]:8080/x", local, true),
            ("http://[::2]/x", local, false),
            ("http://127.0.0.1?x=1", local, true),
            ("http://127.0.0.1@evil.example/x", local, false),
            ("http://localhost.evil.example/x", local, false),
            ("http://LOCALHOST/x", local, false),
            ("HTTP://127.0.0.1/x", local, false),
            ("http://127.0.0.1:9/p", secure, true),
            ("http://127.0.0.1:9/p", "ftp://127.0.0.1/x", false),
            ("http://example.test/p", secure, false),
        ] {
            assert_eq!(address_allowed(url, api), allowed, "{url} with {api}");
        }
    }

    #[test]
    fn the_notes_of_a_release_lose_their_marks_and_keep_their_words() {
        assert_eq!(
            plain_notes("# Heading\n\n## Sub  \n#\n#hashtag"),
            ["Heading", "Sub", "hashtag"]
        );
        assert_eq!(
            plain_notes("* one\n+ two\n- three"),
            ["- one", "- two", "- three"]
        );
        assert_eq!(plain_notes("-no space"), ["-no space"]);
        assert_eq!(plain_notes("- "), ["-"]);
        assert!(plain_notes("- **").is_empty());
        assert!(plain_notes("").is_empty() && plain_notes("\n \r\n\t\n").is_empty());
        assert_eq!(plain_notes("[a](x) and [b](y)"), ["a and b"]);
        assert_eq!(
            plain_notes("see [broken](no close"),
            ["see [broken](no close"]
        );
        assert_eq!(plain_notes("text ](x) stray"), ["text ](x) stray"]);
        assert_eq!(plain_notes("**bold** and `code`"), ["bold and code"]);
        assert_eq!(
            plain_notes("- é ñandú\r\n* 日本語"),
            ["- é ñandú", "- 日本語"]
        );
        let long = format!("- {}", "word ".repeat(2000));
        assert_eq!(plain_notes(&long).len(), 1);
    }

    #[test]
    fn a_release_is_read_for_its_package_its_signature_and_its_checksum_of_the_api() {
        let pkg = package_name("1.2.3", LABEL);
        let text = release_text(
            "v1.2.3",
            vec![
                asset(&format!("{pkg}.sig"), "https://e.test/s"),
                asset(&pkg, "https://e.test/p"),
                asset(&format!("{pkg}.sha256"), "https://e.test/h"),
            ],
        );
        let rel = parse_release(&text, LABEL, API).unwrap();
        assert_eq!(
            (rel.tag.as_str(), rel.version.as_str()),
            ("v1.2.3", "1.2.3")
        );
        assert_eq!(rel.notes, vec!["- a".to_string()]);
        assert_eq!(rel.page, "https://example.test/r");
        let p = rel.package.unwrap();
        assert_eq!(p.name, pkg);
        assert_eq!(
            (p.url.as_str(), p.sum_url.as_str()),
            ("https://e.test/p", "https://e.test/h")
        );
        assert_eq!(p.sig_url.as_deref(), Some("https://e.test/s"));
        assert_eq!((p.size, p.digest), (5, None));
        // the checksum the API gives is lower-cased; another kind of checksum is not taken
        let with_digest = |digest: &str| {
            let mut a =
                json!({"name": pkg, "browser_download_url": "https://e.test/p", "digest": digest});
            a["size"] = json!(7);
            release_text(
                "1.2.3",
                vec![a, asset(&format!("{pkg}.sha256"), "https://e.test/h")],
            )
        };
        let p = parse_release(&with_digest("sha256:ABCDEF0123"), LABEL, API)
            .unwrap()
            .package
            .unwrap();
        assert_eq!((p.digest.as_deref(), p.size), (Some("abcdef0123"), 7));
        let p = parse_release(&with_digest("md5:abc"), LABEL, API)
            .unwrap()
            .package
            .unwrap();
        assert_eq!(p.digest, None);
        // a capital V is a tag too, and the first asset of a name is the one
        let pkg2 = package_name("1.2", LABEL);
        let text = release_text(
            "V1.2",
            vec![
                asset(&pkg2, "https://e.test/first"),
                asset(&pkg2, "https://e.test/second"),
                asset(&format!("{pkg2}.sha256"), "https://e.test/h"),
            ],
        );
        let rel = parse_release(&text, LABEL, API).unwrap();
        assert_eq!((rel.tag.as_str(), rel.version.as_str()), ("V1.2", "1.2"));
        assert_eq!(rel.package.unwrap().url, "https://e.test/first");
    }

    #[test]
    fn a_release_that_is_not_one_to_update_to_is_refused_with_its_reason() {
        let pkg = package_name("1.0.0", LABEL);
        let good = vec![
            asset(&pkg, "https://e.test/p"),
            asset(&format!("{pkg}.sha256"), "https://e.test/h"),
        ];
        let mut pre: serde_json::Value =
            serde_json::from_str(&release_text("v1.0.0", good.clone())).unwrap();
        pre["prerelease"] = json!(true);
        assert_eq!(
            parse_release(&pre.to_string(), LABEL, API).unwrap_err(),
            Failure::Answer("the latest release is a draft or a pre-release".into())
        );
        pre["prerelease"] = json!(false);
        pre["draft"] = json!(true);
        assert!(parse_release(&pre.to_string(), LABEL, API).is_err());
        assert_eq!(
            parse_release(&json!({"assets": []}).to_string(), LABEL, API).unwrap_err(),
            Failure::Answer("the tag \"\" is not a version".into())
        );
        for bad in ["[]", "null", "3"] {
            assert!(
                matches!(parse_release(bad, LABEL, API), Err(Failure::Answer(_))),
                "{bad}"
            );
        }
        assert!(matches!(
            parse_release("", LABEL, API).unwrap_err(),
            Failure::Answer(m) if m.starts_with("not JSON:")
        ));
        assert!(parse_release(&release_text("1.2.3.4.5", good.clone()), LABEL, API).is_err());
        // no assets, only the checksum, or no address: no package, or a refusal
        let none = parse_release(&json!({"tag_name": "v1.0.0"}).to_string(), LABEL, API).unwrap();
        assert!(none.package.is_none() && none.notes.is_empty() && none.page.is_empty());
        let only_sum = release_text("v1.0.0", vec![good[1].clone()]);
        assert!(parse_release(&only_sum, LABEL, API)
            .unwrap()
            .package
            .is_none());
        let no_url = release_text(
            "v1.0.0",
            vec![
                json!({"name": pkg}),
                asset(&format!("{pkg}.sha256"), "https://e.test/h"),
            ],
        );
        assert!(matches!(
            parse_release(&no_url, LABEL, API),
            Err(Failure::Refused(_))
        ));
        let plain = release_text(
            "v1.0.0",
            vec![
                asset(&pkg, "http://e.test/p"),
                asset(&format!("{pkg}.sha256"), "https://e.test/h"),
            ],
        );
        assert_eq!(
            parse_release(&plain, LABEL, API).unwrap_err(),
            Failure::Refused("the package is not at a secure address".into())
        );
    }

    #[test]
    fn a_checksum_file_must_hold_a_sha256_for_this_package_on_its_first_line_with_text() {
        let h = ABC_SUM;
        assert_eq!(
            parse_sum_file(&format!("\n\n{h}  a.zip\n{EMPTY_SUM}  b.zip\n"), "a.zip").unwrap(),
            h
        );
        assert_eq!(parse_sum_file(&format!("{h}\r\n"), "anything").unwrap(), h);
        assert_eq!(parse_sum_file(&format!("{h} *a.zip"), "a.zip").unwrap(), h);
        assert_eq!(
            parse_sum_file(&format!("  {}\ta.zip", h.to_uppercase()), "a.zip").unwrap(),
            h
        );
        let not_sha = Failure::Checksum("the checksum file is not a SHA-256".into());
        let too_long = format!("{h}0");
        let not_hex = format!("g{}", &h[1..]);
        for bad in [&h[..63], too_long.as_str(), not_hex.as_str(), "", "   \n  "] {
            assert_eq!(
                parse_sum_file(&format!("{bad}  a.zip"), "a.zip").unwrap_err(),
                not_sha,
                "{bad:?}"
            );
        }
        assert_eq!(
            parse_sum_file(&format!("{h}  other.zip"), "a.zip").unwrap_err(),
            Failure::Checksum("the checksum file is for other.zip, not for a.zip".into())
        );
    }

    #[test]
    fn the_sha256_of_a_file_is_the_known_one_whatever_its_size_and_a_download_is_checked_against_it(
    ) {
        let d = scratch("sum");
        let f = d.join("f.bin");
        std::fs::write(&f, b"").unwrap();
        assert_eq!(sha256_file(&f).unwrap(), EMPTY_SUM);
        std::fs::write(&f, b"abc").unwrap();
        assert_eq!(sha256_file(&f).unwrap(), ABC_SUM);
        let big = d.join("big.bin");
        std::fs::write(&big, vec![b'a'; 1_000_000]).unwrap();
        assert_eq!(sha256_file(&big).unwrap(), MILLION_A_SUM);
        assert!(sha256_file(&d.join("none.bin")).is_err());
        let sum = format!("{ABC_SUM}  a.zip\n");
        let pkg = package("a.zip");
        assert_eq!(verify(&f, &pkg, &sum), Ok(()));
        let agreeing = Package {
            digest: Some(ABC_SUM.into()),
            ..package("a.zip")
        };
        assert_eq!(verify(&f, &agreeing, &sum), Ok(()));
        // the checksum of the API is looked at before the file is opened
        let clash = Package {
            digest: Some(EMPTY_SUM.into()),
            ..package("a.zip")
        };
        let gone = d.join("gone.bin");
        assert_eq!(
            verify(&gone, &clash, &sum),
            Err(Failure::Checksum(
                "the checksum file and the release do not agree".into()
            ))
        );
        assert!(matches!(verify(&gone, &pkg, &sum), Err(Failure::Disk(_))));
        std::fs::write(&f, b"abd").unwrap();
        match verify(&f, &pkg, &sum) {
            Err(Failure::Checksum(m)) => {
                assert!(
                    m.starts_with("the package has SHA-256 ") && m.contains(ABC_SUM),
                    "{m}"
                );
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(
            verify(&f, &pkg, "nonsense"),
            Err(Failure::Checksum(
                "the checksum file is not a SHA-256".into()
            ))
        );
    }

    #[test]
    fn the_history_keeps_fifty_lines_cleans_what_it_writes_and_skips_what_it_cannot_read() {
        let d = scratch("history");
        let file = d.join("deep").join("h.log");
        assert!(read_history(&file).is_empty());
        for i in 0..50 {
            add_history(&file, &entry(i));
        }
        let all = read_history(&file);
        assert_eq!((all.len(), all[0].time, all[49].time), (50, 0, 49));
        add_history(&file, &entry(50));
        let all = read_history(&file);
        assert_eq!((all.len(), all[0].time, all[49].time), (50, 1, 50));
        let dirty = d.join("dirty.log");
        add_history(
            &dirty,
            &Entry {
                time: 9,
                what: "up\tdate".into(),
                from: "a\r\nb".into(),
                to: "c\nd".into(),
                note: "x\ty ünï 日本語".into(),
            },
        );
        let back = read_history(&dirty);
        assert_eq!(back.len(), 1);
        assert_eq!(
            (
                back[0].what.as_str(),
                back[0].from.as_str(),
                back[0].to.as_str()
            ),
            ("up date", "a  b", "c d")
        );
        assert_eq!(back[0].note, "x y ünï 日本語");
        let messy = d.join("messy.log");
        std::fs::write(
            &messy,
            "1\tupdate\t0.1\t0.2\tnote\nx\tupdate\t0.1\t0.2\n3\tupdate\t0.1\n4\tundo\t0.2\t0.1\n5\tfailed\ta\tb\tn\textra\tmore\n\n  6\tupdate\ta\tb\n7\tupdate\ta\tb\tn\r\n",
        )
        .unwrap();
        let rows = read_history(&messy);
        let times: Vec<u64> = rows.iter().map(|e| e.time).collect();
        assert_eq!(times, vec![1, 4, 5, 7]);
        assert_eq!(rows[1].note, "");
        assert_eq!(rows[2].note, "n");
        assert_eq!(rows[3].note, "n");
    }

    #[test]
    fn the_time_of_the_history_is_written_in_utc_through_leap_days_and_the_century_that_is_not_leap(
    ) {
        for (secs, text) in [
            (0u64, "1970-01-01 00:00"),
            (59, "1970-01-01 00:00"),
            (60, "1970-01-01 00:01"),
            (3_599, "1970-01-01 00:59"),
            (3_600, "1970-01-01 01:00"),
            (86_399, "1970-01-01 23:59"),
            (86_400, "1970-01-02 00:00"),
            (951_782_400, "2000-02-29 00:00"),
            (951_868_800, "2000-03-01 00:00"),
            (1_709_164_800, "2024-02-29 00:00"),
            (4_107_542_399, "2100-02-28 23:59"),
            (4_107_542_400, "2100-03-01 00:00"),
        ] {
            assert_eq!(format_time(secs), text, "{secs}");
        }
        // the text sorts as the time does
        let mut last = String::new();
        for t in (0..4_200_000_000u64).step_by(3_196_800) {
            let now = format_time(t);
            assert!(now > last, "{t}: {now} after {last}");
            last = now;
        }
        assert!(!format_time(u64::MAX).is_empty());
    }

    #[test]
    fn an_install_keeps_the_old_program_an_undo_swaps_the_two_and_neither_leaves_a_half_done_state()
    {
        let d = scratch("install");
        let exe = d.join("bungard-gui");
        let version_file = |exe: &Path| {
            let mut v = previous_path(exe).into_os_string();
            v.push(".version");
            PathBuf::from(v)
        };
        assert_eq!(
            previous_path(Path::new("x/bungard-gui")),
            Path::new("x/bungard-gui.previous")
        );
        assert_eq!(
            staging_path(Path::new("x/bungard-gui")),
            Path::new("x/bungard-gui.new")
        );
        assert_eq!(previous_version(&exe), None);
        std::fs::write(&exe, b"OLD").unwrap();
        // old files of an earlier install are replaced
        std::fs::write(previous_path(&exe), b"STALE").unwrap();
        std::fs::write(version_file(&exe), "9.9.9").unwrap();
        std::fs::write(staging_path(&exe), b"NEW").unwrap();
        install(&staging_path(&exe), &exe, "0.3.0").unwrap();
        assert_eq!(std::fs::read(&exe).unwrap(), b"NEW");
        assert_eq!(std::fs::read(previous_path(&exe)).unwrap(), b"OLD");
        assert_eq!(previous_version(&exe).as_deref(), Some("0.3.0"));
        assert!(!staging_path(&exe).exists());
        // an undo and its undo are back where they started
        assert_eq!(undo(&exe, "0.4.0").unwrap(), "0.3.0");
        assert_eq!(std::fs::read(&exe).unwrap(), b"OLD");
        assert_eq!(previous_version(&exe).as_deref(), Some("0.4.0"));
        assert_eq!(undo(&exe, "0.3.0").unwrap(), "0.4.0");
        assert_eq!(std::fs::read(&exe).unwrap(), b"NEW");
        assert_eq!(std::fs::read(previous_path(&exe)).unwrap(), b"OLD");
        // the file of the version is read trimmed, and empty is empty
        std::fs::write(version_file(&exe), "  0.1.9 \n").unwrap();
        assert_eq!(previous_version(&exe).as_deref(), Some("0.1.9"));
        std::fs::write(version_file(&exe), " \n").unwrap();
        assert_eq!(previous_version(&exe).as_deref(), Some(""));
        // no program to replace, or no new program: an error, and the old one stays
        let d2 = scratch("install-fail");
        let lone = d2.join("lone");
        std::fs::write(d2.join("new"), b"NEW").unwrap();
        assert!(matches!(
            install(&d2.join("new"), &lone, "1"),
            Err(Failure::Disk(_))
        ));
        assert!(d2.join("new").exists());
        std::fs::write(&lone, b"OLD").unwrap();
        assert!(matches!(
            install(&d2.join("absent"), &lone, "1"),
            Err(Failure::Disk(_))
        ));
        assert_eq!(std::fs::read(&lone).unwrap(), b"OLD");
        assert!(!previous_path(&lone).exists());
        assert_eq!(previous_version(&lone), None);
        // nothing kept: nothing to undo; a version file that is not there gives an empty version; a stale side file is cleared
        assert_eq!(
            undo(&lone, "2").unwrap_err(),
            Failure::Refused("there is no earlier version kept".into())
        );
        std::fs::write(previous_path(&lone), b"B").unwrap();
        let mut swap = lone.clone().into_os_string();
        swap.push(".swap");
        std::fs::write(&swap, b"junk").unwrap();
        assert_eq!(undo(&lone, "cur").unwrap(), "");
        assert_eq!(std::fs::read(&lone).unwrap(), b"B");
        assert_eq!(std::fs::read(previous_path(&lone)).unwrap(), b"OLD");
        assert!(!Path::new(&swap).exists());
        assert_eq!(previous_version(&lone).as_deref(), Some("cur"));
    }

    #[test]
    fn only_the_program_of_a_package_is_taken_and_a_package_that_is_wrong_is_refused() {
        let d = scratch("extract");
        let out = d.join("out");
        let tgz = d.join("p.tar.gz");
        std::fs::write(
            &tgz,
            tar_gz(&[
                ("pkg/README", "read"),
                ("pkg/bungard-gui.txt", "decoy"),
                ("one/bungard-gui", "FIRST"),
                ("two/bungard-gui", "SECOND"),
            ]),
        )
        .unwrap();
        extract_program(&tgz, LABEL, &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"FIRST");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_ne!(
                std::fs::metadata(&out).unwrap().permissions().mode() & 0o111,
                0
            );
        }
        let bare = d.join("bare.tar.gz");
        std::fs::write(&bare, tar_gz(&[("bungard-gui", "TOP")])).unwrap();
        extract_program(&bare, LABEL, &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"TOP");
        let without = d.join("without.tar.gz");
        std::fs::write(&without, tar_gz(&[("pkg/README", "read")])).unwrap();
        assert_eq!(
            extract_program(&without, LABEL, &out).unwrap_err(),
            Failure::Archive("the package has no bungard-gui".into())
        );
        let empty = d.join("empty.tar.gz");
        std::fs::write(&empty, tar_gz(&[("pkg/bungard-gui", "")])).unwrap();
        assert_eq!(
            extract_program(&empty, LABEL, &out).unwrap_err(),
            Failure::Archive("the program in the package is empty".into())
        );
        let z = d.join("p.zip");
        std::fs::write(
            &z,
            zip_of(&[
                ("pkg/LICENSE", "l"),
                ("pkg/bungard-gui", "decoy"),
                ("pkg/bungard-gui.exe", "WIN"),
            ]),
        )
        .unwrap();
        extract_program(&z, "windows-x86", &out).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"WIN");
        let no_exe = d.join("no-exe.zip");
        std::fs::write(&no_exe, zip_of(&[("pkg/bungard-gui", "decoy")])).unwrap();
        assert_eq!(
            extract_program(&no_exe, "windows-x86_64", &out).unwrap_err(),
            Failure::Archive("the package has no bungard-gui.exe".into())
        );
        let empty_zip = d.join("empty.zip");
        std::fs::write(&empty_zip, zip_of(&[("bungard-gui.exe", "")])).unwrap();
        assert_eq!(
            extract_program(&empty_zip, "windows-x86_64", &out).unwrap_err(),
            Failure::Archive("the program in the package is empty".into())
        );
        // the format of the system, not the other one
        assert!(matches!(
            extract_program(&tgz, "windows-x86_64", &out),
            Err(Failure::Archive(_))
        ));
        assert!(matches!(
            extract_program(&z, LABEL, &out),
            Err(Failure::Archive(_))
        ));
        assert!(matches!(
            extract_program(&d.join("absent.zip"), LABEL, &out),
            Err(Failure::Disk(_))
        ));
        // a package that is cut short is refused
        let mut g = Lcg(5);
        let random: Vec<u8> = (0..50_000).map(|_| (g.step() & 0xff) as u8).collect();
        let whole = tar_gz_bytes(&[("pkg/bungard-gui", &random[..])]);
        let cut = d.join("cut.tar.gz");
        std::fs::write(&cut, &whole[..20_000]).unwrap();
        assert!(matches!(
            extract_program(&cut, LABEL, &out),
            Err(Failure::Archive(_))
        ));
        let nothing = d.join("nothing.tar.gz");
        std::fs::write(&nothing, "").unwrap();
        assert!(matches!(
            extract_program(&nothing, LABEL, &out),
            Err(Failure::Archive(_))
        ));
    }

    #[test]
    fn a_failure_says_what_kind_it_is_and_a_release_without_a_signature_is_told_apart() {
        let all = [
            (Failure::Network("a".into()), "network: a"),
            (Failure::Answer("a".into()), "answer of the server: a"),
            (
                Failure::NoPackage,
                "the release has no package for this system",
            ),
            (Failure::Checksum("a".into()), "checksum: a"),
            (Failure::Signature("a".into()), "signature: a"),
            (Failure::Archive("a".into()), "package: a"),
            (Failure::Disk("a".into()), "disk: a"),
            (Failure::Refused("a".into()), "refused: a"),
            (Failure::Cancelled, "cancelled"),
        ];
        for (failure, text) in all {
            assert_eq!(failure.to_string(), text);
            let boxed: Box<dyn std::error::Error> = Box::new(failure);
            assert_eq!(boxed.to_string(), text);
        }
        let cfg = |require_signature: bool| Config {
            api_url: API.into(),
            timeout_s: 1,
            require_signature,
        };
        let unsigned = package("a.zip");
        assert_eq!(
            check_signature(&cfg(false), &unsigned, Path::new("unused")),
            Ok(false)
        );
        assert_eq!(
            check_signature(&cfg(true), &unsigned, Path::new("unused")),
            Err(Failure::Signature(
                "the release is not signed and the settings ask for signed releases".into()
            ))
        );
    }

    #[test]
    fn an_update_that_is_refused_before_the_network_changes_nothing_and_is_written_in_the_history()
    {
        let d = scratch("refused");
        let exe = d.join("bungard-gui");
        std::fs::write(&exe, b"OLD").unwrap();
        let target = Target {
            exe: exe.clone(),
            version: "1.0.0".into(),
            work: d.join("work"),
            history: d.join("h.log"),
            label: LABEL,
        };
        let cfg = Config {
            api_url: API.into(),
            timeout_s: 1,
            require_signature: false,
        };
        let state: State = Arc::default();
        let release = |version: &str, package: Option<Package>| Release {
            tag: format!("v{version}"),
            version: version.into(),
            notes: vec![],
            page: String::new(),
            package,
        };
        let run = |rel: Release| {
            let h = spawn_update(
                &state,
                cfg.clone(),
                rel,
                target.clone(),
                Arc::new(AtomicBool::new(false)),
            );
            h.join().unwrap();
            state.lock().unwrap().clone()
        };
        for version in ["1.0.0", "0.9.9"] {
            let s = run(release(
                version,
                Some(package(&package_name(version, LABEL))),
            ));
            assert_eq!(
                s.error.as_deref(),
                Some("refused: the release is not newer")
            );
            assert!(!s.busy && !s.restart_needed && s.signed.is_none());
            assert_eq!(
                s.steps,
                vec![
                    (Step::Download, StepState::Waiting, 0.0),
                    (Step::Verify, StepState::Waiting, 0.0),
                    (Step::Install, StepState::Waiting, 0.0),
                ]
            );
            let last = s.last.unwrap();
            assert_eq!(last.what, "failed");
            assert_eq!((last.from.as_str(), last.to.as_str()), ("1.0.0", version));
            assert_eq!(last.note, "refused: the release is not newer");
        }
        let s = run(release("2.0.0", None));
        assert_eq!(
            s.error.as_deref(),
            Some("the release has no package for this system")
        );
        let history = read_history(&target.history);
        assert_eq!(history.len(), 3);
        assert!(history.iter().all(|e| e.what == "failed"));
        assert_eq!(std::fs::read(&exe).unwrap(), b"OLD");
        assert!(!previous_path(&exe).exists());
        assert!(!target.work.exists());
    }

    #[test]
    fn a_look_for_a_release_without_a_system_label_fails_at_once_and_the_undo_tells_the_window_what_it_did(
    ) {
        let cfg = Config {
            api_url: API.into(),
            timeout_s: 1,
            require_signature: false,
        };
        let state: State = Arc::default();
        spawn_check(&state, cfg, None).join().unwrap();
        let s = state.lock().unwrap().clone();
        assert!(!s.busy && s.checked && s.release.is_none());
        assert_eq!(
            s.error.as_deref(),
            Some("the release has no package for this system")
        );
        assert_eq!(s.steps, vec![(Step::Ask, StepState::Failed, 0.0)]);
        assert_eq!(
            s.log,
            vec!["the release has no package for this system".to_string()]
        );
        // the undo
        let d = scratch("run-undo");
        let exe = d.join("bungard-gui");
        let target = Target {
            exe: exe.clone(),
            version: "1.0.0".into(),
            work: d.join("work"),
            history: d.join("h.log"),
            label: LABEL,
        };
        let state: State = Arc::default();
        std::fs::write(&exe, b"NEW").unwrap();
        let failed = run_undo(&state, &target);
        assert_eq!(
            failed,
            Err(Failure::Refused("there is no earlier version kept".into()))
        );
        let s = state.lock().unwrap().clone();
        assert_eq!(
            s.error.as_deref(),
            Some("refused: there is no earlier version kept")
        );
        assert!(!s.restart_needed);
        let last = s.last.unwrap();
        assert_eq!((last.what.as_str(), last.to.as_str()), ("failed", ""));
        std::fs::write(previous_path(&exe), b"OLD").unwrap();
        let mut version = previous_path(&exe).into_os_string();
        version.push(".version");
        std::fs::write(PathBuf::from(version), "0.9.0").unwrap();
        assert_eq!(run_undo(&state, &target), Ok("0.9.0".to_string()));
        let s = state.lock().unwrap().clone();
        assert!(s.restart_needed && s.error.is_none());
        assert_eq!(s.log, vec!["back to 0.9.0".to_string()]);
        let last = s.last.unwrap();
        assert_eq!(
            (last.what.as_str(), last.from.as_str(), last.to.as_str()),
            ("undo", "1.0.0", "0.9.0")
        );
        assert_eq!(std::fs::read(&exe).unwrap(), b"OLD");
        let rows = read_history(&target.history);
        assert_eq!(
            rows.iter().map(|e| e.what.as_str()).collect::<Vec<_>>(),
            ["failed", "undo"]
        );
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// sshsig (needs the feature of the updates)
// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "updates")]
mod sshsig_tests {
    use super::scratch;
    use eh3d_bungard_gui::sshsig::{
        verify_file, verify_with, MAX_SIGNATURE_BYTES, NAMESPACE, RELEASE_KEY,
    };
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use sha2::{Digest, Sha256, Sha512};
    use std::io::Read;

    fn ssh_string(out: &mut Vec<u8>, s: &[u8]) {
        out.extend_from_slice(&(s.len() as u32).to_be_bytes());
        out.extend_from_slice(s);
    }

    fn b64(data: &[u8]) -> String {
        const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut out = String::new();
        for chunk in data.chunks(3) {
            let n = match chunk.len() {
                3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32),
                2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
                _ => (chunk[0] as u32) << 16,
            };
            out.push(T[((n >> 18) & 63) as usize] as char);
            out.push(T[((n >> 12) & 63) as usize] as char);
            out.push(if chunk.len() > 1 {
                T[((n >> 6) & 63) as usize] as char
            } else {
                '='
            });
            out.push(if chunk.len() > 2 {
                T[(n & 63) as usize] as char
            } else {
                '='
            });
        }
        out
    }

    fn armor(raw: &[u8]) -> String {
        let mut out = String::from("-----BEGIN SSH SIGNATURE-----\n");
        for line in b64(raw).as_bytes().chunks(70) {
            out.push_str(std::str::from_utf8(line).unwrap());
            out.push('\n');
        }
        out.push_str("-----END SSH SIGNATURE-----\n");
        out
    }

    struct TestKey {
        pair: Ed25519KeyPair,
        blob: Vec<u8>,
    }

    impl TestKey {
        fn public_b64(&self) -> String {
            b64(&self.blob)
        }
    }

    fn key(seed: u8) -> TestKey {
        let pair = Ed25519KeyPair::from_seed_unchecked(&[seed; 32]).unwrap();
        let mut blob = Vec::new();
        ssh_string(&mut blob, b"ssh-ed25519");
        ssh_string(&mut blob, pair.public_key().as_ref());
        TestKey { pair, blob }
    }

    fn digest_of(hash: &str, data: &[u8]) -> Vec<u8> {
        match hash {
            "sha256" => Sha256::digest(data).to_vec(),
            _ => Sha512::digest(data).to_vec(),
        }
    }

    /// The 64 bytes of the signature of the data in a namespace.
    fn sign(key: &TestKey, namespace: &str, hash: &str, data: &[u8]) -> Vec<u8> {
        let mut message = b"SSHSIG".to_vec();
        ssh_string(&mut message, namespace.as_bytes());
        ssh_string(&mut message, b"");
        ssh_string(&mut message, hash.as_bytes());
        ssh_string(&mut message, &digest_of(hash, data));
        let sig = key.pair.sign(&message);
        let bytes: &[u8] = sig.as_ref();
        bytes.to_vec()
    }

    fn inner(kind: &[u8], sig: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        ssh_string(&mut out, kind);
        ssh_string(&mut out, sig);
        out
    }

    fn assemble(
        key_blob: &[u8],
        magic: &[u8],
        version: u32,
        namespace: &str,
        hash: &str,
        blob: &[u8],
    ) -> Vec<u8> {
        let mut raw = magic.to_vec();
        raw.extend_from_slice(&version.to_be_bytes());
        ssh_string(&mut raw, key_blob);
        ssh_string(&mut raw, namespace.as_bytes());
        ssh_string(&mut raw, b"");
        ssh_string(&mut raw, hash.as_bytes());
        ssh_string(&mut raw, blob);
        raw
    }

    fn good(key: &TestKey, hash: &str, data: &[u8]) -> Vec<u8> {
        let blob = inner(b"ssh-ed25519", &sign(key, NAMESPACE, hash, data));
        assemble(&key.blob, b"SSHSIG", 1, NAMESPACE, hash, &blob)
    }

    fn check(signature: &str, key_b64: &str, data: &[u8]) -> Result<(), String> {
        let mut input: &[u8] = data;
        verify_with(signature, key_b64, &mut input)
    }

    const DATA: &[u8] = b"release bytes\n";

    #[test]
    fn a_signature_of_the_namespace_of_the_releases_verifies_with_either_hash_and_only_for_its_bytes(
    ) {
        let k = key(7);
        for hash in ["sha256", "sha512"] {
            let sig = armor(&good(&k, hash, DATA));
            assert_eq!(check(&sig, &k.public_b64(), DATA), Ok(()), "{hash}");
            assert_eq!(
                check(&sig, &k.public_b64(), b"release bytes!"),
                Err("the signature does not match the package".to_string()),
                "{hash}"
            );
            assert!(check(&sig, &k.public_b64(), b"").is_err());
            let empty = armor(&good(&k, hash, b""));
            assert_eq!(check(&empty, &k.public_b64(), b""), Ok(()));
            assert!(check(&empty, &k.public_b64(), b"x").is_err());
        }
        // the same key and another namespace; another key; a hash that is not accepted
        let sig = armor(&assemble(
            &k.blob,
            b"SSHSIG",
            1,
            "git",
            "sha512",
            &inner(b"ssh-ed25519", &sign(&k, "git", "sha512", DATA)),
        ));
        assert_eq!(
            check(&sig, &k.public_b64(), DATA),
            Err("the signature was made for something else".to_string())
        );
        let other = key(8);
        assert_eq!(
            check(&armor(&good(&other, "sha512", DATA)), &k.public_b64(), DATA),
            Err("the signature was not made with the key of the releases".to_string())
        );
        let sha1 = armor(&assemble(
            &k.blob,
            b"SSHSIG",
            1,
            NAMESPACE,
            "sha1",
            &inner(b"ssh-ed25519", &sign(&k, NAMESPACE, "sha1", DATA)),
        ));
        assert_eq!(
            check(&sha1, &k.public_b64(), DATA),
            Err("the signature uses a hash that is not accepted".to_string())
        );
    }

    #[test]
    fn each_part_of_a_signature_that_is_wrong_is_refused_with_its_own_words() {
        let k = key(7);
        let sig_bytes = sign(&k, NAMESPACE, "sha512", DATA);
        let blob = inner(b"ssh-ed25519", &sig_bytes);
        let with = |raw: Vec<u8>| check(&armor(&raw), &k.public_b64(), DATA);
        let version = "not an OpenSSH signature of version 1".to_string();
        assert_eq!(
            with(assemble(&k.blob, b"SSHSIG", 2, NAMESPACE, "sha512", &blob)),
            Err(version.clone())
        );
        assert_eq!(
            with(assemble(&k.blob, b"SSHSIX", 1, NAMESPACE, "sha512", &blob)),
            Err(version.clone())
        );
        assert_eq!(with(Vec::new()), Err(version));
        let mut trailing = assemble(&k.blob, b"SSHSIG", 1, NAMESPACE, "sha512", &blob);
        trailing.push(0);
        let more = "the signature has more bytes than it should".to_string();
        assert_eq!(with(trailing), Err(more.clone()));
        let mut long_blob = blob.clone();
        long_blob.push(0);
        assert_eq!(
            with(assemble(
                &k.blob, b"SSHSIG", 1, NAMESPACE, "sha512", &long_blob
            )),
            Err(more)
        );
        assert_eq!(
            with(assemble(
                &k.blob,
                b"SSHSIG",
                1,
                NAMESPACE,
                "sha512",
                &inner(b"ssh-rsa", &sig_bytes)
            )),
            Err("the signature is not an Ed25519 signature".to_string())
        );
        let mut one_more = sig_bytes.clone();
        one_more.push(0);
        for wrong_size in [sig_bytes[..63].to_vec(), one_more] {
            assert_eq!(
                with(assemble(
                    &k.blob,
                    b"SSHSIG",
                    1,
                    NAMESPACE,
                    "sha512",
                    &inner(b"ssh-ed25519", &wrong_size)
                )),
                Err("the signature is wrong".to_string())
            );
        }
    }

    #[test]
    fn a_key_of_the_releases_that_is_not_an_ed25519_public_key_is_refused_before_the_signature_is_read(
    ) {
        let k = key(7);
        let sig = armor(&good(&k, "sha512", DATA));
        assert_eq!(
            check(&sig, "!!", DATA),
            Err("the key of the releases is not base64".to_string())
        );
        let mut rsa = Vec::new();
        ssh_string(&mut rsa, b"ssh-rsa");
        ssh_string(&mut rsa, &[1u8; 32]);
        assert_eq!(
            check(&sig, &b64(&rsa), DATA),
            Err("the key of the releases is not an Ed25519 key".to_string())
        );
        let mut short = Vec::new();
        ssh_string(&mut short, b"ssh-ed25519");
        ssh_string(&mut short, &[1u8; 31]);
        assert_eq!(
            check("not even a signature", &b64(&short), DATA),
            Err("the key of the releases is wrong".to_string())
        );
        assert_eq!(
            check(&sig, "", DATA),
            Err("the key of the releases is not an Ed25519 key".to_string())
        );
        // a key written with blanks inside is the same key
        let spaced = k.public_b64().replace("AAAA", "AAAA \n");
        assert_eq!(check(&sig, &spaced, DATA), Ok(()));
    }

    #[test]
    fn the_armor_may_have_blank_lines_indent_and_junk_after_it_but_not_a_missing_end_or_a_bad_body()
    {
        let k = key(7);
        let sig = armor(&good(&k, "sha256", DATA));
        let padded = format!("\n\n{}\ntrailing words\n", sig.replace('\n', "  \r\n   "));
        assert_eq!(check(&padded, &k.public_b64(), DATA), Ok(()));
        assert_eq!(
            check(
                &format!("{sig}garbage after the end\n"),
                &k.public_b64(),
                DATA
            ),
            Ok(())
        );
        let not_ssh = "not an OpenSSH signature".to_string();
        for text in [
            "",
            "   \n\n",
            "-----BEGIN PGP SIGNATURE-----\nAAAA\n-----END PGP SIGNATURE-----\n",
            "junk\n-----BEGIN SSH SIGNATURE-----\n",
        ] {
            assert_eq!(
                check(text, &k.public_b64(), DATA),
                Err(not_ssh.clone()),
                "{text:?}"
            );
        }
        let cut = sig.replace("-----END SSH SIGNATURE-----", "");
        assert_eq!(
            check(&cut, &k.public_b64(), DATA),
            Err("the signature is cut short".to_string())
        );
        let body = "-----BEGIN SSH SIGNATURE-----\n***\n-----END SSH SIGNATURE-----\n";
        assert_eq!(
            check(body, &k.public_b64(), DATA),
            Err("the signature is not base64".to_string())
        );
        let empty = "-----BEGIN SSH SIGNATURE-----\n-----END SSH SIGNATURE-----\n";
        assert_eq!(
            check(empty, &k.public_b64(), DATA),
            Err("not an OpenSSH signature of version 1".to_string())
        );
        assert!(sig.len() < MAX_SIGNATURE_BYTES as usize);
    }

    #[test]
    fn every_cut_of_a_signature_is_refused_and_only_the_whole_one_is_taken() {
        let k = key(7);
        for hash in ["sha256", "sha512"] {
            let raw = good(&k, hash, DATA);
            for cut in 0..raw.len() {
                assert!(
                    check(&armor(&raw[..cut]), &k.public_b64(), DATA).is_err(),
                    "{hash} cut at {cut}"
                );
            }
            assert_eq!(check(&armor(&raw), &k.public_b64(), DATA), Ok(()));
        }
    }

    #[test]
    fn no_single_changed_bit_of_a_signature_goes_unnoticed() {
        let k = key(7);
        let raw = good(&k, "sha512", DATA);
        for (at, _) in raw.iter().enumerate() {
            for bit in [0u8, 7] {
                let mut changed = raw.clone();
                changed[at] ^= 1 << bit;
                assert!(
                    check(&armor(&changed), &k.public_b64(), DATA).is_err(),
                    "byte {at} bit {bit}"
                );
            }
        }
    }

    #[test]
    fn the_bytes_are_hashed_as_a_stream_whatever_the_size_of_the_pieces_and_a_failure_of_the_reader_is_told(
    ) {
        struct OneByte<'a>(&'a [u8]);
        impl Read for OneByte<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                match self.0.split_first() {
                    Some((first, rest)) => {
                        buf[0] = *first;
                        self.0 = rest;
                        Ok(1)
                    }
                    None => Ok(0),
                }
            }
        }
        struct Broken;
        impl Read for Broken {
            fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("boom"))
            }
        }
        let k = key(7);
        let big = vec![b'a'; 3_000_000];
        let sig = armor(&good(&k, "sha256", &big));
        let mut stream = std::io::repeat(b'a').take(3_000_000);
        assert_eq!(verify_with(&sig, &k.public_b64(), &mut stream), Ok(()));
        let mut shorter = std::io::repeat(b'a').take(2_999_999);
        assert!(verify_with(&sig, &k.public_b64(), &mut shorter).is_err());
        let small: Vec<u8> = (0..20_000u32).map(|i| (i % 251) as u8).collect();
        let sig = armor(&good(&k, "sha512", &small));
        let mut slow = OneByte(&small);
        assert_eq!(verify_with(&sig, &k.public_b64(), &mut slow), Ok(()));
        assert_eq!(
            verify_with(&sig, &k.public_b64(), &mut Broken),
            Err("boom".to_string())
        );
    }

    #[test]
    fn the_file_check_uses_the_key_of_the_releases_and_the_constants_are_the_documented_ones() {
        let d = scratch("sshsig");
        let file = d.join("package.bin");
        std::fs::write(&file, DATA).unwrap();
        let k = key(7);
        let sig = armor(&good(&k, "sha512", DATA));
        assert_eq!(
            verify_file(&sig, &file),
            Err("the signature was not made with the key of the releases".to_string())
        );
        assert_eq!(
            verify_file("junk", &file),
            Err("not an OpenSSH signature".to_string())
        );
        assert!(!verify_file(&sig, &d.join("absent.bin"))
            .unwrap_err()
            .is_empty());
        assert_eq!(NAMESPACE, "eh3d-release");
        assert_eq!(MAX_SIGNATURE_BYTES, 4096);
        // the key of the releases is the base64 of "ssh-ed25519" and 32 bytes
        assert!(RELEASE_KEY.starts_with("AAAAC3NzaC1lZDI1NTE5AAAA"));
        assert_eq!(RELEASE_KEY.len() % 4, 0);
        assert!(!RELEASE_KEY.contains('='));
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
// updates_text (needs the feature of the updates)
// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg(feature = "updates")]
mod updates_text_tests {
    use eh3d_bungard_gui::updates_text::signature_state;
    use eh3d_view::texts::Lang;

    fn lang(code: &str) -> Lang {
        Lang::from_code(code).unwrap_or_else(|| panic!("{code} is not a language"))
    }

    #[test]
    fn the_two_phrases_of_the_main_languages_are_the_ones_written_and_each_one_is_its_own() {
        for (code, signed, unsigned) in [
            (
                "en",
                "signed by the release key",
                "not signed: only the checksum was checked",
            ),
            (
                "es",
                "firmada con la clave de la versión",
                "sin firma: solo se comprobó la suma",
            ),
            (
                "it",
                "firmato con la chiave della versione",
                "senza firma: è stata controllata solo la somma",
            ),
            (
                "de",
                "mit dem Schlüssel der Version signiert",
                "nicht signiert: nur die Prüfsumme wurde geprüft",
            ),
            (
                "pt",
                "assinada com a chave da versão",
                "sem assinatura: só se verificou a soma",
            ),
        ] {
            assert_eq!(signature_state(lang(code), true), signed, "{code}");
            assert_eq!(signature_state(lang(code), false), unsigned, "{code}");
        }
        assert!(signature_state(lang("fr"), true).contains("signé"));
        assert!(signature_state(lang("fr"), false).contains("non signée"));
        assert!(signature_state(lang("zh"), true).contains("签名"));
        assert!(signature_state(lang("ja"), false).contains("署名"));
        let main = ["en", "es", "it", "de", "fr", "pt", "zh", "ja"];
        for signed in [true, false] {
            let mut phrases: Vec<&str> = main
                .iter()
                .map(|c| signature_state(lang(c), signed))
                .collect();
            phrases.sort_unstable();
            phrases.dedup();
            assert_eq!(phrases.len(), main.len(), "{signed}");
        }
    }

    #[test]
    fn every_language_gives_a_clean_phrase_for_both_states() {
        for l in Lang::ALL {
            for signed in [true, false] {
                let p = signature_state(l, signed);
                assert!(!p.is_empty(), "{}", l.code());
                assert_eq!(p, p.trim(), "{}", l.code());
                assert!(!p.contains('\n'), "{}", l.code());
            }
            assert_ne!(
                signature_state(l, true),
                signature_state(l, false),
                "{}",
                l.code()
            );
        }
    }
}
