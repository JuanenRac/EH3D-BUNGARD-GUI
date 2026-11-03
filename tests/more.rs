//! More checks of the parts of the interface that need no window: how the width of the main screen is shared, what is saved to go on after a crash, the file of the connection
//! that the window rewrites, the diameters of the tools and what is remembered between runs.

use eh3d_bungard_gui::jobs::{read_latin1, table_from_config, tool_diameter, DEFAULT_TABLE};
use eh3d_bungard_gui::launcher::Settings;
use eh3d_bungard_gui::link::update;
use eh3d_bungard_gui::panes::{
    bars, drag_left, drag_right, parse, split, BOARD_W, DEFAULT_LEFT, DEFAULT_RIGHT, FRAME, MIN,
    TOTAL,
};
use eh3d_bungard_gui::recovery::{fingerprint, Recovery};
use std::path::{Path, PathBuf};

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

fn dir(name: &str) -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("more-{name}"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

// ---- the panes ----

#[test]
fn whatever_is_asked_the_three_panes_fill_the_room_and_keep_their_least_width() {
    let wanted = [
        -1e9,
        -5.0,
        0.0,
        100.0,
        220.0,
        280.0,
        608.0,
        900.0,
        1e9,
        f32::MAX,
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
    ];
    for &l in &wanted {
        for &r in &wanted {
            let (a, b, c) = split(l, r);
            assert!(near(a + b + c, TOTAL), "{l} {r}: {a} {b} {c}");
            assert!(
                a >= MIN[0] - 1e-3 && b >= MIN[1] - 1e-3 && c >= MIN[2] - 1e-3,
                "{l} {r}: {a} {b} {c}"
            );
        }
    }
    assert!(near(BOARD_W - FRAME, TOTAL));
}

#[test]
fn a_split_that_is_asked_again_does_not_move() {
    for (l, r) in [
        (0.0, 0.0),
        (500.0, 700.0),
        (DEFAULT_LEFT, DEFAULT_RIGHT),
        (1e9, 1e9),
    ] {
        let (a, _, c) = split(l, r);
        let (a2, _, c2) = split(a, c);
        assert!(near(a, a2) && near(c, c2), "{l} {r}");
    }
}

#[test]
fn dragging_a_bar_and_dragging_it_back_returns_where_it_was_while_nothing_hits_a_limit() {
    for delta in [-50.0, -1.0, 0.0, 1.0, 37.5, 80.0] {
        let (l, r) = drag_left(DEFAULT_LEFT, DEFAULT_RIGHT, delta);
        let (l2, r2) = drag_left(l, r, -delta);
        assert!(
            near(l2, DEFAULT_LEFT) && near(r2, DEFAULT_RIGHT),
            "left {delta}"
        );
        let (l, r) = drag_right(DEFAULT_LEFT, DEFAULT_RIGHT, delta);
        let (l2, r2) = drag_right(l, r, -delta);
        assert!(
            near(l2, DEFAULT_LEFT) && near(r2, DEFAULT_RIGHT),
            "right {delta}"
        );
    }
    // a bar that is dragged to the limit and further stays at the limit
    assert_eq!(drag_left(DEFAULT_LEFT, DEFAULT_RIGHT, -1e6).0, MIN[0]);
    assert_eq!(drag_right(DEFAULT_LEFT, DEFAULT_RIGHT, 1e6).1, MIN[2]);
}

#[test]
fn the_bars_sit_in_the_gaps_and_the_second_is_always_after_the_first() {
    for (l, r) in [
        (DEFAULT_LEFT, DEFAULT_RIGHT),
        (MIN[0], MIN[2]),
        (900.0, 900.0),
        (f32::NAN, 0.0),
    ] {
        let (b1, b2) = bars(l, r);
        let (a, m, _) = split(l, r);
        assert!(
            near(b1, 20.0 + a + 8.0) && near(b2 - b1, 16.0 + m),
            "{l} {r}"
        );
        assert!(b1 > 20.0 && b2 < BOARD_W - 20.0 && b2 > b1);
    }
}

#[test]
fn the_panes_in_the_file_are_two_numbers_with_a_comma() {
    assert_eq!(parse("280,608"), Some((280.0, 608.0)));
    assert_eq!(parse(" 1 , 2 "), Some((1.0, 2.0)));
    assert_eq!(parse("1e2,2.5e2"), Some((100.0, 250.0)));
    for bad in ["", ",", "1", "1;2", "a,b", "1,2,3", "1,", ",2", "1 2"] {
        assert_eq!(parse(bad), None, "{bad:?}");
    }
    // not-a-number is read as a number, and the split brings it to the default
    let (l, r) = parse("nan,inf").unwrap();
    assert_eq!(split(l, r), split(DEFAULT_LEFT, DEFAULT_RIGHT));
}

// ---- what is saved to go on after a crash ----

fn sample() -> Recovery {
    Recovery {
        program: "route".into(),
        drill: Some(PathBuf::from("/jobs/ñandú 日本/board.ncd")),
        route: Some(PathBuf::from("/jobs/a=b.plt")),
        columns: 4,
        rows: 3,
        mode: "pitch".into(),
        spacing: (1.5, 2.25),
        tabs_on: false,
        tab_count: 0,
        tab_width: 2.5,
        tab_place: "middle".into(),
        done: 0,
        total: 12,
        drilled: vec![(1.125, -2.5), (0.0, 0.001)],
        queue: vec![],
        off: vec![("route".into(), 7)],
        fingerprint: "0123456789abcdef".into(),
    }
}

#[test]
fn what_is_saved_is_what_comes_back_even_with_odd_names_and_edge_numbers() {
    let r = sample();
    assert_eq!(Recovery::from_text(&r.to_text()), Some(r.clone()));
    let only_route = Recovery {
        drill: None,
        program: "route".into(),
        ..r.clone()
    };
    assert_eq!(Recovery::from_text(&only_route.to_text()), Some(only_route));
    // the text of a text is the same text
    assert_eq!(
        Recovery::from_text(&r.to_text()).unwrap().to_text(),
        r.to_text()
    );
}

#[test]
fn a_file_that_is_cut_damaged_or_foreign_is_not_a_job() {
    let ok = sample().to_text();
    assert!(Recovery::from_text(&ok).is_some());
    for (what, bad) in [
        ("columns", ok.replace("columns=4", "columns=four")),
        ("rows", ok.replace("rows=3", "rows=-1")),
        (
            "spacing",
            ok.replace("spacing=1.5,2.25", "spacing=1.5;2.25"),
        ),
        ("done", ok.replace("done=0", "done=")),
        ("hole", ok.replace("hole=1.125,-2.5", "hole=1.125")),
        ("off", ok.replace("off=route:7", "off=route:seven")),
        ("off colon", ok.replace("off=route:7", "off=route")),
        ("program", ok.replace("program=route", "program=other")),
    ] {
        assert!(Recovery::from_text(&bad).is_none(), "{what}");
    }
    assert!(
        Recovery::from_text("program=route\n").is_none(),
        "no file at all"
    );
    assert!(Recovery::from_text("drill=a.ncd\n").is_none(), "no program");
}

#[test]
fn a_queue_or_a_tool_off_that_names_no_program_is_left_out_and_unknown_keys_do_not_matter() {
    let text = format!(
        "{}queue=other\nqueue=drill\noff=other:3\nflavour=vanilla\nnot a pair\n",
        sample().to_text()
    );
    let r = Recovery::from_text(&text).unwrap();
    assert_eq!(r.queue, vec!["drill".to_string()]);
    assert_eq!(r.off, vec![("route".to_string(), 7)]);
}

#[test]
fn saving_replaces_the_last_one_leaves_no_side_file_and_clearing_removes_both() {
    let d = dir("recovery");
    let path = d.join("recovery.cfg");
    sample().save(&path);
    assert_eq!(Recovery::load(&path), Some(sample()));
    let newer = Recovery {
        done: 5,
        ..sample()
    };
    newer.save(&path);
    assert_eq!(Recovery::load(&path).unwrap().done, 5);
    assert_eq!(std::fs::read_dir(&d).unwrap().count(), 1);
    std::fs::write(path.with_extension("tmp"), "left behind").unwrap();
    Recovery::clear(&path);
    assert_eq!(std::fs::read_dir(&d).unwrap().count(), 0);
    Recovery::clear(&path);
    assert_eq!(Recovery::load(&path), None);
    // a folder that is not there is not a reason to stop
    Recovery {
        done: 1,
        ..sample()
    }
    .save(&d.join("missing").join("recovery.cfg"));
    assert_eq!(
        Recovery::load(&d.join("missing").join("recovery.cfg")),
        None
    );
}

#[test]
fn the_fingerprint_cannot_be_fooled_by_moving_a_byte_from_one_file_to_the_next() {
    let d = dir("fingerprint");
    let (a, b, c, e) = (d.join("a"), d.join("b"), d.join("c"), d.join("e"));
    std::fs::write(&a, "a").unwrap();
    std::fs::write(&b, "b").unwrap();
    std::fs::write(&c, "ab").unwrap();
    std::fs::write(&e, "").unwrap();
    let apart = fingerprint(&[a.as_path(), b.as_path()], "s");
    assert_ne!(apart, fingerprint(&[c.as_path(), e.as_path()], "s"));
    assert_ne!(apart, fingerprint(&[b.as_path(), a.as_path()], "s"));
    assert_ne!(
        fingerprint(&[c.as_path()], "x"),
        fingerprint(&[c.as_path()], "y")
    );
    assert_ne!(
        fingerprint(&[c.as_path()], "xy"),
        fingerprint(&[c.as_path(), e.as_path()], "xy")
    );
    // a file that is not there reads as an empty one
    assert_eq!(
        fingerprint(&[d.join("none").as_path()], "s"),
        fingerprint(&[e.as_path()], "s")
    );
    let f = fingerprint(&[a.as_path()], "");
    assert_eq!(f.len(), 16);
    assert!(f
        .chars()
        .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase()));
}

