//! Behaviour checks of the parts of the job that need no window: the hand jog, the order of a job that drills and routes, what the bar under the views says,
//! the scene of a file, the starting values of the tools, the profile of the axes, the saved state of a job, the machine file and the width of the panes.

use eh3d_bungard_gui::boards::Core;
use eh3d_bungard_gui::catalogue::{all, Kind as BitKind};
use eh3d_bungard_gui::defaults::{drill, machining, material_factors, route, transfer, Machining};
use eh3d_bungard_gui::jobflow::{
    after_program, hand_over, plan_loads, slot_tools, start_decision, Next, Piece, Start,
};
use eh3d_bungard_gui::jobinfo::{
    advance_reached, clock, end_clock, locate, point_segment_dist, seconds_left, tool_stats,
};
use eh3d_bungard_gui::jobs::{
    read_latin1, scene_from_drill, scene_from_plot, table_from_config, tool_diameter, DEFAULT_TABLE,
};
use eh3d_bungard_gui::jog::{
    number, Axis, Config, Dir, Gate, Jog, Limits, Order, Pos, Refusal, Speed, DEFAULTS, SLOTS,
};
use eh3d_bungard_gui::launcher::Program;
use eh3d_bungard_gui::link;
use eh3d_bungard_gui::panes;
use eh3d_bungard_gui::profile::ProfileDims;
use eh3d_bungard_gui::recovery::{fingerprint, Recovery};
use eh3d_bungard_gui::spindle::{Spindle, SpindleId};
use eh3d_drillpro::ncd::{DrillFile, Hole as DrillHole};
use eh3d_drillpro::protocol::connection::{ConnectionConfig, Kind};
use eh3d_routepro::hpgl::{Plot, Polyline};
use eh3d_view::scene::{Hole, Path as ScenePath, Table};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A folder of its own for one test.
fn tmp(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("behaviour-jobs-{n}-{name}"));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// A small generator with a fixed seed: the same cases on every run.
struct Rng(u64);

impl Rng {
    fn draw(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0 >> 33
    }

    fn below(&mut self, n: u64) -> u64 {
        self.draw() % n
    }

