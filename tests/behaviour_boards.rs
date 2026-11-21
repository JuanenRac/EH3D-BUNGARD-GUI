//! Behaviour tests for the board library, the bit catalogue, the machine profiles, the places of the tool changer, the camera check and the spindles.
//! They only use the public API and look at limits, exact text, stable formats, ordering and round trips.

use std::cmp::Ordering;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering as Atomic};

use eh3d_bungard_gui::boards::{
    fit_on_table, from_text, maker_text, makers, materials, materials_view, natural_cmp, of_maker,
    to_text, Board, Core, Family, Filter, Fit, Material, Sides, WORKABLE,
};
use eh3d_bungard_gui::catalogue::{self, Bit, Kind, SHANK_MM, TOTAL_MM};
use eh3d_bungard_gui::inspection::{
    calibrated_scale, finding_text, holes_in_view, load_profile, preset_for, save_profile, Param,
    DEFAULT_PICTURE, DEFAULT_PX_PER_MM, LIGHT_CHOICES,
};
use eh3d_bungard_gui::launcher::Program;
use eh3d_bungard_gui::machines::{self, Model, ToolChange};
use eh3d_bungard_gui::slots::{
    check, engine_slots, fill_in_order, selectable, set, slot_of, Assignment, Book, JobTool,
    Problem, MAX_TOOL_SLOT,
};
use eh3d_bungard_gui::spindle::{Fit as SpindleFit, HeadDims, Spindle, SpindleId, Why};
use eh3d_inspect::{inspect, ExpectedHole, Gray, HoleResult, Profile, Report};
use eh3d_materials::Kind as DbKind;
use eh3d_view::scene::Table;
use eh3d_view::texts::Text;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A fresh, empty directory that no other test uses.
fn scratch(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Atomic::SeqCst);
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("behaviour-boards-{n}-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// A small generator with a fixed seed, so that the tests are the same on every run.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }
}

// ---------------------------------------------------------------------------------------------
// boards
// ---------------------------------------------------------------------------------------------

fn custom(
    name: &'static str,
    core: Core,
    sides: Sides,
    thickness_mm: &'static [f64],
    copper_um: &'static [f64],
    kind: Option<DbKind>,
) -> Material {
    Material {
        maker: "Test",
        name,
        core,
        thickness_mm,
        copper_um,
        sides,
        formats_mm: &[],
        dk: None,
        df: None,
        tg_c: None,
        photoresist: false,
        id: "",
        kind,
        curated: false,
        note: "",
    }
}

fn words(text: &str) -> Filter<'_> {
    Filter {
        text,
        ..Filter::default()
    }
}

#[test]
fn standard_board_is_generic_fr4_with_two_copper_sides() {
    let b = Board::standard();
    assert_eq!(b.thickness_mm, 1.6);
    assert_eq!(b.copper_um, 35.0);
    assert_eq!(b.layers, 2);
    assert!(b.size_mm.is_none());
    assert!(!b.has_no_copper());
    let m = b.material();
    assert_eq!(m.maker, "Generic");
    assert!(m.name.starts_with("FR-4 (IPC"), "{}", m.name);
}