// ---- the file of the connection ----

#[test]
fn the_keys_of_the_window_are_rewritten_and_every_other_line_stays_as_it_was() {
    let pairs = [
        ("connection", "usb".to_string()),
        ("usb_port", "COM8".to_string()),
    ];
    let old = "# my machine ; with notes\n  CONNECTION =  demo \nx_search_hz = 2700\n; usb_port = COM1\nusb_port=COM3\nconnection = server\nplain line\n";
    let new = update(old, &pairs);
    assert_eq!(
        new,
        "# my machine ; with notes\nconnection = usb\nx_search_hz = 2700\n; usb_port = COM1\nusb_port = COM8\nplain line\n",
        "the key written twice is written once, comments are not keys"
    );
    assert_eq!(update(&new, &pairs), new, "again changes nothing");
    assert_eq!(update("", &pairs), "connection = usb\nusb_port = COM8\n");
    assert_eq!(update("a = 1", &[]), "a = 1\n");
    assert_eq!(
        update("a = 1\r\nb = 2\r\n", &[("b", "3".to_string())]),
        "a = 1\nb = 3\n"
    );
    assert_eq!(
        update("k = a=b\n", &[("k", "x=y".to_string())]),
        "k = x=y\n"
    );
}

// ---- the tools and the tables ----

