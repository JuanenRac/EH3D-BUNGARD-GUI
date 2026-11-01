//! Starting values for the tools of a job: what to take when nothing better is known. They are the usual ones for solid carbide drills and
//! two millimetre routers on a spindle that goes up to 63000 rpm (the old control unit gives 60000 at most); the material of the board changes them (paper boards are softer, the
//! PTFE and ceramic filled laminates wear the tools and want less speed, aluminium wants much less). They are a start, to be tuned on the
//! machine and then typed in: the values of the tool table are always the operator's to change.

use crate::boards::Core;
use crate::catalogue::Bit;
use crate::spindle::{Fit, Spindle};

/// The extra depth below the board that a router goes, in mm.
pub const ROUTE_EXTRA_MM: f64 = 0.3;

/// What the material of the board asks of the tools: the factors of the speed and the feed over the values of the bit, and the deepest cut of a router in one pass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Machining {
    /// The factor of the turns (1.0 leaves the speed of the bit).
    pub rpm_factor: f64,
    /// The factor of the feed in the plane and in Z.
    pub feed_factor: f64,
    /// The deepest cut of a router in one pass, in mm; 0 leaves it to the bite of the bit (one millimetre without it).
    pub pass_mm: f64,
}

/// The table of the materials: the one place where these starting values are set (to be tuned on the machine). Prudent figures for the tools of a small
/// spindle: the woods and the plastics that cut clean keep the high turns (a single-flute tool clears the chips); the plastics that soften turn less and
/// feed faster so the tool does not heat them; aluminium and copper turn slow with a moderate feed and shallow passes (and want a lubricant); the PTFE
/// turns less and is not heated; none of them go above the speed of the bit, so none passes the top of the spindle.
pub fn machining(core: Core) -> Machining {
    let (rpm_factor, feed_factor, pass_mm) = match core {
        Core::Fr2 | Core::Fr3 | Core::Cem1 => (1.0, 1.3, 0.0),
        Core::Cem3 | Core::Fr4 | Core::Polyimide => (1.0, 1.0, 0.0),
        Core::Hydrocarbon | Core::Ptfe => (0.8, 0.8, 0.0),
        Core::Flex => (1.0, 1.2, 0.0),
        Core::Aluminium => (0.5, 0.5, 0.0),
        Core::Ceramic => (0.3, 0.2, 0.0),
        Core::Metal => (0.4, 0.4, 0.3),
        Core::Copper => (0.3, 0.5, 0.2),
        Core::Wood => (1.0, 1.5, 1.5),
        Core::Plastic => (0.8, 1.2, 1.0),
        Core::SoftPlastic => (0.6, 1.5, 1.0),
    };
    Machining {
        rpm_factor,
        feed_factor,
        pass_mm,
    }
}

/// How the material changes the speed and the feed: (rpm factor, feed factor).
pub fn material_factors(core: Core) -> (f64, f64) {
    let m = machining(core);
    (m.rpm_factor, m.feed_factor)
}

fn round_to(v: f64, step: f64) -> f64 {
    (v / step).round() * step
}

/// The extra stroke below the board that a drill of `diameter_mm` starts from, in mm.
pub fn drill_extra(diameter_mm: f64) -> f64 {
    drill(diameter_mm, Core::Fr4).0
}

/// What the numbers of a bit of the catalogue need to be checked for: the material of the board asks for fewer turns than the bit, the material asks for fewer turns
/// than the spindle can give, the cutting part is shorter than the board and the extra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Note {
    LowerRpm,
    BelowSpindle,
    FluteShort,
}

/// The numbers a bit of the catalogue gives a tool of the job: the spindle's, then the material of the board.
#[derive(Debug, Clone, PartialEq)]
pub struct Transfer {
    pub rpm: u32,
    pub feed_mm_min: u32,
    pub plunge_mm_min: u32,
    /// The factors of the material that were applied (turns, feed).
    pub rpm_factor: f64,
    pub feed_factor: f64,
    /// The depth in mm: of a router, the cut (the board and the extra, never less); of a drill, the board and the extra stroke.
    pub depth_mm: f64,
    /// The passes of a router (the depth over the bite of the bit); `None` for a drill.
    pub passes: Option<u32>,
    pub notes: Vec<Note>,
}

