//! The catalogue of bits for a PCB: drills, slot drills, end mills, routers, V-bits and the rest, all with the shank of 1/8 inch (3.175 mm) that the chuck of a Bungard machine
//! holds, from the smallest micro drill to the 7 mm drill. The drills and the routers are the series of a maker of PCB tools (the diameters, the length of the flute and the turns
//! and feeds of the maker's table for FR-4); the end mills, V-bits, ball noses, spot drills and countersinks are generic. Every number is a starting point for a try on a
//! scrap board, not the word of a maker.

use std::sync::OnceLock;

/// The shank every bit of the catalogue has: 1/8 inch.
pub const SHANK_MM: f64 = 3.175;
/// The total length of a bit with that shank, from the tip to the end of the shank.
pub const TOTAL_MM: f64 = 38.1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A drill for through holes.
    Drill,
    /// A drill with a stiff web, made to cut oblong holes (slots) by moving sideways while it drills.
    SlotDrill,
    /// A square end mill with two flutes: slots, isolation and the outline.
    EndMill,
    /// A router that pushes the chips down, for a clean upper edge of the outline.
    DownCut,
    /// A router that pulls the chips up, for slots and pockets.
    UpCut,
    /// A router with the cut of a diamond (the "corn" pattern), for the outline of FR-4 and G10.
    Diamond,
    /// A router whose teeth break the chips into short pieces.
    ChipBreaker,
    /// A flat-end router with a diamond coating, for abrasive boards and boards with an aluminium back.
    Coated,
    /// A V-bit (engraving bit) for the isolation of tracks: the groove is as wide as the depth makes it.
    VBit,
    /// A ball nose, for engraving.
    BallNose,
    /// A spot drill, that starts a hole without wandering.
    Spot,
    /// A countersink, that chamfers a hole.
    Countersink,
}

impl Kind {
    pub const ALL: [Kind; 12] = [
        Kind::Drill,
        Kind::SlotDrill,
        Kind::EndMill,
        Kind::DownCut,
        Kind::UpCut,
        Kind::Diamond,
        Kind::ChipBreaker,
        Kind::Coated,
        Kind::VBit,
        Kind::BallNose,
        Kind::Spot,
        Kind::Countersink,
    ];

    /// A bit that only goes in and out (and a slot drill, which also moves sideways but is chosen as a drill).
    pub fn is_drill(self) -> bool {
        matches!(
            self,
            Kind::Drill | Kind::SlotDrill | Kind::Spot | Kind::Countersink
        )
    }
}

/// One bit of the catalogue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bit {
    pub kind: Kind,
    /// The name the maker gives to the series (empty for the generic bits).
    pub series: &'static str,
    /// The diameter of the cut (of the shoulder of a V-bit or a countersink), in mm.
    pub diameter_mm: f64,
    /// The width of the tip of a V-bit, in mm (0 for the rest).
    pub tip_mm: f64,
    /// The included angle of a V-bit, a spot drill or a countersink, in degrees (0 for the rest).
    pub angle_deg: u32,
    pub flutes: u8,
    /// The length of the cutting part (the flute), in mm (0 when the maker's table does not say).
    pub flute_mm: f64,
    /// The turns of the spindle per minute, the feed along the cut and of the plunge in mm per minute, and how deep one pass goes in mm.
    pub rpm: u32,
    pub feed_mm_min: u32,
    pub plunge_mm_min: u32,
    pub pass_mm: f64,
}

impl Bit {
    pub fn shank_mm(&self) -> f64 {
        SHANK_MM
    }

    /// The same bit for a spindle that cannot turn as fast as the maker's table says: it turns at its own top speed and the feeds go down in the same proportion,
    /// so every turn takes the same bite. A spindle that is fast enough leaves the bit as it is.
    pub fn for_spindle(&self, spindle_max_rpm: u32) -> Bit {
        if spindle_max_rpm == 0 || spindle_max_rpm >= self.rpm {
            return *self;
        }
        let ratio = spindle_max_rpm as f64 / self.rpm as f64;
        let scale = |v: u32| {
            if v == 0 {
                0
            } else {
                ((v as f64 * ratio).round() as u32).max(10)
            }
        };
        Bit {
            rpm: spindle_max_rpm,
            feed_mm_min: scale(self.feed_mm_min),
            plunge_mm_min: scale(self.plunge_mm_min),
            ..*self
        }
    }