#[test]
fn the_diameter_of_a_tool_comes_from_the_header_or_from_its_number() {
    let declared = [(1u8, 0.8), (7, 0.0315)];
    assert_eq!(tool_diameter(1, &declared, true), 0.8);
    assert!((tool_diameter(7, &declared, false) - 0.0315 * 25.4).abs() < 1e-12);
    // what the header declares wins even for a number below 4
    assert_eq!(tool_diameter(1, &declared, true), 0.8);
    // without it the number is the diameter in tenths of a millimetre, and the small numbers are a millimetre
    assert_eq!(tool_diameter(4, &[], true), 0.4);
    assert_eq!(tool_diameter(10, &[], false), 1.0);
    assert_eq!(tool_diameter(30, &[], true), 3.0);
    assert_eq!(tool_diameter(255, &[], true), 25.5);
    for t in 0..4u8 {
        assert_eq!(tool_diameter(t, &[], true), 1.0);
    }
}

#[test]
fn a_text_that_is_not_a_configuration_gives_the_table_of_the_machine() {
    for text in [
        "",
        "garbage",
        "\u{0}\u{1}",
        "Clipping X = -5\nClipping Y = 0\n",
    ] {
        for route in [false, true] {
            let t = table_from_config(text, route);
            assert!(
                t.width_mm == DEFAULT_TABLE.width_mm && t.depth_mm == DEFAULT_TABLE.depth_mm,
                "{text:?} {route}"
            );
        }
    }
}