impl Transfer {
    /// Whether the bit should not be used as it is (the person is told in the window of tools).
    pub fn is_unsuitable(&self) -> bool {
        self.notes
            .iter()
            .any(|n| matches!(n, Note::BelowSpindle | Note::FluteShort))
    }
}

/// The numbers `bit` gives a tool on `spindle` for a board of `core` and `thickness_mm`. `current` is the extra stroke of the tool when it is a drill and the depth of
/// the cut when it is a router. `None` when the spindle cannot work with the bit at all.
pub fn transfer(
    bit: &Bit,
    spindle: &Spindle,
    core: Core,
    thickness_mm: f64,
    current: f64,
) -> Option<Transfer> {
    let mine = match spindle.fit(bit) {
        Fit::Fits => *bit,
        Fit::Adjusted { bit, .. } => bit,
        Fit::Incompatible(_) => return None,
    };
    let (kr, kf) = material_factors(core);
    let mut notes = Vec::new();
    let mut rpm = mine.rpm;
    if kr != 1.0 {
        rpm = (round_to(rpm as f64 * kr, 1_000.0).max(1_000.0)) as u32;
    }
    if kr < 1.0 {
        notes.push(Note::LowerRpm);
    }
    if rpm < spindle.min_rpm {
        rpm = spindle.min_rpm;
        notes.push(Note::BelowSpindle);
    }
    let by_feed = |v: u32, top: u32| -> u32 {
        if v == 0 {
            return 0;
        }
        let w = if kf == 1.0 {
            v
        } else {
            (round_to(v as f64 * kf, 10.0).max(10.0)) as u32
        };
        w.min(top)
    };
    let feed = by_feed(mine.feed_mm_min, spindle.max_xy_mm_min);
    let plunge = by_feed(mine.plunge_mm_min, spindle.max_z_mm_min);
    let is_drill = bit.kind.is_drill();
    let (depth, passes) = if is_drill {
        (thickness_mm + current.max(0.0), None)
    } else {
        let depth = current.max(round_to(thickness_mm + ROUTE_EXTRA_MM, 0.1));
        // the bite of the bit, held to what the material allows in one pass
        let cap = machining(core).pass_mm;
        let pass = match (bit.pass_mm > 0.0, cap > 0.0) {
            (true, true) => bit.pass_mm.min(cap),
            (true, false) => bit.pass_mm,
            (false, true) => cap,
            (false, false) => 0.0,
        };
        let passes = if pass > 0.0 {
            (depth / pass).ceil().max(1.0) as u32
        } else {
            1
        };
        (depth, Some(passes))
    };
    // the cutting part has to go through the board and the extra (the router, all the way down: the passes only share the work)
    if bit.flute_mm > 0.0 && bit.flute_mm + 1e-9 < depth {
        notes.push(Note::FluteShort);
    }
    Some(Transfer {
        rpm,
        feed_mm_min: feed,
        plunge_mm_min: plunge,
        rpm_factor: kr,
        feed_factor: kf,
        depth_mm: depth,
        passes,
        notes,
    })
}

/// The extra stroke below the board in mm, the feed of the Z axis in mm/min and the speed in rpm for a drill of `diameter_mm`.
pub fn drill(diameter_mm: f64, core: Core) -> (f64, u32, u32) {
    let (rpm, feed, extra) = match diameter_mm {
        d if d < 0.5 => (60_000.0, 1_500.0, 0.5),
        d if d < 0.8 => (60_000.0, 2_400.0, 0.5),
        d if d < 1.2 => (60_000.0, 3_600.0, 0.5),
        d if d < 2.0 => (40_000.0, 2_400.0, 0.6),
        d if d < 3.0 => (30_000.0, 1_500.0, 0.8),
        _ => (20_000.0, 800.0, 1.0),
    };
    let (kr, kf) = material_factors(core);
    (
        extra,
        round_to(feed * kf, 50.0).clamp(50.0, 9_999.0) as u32,
        round_to(rpm * kr, 1_000.0).clamp(1_000.0, 99_999.0) as u32,
    )
}