#[test]
fn core_names_are_exact_and_unique() {
    let all = [
        (Core::Fr2, "FR-2"),
        (Core::Fr3, "FR-3"),
        (Core::Cem1, "CEM-1"),
        (Core::Cem3, "CEM-3"),
        (Core::Fr4, "FR-4"),
        (Core::Hydrocarbon, "Hydrocarbon"),
        (Core::Ptfe, "PTFE"),
        (Core::Polyimide, "Polyimide"),
        (Core::Ceramic, "Ceramic"),
        (Core::Aluminium, "Aluminium"),
        (Core::Flex, "Flex"),
        (Core::Metal, "Metal"),
        (Core::Copper, "Copper"),
        (Core::Wood, "Wood"),
        (Core::Plastic, "Plastic"),
        (Core::SoftPlastic, "Soft plastic"),
    ];
    for (core, name) in all {
        assert_eq!(core.name(), name, "{core:?}");
    }
    for (i, (_, a)) in all.iter().enumerate() {
        for (_, b) in &all[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn every_core_without_a_database_kind_has_the_family_of_its_material() {
    let expected = [
        (Core::Fr2, Family::Laminate),
        (Core::Fr3, Family::Laminate),
        (Core::Cem1, Family::Laminate),
        (Core::Cem3, Family::Laminate),
        (Core::Fr4, Family::Laminate),
        (Core::Polyimide, Family::Laminate),
        (Core::Flex, Family::Laminate),
        (Core::Ptfe, Family::Rf),
        (Core::Hydrocarbon, Family::Ceramic),
        (Core::Ceramic, Family::Ceramic),
        (Core::Aluminium, Family::MetalCore),
        (Core::Metal, Family::Sheet),
        (Core::Copper, Family::Sheet),
        (Core::Wood, Family::Sheet),
        (Core::Plastic, Family::Sheet),
        (Core::SoftPlastic, Family::Sheet),
    ];
    for (core, family) in expected {
        let m = custom("Sample", core, Sides::Either, &[1.0], &[35.0], None);
        assert_eq!(m.family(), family, "{core:?}");
    }
    // what the maker does not give is told per material
    let none: &'static [f64] = &[];
    let no_thickness = custom("A", Core::Fr4, Sides::Either, none, &[35.0], None);
    assert!(no_thickness.needs_thickness() && !no_thickness.needs_copper());
    assert!(no_thickness.no_data());
    let no_copper_data = custom("B", Core::Fr4, Sides::Either, &[1.6], none, None);
    assert!(!no_copper_data.needs_thickness() && no_copper_data.needs_copper());
    assert!(no_copper_data.no_data());
    // a sheet with no copper does not need any
    let sheet = custom("C", Core::Wood, Sides::NoCopper, &[1.0], none, None);
    assert!(!sheet.needs_copper() && !sheet.no_data());
    let sheet_without_thickness = custom("D", Core::Wood, Sides::NoCopper, none, none, None);
    assert!(sheet_without_thickness.needs_thickness() && sheet_without_thickness.no_data());
    let complete = custom("E", Core::Fr4, Sides::One, &[1.6], &[35.0], None);
    assert!(!complete.no_data());
}

#[test]
fn a_database_kind_decides_the_family_before_the_core() {
    let pick = |kind: DbKind, core: Core| {
        custom("K", core, Sides::Either, &[1.0], &[35.0], Some(kind)).family()
    };
    assert_eq!(pick(DbKind::Ptfe, Core::Fr4), Family::Rf);
    assert_eq!(pick(DbKind::CeramicFilled, Core::Fr4), Family::Ceramic);
    assert_eq!(pick(DbKind::MetalCore, Core::Wood), Family::MetalCore);
    assert_eq!(pick(DbKind::Laminate, Core::Ptfe), Family::Laminate);
    // a kind that a machine for boards does not work falls back to the core
    assert_eq!(pick(DbKind::Prepreg, Core::Copper), Family::Sheet);
    assert_eq!(pick(DbKind::Other, Core::Ptfe), Family::Rf);
    // the workable kinds, in order, and the family each one gives
    assert_eq!(WORKABLE.len(), 4);
    assert!(WORKABLE[0] == DbKind::Laminate && WORKABLE[1] == DbKind::Ptfe);
    assert!(WORKABLE[2] == DbKind::CeramicFilled && WORKABLE[3] == DbKind::MetalCore);
    let families: Vec<Family> = WORKABLE.iter().map(|k| pick(*k, Core::Wood)).collect();
    assert_eq!(
        families,
        [
            Family::Laminate,
            Family::Rf,
            Family::Ceramic,
            Family::MetalCore
        ]
    );
    // the five families are listed once each
    for (i, a) in Family::ALL.iter().enumerate() {
        for b in &Family::ALL[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn an_index_past_the_end_is_clamped_and_the_sides_follow_the_material() {
    let all = materials();
    let last = all.last().unwrap();
    let b = Board {
        material: usize::MAX,
        ..Board::standard()
    };
    assert_eq!(
        (b.material().maker, b.material().name),
        (last.maker, last.name)
    );
    let past = Board {
        material: all.len(),
        ..Board::standard()
    };
    assert_eq!(past.material().name, last.name);
    assert_eq!(
        Board::standard().with_material(usize::MAX).material,
        all.len() - 1
    );
    // copper on one side only, on both sides only, or either
    let one = all.iter().position(|m| m.sides == Sides::One).unwrap();
    let two = all.iter().position(|m| m.sides == Sides::Two).unwrap();
    let either = all.iter().position(|m| m.sides == Sides::Either).unwrap();
    assert_eq!(Board::standard().with_material(one).layers, 1);
    let single = Board {
        layers: 1,
        ..Board::standard()
    };
    assert_eq!(single.with_material(two).layers, 2);
    // a damaged number of layers is brought into 1 or 2 where the material allows both
    let none = Board {
        layers: 0,
        ..Board::standard()
    };
    let many = Board {
        layers: 9,
        ..Board::standard()
    };
    assert_eq!(none.with_material(either).layers, 1);
    assert_eq!(many.with_material(either).layers, 2);
}

#[test]
fn changing_material_twice_changes_nothing_more() {
    let b = Board {
        size_mm: Some((160.0, 100.0)),
        ..Board::standard()
    };
    for (i, _) in materials().iter().enumerate() {
        let once = b.with_material(i);
        assert_eq!(once.with_material(i), once, "{i}");
        assert_eq!(once.size_mm, b.size_mm);
        assert_eq!(once.material, i);
    }
}

#[test]
fn a_changed_board_only_takes_values_the_material_offers() {
    let b = Board {
        thickness_mm: 1.3,
        copper_um: 50.0,
        ..Board::standard()
    };
    for (i, m) in materials().iter().enumerate() {
        let c = b.with_material(i);
        if m.thickness_mm.is_empty() {
            assert_eq!(c.thickness_mm, b.thickness_mm, "{}", m.name);
        } else {
            let d = (c.thickness_mm - b.thickness_mm).abs();
            assert!(m.thickness_mm.contains(&c.thickness_mm), "{}", m.name);
            assert!(
                m.thickness_mm
                    .iter()
                    .all(|t| (t - b.thickness_mm).abs() >= d),
                "{}",
                m.name
            );
        }
        if m.sides == Sides::NoCopper {
            assert_eq!(c.copper_um, 0.0, "{}", m.name);
        } else if m.copper_um.is_empty() {
            assert_eq!(c.copper_um, b.copper_um, "{}", m.name);
        } else {
            let d = (c.copper_um - b.copper_um).abs();
            assert!(m.copper_um.contains(&c.copper_um), "{}", m.name);
            assert!(
                m.copper_um.iter().all(|u| (u - b.copper_um).abs() >= d),
                "{}",
                m.name
            );
        }
        assert!((1..=2).contains(&c.layers), "{}", m.name);
    }
}

#[test]
fn the_description_is_one_stable_line() {
    let b = Board::standard();
    let m = b.material();
    assert_eq!(
        b.describe(),
        format!("{} {} - 1.60 mm - 35 um Cu - 2 layers", m.maker, m.name)
    );
    let thin = Board {
        layers: 1,
        copper_um: 34.6,
        thickness_mm: 0.8,
        ..b
    };
    assert_eq!(
        thin.describe(),
        format!("{} {} - 0.80 mm - 35 um Cu - 1 layer", m.maker, m.name)
    );
    // anything that is not 1 reads as two layers
    let odd = Board { layers: 0, ..b };
    assert!(odd.describe().ends_with("2 layers"));
    let idx = materials()
        .iter()
        .position(|m| m.sides == Sides::NoCopper)
        .unwrap();
    let sheet = Board {
        material: idx,
        thickness_mm: 1.5,
        copper_um: 0.0,
        layers: 2,
        size_mm: None,
    };
    let sm = sheet.material();
    assert!(sheet.has_no_copper());
    assert_eq!(
        sheet.describe(),
        format!("{} {} - 1.50 mm - no copper", sm.maker, sm.name)
    );
}

#[test]
fn the_text_of_a_board_has_a_fixed_shape() {
    let b = Board::standard();
    let m = b.material();
    assert_eq!(to_text(&b), format!("{}|{}|1.6|35|2|", m.maker, m.name));
    let sized = Board {
        size_mm: Some((100.0, 80.5)),
        thickness_mm: 0.8,
        copper_um: 17.5,
        layers: 1,
        ..b
    };
    assert_eq!(
        to_text(&sized),
        format!("{}|{}|0.8|17.5|1|100x80.5", m.maker, m.name)
    );
    assert_eq!(from_text(&to_text(&sized)), Some(sized));
}

#[test]
fn a_damaged_board_text_is_refused_and_outer_spaces_are_forgiven() {
    let b = Board::standard();
    let m = b.material();
    let head = format!("{}|{}", m.maker, m.name);
    assert!(from_text(&format!("{head}|1.6|35|2|")).is_some());
    let bad = [
        format!("{head}|1.6|35|2"),
        format!("{head}|1.6|35|2||"),
        format!("Nobody|{}|1.6|35|2|", m.name),
        format!("{}|No such material|1.6|35|2|", m.maker),
        format!("{head}|1.6|35|0|"),
        format!("{head}|1.6|35|3|"),
        format!("{head}|1.6|35|-1|"),
        format!("{head}|1.6|35|256|"),
        format!("{head}|abc|35|2|"),
        format!("{head}|1.6||2|"),
        format!("{head}|1.6|35|2|100"),
        format!("{head}|1.6|35|2|100x"),
        format!("{head}|1.6|35|2|x80"),
        format!("{head}|1.6|35|2|axb"),
        format!("{head}|1.6|35|2|1x2x3"),
        String::new(),
        "|||||".to_string(),
    ];
    for t in &bad {
        assert_eq!(from_text(t), None, "{t}");
    }
    let sized = Board {
        size_mm: Some((100.0, 80.5)),
        ..b
    };
    let text = to_text(&sized);
    assert!(text.ends_with("|2|100x80.5"));
    assert_eq!(from_text(&format!("  {text}\r\n")), Some(sized));
    // spaces inside a field are not forgiven
    assert_eq!(from_text(&text.replacen("|2|", "| 2|", 1)), None);
}

#[test]
fn every_material_survives_the_settings_text() {
    let base = Board {
        size_mm: Some((123.4, 56.7)),
        ..Board::standard()
    };
    for (i, m) in materials().iter().enumerate() {
        if m.maker.contains('|') || m.maker != m.maker.trim() {
            continue;
        }
        let b = base.with_material(i);
        assert_eq!(from_text(&to_text(&b)), Some(b), "{} {}", m.maker, m.name);
    }
}

const TABLE: Option<(f64, f64)> = Some((495.0, 325.0));

#[test]
fn the_table_tolerance_is_a_hundredth_of_a_millimetre() {
    assert_eq!(fit_on_table(Some((495.0, 325.0)), TABLE), Fit::Fits);
    assert_eq!(fit_on_table(Some((495.005, 325.005)), TABLE), Fit::Fits);
    match fit_on_table(Some((495.05, 325.0)), TABLE) {
        Fit::TooBig { over_w, over_d } => {
            assert!(close(over_w, 0.05), "{over_w}");
            assert_eq!(over_d, 0.0);
        }
        other => panic!("{other:?}"),
    }
    // turned: the board is as long as the table is deep
    assert_eq!(fit_on_table(Some((325.0, 495.0)), TABLE), Fit::FitsTurned);
    assert_eq!(
        fit_on_table(Some((325.005, 495.005)), TABLE),
        Fit::FitsTurned
    );
    // when both ways go over equally, the board as it lies is the one that is reported
    assert_eq!(
        fit_on_table(Some((500.0, 500.0)), TABLE),
        Fit::TooBig {
            over_w: 5.0,
            over_d: 175.0
        }
    );
    // when turning it goes over less, that is what is reported
    assert_eq!(
        fit_on_table(Some((400.0, 600.0)), TABLE),
        Fit::TooBig {
            over_w: 105.0,
            over_d: 75.0
        }
    );
}

#[test]
fn sizes_that_are_not_sizes_are_unknown() {
    assert_eq!(fit_on_table(None, None), Fit::Unknown);
    assert_eq!(fit_on_table(Some((-1.0, 10.0)), TABLE), Fit::Unknown);
    assert_eq!(
        fit_on_table(Some((f64::INFINITY, 10.0)), TABLE),
        Fit::Unknown
    );
    assert_eq!(
        fit_on_table(Some((10.0, 10.0)), Some((0.0, 325.0))),
        Fit::Unknown
    );
    assert_eq!(
        fit_on_table(Some((10.0, 10.0)), Some((f64::NAN, 1.0))),
        Fit::Unknown
    );
    assert_eq!(
        fit_on_table(Some((10.0, 10.0)), Some((-5.0, 5.0))),
        Fit::Unknown
    );
    // the smallest sizes that mean something still fit
    assert_eq!(fit_on_table(Some((0.001, 0.001)), TABLE), Fit::Fits);
}

#[test]
fn natural_order_is_total_and_folds_accents() {
    assert_eq!(natural_cmp("a01", "a1"), Ordering::Less);
    assert_eq!(natural_cmp("a1", "a01"), Ordering::Greater);
    assert_eq!(natural_cmp("", ""), Ordering::Equal);
    assert_eq!(natural_cmp("", "a"), Ordering::Less);
    // text sorts before numbers
    assert_eq!(natural_cmp("a", "1"), Ordering::Less);
    // numbers too big for any integer type tie on their value and are told apart by the raw text
    let big_a = format!("x{}", "9".repeat(60));
    let big_b = format!("x{}", "8".repeat(60));
    assert_eq!(natural_cmp(&big_a, &big_b), big_a.cmp(&big_b));
    assert_eq!(natural_cmp(&big_b, &big_a), big_b.cmp(&big_a));
    assert_eq!(natural_cmp(&big_a, &big_a), Ordering::Equal);
    // an accented capital is sorted as its plain letter: "lodz" comes before "m"
    assert_eq!(natural_cmp("\u{141}\u{f3}d\u{17a}", "m"), Ordering::Less);
    assert_eq!(natural_cmp("m", "\u{141}\u{f3}d\u{17a}"), Ordering::Greater);
}

#[test]
fn sorting_with_natural_order_does_not_depend_on_the_input_order() {
    let items = ["b10", "B2", "a", "A1", "a1", "\u{e9}", "e", "Z"];
    let mut forward = items;
    forward.sort_by(|a, b| natural_cmp(a, b));
    let mut backward = items;
    backward.reverse();
    backward.sort_by(|a, b| natural_cmp(a, b));
    assert_eq!(forward, backward);
    assert_eq!(forward, ["a", "A1", "a1", "B2", "b10", "e", "\u{e9}", "Z"]);
    let mut numbers = ["x10", "X100", "x9", "x1"];
    numbers.sort_by(|a, b| natural_cmp(a, b));
    assert_eq!(numbers, ["x1", "x9", "x10", "X100"]);
}

#[test]
fn the_makers_partition_the_materials() {
    let m = makers();
    let unique: HashSet<&str> = m.iter().copied().collect();
    assert_eq!(unique.len(), m.len());
    for name in ["Bungard", "Rogers", "Generic"] {
        assert!(m.contains(&name), "{name}");
    }
    let total: usize = m.iter().map(|k| of_maker(k).len()).sum();
    assert_eq!(total, materials().len());
    // the maker is matched as it is written
    assert!(of_maker("bungard").is_empty());
    assert!(of_maker("No such maker").is_empty());
}

#[test]
fn the_view_is_ordered_by_the_name_the_caller_gives() {
    let same = |_: &Material| String::from("same");
    let v = materials_view(&Filter::default(), &same);
    assert_eq!(v.len(), materials().len());
    for w in v.windows(2) {
        assert_ne!(natural_cmp(w[0].1.maker, w[1].1.maker), Ordering::Greater);
    }
    // sorted by maker through the name: the sort is stable inside a maker
    let by_maker = |m: &Material| m.maker.to_string();
    let v = materials_view(&Filter::default(), &by_maker);
    for w in v.windows(2) {
        assert_ne!(natural_cmp(w[0].1.maker, w[1].1.maker), Ordering::Greater);
        if w[0].1.maker == w[1].1.maker {
            assert!(w[0].0 < w[1].0);
        }
    }
    // the words are matched against the name that the caller gives
    let tagged = |m: &Material| format!("zzqq {}", m.name);
    let plain = |m: &Material| m.name.to_string();
    let f = words("zzqq");
    assert_eq!(materials_view(&f, &tagged).len(), materials().len());
    assert!(materials_view(&f, &plain).is_empty());
}

#[test]
fn every_word_has_to_match_and_the_maker_and_family_narrow_it() {
    let m = materials()
        .iter()
        .find(|m| m.maker == "Rogers" && m.name == "RO4350B")
        .unwrap();
    assert!(words("   ").accepts(m, "x"));
    assert!(words("rogers ro4350b").accepts(m, m.name));
    assert!(words("RO4350B   ROGERS").accepts(m, m.name));
    assert!(!words("rogers zzqq").accepts(m, m.name));
    assert!(Filter::of_maker("Rogers").accepts(m, m.name));
    assert!(!Filter::of_maker("Taconic").accepts(m, m.name));
    assert!(!Filter::of_maker("rogers").accepts(m, m.name));
    let sheet_only = Filter {
        family: Some(Family::Sheet),
        ..Filter::default()
    };
    assert!(!sheet_only.accepts(m, m.name));
    let both = Filter {
        maker: Some("Rogers"),
        text: "ro4350b",
        family: Some(m.family()),
    };
    assert!(both.accepts(m, m.name));
    // the name that is read may be a translated one, and accents do not count
    assert!(words("creme").accepts(m, "Cr\u{e8}me"));
    assert!(!words("creme").accepts(m, "other"));
    // the identifier of the database is searched too
    let with_id = materials()
        .iter()
        .find(|m| !m.id.is_empty() && !m.id.contains(char::is_whitespace))
        .unwrap();
    assert!(words(with_id.id).accepts(with_id, ""));
}

#[test]
fn the_words_of_the_materials_and_of_the_maker_have_their_text() {
    assert_eq!(maker_text("Generic"), Some(Text::MakerGeneric));
    for other in ["generic", "Generic ", "", "Rogers"] {
        assert_eq!(maker_text(other), None, "{other:?}");
    }
    let names: [&'static str; 20] = [
        "FR4 epoxy, positive photoresist",
        "FR4 epoxy, bare copper",
        "Laminated paper (FR2), photoresist",
        "FR2 / FR3 / CEM-1 sheets",
        "FR4 500 series, 1.6 mm",
        "FR4 500 series, 0.8 mm",
        "FR4 600 series, positive presensitized",
        "Thermal Clad (aluminium)",
        "Pyralux AP flexible",
        "Aluminium sheet (5052 / 6061)",
        "Copper sheet (C110)",
        "Birch plywood",
        "Balsa wood",
        "Basswood (lime)",
        "PTFE sheet",
        "Acrylic (PMMA)",
        "Polycarbonate (PC)",
        "Nylon (PA6)",
        "G10 (FR4 without copper)",
        "Phenolic paper (Pertinax)",
    ];
    let mut texts: Vec<Text> = Vec::new();
    for n in names {
        let m = custom(n, Core::Fr4, Sides::Either, &[1.0], &[35.0], None);
        texts.push(m.name_text().unwrap_or_else(|| panic!("{n}")));
    }
    for (i, a) in texts.iter().enumerate() {
        for b in &texts[i + 1..] {
            assert_ne!(a, b);
        }
    }
    assert_eq!(texts[11], Text::MatBirchPlywood);
    assert_eq!(texts[12], Text::MatBalsa);
    // a designation is the same in every language, and the match is exact
    for n in ["RO4003C", "Cotherm", "balsa wood", "Balsa wood ", ""] {
        let m = custom(n, Core::Fr4, Sides::Either, &[1.0], &[35.0], None);
        assert_eq!(m.name_text(), None, "{n:?}");
    }
}

#[test]
fn the_catalogue_is_built_once_with_sorted_lists() {
    assert!(std::ptr::eq(materials().as_ptr(), materials().as_ptr()));
    for m in materials() {
        assert!(!m.name.is_empty());
        assert!(m.thickness_mm.iter().all(|t| t.is_finite()), "{}", m.name);
        assert!(m.thickness_mm.windows(2).all(|w| w[0] < w[1]), "{}", m.name);
        assert!(m.copper_um.windows(2).all(|w| w[0] < w[1]), "{}", m.name);
        assert!(
            m.formats_mm
                .iter()
                .all(|f| f.0.is_finite() && f.1.is_finite()),
            "{}",
            m.name
        );
    }
}

// ---------------------------------------------------------------------------------------------
// catalogue
// ---------------------------------------------------------------------------------------------

fn plain(kind: Kind, diameter_mm: f64, rpm: u32, feed_mm_min: u32, plunge_mm_min: u32) -> Bit {
    Bit {
        kind,
        series: "",
        diameter_mm,
        tip_mm: 0.0,
        angle_deg: 0,
        flutes: 2,
        flute_mm: 0.0,
        rpm,
        feed_mm_min,
        plunge_mm_min,
        pass_mm: 1.0,
    }
}

fn vbit(angle_deg: u32, tip_mm: f64, diameter_mm: f64) -> Bit {
    Bit {
        tip_mm,
        angle_deg,
        ..plain(Kind::VBit, diameter_mm, 40_000, 400, 150)
    }
}

fn find(kind: Kind, series: &str, d: f64) -> Bit {
    *catalogue::all()
        .iter()
        .find(|b| b.kind == kind && b.series == series && close(b.diameter_mm, d))
        .unwrap_or_else(|| panic!("{kind:?} {series} {d}"))
}

#[test]
fn the_drill_table_changes_row_exactly_at_the_diameter_of_the_row() {
    let cases = [
        (0.0, (125_000, 2800)),
        (-3.0, (125_000, 2800)),
        (0.30, (125_000, 2800)),
        (0.30 + 1e-10, (125_000, 2800)),
        (0.31, (125_000, 3000)),
        (0.35, (125_000, 3000)),
        (0.45, (120_000, 3500)),
        (1.0, (54_000, 3100)),
        (3.175, (20_000, 1100)),
        (3.18, (25_000, 800)),
        (3.95, (25_000, 800)),
        (4.0, (20_000, 700)),
        (8.0, (20_000, 700)),
        (99.0, (20_000, 700)),
        (1000.0, (20_000, 700)),
    ];
    for (d, want) in cases {
        assert_eq!(catalogue::drill_speed(d), want, "{d}");
    }
    // a number that is not a number takes the last row and does not panic
    assert_eq!(catalogue::drill_speed(f64::NAN), (20_000, 700));
}

#[test]
fn the_router_table_changes_row_exactly_at_the_diameter_of_the_row() {
    let cases = [
        (0.0, (40_000, 150)),
        (0.8, (40_000, 150)),
        (0.8 + 1e-10, (40_000, 150)),
        (0.81, (36_500, 300)),
        (1.2, (36_500, 300)),
        (1.21, (32_000, 500)),
        (1.8, (32_000, 500)),
        (2.2, (28_000, 650)),
        (2.21, (26_500, 750)),
        (99.0, (26_500, 750)),
        (1000.0, (26_500, 750)),
    ];
    for (d, want) in cases {
        assert_eq!(catalogue::router_speed(d), want, "{d}");
    }
    assert_eq!(catalogue::router_speed(f64::NAN), (26_500, 750));
}

#[test]
fn bigger_drills_turn_slower_and_bigger_routers_feed_faster() {
    let mut last = catalogue::drill_speed(0.30).0;
    for i in 1..=575 {
        let d = 0.30 + f64::from(i) * 0.005;
        let rpm = catalogue::drill_speed(d).0;
        assert!(rpm <= last, "{d}");
        last = rpm;
    }
    // past the 3.175 mm shank the maker's table turns faster again for the first big drill
    assert_eq!(catalogue::drill_speed(3.9).0, 25_000);
    let mut prev = catalogue::router_speed(0.1);
    for i in 1..400 {
        let d = 0.1 + f64::from(i) * 0.01;
        let now = catalogue::router_speed(d);
        assert!(now.0 <= prev.0 && now.1 >= prev.1, "{d}");
        prev = now;
    }
}

#[test]
fn the_kinds_are_twelve_and_four_of_them_are_drills() {
    let unique: HashSet<Kind> = Kind::ALL.into_iter().collect();
    assert_eq!(unique.len(), 12);
    let drills: Vec<Kind> = Kind::ALL.into_iter().filter(|k| k.is_drill()).collect();
    assert_eq!(
        drills,
        [Kind::Drill, Kind::SlotDrill, Kind::Spot, Kind::Countersink]
    );
    assert_eq!(SHANK_MM, 3.175);
    assert_eq!(TOTAL_MM, 38.1);
}

#[test]
fn the_kinds_split_the_catalogue_without_loss_and_keep_its_order() {
    let all = catalogue::all();
    assert_eq!(catalogue::of_kind(None), all.to_vec());
    let mut sum = 0;
    for k in Kind::ALL {
        let of = catalogue::of_kind(Some(k));
        assert!(!of.is_empty(), "{k:?}");
        assert!(of.iter().all(|b| b.kind == k));
        let expected: Vec<Bit> = all.iter().copied().filter(|b| b.kind == k).collect();
        assert_eq!(of, expected, "{k:?}");
        sum += of.len();
    }
    assert_eq!(sum, all.len());
}

#[test]
fn the_generic_bits_come_in_the_expected_numbers() {
    for (kind, n) in [
        (Kind::EndMill, 14),
        (Kind::VBit, 28),
        (Kind::BallNose, 5),
        (Kind::Spot, 9),
        (Kind::Countersink, 9),
        (Kind::UpCut, 15),
        (Kind::Diamond, 26),
        (Kind::Coated, 13),
    ] {
        assert_eq!(catalogue::of_kind(Some(kind)).len(), n, "{kind:?}");
    }
}

#[test]
fn the_end_mills_follow_the_generic_rules_and_their_clamps() {
    // small: the feed is lifted to its floor
    let e = find(Kind::EndMill, "", 0.2);
    assert_eq!((e.rpm, e.feed_mm_min, e.plunge_mm_min), (60_000, 150, 50));
    assert!(close(e.pass_mm, 0.1));
    let e = find(Kind::EndMill, "", 0.5);
    assert_eq!((e.rpm, e.feed_mm_min, e.plunge_mm_min), (60_000, 350, 116));
    assert!(close(e.pass_mm, 0.25));
    // from 1 mm the spindle turns slower
    let e = find(Kind::EndMill, "", 1.0);
    assert_eq!((e.rpm, e.feed_mm_min, e.plunge_mm_min), (40_000, 700, 233));
    assert!(close(e.pass_mm, 0.5));
    // big: the feed and the pass are cut to their ceilings
    for d in [2.0, 3.175] {
        let e = find(Kind::EndMill, "", d);
        assert_eq!((e.rpm, e.feed_mm_min, e.plunge_mm_min), (40_000, 900, 300));
        assert!(close(e.pass_mm, 0.8), "{d}");
    }
    for d in [0.2, 0.5, 1.0, 2.0, 3.175] {
        let e = find(Kind::EndMill, "", d);
        assert_eq!((e.flutes, e.angle_deg), (2, 0));
        assert_eq!(e.tip_mm, 0.0);
    }
}

#[test]
fn the_v_bits_are_seven_angles_by_four_tips() {
    let v = catalogue::of_kind(Some(Kind::VBit));
    let mut pairs: Vec<(u32, u32)> = v
        .iter()
        .map(|b| (b.angle_deg, (b.tip_mm * 100.0).round() as u32))
        .collect();
    pairs.sort_unstable();
    pairs.dedup();
    assert_eq!(pairs.len(), 28);
    let mut angles: Vec<u32> = pairs.iter().map(|p| p.0).collect();
    angles.dedup();
    assert_eq!(angles, [10, 15, 20, 30, 45, 60, 90]);
    let mut tips: Vec<u32> = pairs.iter().map(|p| p.1).collect();
    tips.sort_unstable();
    tips.dedup();
    assert_eq!(tips, [5, 10, 20, 30]);
    for b in &v {
        assert_eq!(b.diameter_mm, 3.175);
        assert_eq!(
            (b.rpm, b.feed_mm_min, b.plunge_mm_min, b.flutes),
            (40_000, 400, 150, 2)
        );
    }
}

#[test]
fn the_groove_of_a_v_bit_is_bounded_by_the_tip_and_the_shoulder() {
    let v = vbit(90, 0.1, 3.175);
    // 90 degrees: the groove opens by twice the depth
    assert!(close(v.groove_mm(1.0).unwrap(), 2.1));
    assert_eq!(v.groove_mm(50.0), Some(3.175));
    // a depth that is below the surface or not a number is the tip
    assert_eq!(v.groove_mm(-4.0), Some(0.1));
    assert_eq!(v.groove_mm(f64::NAN), Some(0.1));
    // no angle, or not a V-bit: no groove
    assert_eq!(vbit(0, 0.1, 3.175).groove_mm(1.0), None);
    assert_eq!(vbit(0, 0.1, 3.175).depth_for_groove_mm(1.0), None);
    let drill = plain(Kind::Drill, 1.0, 50_000, 0, 600);
    assert!(drill.groove_mm(0.1).is_none() && drill.depth_for_groove_mm(0.3).is_none());
    // the width asked for has to be between the tip and the shoulder
    assert_eq!(v.depth_for_groove_mm(0.1), Some(0.0));
    assert_eq!(v.depth_for_groove_mm(0.0999), None);
    assert_eq!(v.depth_for_groove_mm(3.1751), None);
    assert!(v.depth_for_groove_mm(3.175).is_some());
    // the groove grows with the depth and the depth that gives a width gives it back
    let thin = vbit(30, 0.1, 3.175);
    let mut last = 0.0;
    for i in 0..=40 {
        let w = thin.groove_mm(f64::from(i) * 0.1).unwrap();
        assert!(w >= last);
        last = w;
    }
    for b in catalogue::of_kind(Some(Kind::VBit)) {
        for f in [0.0, 0.25, 0.5, 0.75] {
            let w = b.tip_mm + (b.diameter_mm - b.tip_mm) * f;
            let depth = b.depth_for_groove_mm(w).unwrap();
            assert!(close(b.groove_mm(depth).unwrap(), w), "{b:?} {f}");
        }
    }
}

#[test]
fn a_slower_spindle_never_gives_the_bit_less_than_the_floor_of_the_feed() {
    let b = plain(Kind::Drill, 1.0, 40_000, 0, 20);
    let s = b.for_spindle(1_000);
    assert_eq!(s.rpm, 1_000);
    // a feed that is zero stays zero, and any other is not below 10
    assert_eq!((s.feed_mm_min, s.plunge_mm_min), (0, 10));
    assert_eq!(b.for_spindle(1).plunge_mm_min, 10);
    // the rest of the bit does not change
    assert_eq!(
        (s.kind, s.diameter_mm, s.pass_mm),
        (b.kind, b.diameter_mm, b.pass_mm)
    );
    // exactly its own turns: the same bit; one turn less: another bit
    assert_eq!(b.for_spindle(40_000), b);
    assert_ne!(b.for_spindle(39_999), b);
    assert_eq!(b.for_spindle(39_999).rpm, 39_999);
    // a spindle that is not known and a bit that asks for no turns change nothing
    assert_eq!(b.for_spindle(0), b);
    let idle = plain(Kind::Drill, 1.0, 0, 0, 20);
    assert_eq!(idle.for_spindle(1_000), idle);
    // the same turns again changes nothing more
    let once = plain(Kind::EndMill, 1.0, 40_000, 700, 233).for_spindle(12_345);
    assert_eq!(once.for_spindle(12_345), once);
}

#[test]
fn each_kind_has_its_own_series_and_the_generic_bits_have_none() {
    let series = |k: Kind| -> BTreeSet<&'static str> {
        catalogue::of_kind(Some(k))
            .iter()
            .map(|b| b.series)
            .collect()
    };
    assert_eq!(
        series(Kind::Drill),
        BTreeSet::from([
            "ID30", "ID30C", "MC", "MCV", "MCW", "MD", "MHW", "MV", "ST", "STB", "UC", "UHW",
            "UM30", "UM35", "UV", "UW"
        ])
    );
    assert_eq!(series(Kind::SlotDrill), BTreeSet::from(["SX", "SX24"]));
    assert_eq!(series(Kind::Diamond), BTreeSet::from(["RCM"]));
    assert_eq!(series(Kind::DownCut), BTreeSet::from(["RHM", "RS"]));
    assert_eq!(series(Kind::ChipBreaker), BTreeSet::from(["RLE", "SC"]));
    assert_eq!(series(Kind::UpCut), BTreeSet::from(["RPEM"]));
    assert_eq!(series(Kind::Coated), BTreeSet::from(["DCET", "DCRPEM"]));
    for k in [
        Kind::EndMill,
        Kind::VBit,
        Kind::BallNose,
        Kind::Spot,
        Kind::Countersink,
    ] {
        assert_eq!(series(k), BTreeSet::from([""]), "{k:?}");
    }
    // inside a series the diameters only grow
    for w in catalogue::all().windows(2) {
        if w[0].kind == w[1].kind && w[0].series == w[1].series && !w[0].series.is_empty() {
            assert!(w[0].diameter_mm < w[1].diameter_mm, "{:?} {:?}", w[0], w[1]);
        }
    }
}

#[test]
fn the_small_drills_have_the_flute_of_their_diameter() {
    for (d, flute) in [
        (0.30, 5.0),
        (0.35, 5.0),
        (0.40, 6.5),
        (0.45, 6.5),
        (0.50, 6.5),
    ] {
        assert_eq!(find(Kind::Drill, "UC", d).flute_mm, flute, "{d}");
    }
    assert_eq!(find(Kind::Drill, "UV", 0.55).flute_mm, 7.0);
    assert_eq!(find(Kind::Drill, "UV", 0.60).flute_mm, 8.5);
    assert_eq!(find(Kind::Drill, "UW", 0.55).flute_mm, 7.0);
    // the largest of each series, and the flutes
    let drills = catalogue::of_kind(Some(Kind::Drill));
    for (series, last, flutes) in [
        ("UC", 0.50, 2),
        ("UV", 0.75, 2),
        ("UW", 0.55, 2),
        ("UHW", 0.65, 1),
    ] {
        let of: Vec<&Bit> = drills.iter().filter(|b| b.series == series).collect();
        let max = of.iter().map(|b| b.diameter_mm).fold(0.0, f64::max);
        assert!(close(max, last), "{series} {max}");
        assert!(of.iter().all(|b| b.flutes == flutes), "{series}");
    }
    // the micro series does not say the length of the flute
    assert!(drills
        .iter()
        .filter(|b| b.series == "MD")
        .all(|b| b.flute_mm == 0.0));
}

#[test]
fn the_speeds_of_each_bit_come_from_the_tables_of_its_kind() {
    for kind in [Kind::Drill, Kind::SlotDrill] {
        for b in catalogue::of_kind(Some(kind)) {
            let (rpm, infeed) = catalogue::drill_speed(b.diameter_mm);
            assert_eq!((b.rpm, b.plunge_mm_min), (rpm, infeed), "{b:?}");
            assert_eq!(b.feed_mm_min, 0, "{b:?}");
            assert!(close(b.pass_mm, 1.8), "{b:?}");
        }
    }
    for kind in [
        Kind::Diamond,
        Kind::DownCut,
        Kind::ChipBreaker,
        Kind::UpCut,
        Kind::Coated,
    ] {
        for b in catalogue::of_kind(Some(kind)) {
            let (rpm, feed) = catalogue::router_speed(b.diameter_mm);
            assert_eq!((b.rpm, b.feed_mm_min), (rpm, feed), "{b:?}");
            assert_eq!(b.plunge_mm_min, (feed / 2).clamp(100, 400), "{b:?}");
            assert!(
                close(b.pass_mm, (b.diameter_mm * 0.6).clamp(0.4, 1.0)),
                "{b:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// machines
// ---------------------------------------------------------------------------------------------

#[test]
fn the_models_are_listed_in_the_order_of_the_profiles() {
    let names: Vec<String> = machines::models().iter().map(|m| m.name()).collect();
    assert_eq!(
        names,
        [
            "CCD/2 MTC",
            "CCD/2 ATC",
            "CCD MTC",
            "CCD ATC",
            "CCD Premium MTC",
            "CCD Premium ATC",
            "CCD XL MTC",
            "CCD XL ATC",
            "CCD/2 ECO MTC",
            "CCD/2 Precision ATC"
        ]
    );
}

#[test]
fn a_model_is_found_by_its_exact_name_with_outer_spaces_forgiven() {
    let all = machines::models();
    for (i, m) in all.iter().enumerate() {
        assert_eq!(Model::from_name(&m.name()), Some(i));
        assert_eq!(Model::from_name(&format!("  {}\n", m.name())), Some(i));
    }
    for bad in [
        "ccd mtc",
        "CCD",
        "",
        "CCD  MTC",
        "CCD/2 ECO ATC",
        "CCD/2 Precision MTC",
        "CCD MTC CCD MTC",
    ] {
        assert_eq!(Model::from_name(bad), None, "{bad:?}");
    }
}

#[test]
fn a_profile_number_past_the_end_is_the_last_profile_and_the_rack_follows_the_change() {
    let len = machines::PROFILES.len();
    for profile in [len - 1, len, usize::MAX] {
        let m = Model {
            profile,
            change: ToolChange::Manual,
        };
        assert_eq!(m.profile().name, "CCD/2 Precision");
        assert_eq!(m.name(), "CCD/2 Precision MTC");
        assert_eq!(m.rack_places(), 0);
        let a = Model {
            profile,
            change: ToolChange::Automatic,
        };
        assert_eq!(a.name(), "CCD/2 Precision ATC");
        assert_eq!(a.rack_places(), 15);
    }
    let all = machines::models();
    for (name, places) in [
        ("CCD/2 MTC", 0),
        ("CCD/2 ATC", 15),
        ("CCD ATC", 15),
        ("CCD Premium ATC", 30),
        ("CCD XL ATC", 25),
        ("CCD/2 ECO MTC", 0),
        ("CCD/2 Precision ATC", 15),
    ] {
        let i = Model::from_name(name).unwrap();
        assert_eq!(all[i].rack_places(), places, "{name}");
    }
}

#[test]
fn the_profiles_are_consistent_with_each_other() {
    for (i, p) in machines::PROFILES.iter().enumerate() {
        assert!(p.overall_mm.0 > p.work_mm.0, "{}", p.name);
        assert!(p.overall_mm.1 > p.work_mm.1, "{}", p.name);
        assert!(p.overall_mm.2 > p.work_mm.2, "{}", p.name);
        assert!(p.weight_kg > 0.0 && !p.changes.is_empty(), "{}", p.name);
        // the places of the changer exist exactly where the changer is sold
        assert_eq!(
            p.tool_places == 0,
            !p.changes.contains(&ToolChange::Automatic),
            "{}",
            p.name
        );
        assert_eq!(p.work_mm_xy(), (p.work_mm.0, p.work_mm.1));
        if let Some((lo, hi)) = p.spindle_rpm {
            assert!(lo < hi, "{}", p.name);
        }
        for q in &machines::PROFILES[i + 1..] {
            assert_ne!(p.name, q.name);
            assert_ne!(p.family, q.family);
        }
    }
}

#[test]
fn what_the_maker_publishes_is_kept_per_machine() {
    let by_name = |name: &str| {
        machines::PROFILES
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("{name}"))
    };
    let ccd = by_name("CCD");
    assert_eq!(ccd.spindle_rpm, Some((5_000, 63_000)));
    assert_eq!(
        (ccd.max_feed_mm_min, ccd.step_um),
        (Some(9_000), Some(3.175))
    );
    for name in ["CCD Premium", "CCD XL"] {
        let p = by_name(name);
        assert_eq!(
            (p.spindle_rpm, p.max_feed_mm_min, p.step_um),
            (None, None, None)
        );
    }
    let eco = by_name("CCD/2 ECO");
    assert_eq!(eco.spindle_rpm, Some((5_000, 35_000)));
    assert_eq!((eco.max_feed_mm_min, eco.step_um), (None, None));
    let precision = by_name("CCD/2 Precision");
    assert_eq!(precision.max_feed_mm_min, Some(7_000));
    assert_eq!(precision.step_um, Some(0.79));
    let weights: Vec<f64> = machines::PROFILES.iter().map(|p| p.weight_kg).collect();
    assert_eq!(weights, [30.0, 35.0, 80.0, 49.0, 30.0, 30.0]);
    let places: Vec<u8> = machines::PROFILES.iter().map(|p| p.tool_places).collect();
    assert_eq!(places, [15, 15, 30, 25, 0, 15]);
}

#[test]
fn the_machine_for_a_table_follows_the_distance_in_proportion() {
    for (w, d, name) in [
        (410.0, 300.0, "CCD"),
        (380.0, 290.0, "CCD/2"),
        (500.0, 350.0, "CCD"),
        (500.0, 380.0, "CCD Premium"),
        (1.0e9, 1.0e9, "CCD XL"),
        (1.0, 1.0, "CCD/2"),
    ] {
        assert_eq!(machines::for_table(&Table::new(w, d)).name, name, "{w} {d}");
    }
    // a table of no size is no better for any machine: the first one is taken
    assert_eq!(machines::for_table(&Table::new(0.0, 0.0)).name, "CCD/2");
    // the model of a table always has the manual change and the machine of that table
    for (w, d) in [
        (495.0, 325.0),
        (325.0, 270.0),
        (500.0, 400.0),
        (600.0, 500.0),
        (1.0, 1.0),
        (0.0, 0.0),
        (1.0e9, 1.0e9),
    ] {
        let t = Table::new(w, d);
        let m = machines::models()[Model::for_table(&t)];
        assert_eq!(m.change, ToolChange::Manual);
        assert_eq!(m.profile().name, machines::for_table(&t).name);
    }
    let xl = Model::for_table(&Table::new(600.0, 500.0));
    assert_eq!(machines::models()[xl].name(), "CCD XL MTC");
}

#[test]
fn the_plate_margins_are_exact_for_the_published_sizes_and_never_below_two_centimetres() {
    let by_name = |name: &str| {
        machines::PROFILES
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("{name}"))
    };
    for (name, margins) in [
        ("CCD/2", (147.5, 40.0)),
        ("CCD", (112.5, 87.5)),
        ("CCD Premium", (185.0, 100.0)),
        ("CCD XL", (135.0, 125.0)),
        ("CCD/2 ECO", (147.5, 40.0)),
        ("CCD/2 Precision", (147.5, 40.0)),
    ] {
        let p = by_name(name);
        assert_eq!(p.plate_margins(p.work_mm_xy()), margins, "{name}");
        // a usable area that is smaller than the published one does not move the plate
        assert_eq!(p.plate_margins((1.0, 1.0)), margins, "{name}");
        assert_eq!(p.plate_margins((0.0, 0.0)), margins, "{name}");
    }
    for p in machines::PROFILES {
        assert_eq!(
            p.plate_margins((5_000.0, 5_000.0)),
            (20.0, 20.0),
            "{}",
            p.name
        );
        // the margins never grow with the usable area
        let mut last = p.plate_margins((0.0, 0.0));
        for i in 1..=40 {
            let u = f64::from(i) * 50.0;
            let now = p.plate_margins((u, u));
            assert!(now.0 <= last.0 && now.1 <= last.1, "{} {u}", p.name);
            last = now;
        }
    }
}

#[test]
fn the_default_bar_follows_the_pitch_the_table_allows() {
    let by_name = |name: &str| {
        machines::PROFILES
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("{name}"))
    };
    // the CCD/2: 15 places in 230 mm of the Y that is left, so 14.375 mm apart
    let (first, last, n) = by_name("CCD/2").default_bar();
    assert_eq!(n, 15);
    assert!(close(first.0, 365.0) && close(last.0, 365.0));
    assert!(close(first.1, 34.375) && close(last.1, 235.625));
    assert!(close(first.2, 37.4) && close(last.2, 37.4));
    // the CCD: 17.8125 mm apart, and the bar starts 40 mm past the longer table
    let (first, last, n) = by_name("CCD").default_bar();
    assert_eq!(n, 15);
    assert!(close(first.0, 535.0));
    assert!(close(first.1, 37.8125) && close(last.1, 287.1875));
    // a model with no changer still has a one-place bar that the drawing can start from
    let (first, last, n) = by_name("CCD/2 ECO").default_bar();
    assert_eq!(n, 1);
    assert!(close(first.1, 40.0) && close(last.1, 40.0));
    assert!(close(first.0, last.0) && close(first.2, last.2));
    // the deeper travel of the Premium gives a deeper grip
    let (first, _, n) = by_name("CCD Premium").default_bar();
    assert_eq!(n, 30);
    assert!(close(first.0, 540.0) && close(first.2, 63.4));
}

#[test]
fn a_fitted_plan_is_exact_when_everything_is_a_round_number() {
    let table = Table::new(325.0, 270.0);
    let v = machines::fit_plan(
        &table,
        (0.0, 0.0, 0.0, 0.0),
        (670.0, 560.0),
        (10.0, 10.0, 10.0, 10.0),
    );
    assert_eq!(v.scale, 2.0);
    assert_eq!(v.origin, (10.0, 10.0));
    // a bar on the left of the table moves the origin to the right
    let square = Table::new(100.0, 100.0);
    let v = machines::fit_plan(
        &square,
        (-50.0, 0.0, 0.0, 0.0),
        (200.0, 100.0),
        (0.0, 0.0, 0.0, 0.0),
    );
    assert_eq!(v.scale, 1.0);
    assert_eq!(v.origin, (75.0, 0.0));
    // a bar above it moves the origin down
    let v = machines::fit_plan(
        &square,
        (0.0, 0.0, 100.0, 150.0),
        (100.0, 150.0),
        (0.0, 0.0, 0.0, 0.0),
    );
    assert_eq!(v.scale, 1.0);
    assert_eq!(v.origin, (0.0, 50.0));
}

#[test]
fn a_fitted_plan_in_a_window_smaller_than_its_room_does_not_break() {
    let table = Table::new(325.0, 270.0);
    let v = machines::fit_plan(
        &table,
        (0.0, 0.0, 0.0, 0.0),
        (10.0, 10.0),
        (20.0, 20.0, 20.0, 20.0),
    );
    assert!(v.scale > 0.0 && v.scale.is_finite());
    assert!(v.origin.0.is_finite() && v.origin.1.is_finite());
    // a table of no size
    let v = machines::fit_plan(
        &Table::new(0.0, 0.0),
        (0.0, 0.0, 0.0, 0.0),
        (100.0, 100.0),
        (0.0, 0.0, 0.0, 0.0),
    );
    assert!(close(v.scale, 100.0));
    assert!(v.origin.0.is_finite() && v.origin.1.is_finite());
}

// ---------------------------------------------------------------------------------------------
// slots
// ---------------------------------------------------------------------------------------------

fn drill(tool: u8, um: u32) -> JobTool {
    JobTool {
        prog: Program::Drill,
        tool,
        diameter_um: um,
    }
}

fn cutter(tool: u8) -> JobTool {
    JobTool {
        prog: Program::Route,
        tool,
        diameter_um: 0,
    }
}

/// The places of a machine file: the test pin, fourteen places, the detector and one more.
fn bar() -> Vec<usize> {
    (0..=16).collect()
}

#[test]
fn only_the_places_one_to_fourteen_can_be_chosen_and_each_is_listed_once() {
    assert_eq!(MAX_TOOL_SLOT, 14);
    assert_eq!(selectable(&[0, 1, 14, 15, 16, usize::MAX]), [1, 14]);
    assert_eq!(selectable(&[9, 3, 9, 3, 5]), [3, 5, 9]);
    assert!(selectable(&[]).is_empty());
    assert!(selectable(&[0, 15]).is_empty());
    // giving a tool a place again moves it to the end and never repeats it
    let mut a: Assignment = Vec::new();
    set(&mut a, Program::Drill, 1, None);
    assert!(a.is_empty());
    set(&mut a, Program::Drill, 1, Some(3));
    set(&mut a, Program::Route, 1, Some(4));
    set(&mut a, Program::Drill, 1, Some(3));
    assert_eq!(a, [(Program::Route, 1, 4), (Program::Drill, 1, 3)]);
    // a place is whatever number is given: the check is what says it is not valid
    set(&mut a, Program::Drill, 2, Some(usize::MAX));
    assert_eq!(slot_of(&a, Program::Drill, 2), Some(usize::MAX));
}

#[test]
fn the_problems_come_unassigned_first_and_shared_in_the_order_the_places_were_met() {
    let tools = [
        drill(1, 800),
        drill(2, 800),
        drill(3, 800),
        cutter(7),
        drill(4, 800),
    ];
    let mut a: Assignment = Vec::new();
    set(&mut a, Program::Drill, 1, Some(6));
    set(&mut a, Program::Drill, 3, Some(5));
    set(&mut a, Program::Route, 7, Some(5));
    set(&mut a, Program::Drill, 4, Some(6));
    assert_eq!(
        check(&tools, &a, &bar()),
        [
            Problem::Unassigned(Program::Drill, 2),
            Problem::Shared(6, vec![(Program::Drill, 1), (Program::Drill, 4)]),
            Problem::Shared(5, vec![(Program::Drill, 3), (Program::Route, 7)]),
        ]
    );
    // no tools: nothing to say, whatever the assignment holds
    assert!(check(&[], &a, &bar()).is_empty());
}

#[test]
fn a_place_that_does_not_exist_is_told_for_each_tool_and_for_the_sharing() {
    let mut a: Assignment = Vec::new();
    set(&mut a, Program::Drill, 1, Some(20));
    set(&mut a, Program::Drill, 2, Some(20));
    assert_eq!(
        check(&[drill(1, 800), drill(2, 800)], &a, &bar()),
        [
            Problem::Undefined(Program::Drill, 1, 20),
            Problem::Undefined(Program::Drill, 2, 20),
            Problem::Shared(20, vec![(Program::Drill, 1), (Program::Drill, 2)]),
        ]
    );
    // the edges of the bar: 0 and 15 are not places for a tool, 1 and 14 are
    let tools = [drill(1, 800), drill(2, 800), drill(3, 800), drill(4, 800)];
    let mut b: Assignment = Vec::new();
    for (tool, slot) in [(1, 0), (2, 1), (3, 14), (4, 15)] {
        set(&mut b, Program::Drill, tool, Some(slot));
    }
    assert_eq!(
        check(&tools, &b, &bar()),
        [
            Problem::Undefined(Program::Drill, 1, 0),
            Problem::Undefined(Program::Drill, 4, 15)
        ]
    );
    // a machine file with no places leaves every assigned tool without one, and a huge number does not break
    let mut c: Assignment = Vec::new();
    set(&mut c, Program::Drill, 1, Some(3));
    assert_eq!(
        check(&[drill(1, 800)], &c, &[]),
        [Problem::Undefined(Program::Drill, 1, 3)]
    );
    set(&mut c, Program::Drill, 1, Some(usize::MAX));
    assert_eq!(
        check(&[drill(1, 800)], &c, &bar()),
        [Problem::Undefined(Program::Drill, 1, usize::MAX)]
    );
}

#[test]
fn filling_in_order_goes_by_size_then_by_number_and_leaves_what_is_not_in_the_job_alone() {
    let tools = [
        drill(5, 800),
        drill(2, 800),
        drill(9, 500),
        cutter(3),
        cutter(1),
    ];
    let mut a: Assignment = Vec::new();
    fill_in_order(&tools, &mut a, &bar());
    assert_eq!(
        a,
        [
            (Program::Drill, 9, 1),
            (Program::Drill, 2, 2),
            (Program::Drill, 5, 3),
            (Program::Route, 1, 4),
            (Program::Route, 3, 5),
        ]
    );
    // the place of a tool that is not in the job does not keep the place reserved
    let mut b: Assignment = vec![(Program::Drill, 99, 1)];
    fill_in_order(&[drill(1, 800)], &mut b, &bar());
    assert_eq!(slot_of(&b, Program::Drill, 99), Some(1));
    assert_eq!(slot_of(&b, Program::Drill, 1), Some(1));
}

#[test]
fn when_the_places_run_out_the_biggest_tools_are_left_and_the_check_says_so() {
    let tools: Vec<JobTool> = (1..=16u8).map(|n| drill(n, u32::from(n) * 100)).collect();
    let mut a: Assignment = Vec::new();
    fill_in_order(&tools, &mut a, &bar());
    assert_eq!(a.len(), 14);
    assert_eq!(slot_of(&a, Program::Drill, 14), Some(14));
    assert_eq!(slot_of(&a, Program::Drill, 15), None);
    let mut used: Vec<usize> = a.iter().map(|x| x.2).collect();
    used.sort_unstable();
    assert_eq!(used, (1..=14).collect::<Vec<usize>>());
    assert_eq!(
        check(&tools, &a, &bar()),
        [
            Problem::Unassigned(Program::Drill, 15),
            Problem::Unassigned(Program::Drill, 16)
        ]
    );
    // only the places the file defines are used, and a tool that has one keeps it
    let three = [drill(1, 500), drill(2, 600), drill(3, 700)];
    let mut b: Assignment = vec![(Program::Drill, 2, 7)];
    fill_in_order(&three, &mut b, &[0, 3, 7, 9, 20]);
    assert_eq!(
        b,
        [
            (Program::Drill, 2, 7),
            (Program::Drill, 1, 3),
            (Program::Drill, 3, 9)
        ]
    );
}

#[test]
fn any_job_that_fits_the_bar_is_filled_without_problems_and_filling_twice_is_the_same() {
    let mut rng = Lcg(7);
    for _ in 0..25 {
        let n = 1 + rng.below(14) as u8;
        let tools: Vec<JobTool> = (1..=n)
            .map(|t| {
                if rng.below(3) == 0 {
                    cutter(t)
                } else {
                    drill(t, rng.below(3_000) as u32)
                }
            })
            .collect();
        let mut a: Assignment = Vec::new();
        fill_in_order(&tools, &mut a, &bar());
        assert!(check(&tools, &a, &bar()).is_empty(), "{tools:?} {a:?}");
        let before = a.clone();
        fill_in_order(&tools, &mut a, &bar());
        assert_eq!(a, before);
    }
}

#[test]
fn the_places_go_to_the_planner_by_program_in_the_order_they_were_given() {
    let mut a: Assignment = Vec::new();
    assert!(engine_slots(&a, Program::Drill).is_empty());
    set(&mut a, Program::Drill, 2, Some(4));
    set(&mut a, Program::Route, 1, Some(3));
    set(&mut a, Program::Drill, 1, Some(5));
    assert_eq!(engine_slots(&a, Program::Drill), [(2, 4), (1, 5)]);
    assert_eq!(engine_slots(&a, Program::Route), [(1, 3)]);
}

#[test]
fn the_text_of_the_places_drops_what_is_out_of_range_and_forgives_spaces() {
    let b = Book::parse(
        "v1;d800#1=0,d800#2=1,d800#3=99,d800#4=100,d800#256=3,d-5#1=3,d4294967295#1=3,r#1=3,\u{e9}800#1=3",
    );
    assert_eq!(b.to_text(), "v1;d800#2=1,d800#3=99,d4294967295#1=3");
    assert_eq!(Book::parse("v1;d 800 # 1 = 5").to_text(), "v1;d800#1=5");
    // the version has to be exactly v1 followed by a semicolon
    for bad in [
        "",
        "v1",
        "v1;",
        "v1;;;",
        "V1;d800#1=5",
        "v10;d800#1=5",
        "v1,d800#1=5",
        "v1 ;d800#1=5",
    ] {
        assert!(Book::parse(bad).is_empty(), "{bad:?}");
    }
    // what is left out of a line does not take the rest with it
    assert_eq!(
        Book::parse("v1;d800#1=5;extra,r0#7=14").to_text(),
        "v1;r0#7=14"
    );
}

#[test]
fn a_text_with_more_places_than_the_book_holds_keeps_the_first_two_hundred() {
    let body: Vec<String> = (1..=250).map(|d| format!("d{d}#1=3")).collect();
    let b = Book::parse(&format!("v1;{}", body.join(",")));
    let text = b.to_text();
    assert_eq!(text.matches(',').count(), 199);
    assert!(text.starts_with("v1;d1#1=3,"));
    assert!(text.ends_with(",d200#1=3"));
}

#[test]
fn remembering_more_than_the_book_holds_keeps_the_newest_two_hundred() {
    let tools: Vec<JobTool> = (0..=249u8).map(|t| drill(t, 800)).collect();
    let a: Assignment = tools.iter().map(|t| (t.prog, t.tool, 5)).collect();
    let mut book = Book::default();
    book.remember(&tools, &a);
    let text = book.to_text();
    assert_eq!(text.matches(',').count(), 199);
    assert!(text.starts_with("v1;d800#50=5,"));
    assert!(text.ends_with(",d800#249=5"));
    // what is written can be read back to the same book
    assert_eq!(Book::parse(&text), book);
}

#[test]
fn a_tool_is_remembered_once_a_tool_without_a_place_is_skipped_and_unknown_sizes_evict() {
    let mut book = Book::default();
    let mut a: Assignment = Vec::new();
    set(&mut a, Program::Drill, 1, Some(3));
    book.remember(&[drill(1, 800)], &a);
    set(&mut a, Program::Drill, 1, Some(4));
    book.remember(&[drill(1, 800)], &a);
    assert_eq!(book.to_text(), "v1;d800#1=4");
    book.remember(&[drill(2, 600)], &a);
    assert_eq!(book.to_text(), "v1;d800#1=4");
    // two drills of one size can be in the same place: their numbers tell them apart
    let mut shared: Assignment = Vec::new();
    set(&mut shared, Program::Drill, 1, Some(5));
    set(&mut shared, Program::Drill, 2, Some(5));
    let mut b = Book::default();
    b.remember(&[drill(1, 800), drill(2, 800)], &shared);
    assert_eq!(b.to_text(), "v1;d800#1=5,d800#2=5");
    // two cutters, whose size is not known, cannot: the last one takes the place
    let mut cutters: Assignment = Vec::new();
    set(&mut cutters, Program::Route, 7, Some(5));
    set(&mut cutters, Program::Route, 8, Some(5));
    let mut c = Book::default();
    c.remember(&[cutter(7), cutter(8)], &cutters);
    assert_eq!(c.to_text(), "v1;r0#8=5");
}

#[test]
fn a_tool_is_found_by_size_then_by_number_and_only_where_the_machine_has_the_place() {
    let book = Book::parse("v1;d800#1=3,d800#2=6");
    let got = |tools: &[JobTool], defined: &[usize], prog: Program, tool: u8| {
        slot_of(&book.recall(tools, defined), prog, tool)
    };
    // among the places of one size the one with the same number
    assert_eq!(got(&[drill(2, 800)], &bar(), Program::Drill, 2), Some(6));
    // and when there is none, the first one that was remembered
    assert_eq!(got(&[drill(7, 800)], &bar(), Program::Drill, 7), Some(3));
    // a size that is not known is found by the number
    assert_eq!(got(&[drill(2, 0)], &bar(), Program::Drill, 2), Some(6));
    assert_eq!(got(&[drill(9, 0)], &bar(), Program::Drill, 9), None);
    // a cutter does not take the place of a drill
    assert_eq!(got(&[cutter(1)], &bar(), Program::Route, 1), None);
    // the place that the machine no longer has is not given
    assert_eq!(got(&[drill(2, 800)], &[3, 4], Program::Drill, 2), Some(3));
    assert_eq!(got(&[drill(2, 800)], &[4], Program::Drill, 2), None);
}

#[test]
fn forgetting_by_number_when_the_size_is_not_known_and_never_across_programs() {
    let mut book = Book::parse("v1;d800#1=3,d1000#2=4,r0#7=9");
    book.forget(&[]);
    assert_eq!(book.to_text(), "v1;d800#1=3,d1000#2=4,r0#7=9");
    book.forget(&[cutter(1)]);
    assert_eq!(book.to_text(), "v1;d800#1=3,d1000#2=4,r0#7=9");
    book.forget(&[drill(1, 0)]);
    assert_eq!(book.to_text(), "v1;d1000#2=4,r0#7=9");
    book.forget(&[cutter(7)]);
    assert_eq!(book.to_text(), "v1;d1000#2=4");
    book.forget(&[drill(2, 1000)]);
    assert!(book.is_empty());
    assert!(book.to_text().is_empty());
}

#[test]
fn a_messy_text_becomes_a_canonical_one_that_reads_back_the_same() {
    let book = Book::parse(" v1; d800#1=5 ,x,d800#1=5, r0#7=14 ,d10#255=99,d10#256=99 ");
    let text = book.to_text();
    assert_eq!(text, "v1;d800#1=5,d800#1=5,r0#7=14,d10#255=99");
    let again = Book::parse(&text);
    assert_eq!(again, book);
    assert_eq!(again.to_text(), text);
}

// ---------------------------------------------------------------------------------------------
// inspection
// ---------------------------------------------------------------------------------------------

#[test]
fn a_hole_is_in_view_when_it_is_wholly_inside_and_the_centre_comes_first() {
    let job = [
        (10.0, 10.0, 1.0),
        (16.5, 10.0, 1.0),
        (16.75, 10.0, 1.0),
        (10.0, 14.5, 1.0),
        (10.0, 14.75, 1.0),
        (3.5, 10.0, 1.0),
        (3.25, 10.0, 1.0),
    ];
    let v = holes_in_view(&job, 0, (14.0, 10.0));
    let xy: Vec<(f64, f64)> = v.iter().map(|h| (h.x_mm, h.y_mm)).collect();
    assert_eq!(xy, [(0.0, 0.0), (6.5, 0.0), (0.0, 4.5), (-6.5, 0.0)]);
    // the centre is put first even when it comes last in the job
    let line = [(10.0, 10.0, 1.0), (11.0, 10.0, 1.0), (12.0, 10.0, 1.0)];
    let v = holes_in_view(&line, 2, (14.0, 10.0));
    let xs: Vec<f64> = v.iter().map(|h| h.x_mm).collect();
    assert_eq!(xs, [0.0, -2.0, -1.0]);
    // a centre hole wider than the picture is not offered either
    assert!(holes_in_view(&[(10.0, 10.0, 20.0)], 0, (14.0, 10.0)).is_empty());
}

#[test]
fn a_picture_with_no_size_or_no_job_sees_nothing() {
    assert!(holes_in_view(&[], 0, (14.0, 10.0)).is_empty());
    assert!(holes_in_view(&[(1.0, 1.0, 0.5)], 1, (14.0, 10.0)).is_empty());
    assert!(holes_in_view(&[(1.0, 1.0, 0.5)], usize::MAX, (14.0, 10.0)).is_empty());
    assert!(holes_in_view(&[(1.0, 1.0, 0.5)], 0, (-4.0, -4.0)).is_empty());
    assert!(holes_in_view(&[(1.0, 1.0, 0.5)], 0, (0.0, 0.0)).is_empty());
    // a hole with no width in a picture with no size is on the spot
    assert_eq!(holes_in_view(&[(1.0, 1.0, 0.0)], 0, (0.0, 0.0)).len(), 1);
    assert_eq!(DEFAULT_PICTURE, (640.0, 480.0));
}

fn hole(found: bool, expected_mm: f64, measured_mm: f64) -> HoleResult {
    HoleResult {
        index: 0,
        expected: ExpectedHole {
            x_mm: 0.0,
            y_mm: 0.0,
            dia_mm: expected_mm,
        },
        found,
        cx_px: 4.0,
        cy_px: 4.0,
        dia_mm: measured_mm,
        offset_mm: 0.0,
        roundness: 1.0,
        debris: 0.0,
        issues: vec![],
    }
}

fn report_of(holes: Vec<HoleResult>) -> Report {
    let mut r = inspect(&Gray::new(8, 8, 128), &[], &Profile::default());
    r.holes.extend(holes);
    r
}

#[test]
fn the_scale_comes_from_the_first_hole_that_was_found_and_has_a_known_width() {
    let p = Profile {
        px_per_mm: 40.0,
        ..Profile::default()
    };
    assert_eq!(calibrated_scale(&report_of(vec![]), &p), None);
    // a hole that was not found is skipped: 0.5 mm seen as 20 px for a hole of 1 mm
    let r = report_of(vec![hole(false, 0.8, 0.7), hole(true, 1.0, 0.5)]);
    assert!(close(calibrated_scale(&r, &p).unwrap(), 20.0));
    // a hole whose width in the job is zero tells nothing
    assert_eq!(
        calibrated_scale(&report_of(vec![hole(true, 0.0, 0.5)]), &p),
        None
    );
    let r = report_of(vec![hole(true, 0.0, 0.5), hole(true, 2.0, 1.0)]);
    assert!(close(calibrated_scale(&r, &p).unwrap(), 20.0));
    // a spot of one pixel or less is not a hole to measure with
    assert_eq!(
        calibrated_scale(&report_of(vec![hole(true, 0.8, 0.02)]), &p),
        None
    );
    let r = report_of(vec![hole(true, 0.8, 0.03)]);
    assert!(close(calibrated_scale(&r, &p).unwrap(), 1.5));
    // the scale follows the scale of the picture it was measured with
    let double = Profile {
        px_per_mm: 80.0,
        ..Profile::default()
    };
    let r = report_of(vec![hole(true, 1.0, 0.5)]);
    assert!(close(
        calibrated_scale(&r, &double).unwrap(),
        2.0 * calibrated_scale(&r, &p).unwrap()
    ));
}

#[test]
fn the_ten_numbers_have_each_their_own_label_and_field() {
    assert_eq!(Param::ALL.len(), 10);
    for (i, a) in Param::ALL.iter().enumerate() {
        for b in &Param::ALL[i + 1..] {
            assert_ne!(a, b);
            assert_ne!(a.label(), b.label());
        }
    }
    let p = Profile {
        hole_dia_tol_mm: 0.11,
        hole_pos_tol_mm: 0.22,
        hole_min_roundness: 0.33,
        hole_max_debris: 0.44,
        mill_width_mm: 0.55,
        mill_width_tol: 0.66,
        mill_min_width_mm: 0.77,
        mill_max_spread: 0.88,
        min_mean: 99.0,
        min_sharpness: 111.0,
        ..Profile::default()
    };
    let got = Param::ALL.map(|q| q.get(&p));
    assert_eq!(
        got,
        [0.11, 0.22, 0.33, 0.44, 0.55, 0.66, 0.77, 0.88, 99.0, 111.0]
    );
}

#[test]
fn the_numbers_are_written_with_the_digits_and_the_unit_of_the_window() {
    let p = Profile {
        hole_dia_tol_mm: 0.05,
        hole_pos_tol_mm: 1.234,
        hole_min_roundness: 0.9,
        hole_max_debris: 0.05,
        mill_width_mm: 0.2,
        mill_width_tol: 0.25,
        mill_min_width_mm: 0.1,
        mill_max_spread: 0.4,
        min_mean: 44.6,
        min_sharpness: 123.2,
        ..Profile::default()
    };
    let shown: Vec<String> = Param::ALL.iter().map(|q| q.show(&p)).collect();
    assert_eq!(
        shown,
        ["0.05 mm", "1.23 mm", "0.90", "0.05", "0.20 mm", "0.25", "0.10 mm", "0.40", "45", "123"]
    );
}

#[test]
fn nudging_one_number_leaves_the_other_nine_alone() {
    for p in Param::ALL {
        for dir in [1, -1] {
            let mut prof = Profile::default();
            let before = Param::ALL.map(|q| q.get(&prof));
            p.nudge(&mut prof, dir);
            let after = Param::ALL.map(|q| q.get(&prof));
            for (i, q) in Param::ALL.iter().enumerate() {
                if *q != p {
                    assert_eq!(before[i], after[i], "{q:?} moved by {p:?}");
                }
            }
        }
    }
}

#[test]
fn every_number_stops_at_the_end_of_its_range_and_stays_there() {
    let ranges = [
        (Param::HoleDia, 0.005, 1.0),
        (Param::HolePos, 0.01, 2.0),
        (Param::HoleRound, 0.5, 1.0),
        (Param::HoleDebris, 0.0, 1.0),
        (Param::MillWidth, 0.05, 2.0),
        (Param::MillTol, 0.05, 1.0),
        (Param::MillMin, 0.02, 1.0),
        (Param::MillSpread, 0.1, 2.0),
        (Param::MinMean, 5.0, 120.0),
        (Param::MinSharp, 1.0, 400.0),
    ];
    for (p, lo, hi) in ranges {
        let mut up = Profile::default();
        for _ in 0..2_000 {
            p.nudge(&mut up, 1);
        }
        let top = p.get(&up);
        assert!(
            close(top, hi) || (hi - 1e-6..=hi).contains(&top),
            "{p:?} {top}"
        );
        p.nudge(&mut up, 1);
        assert_eq!(p.get(&up), top, "{p:?}");
        let mut down = Profile::default();
        for _ in 0..2_000 {
            p.nudge(&mut down, -1);
        }
        let bottom = p.get(&down);
        assert!((lo - 1e-9..=lo + 5.0).contains(&bottom), "{p:?} {bottom}");
        p.nudge(&mut down, -1);
        assert_eq!(p.get(&down), bottom, "{p:?}");
    }
}

#[test]
fn only_the_sign_of_the_direction_counts_and_up_then_down_comes_back() {
    let mut one = Profile::default();
    let mut many = Profile::default();
    let mut neg = Profile::default();
    let mut min = Profile::default();
    Param::MinMean.nudge(&mut one, 1);
    Param::MinMean.nudge(&mut many, 1_000);
    Param::MinMean.nudge(&mut neg, -1);
    Param::MinMean.nudge(&mut min, i32::MIN);
    assert_eq!(Param::MinMean.get(&one), 50.0);
    assert_eq!(Param::MinMean.get(&many), 50.0);
    assert_eq!(Param::MinMean.get(&neg), 40.0);
    assert_eq!(Param::MinMean.get(&min), 40.0);
    // zero moves nothing
    let mut still = Profile::default();
    Param::MinMean.nudge(&mut still, 0);
    assert_eq!(Param::MinMean.get(&still), 45.0);
    for p in [Param::MinMean, Param::MinSharp] {
        let mut prof = Profile::default();
        let start = p.get(&prof);
        p.nudge(&mut prof, 1);
        p.nudge(&mut prof, -1);
        assert_eq!(p.get(&prof), start, "{p:?}");
    }
}

#[test]
fn the_narrowest_channel_is_kept_below_the_widest() {
    let mut p = Profile {
        mill_max_channel_mm: 0.5,
        ..Profile::default()
    };
    for _ in 0..100 {
        Param::MillMin.nudge(&mut p, 1);
    }
    assert!(close(p.mill_min_width_mm, 0.49), "{}", p.mill_min_width_mm);
    // any other number that is nudged repairs it as well
    let mut q = Profile {
        mill_min_width_mm: 2.0,
        ..Profile::default()
    };
    Param::MinMean.nudge(&mut q, 1);
    assert!(close(q.mill_min_width_mm, q.mill_max_channel_mm - 0.01));
}

#[test]
fn every_finding_code_has_its_own_words_and_a_near_miss_has_none() {
    let codes = [
        "hole_missing",
        "hole_wrong_size",
        "hole_off_centre",
        "hole_not_round",
        "hole_debris",
        "hole_out_of_view",
        "mill_no_channels",
        "mill_too_narrow",
        "mill_too_wide",
        "mill_pinched",
        "mill_uneven",
        "image_too_dark",
        "image_too_bright",
        "image_low_contrast",
        "image_blurry",
        "not_calibrated",
        "nothing_to_check",
        "no_expected_holes",
    ];
    let texts: Vec<Text> = codes.iter().map(|c| finding_text(c).unwrap()).collect();
    for (i, a) in texts.iter().enumerate() {
        for b in &texts[i + 1..] {
            assert_ne!(a, b);
        }
    }
    for near in [
        "",
        "HOLE_MISSING",
        " hole_missing",
        "hole_missing ",
        "hole-missing",
    ] {
        assert_eq!(finding_text(near), None, "{near:?}");
    }
}

#[test]
fn a_profile_file_that_is_missing_empty_or_damaged_gives_the_standard_one() {
    let dir = scratch("profile-missing");
    let file = dir.join("profile.json");
    let missing = load_profile(&file);
    assert_eq!(missing.name, "standard");
    assert_eq!(missing.px_per_mm, DEFAULT_PX_PER_MM);
    for bytes in [&b""[..], &b"{ nope"[..], &[0u8, 159, 146, 150][..]] {
        fs::write(&file, bytes).unwrap();
        let p = load_profile(&file);
        assert_eq!(p.name, "standard", "{bytes:?}");
        assert_eq!(p.px_per_mm, DEFAULT_PX_PER_MM);
    }
    // a folder where the file should be is no crash either
    assert_eq!(load_profile(&dir).px_per_mm, DEFAULT_PX_PER_MM);
}

#[test]
fn a_profile_is_saved_with_its_folders_as_json_and_the_last_save_wins() {
    let dir = scratch("profile-save");
    let file = dir.join("a").join("b").join("profile.json");
    let mut p = preset_for(1, &Profile::default());
    p.px_per_mm = 63.0;
    save_profile(&file, &p).unwrap();
    let text = fs::read_to_string(&file).unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&text)
        .unwrap()
        .is_object());
    assert_eq!(load_profile(&file), p);
    p.px_per_mm = 71.0;
    save_profile(&file, &p).unwrap();
    assert_eq!(load_profile(&file).px_per_mm, 71.0);
    // a profile that was never calibrated comes back with the guessed scale and keeps its name
    let uncalibrated = preset_for(1, &Profile::default());
    save_profile(&file, &uncalibrated).unwrap();
    let back = load_profile(&file);
    assert_eq!(back.name, "fine_pitch");
    assert_eq!(back.px_per_mm, DEFAULT_PX_PER_MM);
    // a file in the way of the folders is an error and not a panic
    let blocker = dir.join("blocker");
    fs::write(&blocker, b"x").unwrap();
    assert!(save_profile(&blocker.join("sub").join("p.json"), &p).is_err());
}

#[test]
fn the_buttons_of_the_presets_keep_the_scale_and_the_checks_and_clamp_the_number() {
    let cur = Profile {
        px_per_mm: 12.5,
        check_holes: false,
        ..Profile::default()
    };
    for (n, name) in [
        (0, "standard"),
        (1, "fine_pitch"),
        (2, "prototype"),
        (3, "prototype"),
        (usize::MAX, "prototype"),
    ] {
        let p = preset_for(n, &cur);
        assert_eq!(p.name, name, "{n}");
        assert_eq!(p.px_per_mm, 12.5);
        assert!(!p.check_holes && p.check_milling);
    }
    assert_eq!(preset_for(0, &Profile::default()), Profile::default());
    // the choices of the ring and the guessed scale
    assert_eq!(LIGHT_CHOICES, [None, Some(30), Some(60), Some(80)]);
    assert_eq!(DEFAULT_PX_PER_MM, 40.0);
}

// ---------------------------------------------------------------------------------------------
// spindle
// ---------------------------------------------------------------------------------------------

fn limited(id: SpindleId) -> Spindle {
    Spindle {
        id,
        min_rpm: 5_000,
        max_rpm: 24_000,
        power_w: 100,
        max_xy_mm_min: 1_500,
        max_z_mm_min: 1_200,
        max_bit_mm: 3.0,
    }
}

#[test]
fn the_spindle_codes_are_exact_and_in_order() {
    assert_eq!(SpindleId::ALL, [SpindleId::Original, SpindleId::Vfd]);
    for id in SpindleId::ALL {
        assert_eq!(SpindleId::from_code(id.code()), Some(id));
        assert_eq!(
            SpindleId::from_code(&format!("\t{}\n", id.code())),
            Some(id)
        );
    }
    for bad in ["VFD", "Original", "", "vfd2", "vf d"] {
        assert_eq!(SpindleId::from_code(bad), None, "{bad:?}");
    }
    for id in SpindleId::ALL {
        let s = Spindle::default_for(id);
        assert_eq!(s.id, id);
        assert!(s.min_rpm <= s.max_rpm && s.power_w > 0 && s.max_bit_mm > 0.0);
    }
    assert!(
        Spindle::default_for(SpindleId::Original).max_bit_mm
            < Spindle::default_for(SpindleId::Vfd).max_bit_mm
    );
}

#[test]
fn the_line_of_a_spindle_has_the_six_limits_in_order() {
    let s = Spindle::from_text(SpindleId::Vfd, "8000,20000,500,1200,900,3.5");
    assert_eq!(s.id, SpindleId::Vfd);
    assert_eq!((s.min_rpm, s.max_rpm, s.power_w), (8_000, 20_000, 500));
    assert_eq!((s.max_xy_mm_min, s.max_z_mm_min), (1_200, 900));
    assert_eq!(s.max_bit_mm, 3.5);
    assert_eq!(s.to_text(), "8000,20000,500,1200,900,3.5");
    // a whole number of millimetres is written without a decimal point
    let whole = Spindle {
        max_bit_mm: 3.0,
        ..s
    };
    assert_eq!(whole.to_text(), "8000,20000,500,1200,900,3");
    assert_eq!(Spindle::from_text(SpindleId::Vfd, &whole.to_text()), whole);
    // what follows the sixth number is ignored and the numbers are cut, not rounded
    let cut = Spindle::from_text(SpindleId::Vfd, "1000.9,20000.9,5,6,7,8,999,abc");
    assert_eq!((cut.min_rpm, cut.max_rpm), (1_000, 20_000));
    assert_eq!(cut.max_bit_mm, 8.0);
}

#[test]
fn a_line_that_makes_no_sense_keeps_the_starting_values_and_the_turns_are_ordered() {
    for id in SpindleId::ALL {
        for bad in [
            "",
            "0,0,0,0,0,0",
            "-5,-5,-5,-5,-5,-5",
            "nan,nan,nan,nan,nan,nan",
            "a,b,c,d,e,f",
        ] {
            assert_eq!(
                Spindle::from_text(id, bad),
                Spindle::default_for(id),
                "{bad:?}"
            );
        }
        let floor = Spindle::from_text(id, "200,500");
        assert_eq!((floor.min_rpm, floor.max_rpm), (200, 1_000));
        let swapped = Spindle::from_text(id, "30000,20000");
        assert_eq!(swapped.max_rpm, 20_000);
        assert_eq!(
            swapped.min_rpm,
            Spindle::default_for(id).min_rpm.min(20_000)
        );
    }
}

#[test]
fn the_thickest_bit_is_the_one_as_thick_as_the_spindle_takes() {
    let s = limited(SpindleId::Vfd);
    let at_limit = plain(Kind::Drill, 3.0, 20_000, 0, 500);
    assert_eq!(s.fit(&at_limit), SpindleFit::Fits);
    let over = plain(Kind::Drill, 3.000_001, 20_000, 0, 500);
    assert_eq!(s.fit(&over), SpindleFit::Incompatible(Why::TooThick));
    // the thickness is judged before the turns
    let both = plain(Kind::Drill, 9.0, 1_000_000, 0, 500);
    assert_eq!(s.fit(&both), SpindleFit::Incompatible(Why::TooThick));
}

#[test]
fn a_bit_needs_at_least_thirty_five_per_cent_of_the_turns_it_asks_for() {
    let s = limited(SpindleId::Vfd);
    // 24000 / 0.35 is 68571.4: one turn more than that is too slow
    match s.fit(&plain(Kind::Drill, 1.0, 68_571, 0, 100)) {
        SpindleFit::Adjusted {
            bit,
            lower_rpm,
            xy_slower,
            z_slower,
        } => {
            assert_eq!(bit.rpm, 24_000);
            assert!(lower_rpm && !xy_slower && !z_slower);
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        s.fit(&plain(Kind::Drill, 1.0, 68_572, 0, 100)),
        SpindleFit::Incompatible(Why::TooSlow)
    );
    // a bit that asks for no turns is given the least the spindle can turn
    match s.fit(&plain(Kind::Drill, 1.0, 0, 0, 100)) {
        SpindleFit::Adjusted { bit, lower_rpm, .. } => {
            assert_eq!(bit.rpm, 5_000);
            assert!(lower_rpm);
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_original_spindle_turns_at_the_nearest_of_its_sixteen_levels() {
    let original = Spindle {
        id: SpindleId::Original,
        min_rpm: 1_000,
        max_rpm: 63_000,
        power_w: 100,
        max_xy_mm_min: 99_999,
        max_z_mm_min: 99_999,
        max_bit_mm: 10.0,
    };
    let gets = |asked: u32| match original.fit(&plain(Kind::Drill, 1.0, asked, 0, 100)) {
        SpindleFit::Fits => asked,
        SpindleFit::Adjusted { bit, .. } => bit.rpm,
        SpindleFit::Incompatible(why) => panic!("{asked}: {why:?}"),
    };
    for (asked, want) in [
        (5_000, 30_000),
        (29_999, 30_000),
        (30_000, 30_000),
        (30_999, 30_000),
        (31_000, 32_000),
        (32_000, 32_000),
        (58_999, 58_000),
        (59_000, 60_000),
        (60_000, 60_000),
        (62_999, 60_000),
        (63_000, 60_000),
        (70_000, 60_000),
    ] {
        assert_eq!(gets(asked), want, "asked {asked}");
    }
    // between the ends the level is always an even thousand within one thousand of what was asked
    for asked in (30_000..=60_000).step_by(137) {
        let level = i64::from(gets(asked));
        assert_eq!(level % 2_000, 0, "{asked}");
        assert!(
            (-999..=1_000).contains(&(level - i64::from(asked))),
            "{asked}"
        );
    }
}

#[test]
fn what_a_spindle_gives_a_bit_never_goes_over_what_the_spindle_holds() {
    for id in SpindleId::ALL {
        let s = Spindle::default_for(id);
        for b in catalogue::all() {
            match s.fit(b) {
                SpindleFit::Fits => {
                    assert!((s.min_rpm..=s.max_rpm).contains(&b.rpm), "{b:?}");
                    assert!(b.feed_mm_min <= s.max_xy_mm_min, "{b:?}");
                    assert!(b.plunge_mm_min <= s.max_z_mm_min, "{b:?}");
                    assert!(b.diameter_mm <= s.max_bit_mm + 1e-9, "{b:?}");
                }
                SpindleFit::Adjusted {
                    bit,
                    xy_slower,
                    z_slower,
                    ..
                } => {
                    assert!(bit.feed_mm_min <= s.max_xy_mm_min, "{b:?}");
                    assert!(bit.plunge_mm_min <= s.max_z_mm_min, "{b:?}");
                    assert!(!xy_slower || bit.feed_mm_min == s.max_xy_mm_min, "{b:?}");
                    assert!(!z_slower || bit.plunge_mm_min == s.max_z_mm_min, "{b:?}");
                    assert_eq!(
                        (bit.kind, bit.series, bit.flutes, bit.diameter_mm),
                        (b.kind, b.series, b.flutes, b.diameter_mm)
                    );
                }
                SpindleFit::Incompatible(Why::TooThick) => {
                    assert!(b.diameter_mm > s.max_bit_mm, "{b:?}");
                }
                SpindleFit::Incompatible(Why::TooSlow) => {
                    assert!(b.rpm > s.max_rpm, "{b:?}");
                }
            }
        }
    }
}

#[test]
fn the_drawing_of_the_head_has_its_default_text_and_keeps_the_sizes_that_make_sense() {
    let d = HeadDims::default();
    assert_eq!(d.to_text(), "174,33,20,27,14,88,33,59");
    // the edge of what is sensible: 1000 mm is taken, a little more is not, and the smallest positive number is
    let top = HeadDims::from_text("1000,1000,1000,1000,1000,1000,1000,1000");
    assert_eq!(top.to_text(), "1000,1000,1000,1000,1000,1000,1000,1000");
    let over = "1000.01,1000.01,1000.01,1000.01,1000.01,1000.01,1000.01,1000.01";
    assert_eq!(HeadDims::from_text(over), d);
    let tiny = HeadDims::from_text("0.001,0.001,0.001,0.001,0.001,0.001,0.001,0.001");
    assert_eq!(tiny.total_mm, 0.001);
    for bad in [
        "",
        "0,0,0,0,0,0,0,0",
        "-1,-1,-1,-1,-1,-1,-1,-1",
        "inf,inf,inf,inf,inf,inf,inf,inf",
        "nan,nan,nan,nan,nan,nan,nan,nan",
    ] {
        assert_eq!(HeadDims::from_text(bad), d, "{bad:?}");
    }
    // spaces and numbers past the eighth do not matter
    let spaced = HeadDims::from_text(" 200 , 40 , 30 , 20 , 10 , 80 , 30 , 50 , 999");
    assert_eq!(spaced.to_text(), "200,40,30,20,10,80,30,50");
    // the collar is cut to a nose that is shorter than it
    let short_nose = HeadDims::from_text("174,10");
    assert_eq!((short_nose.nose_mm, short_nose.collar_mm), (10.0, 10.0));
    assert_eq!(short_nose.body_mm, d.body_mm);
}