    /// The width of the groove a V-bit leaves when it goes `depth_mm` deep: the tip, plus what the angle opens on each side.
    pub fn groove_mm(&self, depth_mm: f64) -> Option<f64> {
        (self.kind == Kind::VBit && self.angle_deg > 0).then(|| {
            let half = (self.angle_deg as f64 / 2.0).to_radians();
            (self.tip_mm + 2.0 * depth_mm.max(0.0) * half.tan()).min(self.diameter_mm)
        })
    }

    /// The depth at which a V-bit cuts a groove of `width_mm` (None when the width is less than the tip or more than the bit can make).
    pub fn depth_for_groove_mm(&self, width_mm: f64) -> Option<f64> {
        if self.kind != Kind::VBit
            || self.angle_deg == 0
            || width_mm < self.tip_mm
            || width_mm > self.diameter_mm
        {
            return None;
        }
        let half = (self.angle_deg as f64 / 2.0).to_radians();
        Some((width_mm - self.tip_mm) / (2.0 * half.tan()))
    }
}

#[allow(clippy::too_many_arguments)]
fn bit(
    kind: Kind,
    d: f64,
    tip: f64,
    angle: u32,
    flutes: u8,
    rpm: u32,
    feed: u32,
    plunge: u32,
    pass: f64,
) -> Bit {
    Bit {
        kind,
        series: "",
        diameter_mm: d,
        tip_mm: tip,
        angle_deg: angle,
        flutes,
        flute_mm: 0.0,
        rpm,
        feed_mm_min: feed,
        plunge_mm_min: plunge,
        pass_mm: pass,
    }
}

/// The turns and the infeed (mm per minute) that the maker's table gives for a drill of this diameter in FR-4 of a double-sided board: each row is the largest diameter
/// it holds, the turns per minute and the infeed in m/min.
const DRILL_FR4: [(f64, u32, f64); 33] = [
    (0.30, 125_000, 2.8),
    (0.35, 125_000, 3.0),
    (0.40, 125_000, 3.4),
    (0.45, 120_000, 3.5),
    (0.50, 108_000, 3.4),
    (0.55, 98_000, 3.6),
    (0.60, 90_000, 3.6),
    (0.65, 83_000, 3.7),
    (0.70, 77_000, 3.7),
    (0.75, 72_000, 3.8),
    (0.80, 68_000, 3.6),
    (0.85, 64_000, 3.4),
    (0.90, 60_000, 3.3),
    (0.95, 57_000, 3.2),
    (1.00, 54_000, 3.1),
    (1.10, 49_000, 2.9),
    (1.20, 45_000, 2.7),
    (1.30, 42_000, 2.5),
    (1.40, 39_000, 2.3),
    (1.50, 36_000, 2.1),
    (1.60, 34_000, 2.0),
    (1.70, 32_000, 1.9),
    (1.80, 30_000, 1.7),
    (2.00, 27_000, 1.6),
    (2.20, 25_000, 1.4),
    (2.50, 22_000, 1.4),
    (2.60, 21_000, 1.2),
    (3.175, 20_000, 1.1),
    (3.95, 25_000, 0.8),
    (4.95, 20_000, 0.7),
    (5.95, 20_000, 0.8),
    (8.00, 20_000, 0.7),
    (99.0, 20_000, 0.7),
];

/// The turns and the feed for a drill: the smallest ones keep the speed of the 0.30 mm row.
pub fn drill_speed(d: f64) -> (u32, u32) {
    let row = DRILL_FR4
        .iter()
        .find(|r| d <= r.0 + 1e-9)
        .unwrap_or(&DRILL_FR4[DRILL_FR4.len() - 1]);
    (row.1, (row.2 * 1000.0).round() as u32)
}