#[test]
fn a_file_is_read_one_byte_for_one_letter_whatever_the_code_page() {
    let d = dir("latin1");
    let f = d.join("odd.ncd");
    let bytes: Vec<u8> = (0..=255u8).collect();
    std::fs::write(&f, &bytes).unwrap();
    let text = read_latin1(&f).unwrap();
    assert_eq!(text.chars().count(), 256);
    assert_eq!(
        text.chars().map(|c| c as u32).collect::<Vec<_>>(),
        (0..256).collect::<Vec<_>>()
    );
    std::fs::write(&f, b"T01C0.8 (cota \xF1)\r\n").unwrap();
    assert_eq!(read_latin1(&f).unwrap(), "T01C0.8 (cota \u{f1})\r\n");
    assert!(read_latin1(&d.join("missing")).is_err());
    std::fs::write(&f, b"").unwrap();
    assert_eq!(read_latin1(&f).unwrap(), "");
}

// ---- what is remembered between runs ----

#[test]
fn a_file_name_with_a_line_break_cannot_add_a_setting_to_the_file() {
    let mut s = Settings::default();
    let before = s.updates_api_url.clone();
    s.remember_recent("/jobs/x.ncd\nupdates_api_url = https://example.invalid/latest");
    s.updates_api_url = format!("{before}\nupdates = no");
    let back = Settings::parse(&s.to_text());
    assert_eq!(back.updates_api_url, before.clone() + " updates = no");
    assert!(back.updates_enabled);
    assert!(!back.recent.iter().any(|r| r.contains('\n')));
    assert_eq!(back.recent.len(), 1);
    assert!(back.recent[0].starts_with("/jobs/x.ncd"));
}

#[test]
fn the_settings_survive_a_round_trip_and_the_recent_files_keep_their_order() {
    let mut s = Settings::default();
    for i in 0..10 {
        s.remember_recent(&format!("/jobs/f{i}.ncd"));
    }
    s.remember_recent("/jobs/f5.ncd");
    assert_eq!(s.recent[0], "/jobs/f5.ncd");
    assert_eq!(s.recent.iter().filter(|p| *p == "/jobs/f5.ncd").count(), 1);
    let back = Settings::parse(&s.to_text());
    assert_eq!(back.recent, s.recent);
    assert_eq!(Settings::parse(&back.to_text()).to_text(), back.to_text());
    assert_eq!(Settings::parse("").to_text(), Settings::default().to_text());
}

#[test]
fn a_wrong_value_in_the_settings_keeps_the_default() {
    let d = Settings::default();
    let s = Settings::parse(
        "sim_speed = 0\nupdates_interval_hours = 0\nupdates_timeout_seconds = 1\nupdates_timeout_seconds = x\nestop_pos = a,b\nmodel = no such machine\nlang = xx\npanes = nan,nan\nholes_drilled = a:1,2:b,3:4\n",
    );
    assert_eq!(s.sim_speed, d.sim_speed);
    assert_eq!(s.updates_interval_h, d.updates_interval_h);
    assert_eq!(s.updates_timeout_s, d.updates_timeout_s);
    assert_eq!(s.estop_pos, d.estop_pos);
    assert_eq!(s.model, d.model);
    assert_eq!(s.lang, d.lang);
    assert_eq!((s.pane_left, s.pane_right), (d.pane_left, d.pane_right));
    assert_eq!(
        s.holes_drilled.into_iter().collect::<Vec<_>>(),
        vec![(3u8, 4u64)]
    );
}

#[test]
fn the_position_of_the_stop_button_is_kept_inside_the_drawing_board() {
    let s = Settings::parse("estop_pos = -50,5000\n");
    assert_eq!(s.estop_pos, (0.0, 900.0));
    let s = Settings::parse("estop_pos = 1700.5, 12\n");
    assert_eq!(s.estop_pos, (1600.0, 12.0));
}

#[test]
fn the_limits_of_the_updates_are_inclusive() {
    let s = Settings::parse("updates_interval_hours = 1\nupdates_timeout_seconds = 300\n");
    assert_eq!((s.updates_interval_h, s.updates_timeout_s), (1, 300));
    let s = Settings::parse(&format!(
        "updates_interval_hours = {}\nupdates_timeout_seconds = 2\n",
        24 * 90
    ));
    assert_eq!((s.updates_interval_h, s.updates_timeout_s), (24 * 90, 2));
    let s = Settings::parse(&format!(
        "updates_interval_hours = {}\nupdates_timeout_seconds = 301\n",
        24 * 90 + 1
    ));
    assert_eq!(
        (s.updates_interval_h, s.updates_timeout_s),
        (
            Settings::default().updates_interval_h,
            Settings::default().updates_timeout_s
        )
    );
}