/// The depth of a cut in mm, the passes, the feed in the plane and in Z in mm/min and the speed in rpm for a router of two millimetres
/// that cuts through a board of `thickness_mm`.
pub fn route(thickness_mm: f64, core: Core) -> (f64, u32, u32, u32, u32) {
    let depth = round_to(thickness_mm + 0.3, 0.1);
    let m = machining(core);
    let pass = if m.pass_mm > 0.0 { m.pass_mm } else { 1.0 };
    let passes = (thickness_mm / pass).ceil().max(1.0) as u32;
    let (kr, kf) = (m.rpm_factor, m.feed_factor);
    (
        depth,
        passes,
        round_to(800.0 * kf, 50.0) as u32,
        round_to(300.0 * kf, 50.0).max(50.0) as u32,
        round_to(60_000.0 * kr, 1_000.0) as u32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bigger_drill_turns_slower_and_goes_down_slower() {
        let small = drill(0.6, Core::Fr4);
        let big = drill(3.2, Core::Fr4);
        assert!(small.2 > big.2 && small.1 > big.1);
        assert_eq!(drill(0.8, Core::Fr4).2, 60_000);
    }

    #[test]
    fn the_material_changes_the_values() {
        let fr4 = drill(1.0, Core::Fr4);
        assert!(drill(1.0, Core::Fr2).1 > fr4.1);
        assert!(drill(1.0, Core::Ptfe).2 < fr4.2);
        assert!(drill(1.0, Core::Aluminium).2 <= 30_000);
        for d in [0.3, 0.8, 1.5, 2.5, 6.0] {
            let (e, f, r) = drill(d, Core::Fr4);
            assert!(e > 0.0 && (50..=9_999).contains(&f) && (1_000..=99_999).contains(&r));
        }
    }

    #[test]
    fn the_router_cuts_through_the_board_in_passes() {
        let (depth, passes, xy, z, rpm) = route(1.6, Core::Fr4);
        assert!(depth > 1.6 && depth < 2.1);
        assert_eq!(passes, 2);
        assert!(xy > z && rpm == 60_000);
        assert_eq!(route(0.5, Core::Fr4).1, 1);
    }

    fn a_drill(d: f64) -> Bit {
        *crate::catalogue::all()
            .iter()
            .find(|b| b.kind.is_drill() && (b.diameter_mm - d).abs() < 1e-9 && b.flute_mm > 0.0)
            .expect("a drill of the size")
    }

    fn a_router(d: f64) -> Bit {
        *crate::catalogue::all()
            .iter()
            .find(|b| b.kind == crate::catalogue::Kind::Diamond && (b.diameter_mm - d).abs() < 1e-9)
            .expect("a router of the size")
    }

    fn original() -> Spindle {
        Spindle::default_for(crate::spindle::SpindleId::Original)
    }

    #[test]
    fn a_board_of_fr4_leaves_the_numbers_of_the_bit_as_the_spindle_gives_them() {
        let b = a_drill(1.0);
        let t = transfer(&b, &original(), Core::Fr4, 1.6, 0.5).unwrap();
        assert_eq!((t.rpm, t.plunge_mm_min), (b.rpm, b.plunge_mm_min));
        assert_eq!((t.rpm_factor, t.feed_factor), (1.0, 1.0));
        assert!(!t.notes.contains(&Note::LowerRpm) && t.passes.is_none());
    }

    #[test]
    fn a_material_that_wants_fewer_turns_lowers_the_turns_and_the_feed_and_says_so() {
        let b = a_drill(1.0);
        let fr4 = transfer(&b, &original(), Core::Fr4, 1.6, 0.5).unwrap();
        for core in [Core::Ptfe, Core::Aluminium, Core::Ceramic] {
            let t = transfer(&b, &original(), core, 1.6, 0.5).unwrap();
            let (kr, kf) = material_factors(core);
            assert!(
                t.rpm < fr4.rpm && t.plunge_mm_min < fr4.plunge_mm_min,
                "{core:?}"
            );
            assert!(
                (t.rpm as f64 - b.rpm as f64 * kr).abs() <= 500.0,
                "{core:?}"
            );
            assert!(
                (t.plunge_mm_min as f64 - b.plunge_mm_min as f64 * kf).abs() <= 5.0,
                "{core:?}"
            );
            assert!(t.notes.contains(&Note::LowerRpm), "{core:?}");
        }
        // a softer material turns as much and goes faster, and nothing is said
        let paper = transfer(&b, &original(), Core::Fr2, 1.6, 0.5).unwrap();
        assert!(paper.plunge_mm_min > fr4.plunge_mm_min && paper.rpm == fr4.rpm);
        assert!(!paper.notes.contains(&Note::LowerRpm));
    }

    #[test]
    fn a_material_that_wants_fewer_turns_than_the_spindle_can_give_keeps_the_least_and_warns() {
        let mut vfd = Spindle::default_for(crate::spindle::SpindleId::Vfd);
        vfd.min_rpm = 9_000;
        let b = a_drill(1.0);
        let t = transfer(&b, &vfd, Core::Ceramic, 1.6, 0.5).unwrap();
        assert_eq!(t.rpm, 9_000);
        assert!(t.notes.contains(&Note::BelowSpindle) && t.is_unsuitable());
    }

    #[test]
    fn the_feed_of_a_soft_material_is_held_by_the_spindle() {
        let mut s = original();
        s.max_z_mm_min = 1_000;
        let mut b = a_drill(1.0);
        b.plunge_mm_min = 900;
        let t = transfer(&b, &s, Core::Fr2, 1.6, 0.5).unwrap();
        assert_eq!(t.plunge_mm_min, 1_000);
    }

    #[test]
    fn a_bit_the_spindle_cannot_work_with_gives_no_numbers() {
        let mut b = a_drill(1.0);
        b.rpm = 100_000;
        let vfd = Spindle::default_for(crate::spindle::SpindleId::Vfd);
        assert!(transfer(&b, &vfd, Core::Fr4, 1.6, 0.5).is_none());
    }

    #[test]
    fn the_passes_of_a_router_come_from_the_bite_of_the_bit_and_the_board_with_the_extra() {
        let mut r = a_router(2.0);
        r.pass_mm = 0.5;
        r.flute_mm = 8.0;
        let t = transfer(&r, &original(), Core::Fr4, 1.6, 0.0).unwrap();
        // a cut that is shorter than the board and the extra is raised to them
        assert!((t.depth_mm - 1.9).abs() < 1e-9, "{}", t.depth_mm);
        assert_eq!(t.passes, Some(4));
        // a deeper cut typed by the person is kept
        assert_eq!(
            transfer(&r, &original(), Core::Fr4, 1.6, 2.4)
                .unwrap()
                .passes,
            Some(5)
        );
        r.pass_mm = 5.0;
        assert_eq!(
            transfer(&r, &original(), Core::Fr4, 0.5, 0.0)
                .unwrap()
                .passes,
            Some(1)
        );
    }

    #[test]
    fn a_bit_whose_cutting_part_does_not_go_through_the_board_and_the_extra_is_warned() {
        let mut d = a_drill(1.0);
        d.flute_mm = 1.8;
        let t = transfer(&d, &original(), Core::Fr4, 1.6, 0.5).unwrap();
        assert!(t.notes.contains(&Note::FluteShort) && t.is_unsuitable());
        d.flute_mm = 2.2;
        assert!(!transfer(&d, &original(), Core::Fr4, 1.6, 0.5)
            .unwrap()
            .notes
            .contains(&Note::FluteShort));
        // a thinner board is enough for the short one
        d.flute_mm = 1.8;
        assert!(transfer(&d, &original(), Core::Fr4, 0.8, 0.5)
            .unwrap()
            .notes
            .is_empty());
        let mut r = a_router(2.0);
        r.flute_mm = 1.5;
        assert!(transfer(&r, &original(), Core::Fr4, 1.6, 0.0)
            .unwrap()
            .notes
            .contains(&Note::FluteShort));
        // a bit that does not say its length is not judged
        d.flute_mm = 0.0;
        assert!(transfer(&d, &original(), Core::Fr4, 1.6, 0.5)
            .unwrap()
            .notes
            .is_empty());
    }

    const NEW_CORES: [Core; 5] = [
        Core::Metal,
        Core::Copper,
        Core::Wood,
        Core::Plastic,
        Core::SoftPlastic,
    ];

    #[test]
    fn every_material_keeps_inside_the_limits_of_the_spindle() {
        let top = original().max_rpm;
        let all = [
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
        ]
        .into_iter()
        .chain(NEW_CORES);
        for core in all {
            let m = machining(core);
            assert!(m.rpm_factor > 0.0 && m.rpm_factor <= 1.0, "{core:?}");
            assert!(m.feed_factor > 0.0 && m.feed_factor <= 2.0, "{core:?}");
            assert!(m.pass_mm >= 0.0 && m.pass_mm <= 2.0, "{core:?}");
            for d in [0.3, 0.8, 1.5, 2.5, 6.0] {
                let (e, f, r) = drill(d, core);
                assert!(e > 0.0 && (50..=9_999).contains(&f), "{core:?}");
                assert!((1_000..=top).contains(&r), "{core:?} {d}");
            }
            for t in [0.2, 1.6, 5.0] {
                let (depth, passes, xy, z, rpm) = route(t, core);
                assert!(depth > t && passes >= 1 && xy >= z, "{core:?}");
                assert!((1_000..=top).contains(&rpm), "{core:?}");
            }
            // the bits of the catalogue never go over the top turns once the material is applied
            for b in crate::catalogue::all().iter().take(400) {
                if let Some(t) = transfer(b, &original(), core, 1.6, 0.5) {
                    assert!(t.rpm <= top, "{core:?}");
                }
            }
        }
    }

    #[test]
    fn the_sheets_without_copper_are_cut_as_each_material_asks() {
        let fr4 = drill(1.0, Core::Fr4);
        // the woods keep the turns and go faster; the plastics that soften turn less and go faster still
        assert_eq!(drill(1.0, Core::Wood).2, fr4.2);
        assert!(drill(1.0, Core::Wood).1 > fr4.1);
        assert!(drill(1.0, Core::SoftPlastic).2 < drill(1.0, Core::Plastic).2);
        assert!(drill(1.0, Core::Plastic).2 < fr4.2);
        assert!(drill(1.0, Core::SoftPlastic).1 > fr4.1);
        // the metals turn slow, the copper slower than the aluminium alloy, and both go in shallow passes
        assert!(drill(1.0, Core::Metal).2 <= 30_000);
        assert!(drill(1.0, Core::Copper).2 < drill(1.0, Core::Metal).2);
        assert!(machining(Core::Metal).pass_mm < 0.5 && machining(Core::Copper).pass_mm < 0.5);
        assert!(route(3.0, Core::Metal).1 >= 10 && route(3.0, Core::Copper).1 >= 15);
        assert_eq!(route(3.0, Core::Wood).1, 2);
        // the PTFE is not heated: fewer turns than the glass laminate
        assert!(drill(1.0, Core::Ptfe).2 < fr4.2);
        // the laminates keep what they had
        assert_eq!(machining(Core::Fr4).pass_mm, 0.0);
        assert_eq!(route(1.6, Core::Fr4).1, 2);
    }

    #[test]
    fn the_pass_of_the_material_holds_the_bite_of_the_bit() {
        let mut r = a_router(2.0);
        r.pass_mm = 1.0;
        r.flute_mm = 8.0;
        let fr4 = transfer(&r, &original(), Core::Fr4, 3.0, 0.0).unwrap();
        let copper = transfer(&r, &original(), Core::Copper, 3.0, 0.0).unwrap();
        assert_eq!(fr4.passes, Some(4));
        assert_eq!(copper.passes, Some(17));
        // a bit that gives no bite takes the one of the material
        r.pass_mm = 0.0;
        let metal = transfer(&r, &original(), Core::Metal, 3.0, 0.0)
            .unwrap()
            .passes
            .unwrap();
        assert!((11..=12).contains(&metal), "{metal}");
        assert_eq!(
            transfer(&r, &original(), Core::Fr4, 3.0, 0.0)
                .unwrap()
                .passes,
            Some(1)
        );
    }
}