/// The turns and the feed (mm per minute) for a router in FR-4: each row is the largest diameter it holds.
pub fn router_speed(d: f64) -> (u32, u32) {
    const ROWS: [(f64, u32, f64); 5] = [
        (0.8, 40_000, 0.15),
        (1.2, 36_500, 0.30),
        (1.8, 32_000, 0.50),
        (2.2, 28_000, 0.65),
        (99.0, 26_500, 0.75),
    ];
    let row = ROWS.iter().find(|r| d <= r.0 + 1e-9).unwrap_or(&ROWS[4]);
    (row.1, (row.2 * 1000.0).round() as u32)
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The diameters from `from` to `to` in steps of `step`.
fn steps(from: f64, to: f64, step: f64) -> Vec<f64> {
    let n = ((to - from) / step).round() as i32;
    (0..=n).map(|i| round3(from + i as f64 * step)).collect()
}

/// The shortest length of the flute that the maker lists for the small drills (0.30 to 0.75 mm).
fn small_flute(d: f64) -> f64 {
    if d < 0.375 {
        5.0
    } else if d < 0.525 {
        6.5
    } else if d < 0.575 {
        7.0
    } else {
        8.5
    }
}

fn drill(v: &mut Vec<Bit>, kind: Kind, series: &'static str, flutes: u8, d: f64, flute: f64) {
    let (rpm, infeed) = drill_speed(d);
    v.push(Bit {
        kind,
        series,
        diameter_mm: d,
        tip_mm: 0.0,
        angle_deg: 0,
        flutes,
        flute_mm: flute,
        rpm,
        feed_mm_min: 0,
        plunge_mm_min: infeed,
        pass_mm: 1.8,
    });
}

fn router(v: &mut Vec<Bit>, kind: Kind, series: &'static str, flutes: u8, list: &[(f64, f64)]) {
    for &(d, flute) in list {
        let (rpm, feed) = router_speed(d);
        v.push(Bit {
            kind,
            series,
            diameter_mm: d,
            tip_mm: 0.0,
            angle_deg: 0,
            flutes,
            flute_mm: flute,
            rpm,
            feed_mm_min: feed,
            plunge_mm_min: (feed / 2).clamp(100, 400),
            pass_mm: (d * 0.6).clamp(0.4, 1.0),
        });
    }
}

fn build() -> Vec<Bit> {
    let mut v = Vec::new();
    // ---- drills: the micro series (the length of the flute is in the maker's own table), the small ones, the medium range, the large ones with a chip breaker
    for (series, list) in [
        ("MD", &[0.05, 0.075, 0.10, 0.15][..]),
        ("MC", &[0.105, 0.12, 0.15, 0.20, 0.25][..]),
        ("MV", &[0.105, 0.12, 0.15][..]),
        ("MCV", &[0.12, 0.15, 0.20, 0.25][..]),
        ("MCW", &[0.15, 0.20, 0.25, 0.275][..]),
        ("MHW", &[0.20, 0.25, 0.275][..]),
    ] {
        for &d in list {
            drill(&mut v, Kind::Drill, series, 2, d, 0.0);
        }
    }
    for (series, flutes, last) in [
        ("UC", 2u8, 0.50),
        ("UV", 2, 0.75),
        ("UW", 2, 0.55),
        ("UHW", 1, 0.65),
    ] {
        for d in steps(0.30, last, 0.05) {
            drill(&mut v, Kind::Drill, series, flutes, d, small_flute(d));
        }
    }
    for d in steps(0.60, 0.65, 0.05) {
        drill(&mut v, Kind::Drill, "ST", 2, d, 7.0);
    }
    for d in steps(0.70, 1.60, 0.05) {
        drill(&mut v, Kind::Drill, "ST", 2, d, 10.0);
    }
    for d in steps(1.65, 3.15, 0.05).into_iter().chain([3.175]) {
        drill(&mut v, Kind::Drill, "STB", 2, d, 10.0);
    }
    for d in steps(0.80, 1.60, 0.05) {
        drill(&mut v, Kind::Drill, "UM35", 2, d, 10.0);
    }
    for d in steps(1.65, 3.15, 0.05).into_iter().chain([3.175]) {
        drill(&mut v, Kind::Drill, "UM30", 2, d, 10.0);
    }
    for d in steps(3.20, 3.95, 0.05)
        .into_iter()
        .chain(steps(4.0, 6.5, 0.1))
    {
        drill(&mut v, Kind::Drill, "ID30C", 2, d, 12.0);
    }
    for d in steps(6.55, 7.0, 0.05) {
        drill(&mut v, Kind::Drill, "ID30", 2, d, 12.0);
    }
    for d in steps(0.50, 0.65, 0.05) {
        drill(&mut v, Kind::SlotDrill, "SX24", 2, d, 4.7);
    }
    for d in steps(0.70, 1.60, 0.05) {
        drill(&mut v, Kind::SlotDrill, "SX24", 2, d, 6.7);
    }
    for d in steps(1.65, 3.15, 0.05).into_iter().chain([3.175]) {
        drill(&mut v, Kind::SlotDrill, "SX", 2, d, 10.0);
    }

    // ---- routers: diamond cut (right hand), diamond cut (left hand, down draft), chip breakers, slitting routers, diamond-coated flat ends
    router(
        &mut v,
        Kind::Diamond,
        "RCM",
        0,
        &[
            (0.6, 4.0),
            (0.7, 4.0),
            (0.8, 4.5),
            (0.9, 4.5),
            (1.0, 4.5),
            (1.1, 4.5),
            (1.2, 5.0),
            (1.3, 5.0),
            (1.4, 5.0),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.8, 6.5),
            (1.9, 8.0),
            (2.0, 8.0),
            (2.1, 8.0),
            (2.2, 8.0),
            (2.3, 9.5),
            (2.4, 9.5),
            (2.5, 9.5),
            (2.6, 9.5),
            (2.7, 9.5),
            (2.8, 9.5),
            (2.9, 9.5),
            (3.0, 10.0),
            (3.1, 10.0),
            (3.175, 10.0),
        ],
    );
    router(
        &mut v,
        Kind::DownCut,
        "RHM",
        0,
        &[
            (0.6, 4.0),
            (0.8, 4.5),
            (0.9, 4.5),
            (1.0, 4.5),
            (1.1, 4.5),
            (1.2, 5.0),
            (1.3, 5.0),
            (1.4, 5.0),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.7, 6.5),
            (1.8, 6.5),
            (1.9, 8.0),
            (2.0, 8.0),
            (2.1, 8.0),
            (2.2, 8.0),
            (2.3, 9.5),
            (2.4, 9.5),
            (2.5, 9.5),
            (2.6, 9.5),
            (2.7, 9.5),
            (2.8, 9.5),
            (2.9, 9.5),
            (3.0, 10.0),
            (3.1, 10.0),
            (3.175, 10.0),
        ],
    );
    router(
        &mut v,
        Kind::ChipBreaker,
        "SC",
        0,
        &[
            (0.5, 2.5),
            (0.6, 3.0),
            (0.7, 4.0),
            (0.8, 4.0),
            (0.9, 4.0),
            (1.0, 4.5),
            (1.1, 4.5),
            (1.2, 5.0),
            (1.3, 5.0),
            (1.4, 5.0),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.7, 6.5),
            (1.8, 6.5),
            (1.9, 8.0),
            (2.0, 8.0),
            (2.1, 8.0),
            (2.2, 8.0),
            (2.3, 9.5),
            (2.4, 9.5),
            (2.5, 9.5),
            (2.6, 9.5),
            (2.7, 9.5),
            (2.8, 9.5),
            (3.0, 10.0),
            (3.1, 10.0),
            (3.175, 10.0),
        ],
    );
    router(
        &mut v,
        Kind::ChipBreaker,
        "RLE",
        0,
        &[
            (0.8, 4.5),
            (0.9, 4.5),
            (1.0, 4.5),
            (1.2, 5.0),
            (1.4, 5.0),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.8, 8.0),
            (2.0, 8.0),
            (2.4, 9.5),
            (2.6, 9.5),
            (3.0, 9.5),
        ],
    );
    router(
        &mut v,
        Kind::UpCut,
        "RPEM",
        2,
        &[
            (0.4, 2.5),
            (0.6, 3.0),
            (0.8, 3.0),
            (1.0, 4.5),
            (1.2, 4.5),
            (1.3, 5.0),
            (1.4, 5.0),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.8, 6.5),
            (2.0, 8.0),
            (2.4, 8.0),
            (2.6, 8.0),
            (2.8, 8.0),
            (3.0, 8.0),
        ],
    );
    router(
        &mut v,
        Kind::DownCut,
        "RS",
        2,
        &[
            (0.8, 4.5),
            (1.0, 4.5),
            (1.2, 5.0),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.8, 6.5),
            (2.0, 8.0),
            (2.4, 8.0),
            (2.5, 8.0),
            (3.0, 9.5),
        ],
    );
    router(
        &mut v,
        Kind::Coated,
        "DCET",
        2,
        &[
            (0.8, 4.5),
            (1.0, 4.5),
            (1.2, 6.5),
            (1.6, 6.5),
            (2.0, 8.0),
            (2.4, 10.5),
            (2.5, 10.5),
        ],
    );
    router(
        &mut v,
        Kind::Coated,
        "DCRPEM",
        2,
        &[
            (1.0, 4.5),
            (1.5, 6.5),
            (1.6, 6.5),
            (1.8, 6.5),
            (2.0, 8.0),
            (2.4, 8.0),
        ],
    );

    // ---- generic bits
    for d in [
        0.2f64, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.5, 2.0, 2.4, 3.175,
    ] {
        let feed = (d * 700.0).clamp(150.0, 900.0) as u32;
        let rpm = if d < 1.0 { 60_000 } else { 40_000 };
        v.push(bit(
            Kind::EndMill,
            d,
            0.0,
            0,
            2,
            rpm,
            feed,
            feed / 3,
            (d * 0.5).clamp(0.05, 0.8),
        ));
    }
    // V-bits: the angle and the width of the tip decide how narrow the groove can be
    for (angle, shoulder) in [
        (10u32, 3.175f64),
        (15, 3.175),
        (20, 3.175),
        (30, 3.175),
        (45, 3.175),
        (60, 3.175),
        (90, 3.175),
    ] {
        for tip in [0.05, 0.1, 0.2, 0.3] {
            v.push(bit(
                Kind::VBit,
                shoulder,
                tip,
                angle,
                2,
                40_000,
                400,
                150,
                0.05,
            ));
        }
    }
    for d in [0.5f64, 1.0, 1.5, 2.0, 3.175] {
        v.push(bit(
            Kind::BallNose,
            d,
            0.0,
            0,
            2,
            40_000,
            (d * 400.0).clamp(200.0, 900.0) as u32,
            200,
            (d * 0.3).clamp(0.05, 0.5),
        ));
    }
    for angle in [60u32, 90, 120] {
        for d in [1.0, 2.0, 3.175] {
            v.push(bit(Kind::Spot, d, 0.0, angle, 2, 40_000, 0, 300, 0.3));
        }
    }
    for angle in [60u32, 90, 120] {
        for d in [3.175, 4.0, 6.0] {
            v.push(bit(
                Kind::Countersink,
                d,
                0.0,
                angle,
                1,
                12_000,
                0,
                200,
                0.5,
            ));
        }
    }
    v
}