    fn unit(&mut self) -> f64 {
        (self.draw() % 1_000_000) as f64 / 1_000_000.0
    }
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn nearf(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

// ------------------------------------------------------------------------------------------------------------- panes

#[test]
fn a_window_size_that_is_not_a_size_gives_the_least_board() {
    for (w, h) in [
        (f32::NAN, 900.0),
        (1600.0, f32::NAN),
        (f32::INFINITY, 900.0),
        (1600.0, f32::NEG_INFINITY),
        (0.0, 900.0),
        (1600.0, 0.0),
        (-1.0, 900.0),
        (1600.0, -1.0),
    ] {
        assert_eq!(panes::board_width_for(w, h), panes::BOARD_W, "{w} {h}");
    }
}

#[test]
fn the_board_follows_the_shape_of_the_window_between_its_two_limits() {
    assert_eq!(panes::board_width_for(800.0, 900.0), panes::BOARD_W);
    assert_eq!(panes::board_width_for(10_000.0, 900.0), panes::BOARD_W_MAX);
    assert!(nearf(panes::board_width_for(1600.0, 900.0), panes::BOARD_W));
    let mut last = 0.0;
    for w in (1000..=4000).step_by(100) {
        let b = panes::board_width_for(w as f32, 900.0);
        assert!((panes::BOARD_W..=panes::BOARD_W_MAX).contains(&b), "{w}");
        assert!(b >= last, "{w}");
        last = b;
    }
}

#[test]
fn a_drag_that_is_not_a_number_falls_back_to_the_default_panes() {
    let default = (panes::DEFAULT_LEFT, panes::DEFAULT_RIGHT);
    assert_eq!(panes::drag_left(280.0, 608.0, f32::NAN), default);
    assert_eq!(panes::drag_right(280.0, 608.0, f32::NAN), default);
    assert_eq!(
        panes::bars(f32::NAN, f32::NAN),
        panes::bars(panes::DEFAULT_LEFT, panes::DEFAULT_RIGHT)
    );
}

#[test]
fn the_pane_line_of_the_file_reads_the_odd_but_valid_forms_and_feeds_the_split() {
    assert_eq!(panes::parse("  4 , 5 "), Some((4.0, 5.0)));
    assert_eq!(panes::parse("1e2,2.5"), Some((100.0, 2.5)));
    assert_eq!(panes::parse(""), None);
    assert_eq!(panes::parse(","), None);
    assert_eq!(panes::parse("1,2,3"), None);
    let (a, b) = panes::parse("nan,inf").expect("both words are numbers");
    assert!(a.is_nan() && b.is_infinite());
    assert_eq!(
        panes::split(a, b),
        panes::split(panes::DEFAULT_LEFT, panes::DEFAULT_RIGHT)
    );
}

// ------------------------------------------------------------------------------------------------------------- recovery

fn minimal() -> Recovery {
    Recovery {
        program: "route".into(),
        route: Some(PathBuf::from("a.plt")),
        ..Recovery::default()
    }
}

fn full() -> Recovery {
    Recovery {
        program: "drill".into(),
        drill: Some(PathBuf::from("board.ncd")),
        route: Some(PathBuf::from("board.plt")),
        columns: 2,
        rows: 3,
        mode: "pitch".into(),
        spacing: (1.5, 2.25),
        tabs_on: true,
        tab_count: 4,
        tab_width: 2.5,
        tab_place: "middle".into(),
        done: 7,
        total: 9,
        drilled: vec![(1.0, 2.0), (3.5, 4.5)],
        queue: vec!["route".into()],
        off: vec![("drill".into(), 2), ("route".into(), 3)],
        fingerprint: "0123456789abcdef".into(),
    }
}

#[test]
fn the_text_of_a_bare_job_is_exactly_these_lines() {
    let text = minimal().to_text();
    assert_eq!(
        text,
        "program=route\nroute=a.plt\ncolumns=0\nrows=0\nmode=\nspacing=0,0\ntabs_on=false\ntab_count=0\ntab_width=0\ntab_place=\ndone=0\ntotal=0\nfingerprint=\n"
    );
    assert_eq!(Recovery::from_text(&text), Some(minimal()));
}

#[test]
fn the_holes_are_written_with_three_decimals_and_read_back_at_that_precision() {
    let mut r = minimal();
    r.drilled = vec![(1.23456, 2.0)];
    let text = r.to_text();
    assert!(text.contains("hole=1.235,2.000\n"), "{text}");
    let back = Recovery::from_text(&text).unwrap();
    assert_eq!(back.drilled, vec![(1.235, 2.0)]);
}

#[test]
fn twenty_thousand_holes_come_back_in_order() {
    let mut r = full();
    r.drilled = (0..20_000u32)
        .map(|i| (f64::from(i) * 0.125, f64::from(i % 97) * 0.5))
        .collect();
    r.total = 20_000;
    r.done = 20_000;
    assert_eq!(Recovery::from_text(&r.to_text()), Some(r));
}

#[test]
fn a_file_with_windows_line_ends_reads_the_same() {
    let text = full().to_text().replace('\n', "\r\n");
    assert_eq!(Recovery::from_text(&text), Some(full()));
}

#[test]
fn the_numbers_of_the_job_are_taken_up_to_their_type_and_not_one_more() {
    let head = "program=drill\ndrill=a\n";
    let max = Recovery::from_text(&format!("{head}columns=4294967295\n")).unwrap();
    assert_eq!(max.columns, u32::MAX);
    assert!(Recovery::from_text(&format!("{head}columns=4294967296\n")).is_none());
    assert!(Recovery::from_text(&format!("{head}rows=-1\n")).is_none());
    assert!(Recovery::from_text(&format!("{head}done=99999999999999999999999999\n")).is_none());
    assert!(Recovery::from_text(&format!("{head}tab_width=abc\n")).is_none());
    assert_eq!(
        Recovery::from_text(&format!("{head}tab_width=1e1\n"))
            .unwrap()
            .tab_width,
        10.0
    );
}

#[test]
fn the_tool_numbers_that_are_switched_off_are_held_to_a_byte() {
    let head = "program=drill\ndrill=a\n";
    let top = Recovery::from_text(&format!("{head}off=drill:255\n")).unwrap();
    assert_eq!(top.off, vec![("drill".to_string(), 255)]);
    assert!(Recovery::from_text(&format!("{head}off=drill:256\n")).is_none());
    assert!(Recovery::from_text(&format!("{head}off=drill\n")).is_none());
    // a program that is not one of the two is left out before its number is read
    let other = Recovery::from_text(&format!("{head}off=other:xyz\n")).unwrap();
    assert!(other.off.is_empty());
}

#[test]
fn a_hole_with_three_numbers_or_one_makes_the_file_no_job() {
    let head = "program=drill\ndrill=a\n";
    assert!(Recovery::from_text(&format!("{head}hole=1,2,3\n")).is_none());
    assert!(Recovery::from_text(&format!("{head}hole=1\n")).is_none());
    assert!(Recovery::from_text(&format!("{head}spacing=5\n")).is_none());
    let spaced = Recovery::from_text(&format!("{head}spacing= 5 , 7.5\n")).unwrap();
    assert_eq!(spaced.spacing, (5.0, 7.5));
}

#[test]
fn saving_in_a_folder_that_is_not_there_is_silent_and_leaves_nothing() {
    let dir = tmp("nofolder");
    let path = dir.join("missing").join("job.cfg");
    full().save(&path);
    assert!(!path.exists());
    assert!(Recovery::load(&path).is_none());
    Recovery::clear(&path);
}

#[test]
fn a_file_that_is_gone_counts_as_an_empty_one_and_the_name_does_not_count() {
    let dir = tmp("fingerprint");
    let empty = dir.join("empty.ncd");
    fs::write(&empty, "").unwrap();
    let gone = dir.join("gone.ncd");
    assert_eq!(
        fingerprint(&[empty.as_path()], "s"),
        fingerprint(&[gone.as_path()], "s")
    );
    let (one, two) = (dir.join("one.ncd"), dir.join("two.ncd"));
    fs::write(&one, "T01\nX1Y1\n").unwrap();
    fs::write(&two, "T01\nX1Y1\n").unwrap();
    assert_eq!(
        fingerprint(&[one.as_path()], "s"),
        fingerprint(&[two.as_path()], "s")
    );
}

#[test]
fn the_number_and_the_order_of_the_files_count_in_the_fingerprint() {
    let dir = tmp("order");
    let (a, b) = (dir.join("a.ncd"), dir.join("b.ncd"));
    fs::write(&a, "AAA").unwrap();
    fs::write(&b, "BBB").unwrap();
    let ab = fingerprint(&[a.as_path(), b.as_path()], "s");
    let ba = fingerprint(&[b.as_path(), a.as_path()], "s");
    assert_ne!(ab, ba);
    assert_ne!(fingerprint(&[], "s"), fingerprint(&[a.as_path()], "s"));
    assert_ne!(
        fingerprint(&[a.as_path()], "s"),
        fingerprint(&[a.as_path(), a.as_path()], "s")
    );
}

// ------------------------------------------------------------------------------------------------------------- jobflow

fn sizes(loads: &[Vec<Piece>]) -> Vec<usize> {
    loads
        .iter()
        .map(|l| l.iter().map(|(_, t)| t.len()).sum())
        .collect()
}

#[test]
fn a_rack_one_place_short_of_the_tools_takes_one_more_load() {
    let tools = [(Program::Drill, (1..=5u8).collect::<Vec<u8>>())];
    assert_eq!(sizes(&plan_loads(&tools, 4)), [4, 1]);
    assert_eq!(sizes(&plan_loads(&tools, 5)), [5]);
    assert_eq!(sizes(&plan_loads(&tools, 6)), [5]);
    assert_eq!(sizes(&plan_loads(&tools, 1)), [1, 1, 1, 1, 1]);
}

#[test]
fn the_routing_joins_the_last_drills_when_it_fits_with_them() {
    let tools = [
        (Program::Drill, (1..=5u8).collect::<Vec<u8>>()),
        (Program::Route, vec![9]),
    ];
    let loads = plan_loads(&tools, 2);
    assert_eq!(loads.len(), 3);
    assert_eq!(
        loads[2],
        vec![(Program::Drill, vec![5]), (Program::Route, vec![9])]
    );
}

#[test]
fn a_piece_that_does_not_fit_beside_the_one_before_starts_the_next_load() {
    let tools = [(Program::Drill, vec![1]), (Program::Route, vec![2, 3])];
    assert_eq!(
        plan_loads(&tools, 2),
        vec![
            vec![(Program::Drill, vec![1])],
            vec![(Program::Route, vec![2, 3])]
        ]
    );
    let fits = [(Program::Drill, vec![1, 2]), (Program::Route, vec![3])];
    assert_eq!(plan_loads(&fits, 3).len(), 1);
}

#[test]
fn all_the_tools_a_byte_can_name_fill_full_racks_of_fifteen() {
    let tools = [(Program::Drill, (1..=255u8).collect::<Vec<u8>>())];
    let loads = plan_loads(&tools, 15);
    assert_eq!(loads.len(), 17);
    assert!(sizes(&loads).iter().all(|&n| n == 15));
}

#[test]
fn the_slots_skip_a_program_with_no_tools_and_keep_counting() {
    let load = [
        (Program::Drill, vec![3]),
        (Program::Route, vec![]),
        (Program::Drill, vec![8, 9]),
    ];
    assert_eq!(
        slot_tools(&load),
        vec![
            (1, Program::Drill, 3),
            (2, Program::Drill, 8),
            (3, Program::Drill, 9)
        ]
    );
}

#[test]
fn a_start_with_nothing_on_never_says_outside_or_changer() {
    for file in [false, true] {
        for outside in [false, true] {
            for changer in [false, true] {
                let s = start_decision(true, file, outside, changer);
                assert_eq!(
                    s,
                    if file {
                        Start::NothingOn
                    } else {
                        Start::Silent
                    }
                );
            }
        }
    }
}

#[test]
fn a_start_with_programs_on_never_says_silent_and_outside_beats_the_changer() {
    for file in [false, true] {
        assert_eq!(start_decision(false, file, true, true), Start::Outside);
        assert_eq!(start_decision(false, file, true, false), Start::Outside);
        assert_eq!(start_decision(false, file, false, true), Start::Changer);
        assert_eq!(start_decision(false, file, false, false), Start::Plain);
    }
}

#[test]
fn what_follows_a_program_depends_only_on_the_counts_that_belong_to_the_mode() {
    for rack in [false, true] {
        for pieces in 0..3usize {
            for loads in 0..3usize {
                for queue in 0..3usize {
                    let next = after_program(rack, pieces, loads, queue);
                    if rack {
                        assert!(!matches!(next, Next::Program));
                        assert_eq!(next == Next::Over, pieces == 0 && loads == 0);
                        // the queue does not matter with the rack in use
                        assert_eq!(next, after_program(true, pieces, loads, 9));
                    } else {
                        assert!(matches!(next, Next::Program | Next::Over));
                        assert_eq!(next == Next::Over, queue == 0);
                        assert_eq!(next, after_program(false, 9, 9, queue));
                    }
                }
            }
        }
    }
}

#[test]
fn the_chuck_is_handed_on_along_a_chain_of_pieces() {
    let pieces: [Piece; 2] = [
        (Program::Drill, vec![1, 2]),
        (Program::Route, vec![7, 8, 9]),
    ];
    let (first, chuck) = hand_over(0, None);
    assert_eq!((first, chuck), (0, None));
    let (first, chuck) = hand_over(first, Some(&pieces[0]));
    assert_eq!((first, chuck), (2, Some((Program::Drill, 2))));
    let (first, chuck) = hand_over(first, Some(&pieces[1]));
    assert_eq!((first, chuck), (5, Some((Program::Route, 9))));
}

// ------------------------------------------------------------------------------------------------------------- jobs

fn drill_file(holes: &[(u8, i32, i32)], declared: &[(u8, f64)], metric: bool) -> DrillFile {
    DrillFile {
        holes: holes
            .iter()
            .map(|&(tool, x, y)| DrillHole {
                tool,
                x_mil: x,
                y_mil: y,
            })
            .collect(),
        diameters: declared.to_vec(),
        metric,
        ..DrillFile::default()
    }
}

#[test]
fn nothing_that_is_not_a_configuration_changes_the_table() {
    let noise = "\0\u{1}garbage\n".repeat(10_000);
    for text in ["", " ", "\n\n", "x=1", noise.as_str()] {
        assert_eq!(table_from_config(text, false), DEFAULT_TABLE);
        assert_eq!(table_from_config(text, true), DEFAULT_TABLE);
    }
}

#[test]
fn every_tool_number_has_a_diameter_that_grows_with_it_from_four_on() {
    for tool in 0..=255u8 {
        let d = tool_diameter(tool, &[], true);
        if tool < 4 {
            assert_eq!(d, 1.0, "{tool}");
        } else {
            assert!(near(d, f64::from(tool) / 10.0), "{tool}");
        }
        assert!(d > 0.0);
    }
    // the unit of the file does not matter when the header says nothing
    assert_eq!(tool_diameter(30, &[], false), tool_diameter(30, &[], true));
}

#[test]
fn the_holes_keep_their_order_their_tools_and_their_coordinates() {
    let file = drill_file(
        &[(2, 1000, 2000), (1, -50, 0), (2, 7, 8)],
        &[(1, 0.8)],
        true,
    );
    let table = Table::new(100.0, 50.0);
    let s = scene_from_drill(&file, table, 1.0);
    assert_eq!(s.table, table);
    assert!(s.paths.is_empty());
    let got: Vec<(u8, f64, f64)> = s.holes.iter().map(|h| (h.tool, h.x_mm, h.y_mm)).collect();
    assert_eq!(got, [(2, 1000.0, 2000.0), (1, -50.0, 0.0), (2, 7.0, 8.0)]);
    assert!(s.holes.iter().all(|h| !h.done));
    assert_eq!(s.holes[1].diameter_mm, 0.8);
    assert_eq!(s.holes[0].diameter_mm, 1.0);
}

#[test]
fn a_unit_of_nothing_puts_every_hole_at_the_origin_and_a_huge_mil_count_stays_finite() {
    let file = drill_file(&[(1, 500, -500), (1, i32::MAX, i32::MIN)], &[], true);
    let flat = scene_from_drill(&file, DEFAULT_TABLE, 0.0);
    assert!(flat.holes.iter().all(|h| h.x_mm == 0.0 && h.y_mm == 0.0));
    let big = scene_from_drill(&file, DEFAULT_TABLE, 0.0254);
    assert!(big
        .holes
        .iter()
        .all(|h| h.x_mm.is_finite() && h.y_mm.is_finite()));
    assert!(big.holes[1].x_mm > 0.0 && big.holes[1].y_mm < 0.0);
}

#[test]
fn the_paths_keep_their_pen_their_order_and_their_empty_ones() {
    let plot = Plot {
        paths: vec![
            Polyline {
                pen: 3,
                points: vec![(40, 0), (0, 40)],
            },
            Polyline {
                pen: 1,
                points: Vec::new(),
            },
            Polyline {
                pen: 3,
                points: vec![(-40, 80)],
            },
        ],
        ignored: 7,
    };
    let s = scene_from_plot(&plot, DEFAULT_TABLE);
    assert!(s.holes.is_empty());
    let pens: Vec<u8> = s.paths.iter().map(|p| p.tool).collect();
    assert_eq!(pens, [3, 1, 3]);
    assert!(s.paths.iter().all(|p| p.reached == 0));
    assert!(s.paths[1].points.is_empty());
    assert!(near(s.paths[0].points[0].0, 1.0) && near(s.paths[0].points[1].1, 1.0));
    assert!(near(s.paths[2].points[0].0, -1.0) && near(s.paths[2].points[0].1, 2.0));
}

#[test]
fn holes_and_paths_together_count_in_one_progress() {
    let file = drill_file(&[(1, 0, 0), (1, 100, 0)], &[], true);
    let mut s = scene_from_drill(&file, DEFAULT_TABLE, 0.0254);
    let plot = Plot {
        paths: vec![Polyline {
            pen: 1,
            points: vec![(0, 0), (40, 0)],
        }],
        ignored: 0,
    };
    s.overlay(&scene_from_plot(&plot, DEFAULT_TABLE));
    assert_eq!(s.progress(), 0.0);
    s.holes[0].done = true;
    s.paths[0].reached = 2;
    assert!(near(s.progress(), 0.75));
}

#[test]
fn a_file_that_is_not_there_is_not_found_and_an_empty_one_is_empty() {
    let dir = tmp("latin1");
    let err = read_latin1(&dir.join("none.txt")).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    let empty = dir.join("empty.txt");
    fs::write(&empty, "").unwrap();
    assert_eq!(read_latin1(&empty).unwrap(), "");
}

// ------------------------------------------------------------------------------------------------------------- jobinfo

fn hole(tool: u8, d: f64, done: bool) -> Hole {
    Hole {
        x_mm: 0.0,
        y_mm: 0.0,
        diameter_mm: d,
        tool,
        done,
    }
}

fn path(tool: u8, n: usize, reached: usize) -> ScenePath {
    ScenePath {
        tool,
        points: vec![(0.0, 0.0); n],
        reached,
    }
}

#[test]
fn a_drill_and_a_cutter_of_the_same_number_are_two_tools_and_the_drills_come_first() {
    let s = tool_stats(&[hole(5, 0.5, true)], &[path(5, 3, 1)]);
    assert_eq!(s.len(), 2);
    assert!(s[0].drill && !s[1].drill);
    assert_eq!((s[0].done, s[0].total), (1, 1));
    assert_eq!((s[1].done, s[1].total), (1, 3));
    assert_eq!(s[1].diameter_mm, 0.0);
}

#[test]
fn a_path_without_points_is_a_cutter_with_nothing_to_do_and_never_finished() {
    let s = tool_stats(&[], &[path(2, 0, 0)]);
    assert_eq!(s.len(), 1);
    assert_eq!((s[0].done, s[0].total), (0, 0));
    assert!(!s[0].finished());
    let s = tool_stats(&[], &[path(2, 0, 0), path(2, 4, 4)]);
    assert_eq!((s[0].done, s[0].total), (4, 4));
    assert!(s[0].finished());
}

#[test]
fn a_tool_the_program_does_not_have_falls_back_to_the_first_unfinished_cutter() {
    let s = tool_stats(&[], &[path(2, 5, 1)]);
    let w = locate(&s, Some(2), true);
    assert_eq!(w.current.map(|c| (c.tool, c.drill)), Some((2, false)));
}

#[test]
fn the_next_tool_is_never_searched_before_the_current_one() {
    let s = tool_stats(
        &[hole(1, 0.8, false), hole(2, 1.0, false), hole(3, 1.2, true)],
        &[],
    );
    let w = locate(&s, Some(2), true);
    assert_eq!(w.current.map(|c| c.tool), Some(2));
    // tool 3 is finished and tool 1 is before: nothing comes next
    assert_eq!(w.next, None);
}

#[test]
fn the_time_left_waits_for_a_fiftieth_of_the_work_and_ends_with_the_work() {
    assert_eq!(seconds_left(1.0, 0.02), Some(49));
    assert_eq!(seconds_left(1.0, 0.0199), None);
    assert_eq!(seconds_left(1000.0, 0.5), Some(1000));
    assert_eq!(seconds_left(1000.0, 0.99), Some(10));
    assert_eq!(seconds_left(1000.0, 1.0), None);
    assert_eq!(seconds_left(1000.0, 1.5), None);
    assert_eq!(seconds_left(-5.0, 0.5), None);
    assert_eq!(seconds_left(f64::INFINITY, 0.5), None);
    assert_eq!(seconds_left(10.0, f64::NAN), None);
    assert_eq!(seconds_left(10.0, -0.5), None);
}

#[test]
fn the_clock_changes_its_shape_at_the_hour() {
    assert_eq!(clock(59), "0:59");
    assert_eq!(clock(60), "1:00");
    assert_eq!(clock(599), "9:59");
    assert_eq!(clock(3599), "59:59");
    assert_eq!(clock(3600), "1:00:00");
    assert_eq!(clock(86_399), "23:59:59");
    assert_eq!(clock(86_400), "24:00:00");
    assert_eq!(clock(360_000), "100:00:00");
    assert!(clock(u64::MAX).contains(':'));
}

#[test]
fn the_hour_of_the_end_is_worked_out_for_a_moment_far_from_the_epoch() {
    // 1.7 billion seconds after 1970 is 22:13:20 universal time
    assert_eq!(end_clock(1_700_000_000, 0, 0), "22:13");
    assert_eq!(end_clock(1_700_000_000, 40, 0), "22:14");
    assert_eq!(end_clock(1_700_000_000, 0, 120), "00:13");
    assert_eq!(end_clock(0, 0, -60), "23:00");
}

#[test]
fn the_hour_of_the_end_is_always_a_valid_hour_and_moves_a_minute_for_sixty_seconds() {
    let mut rng = Rng(0xC10C);
    let minutes = |s: &str| -> u32 {
        let (h, m) = s.split_once(':').expect("a colon");
        assert_eq!((h.len(), m.len()), (2, 2), "{s}");
        let (h, m): (u32, u32) = (h.parse().unwrap(), m.parse().unwrap());
        assert!(h < 24 && m < 60, "{s}");
        h * 60 + m
    };
    for _ in 0..500 {
        let now = rng.below(4_000_000_000);
        let left = rng.below(200_000);
        let offset = rng.below(1681) as i32 - 840;
        let a = minutes(&end_clock(now, left, offset));
        let b = minutes(&end_clock(now + 60, left, offset));
        assert_eq!((b + 1440 - a) % 1440, 1, "{now} {left} {offset}");
    }
}

#[test]
fn the_distance_to_a_segment_does_not_depend_on_which_end_is_called_first() {
    let mut rng = Rng(0xD157);
    for _ in 0..300 {
        let mut pt = || (rng.unit() * 20.0 - 10.0, rng.unit() * 20.0 - 10.0);
        let (p, a, b) = (pt(), pt(), pt());
        let d = point_segment_dist(p, a, b);
        assert!(near(d, point_segment_dist(p, b, a)));
        let to_a = (p.0 - a.0).hypot(p.1 - a.1);
        let to_b = (p.0 - b.0).hypot(p.1 - b.1);
        assert!(d <= to_a.min(to_b) + 1e-9);
        assert!(d >= 0.0);
    }
}

#[test]
fn whatever_way_the_tool_wanders_a_path_only_goes_forward_and_never_past_its_end() {
    let mut rng = Rng(0xFEED);
    let mut q = ScenePath {
        tool: 1,
        points: (0..6)
            .map(|_| (rng.unit() * 10.0, rng.unit() * 10.0))
            .collect(),
        reached: 0,
    };
    for _ in 0..400 {
        let at = (rng.unit() * 10.0, rng.unit() * 10.0);
        let before = q.reached;
        advance_reached(&mut q, at, 0.5);
        assert!(q.reached >= before);
        assert!(q.reached <= q.points.len());
    }
}

// ------------------------------------------------------------------------------------------------------------- defaults

#[test]
fn the_materials_of_one_family_ask_the_same_of_the_tools() {
    assert_eq!(machining(Core::Fr2), machining(Core::Fr3));
    assert_eq!(machining(Core::Fr3), machining(Core::Cem1));
    assert_eq!(machining(Core::Cem3), machining(Core::Fr4));
    assert_eq!(machining(Core::Fr4), machining(Core::Polyimide));
    assert_eq!(machining(Core::Hydrocarbon), machining(Core::Ptfe));
    assert_ne!(machining(Core::Fr2), machining(Core::Fr4));
}

#[test]
fn the_table_of_the_materials_has_these_values_for_the_ones_that_matter() {
    assert_eq!(
        machining(Core::Fr4),
        Machining {
            rpm_factor: 1.0,
            feed_factor: 1.0,
            pass_mm: 0.0
        }
    );
    assert_eq!(
        machining(Core::Wood),
        Machining {
            rpm_factor: 1.0,
            feed_factor: 1.5,
            pass_mm: 1.5
        }
    );
    assert_eq!(
        machining(Core::Copper),
        Machining {
            rpm_factor: 0.3,
            feed_factor: 0.5,
            pass_mm: 0.2
        }
    );
    assert_eq!(material_factors(Core::Ceramic), (0.3, 0.2));
}

#[test]
fn a_drill_of_fr4_changes_its_numbers_exactly_at_the_edges_of_the_sizes() {
    let cases = [
        (0.0, (0.5, 1_500, 60_000)),
        (0.4999, (0.5, 1_500, 60_000)),
        (0.5, (0.5, 2_400, 60_000)),
        (0.7999, (0.5, 2_400, 60_000)),
        (0.8, (0.5, 3_600, 60_000)),
        (1.1999, (0.5, 3_600, 60_000)),
        (1.2, (0.6, 2_400, 40_000)),
        (1.9999, (0.6, 2_400, 40_000)),
        (2.0, (0.8, 1_500, 30_000)),
        (2.9999, (0.8, 1_500, 30_000)),
        (3.0, (1.0, 800, 20_000)),
        (10.0, (1.0, 800, 20_000)),
    ];
    for (d, want) in cases {
        assert_eq!(drill(d, Core::Fr4), want, "{d}");
    }
}

#[test]
fn a_diameter_that_is_not_a_size_takes_one_of_the_ends_and_does_not_panic() {
    assert_eq!(drill(f64::NAN, Core::Fr4), (1.0, 800, 20_000));
    assert_eq!(drill(f64::INFINITY, Core::Fr4), (1.0, 800, 20_000));
    assert_eq!(drill(-1.0, Core::Fr4), (0.5, 1_500, 60_000));
    assert_eq!(drill(f64::NEG_INFINITY, Core::Fr4), (0.5, 1_500, 60_000));
}

#[test]
fn the_material_scales_the_feed_and_the_turns_of_a_drill_to_round_figures() {
    assert_eq!(drill(0.3, Core::Ceramic), (0.5, 300, 18_000));
    assert_eq!(drill(1.0, Core::Wood), (0.5, 5_400, 60_000));
    assert_eq!(drill(3.0, Core::Copper), (1.0, 400, 6_000));
    assert_eq!(drill(1.5, Core::SoftPlastic), (0.6, 3_600, 24_000));
    assert_eq!(drill(0.6, Core::Plastic), (0.5, 2_900, 48_000));
}

#[test]
fn the_figures_of_a_drill_are_multiples_of_fifty_and_a_thousand_for_every_material_and_size() {
    let cores = [
        Core::Fr2,
        Core::Fr3,
        Core::Cem1,
        Core::Cem3,
        Core::Fr4,
        Core::Hydrocarbon,
        Core::Ptfe,
        Core::Polyimide,
        Core::Ceramic,
        Core::Aluminium,
        Core::Flex,
        Core::Metal,
        Core::Copper,
        Core::Wood,
        Core::Plastic,
        Core::SoftPlastic,
    ];
    for core in cores {
        for k in 1..=70 {
            let d = f64::from(k) * 0.1;
            let (extra, feed, rpm) = drill(d, core);
            assert!(extra > 0.0);
            assert_eq!(feed % 50, 0, "{core:?} {d}");
            assert_eq!(rpm % 1_000, 0, "{core:?} {d}");
            assert!(feed >= 50 && rpm >= 1_000, "{core:?} {d}");
        }
    }
}

#[test]
fn a_router_of_fr4_needs_one_more_pass_after_each_whole_millimetre() {
    for (t, passes) in [(0.0, 1), (1.0, 1), (1.0001, 2), (2.0, 2), (2.0001, 3)] {
        assert_eq!(route(t, Core::Fr4).1, passes, "{t}");
    }
    let (depth, _, xy, z, rpm) = route(1.6, Core::Fr4);
    assert!(near(depth, 1.9));
    assert_eq!((xy, z, rpm), (800, 300, 60_000));
}

#[test]
fn a_board_of_no_thickness_or_a_wrong_one_still_gets_a_pass() {
    assert_eq!(route(0.0, Core::Fr4).1, 1);
    assert!(near(route(0.0, Core::Fr4).0, 0.3));
    assert_eq!(route(-1.0, Core::Fr4).1, 1);
    let nan = route(f64::NAN, Core::Fr4);
    assert_eq!(nan.1, 1);
    assert!(nan.0.is_nan());
}

#[test]
fn the_material_scales_the_feeds_and_the_turns_of_a_router_to_round_figures() {
    assert_eq!(route(1.6, Core::Ceramic).2, 150);
    assert_eq!(route(1.6, Core::Ceramic).3, 50);
    assert_eq!(route(1.6, Core::Ceramic).4, 18_000);
    let wood = route(1.6, Core::Wood);
    assert_eq!((wood.2, wood.3, wood.4), (1_200, 450, 60_000));
    let metal = route(1.6, Core::Metal);
    assert_eq!((metal.2, metal.3, metal.4), (300, 100, 24_000));
}

fn a_drill(d: f64) -> eh3d_bungard_gui::catalogue::Bit {
    *all()
        .iter()
        .find(|b| b.kind.is_drill() && (b.diameter_mm - d).abs() < 1e-9 && b.flute_mm > 0.0)
        .expect("a drill of the size")
}

fn a_router(d: f64) -> eh3d_bungard_gui::catalogue::Bit {
    *all()
        .iter()
        .find(|b| b.kind == BitKind::Diamond && (b.diameter_mm - d).abs() < 1e-9)
        .expect("a router of the size")
}

fn original() -> Spindle {
    Spindle::default_for(SpindleId::Original)
}

#[test]
fn the_depth_of_a_drill_is_the_board_and_the_extra_stroke_and_never_less_than_the_board() {
    let b = a_drill(1.0);
    let t = transfer(&b, &original(), Core::Fr4, 1.6, 0.5).unwrap();
    assert!(near(t.depth_mm, 2.1));
    assert_eq!(t.passes, None);
    let t = transfer(&b, &original(), Core::Fr4, 1.6, -2.0).unwrap();
    assert!(near(t.depth_mm, 1.6));
}

#[test]
fn a_router_goes_at_least_through_the_board_and_the_extra_and_keeps_a_deeper_cut() {
    let mut r = a_router(2.0);
    r.pass_mm = 0.5;
    r.flute_mm = 8.0;
    let shallow = transfer(&r, &original(), Core::Fr4, 1.6, 0.0).unwrap();
    assert!(near(shallow.depth_mm, 1.9));
    let deep = transfer(&r, &original(), Core::Fr4, 1.6, 2.4).unwrap();
    assert_eq!(deep.depth_mm, 2.4);
    assert!(deep.passes.unwrap() >= shallow.passes.unwrap());
}

// ------------------------------------------------------------------------------------------------------------- profile

fn bounds(p: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    p.iter()
        .fold((f64::MAX, f64::MAX, f64::MIN, f64::MIN), |b, &(x, y)| {
            (b.0.min(x), b.1.min(y), b.2.max(x), b.3.max(y))
        })
}

#[test]
fn the_line_of_the_settings_has_the_twelve_numbers_in_this_form() {
    assert_eq!(
        ProfileDims::default().to_text(),
        "80,40,8,12.25,6.8,40,3.04,16.6,69.54,8.3,17.38,11.38"
    );
}

#[test]
fn numbers_that_are_not_round_come_back_the_same() {
    let d = ProfileDims {
        width_mm: 90.123456789,
        height_mm: 45.5,
        slot_mm: 0.1 + 0.2,
        slot_edge_mm: 10.0 / 3.0,
        mass_kg_m: 1e-3,
        ix_cm4: 123456.789,
        ..ProfileDims::default()
    };
    assert_eq!(ProfileDims::from_text(&d.to_text()), d);
}

#[test]
fn a_number_that_is_zero_negative_or_not_finite_keeps_the_starting_value_of_its_place() {
    assert_eq!(
        ProfileDims::from_text("0,-5,nan,inf"),
        ProfileDims::default()
    );
    let d = ProfileDims::from_text("90,-1,8");
    assert_eq!((d.width_mm, d.height_mm), (90.0, 40.0));
    assert_eq!(ProfileDims::from_text(""), ProfileDims::default());
}

#[test]
fn the_short_side_of_ten_millimetres_is_the_least_that_can_be_drawn() {
    let ok = ProfileDims::from_text("80,10,4,12.25,2,40");
    assert_eq!((ok.width_mm, ok.height_mm, ok.slot_mm), (80.0, 10.0, 4.0));
    assert_eq!(
        ProfileDims::from_text("80,9.99,4,12.25,2,40"),
        ProfileDims::default()
    );
}

#[test]
fn the_two_slots_of_a_long_face_and_their_edges_must_fit_in_the_long_side() {
    assert_eq!(ProfileDims::from_text("80,40,8,31.99").slot_edge_mm, 31.99);
    assert_eq!(ProfileDims::from_text("80,40,8,32"), ProfileDims::default());
}

#[test]
fn the_sections_of_sane_measures_have_the_parts_they_should_and_the_size_they_say() {
    let mut rng = Rng(0x5EC7);
    for _ in 0..100 {
        let d = ProfileDims {
            width_mm: 60.0 + rng.unit() * 60.0,
            height_mm: 20.0 + rng.unit() * 30.0,
            slot_mm: 4.0 + rng.unit() * 4.0,
            bore_mm: 3.0 + rng.unit() * 5.0,
            bore_pitch_mm: 20.0 + rng.unit() * 20.0,
            ..ProfileDims::default()
        };
        let s = d.section();
        assert_eq!((s.outline.len(), s.slots.len(), s.bores.len()), (4, 6, 2));
        assert_eq!(s.cavity.len(), 8);
        assert!(s.slots.iter().all(|p| p.len() == 8));
        let (x0, y0, x1, y1) = bounds(&s.outline);
        assert!(near(x1 - x0, d.width_mm) && near(y1 - y0, d.height_mm));
        assert!(s
            .slots
            .iter()
            .flatten()
            .chain(s.outline.iter())
            .chain(s.cavity.iter())
            .all(|p| p.0.is_finite() && p.1.is_finite()));
    }
}

#[test]
fn turning_the_section_four_times_gives_it_back_and_twice_flips_it_through_the_centre() {
    let s = ProfileDims::default().section();
    assert_eq!(s.turned().turned().turned().turned(), s);
    let half = s.turned().turned();
    for (a, b) in s.outline.iter().zip(&half.outline) {
        assert!(near(a.0, -b.0) && near(a.1, -b.1));
    }
}

// ------------------------------------------------------------------------------------------------------------- link

#[test]
fn a_connection_that_is_not_in_use_writes_only_the_seven_common_keys() {
    let p = link::pairs(&ConnectionConfig::default());
    let keys: Vec<&str> = p.iter().map(|(k, _)| *k).collect();
    assert_eq!(
        keys,
        [
            "connection",
            "rs232_port",
            "usb_port",
            "sound_finish",
            "sound_tool_change",
            "sound_check_ok",
            "sound_check_bad"
        ]
    );
    assert_eq!(p[0].1, Kind::Demo.name());
}

#[test]
fn the_node_and_the_server_add_their_own_keys_only_when_they_are_used() {
    let mut c = ConnectionConfig {
        kind: Kind::Node,
        ..ConnectionConfig::default()
    };
    let keys = |c: &ConnectionConfig| -> Vec<&'static str> {
        link::pairs(c).iter().map(|(k, _)| *k).collect()
    };
    assert!(keys(&c).contains(&"node_address") && !keys(&c).contains(&"server_address"));
    c.kind = Kind::Server;
    assert!(keys(&c).contains(&"server_machine") && !keys(&c).contains(&"node_token"));
    // an address that is already there is kept in the file even when the kind is another
    c.kind = Kind::Usb;
    c.server_address = "10.0.0.1:1".into();
    assert!(keys(&c).contains(&"server_address"));
}

#[test]
fn a_key_that_is_written_twice_is_written_once_in_the_place_of_the_first() {
    let out = link::update(
        "usb_port = A\nx=1\nusb_port = B\n",
        &[("usb_port", "C".to_string())],
    );
    assert_eq!(out, "usb_port = C\nx=1\n");
}

#[test]
fn the_name_of_the_key_is_not_case_sensitive_in_the_file_and_the_spaces_go() {
    assert_eq!(
        link::update("USB_PORT = A\n", &[("usb_port", "Z".to_string())]),
        "usb_port = Z\n"
    );
    assert_eq!(
        link::update("   usb_port   =   A   \n", &[("usb_port", "Z".to_string())]),
        "usb_port = Z\n"
    );
}

#[test]
fn a_key_in_a_comment_is_not_the_key() {
    let out = link::update(
        "# usb_port = A\n; usb_port = B\nx=1\n",
        &[("usb_port", "Z".to_string())],
    );
    assert_eq!(out, "# usb_port = A\n; usb_port = B\nx=1\nusb_port = Z\n");
}

#[test]
fn writing_the_same_keys_twice_changes_nothing() {
    let pairs = [
        ("usb_port", "COM8".to_string()),
        ("connection", "usb".to_string()),
    ];
    let old = "# mine\nconnection = demo\nx_search_hz = 2700\n\nusb_port = COM3\n";
    let once = link::update(old, &pairs);
    assert_eq!(link::update(&once, &pairs), once);
    assert!(once.contains("# mine\n") && once.contains("x_search_hz = 2700\n"));
}

// ------------------------------------------------------------------------------------------------------------- jog

const OPEN: Gate = Gate {
    job_running: false,
    estop: false,
};

fn limits() -> Limits {
    Limits::new(500.0, 300.0, 24.0, 4.0, 0.0)
}

fn jog_with(text: &str) -> Jog {
    Jog::new(Config::parse(text), limits())
}

fn only(r: Result<Vec<Order>, Refusal>) -> Order {
    let v = r.expect("the move was refused");
    assert_eq!(v.len(), 1, "{v:?}");
    v[0]
}

#[test]
fn nothing_but_comments_and_blank_lines_is_the_starting_values() {
    assert_eq!(Config::parse(""), Config::defaults());
    assert_eq!(Config::parse(DEFAULTS), Config::defaults());
    assert_eq!(
        Config::parse("# just a note\n\n   \n# steps = 3\nno equals\n"),
        Config::defaults()
    );
}

#[test]
fn the_list_of_steps_takes_eight_numbers_and_five_hundred_and_not_one_more() {
    let defaults = Config::defaults();
    assert_eq!(Config::parse("steps = 1,2,3,4,5,6,7,8").steps.len(), 8);
    assert_eq!(
        Config::parse("steps = 1,2,3,4,5,6,7,8,9").steps,
        defaults.steps
    );
    assert_eq!(Config::parse("steps = 500").steps, vec![500.0]);
    assert_eq!(Config::parse("steps = 500.01").steps, defaults.steps);
    assert_eq!(Config::parse("steps = 0.0001").steps, vec![0.0001]);
}

#[test]
fn a_stretch_of_a_held_arrow_is_never_longer_than_fifty_millimetres() {
    assert_eq!(Config::parse("chunk_mm = 49.5").chunk_mm, 49.5);
    assert_eq!(Config::parse("chunk_mm = 50").chunk_mm, 50.0);
    assert_eq!(Config::parse("chunk_mm = 51").chunk_mm, 50.0);
    assert_eq!(Config::parse("chunk_mm = 1000").chunk_mm, 50.0);
    assert_eq!(Config::parse("chunk_mm = 0").chunk_mm, 2.0);
}

#[test]
fn the_words_of_the_speed_ignore_case_and_the_names_of_the_keys_do_not() {
    assert_eq!(Config::parse("speed = FAST").speed, Speed::Fast);
    assert_eq!(Config::parse("speed =  Slow ").speed, Speed::Slow);
    assert_eq!(Config::parse("Speed = fast").speed, Speed::Medium);
    assert!(!Config::parse("continuous = TRUE").continuous);
}

#[test]
fn a_place_is_set_cleared_and_kept_when_the_new_one_is_wrong() {
    let c = Config::parse("slot2 = 1,2,3\nslot2 =\nslot1 = 4,5,6\nslot1 = x\nslot4 = 7, 8, 9");
    assert_eq!(c.slots[1], None);
    assert_eq!(
        c.slots[0],
        Some(Pos {
            x: 4.0,
            y: 5.0,
            z: 6.0
        })
    );
    assert_eq!(c.slots[3].map(|p| p.z), Some(9.0));
    // there are four places and they are numbered from one
    let none = Config::parse("slot0 = 1,2,3\nslot5 = 1,2,3");
    assert_eq!(none.slots, [None; SLOTS]);
}

#[test]
fn the_file_of_the_starting_values_is_written_in_exactly_this_form() {
    assert_eq!(
        Config::defaults().to_text(),
        "steps = 0.01, 0.1, 1, 10, 50\nstep = 1\ncontinuous = false\nspeed_slow = 300\nspeed_medium = 1200\nspeed_fast = 3000\nspeed = medium\nchunk_mm = 2\nclearance_mm = 5\nbelow_board_mm = 0\npanel = 24, 96\ndock = false\norigin = 0, 0, 0\nslot1 =\nslot2 =\nslot3 =\nslot4 =\n"
    );
}

#[test]
fn what_is_written_is_read_back_whole_and_written_again_the_same() {
    let cfg = Config::parse(
        "steps = 0.05, 0.5, 5\nstep = 0.5\ncontinuous = true\nspeed_slow = 123.4567\nspeed = fast\nchunk_mm = 3.25\nclearance_mm = 2.5\nbelow_board_mm = 0.75\npanel = 10.5, 20.25\ndock = true\norigin = 1.5, -2.25, 3\nslot1 = 1, 2, 3\nslot4 = -1.5, 0, 9\n",
    );
    let text = cfg.to_text();
    let back = Config::parse(&text);
    assert_eq!(back, cfg);
    assert_eq!(back.to_text(), text);
}

#[test]
fn saving_makes_the_folders_replaces_the_old_file_and_leaves_no_side_file() {
    let dir = tmp("jog-save");
    let path = dir.join("deeper").join("still").join("jog.cfg");
    let mut cfg = Config::defaults();
    cfg.save(&path).unwrap();
    assert_eq!(Config::load(&path), cfg);
    cfg.dock = true;
    cfg.chunk_mm = 7.5;
    cfg.save(&path).unwrap();
    assert_eq!(Config::load(&path), cfg);
    assert!(!path.with_extension("tmp").exists());
}

#[test]
fn saving_under_a_file_that_is_not_a_folder_is_an_error_and_the_file_is_untouched() {
    let dir = tmp("jog-blocked");
    let blocker = dir.join("blocker");
    fs::write(&blocker, "I am a file").unwrap();
    let err = Config::defaults().save(&blocker.join("jog.cfg"));
    assert!(err.is_err());
    assert_eq!(fs::read_to_string(&blocker).unwrap(), "I am a file");
}

#[test]
fn numbers_are_written_to_four_decimals_without_the_zeros_that_mean_nothing() {
    assert_eq!(number(1.23456), "1.2346");
    assert_eq!(number(100.0), "100");
    assert_eq!(number(1_000_000.0), "1000000");
    assert_eq!(number(0.00004), "0");
    assert_eq!(number(-10.0), "-10");
    assert_eq!(number(2.5), "2.5");
}

#[test]
fn a_smaller_machine_takes_the_head_to_the_new_edges() {
    let mut j = jog_with("step = 50");
    for _ in 0..6 {
        j.press(Dir::XPlus, OPEN).unwrap();
    }
    assert_eq!(j.pos().x, 300.0);
    j.press(Dir::ZDown, OPEN).unwrap();
    assert_eq!(j.pos().z, 24.0);
    j.set_limits(Limits::new(100.0, 100.0, 10.0, 4.0, 0.0));
    assert_eq!(j.pos().x, 100.0);
    assert_eq!(j.pos().z, 10.0);
    assert_eq!(j.limits().x_max, 100.0);
    assert!(j.shown().x <= 100.0);
}

#[test]
fn a_frame_of_negative_time_owes_nothing_and_the_first_stretch_is_the_only_one() {
    let mut j = jog_with("speed_medium = 600\nchunk_mm = 2.5\ncontinuous = true");
    j.press(Dir::XPlus, OPEN).unwrap();
    let first = j.tick(-100.0, OPEN).unwrap();
    assert_eq!(first.len(), 1);
    assert!(j.tick(0.1, OPEN).unwrap().is_empty());
    assert_eq!(j.pos().x, 2.5);
}

#[test]
fn a_held_z_goes_to_the_floor_sends_what_is_left_and_ends() {
    let mut j = jog_with("speed_medium = 600\nchunk_mm = 2.5\ncontinuous = true");
    j.press(Dir::ZDown, OPEN).unwrap();
    assert_eq!(j.tick(100.0, OPEN).unwrap().len(), 8);
    assert_eq!(j.pos().z, 20.0);
    let rest = j.tick(100.0, OPEN).unwrap();
    assert_eq!(rest.len(), 2);
    assert_eq!(rest[1].dz, 1.5);
    assert_eq!(j.pos().z, 24.0);
    assert_eq!(j.held(), None);
}

#[test]
fn the_picture_never_leaves_the_table_or_gets_further_from_the_head() {
    let mut j = jog_with("speed_medium = 600\nstep = 50");
    let mut rng = Rng(0xA11CE);
    let gap = |j: &Jog| {
        let (a, b) = (j.pos(), j.shown());
        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
    };
    for _ in 0..300 {
        let dir = [Dir::XPlus, Dir::XMinus, Dir::YPlus, Dir::YMinus][rng.below(4) as usize];
        let _ = j.press(dir, OPEN);
        let before = gap(&j);
        j.advance(rng.unit() * 0.5);
        assert!(gap(&j) <= before + 1e-9);
        let s = j.shown();
        assert!((-1e-9..=500.0 + 1e-9).contains(&s.x) && (-1e-9..=300.0 + 1e-9).contains(&s.y));
    }
    j.advance(1e6);
    assert_eq!(j.shown(), j.pos());
}

#[test]
fn going_deep_to_another_place_goes_up_then_across_then_down() {
    let mut j = jog_with("step = 50");
    j.press(Dir::ZDown, OPEN).unwrap();
    let orders = j
        .goto(
            Pos {
                x: 10.0,
                y: 20.0,
                z: 24.0,
            },
            OPEN,
        )
        .unwrap();
    let moves: Vec<(f64, f64, f64)> = orders.iter().map(|o| (o.dx, o.dy, o.dz)).collect();
    assert_eq!(
        moves,
        vec![(0.0, 0.0, -4.0), (10.0, 20.0, 0.0), (0.0, 0.0, 4.0)]
    );
}

#[test]
fn a_zero_of_the_work_outside_the_table_is_reached_at_its_edge() {
    let cfg = Config::parse("origin = 600, 0, -5");
    let mut j = Jog::new(cfg, limits());
    let o = only(j.go_zero(Axis::X, OPEN));
    assert_eq!((o.dx, o.dy, o.dz), (500.0, 0.0, 0.0));
    assert_eq!(j.pos().x, 500.0);
    assert!(j.go_zero(Axis::Z, OPEN).unwrap().is_empty());
}

#[test]
fn the_mils_sent_add_up_to_the_distance_asked_to_within_half_a_mil() {
    let mut j = jog_with("");
    let mut rng = Rng(7);
    let (mut mm, mut mils) = (0.0f64, 0i64);
    for _ in 0..500 {
        let dx = (rng.unit() - 0.5) * 6.0;
        let m = j.mils_of(Order {
            dx,
            dy: 0.0,
            dz: 0.0,
            feed: 1.0,
        });
        mm += dx;
        mils += i64::from(m.0);
        assert_eq!((m.1, m.2), (0, 0));
    }
    assert!((mils as f64 - mm / 0.0254).abs() <= 0.5 + 1e-6);
}

#[test]
fn the_travel_never_has_a_negative_end_and_the_safe_height_is_inside_it() {
    let values = [-5.0, 0.0, 1.0, 24.0, 300.0];
    for &w in &values {
        for &board in &values {
            for &clearance in &values {
                for &below in &values {
                    let l = Limits::new(w, w, board, clearance, below);
                    assert!(l.x_max >= 0.0 && l.y_max >= 0.0 && l.z_floor >= 0.0);
                    assert!((0.0..=l.z_floor).contains(&l.safe_z), "{l:?}");
                }
            }
        }
    }
}

#[test]
fn letting_go_with_escape_keeps_a_head_that_has_a_machine_behind_it_where_it_was_sent() {
    let mut j = jog_with("continuous = true");
    j.press(Dir::XPlus, OPEN).unwrap();
    j.stop(true);
    assert_eq!(j.held(), None);
    assert_eq!(j.pos(), Pos::HOME);
    let mut j = jog_with("");
    j.press(Dir::XPlus, OPEN).unwrap();
    j.stop(false);
    assert_eq!(j.pos(), j.shown());
    assert!(!j.busy());
    assert!(j.touched());
}