/// Every bit of the catalogue, kind by kind and from the smallest.
pub fn all() -> &'static [Bit] {
    static ALL: OnceLock<Vec<Bit>> = OnceLock::new();
    ALL.get_or_init(build)
}

/// The bits of one kind, or all of them.
pub fn of_kind(kind: Option<Kind>) -> Vec<Bit> {
    all()
        .iter()
        .copied()
        .filter(|b| kind.is_none_or(|k| b.kind == k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalogue_is_wide_and_every_bit_fits_the_chuck() {
        let a = all();
        assert!(a.len() >= 400, "{}", a.len());
        for k in Kind::ALL {
            assert!(a.iter().any(|b| b.kind == k), "{k:?}");
        }
        assert!(a.iter().all(|b| b.shank_mm() == 3.175));
        // nothing is listed twice and everything has numbers to start from
        for (i, x) in a.iter().enumerate() {
            assert!(
                x.diameter_mm > 0.0 && x.rpm > 0 && x.plunge_mm_min > 0 && x.pass_mm > 0.0,
                "{x:?}"
            );
            assert!(!a[i + 1..].contains(x), "{x:?}");
            if x.kind == Kind::VBit {
                assert!(x.angle_deg > 0 && x.tip_mm > 0.0 && x.tip_mm < x.diameter_mm);
            }
            // only a drill bit goes in, a mill that cuts sideways has a feed
            if !x.kind.is_drill() {
                assert!(x.feed_mm_min > 0, "{x:?}");
            }
            // the cutting part is not longer than the bit
            assert!(x.flute_mm < TOTAL_MM - SHANK_MM, "{x:?}");
        }
    }

    #[test]
    fn the_drills_go_from_the_micro_series_to_seven_millimetres() {
        let d = of_kind(Some(Kind::Drill));
        let smallest = d.iter().map(|b| b.diameter_mm).fold(f64::MAX, f64::min);
        let largest = d.iter().map(|b| b.diameter_mm).fold(0.0, f64::max);
        assert!((smallest - 0.05).abs() < 1e-9 && (largest - 7.0).abs() < 1e-9);
        // the usual sizes of a board are all there, with the series that makes them
        for size in [0.3, 0.6, 0.8, 1.0, 1.2, 3.0, 3.175, 4.0, 6.5, 7.0] {
            assert!(
                d.iter().any(|b| (b.diameter_mm - size).abs() < 1e-9),
                "{size}"
            );
        }
        assert!(d.iter().any(|b| b.series == "ID30C"));
        assert!(of_kind(Some(Kind::SlotDrill))
            .iter()
            .all(|b| b.series.starts_with("SX")));
    }

    #[test]
    fn the_speeds_are_the_makers_table_and_a_slow_spindle_keeps_the_bite() {
        // the 1 mm drill: 54 000 turns and 3.1 m/min in
        let (rpm, infeed) = drill_speed(1.0);
        assert_eq!((rpm, infeed), (54_000, 3100));
        // a big drill turns slower than a small one
        assert!(drill_speed(3.0).0 < drill_speed(0.5).0);
        let one = of_kind(Some(Kind::Drill))
            .into_iter()
            .find(|b| (b.diameter_mm - 1.0).abs() < 1e-9 && b.series == "ST")
            .unwrap();
        // a spindle of 24 000 turns: the same bite for each turn, so the infeed goes down in the same proportion
        let slow = one.for_spindle(24_000);
        assert_eq!(slow.rpm, 24_000);
        let bite = |b: &Bit| b.plunge_mm_min as f64 / b.rpm as f64;
        assert!((bite(&slow) - bite(&one)).abs() < 1e-3, "{slow:?} {one:?}");
        // a fast enough spindle, or one that is not known, changes nothing
        assert_eq!(one.for_spindle(200_000), one);
        assert_eq!(one.for_spindle(0), one);
        // a router: its feed goes down with the turns too
        let r = of_kind(Some(Kind::Diamond))
            .into_iter()
            .find(|b| (b.diameter_mm - 2.0).abs() < 1e-9)
            .unwrap();
        let r24 = r.for_spindle(24_000);
        assert!(r24.rpm == 24_000 && r24.feed_mm_min < r.feed_mm_min);
    }

    #[test]
    fn a_v_bit_cuts_a_groove_as_wide_as_the_depth_makes_it() {
        let v = bit(Kind::VBit, 3.175, 0.1, 30, 2, 40_000, 400, 150, 0.05);
        // 30 degrees: each side opens by tan(15 degrees) of the depth
        let w = v.groove_mm(0.1).unwrap();
        assert!(
            (w - (0.1 + 2.0 * 0.1 * 15f64.to_radians().tan())).abs() < 1e-9,
            "{w}"
        );
        assert_eq!(v.groove_mm(0.0), Some(0.1));
        assert!(v.groove_mm(100.0).unwrap() <= 3.175);
        let depth = v.depth_for_groove_mm(0.3).unwrap();
        assert!((v.groove_mm(depth).unwrap() - 0.3).abs() < 1e-9);
        assert!(v.depth_for_groove_mm(0.05).is_none() && v.depth_for_groove_mm(9.0).is_none());
        let drill = bit(Kind::Drill, 1.0, 0.0, 0, 2, 50_000, 0, 600, 1.8);
        assert!(drill.groove_mm(0.1).is_none() && drill.depth_for_groove_mm(0.3).is_none());
    }
}
