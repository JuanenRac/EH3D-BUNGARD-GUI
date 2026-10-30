//! The catalogue of bits for a PCB: drills, end mills, V-bits and the rest, all with the shank of 1/8 inch (3.175 mm) that the chuck of a Bungard machine holds. Each bit
//! has the numbers a job starts from on FR-4 (the turns of the spindle, the feed along the cut, the feed of the plunge and how deep one pass goes); they are starting
//! points for a try on a scrap board, not the word of a maker.

/// The shank every bit of the catalogue has: 1/8 inch.
pub const SHANK_MM: f64 = 3.175;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A drill for through holes.
    Drill,
    /// A square end mill with two flutes: slots, isolation and the outline.
    EndMill,
    /// A router that pushes the chips down, for a clean upper edge of the outline.
    DownCut,
    /// A router that pulls the chips up, for slots and pockets.
    UpCut,
    /// A router with the cut of a diamond (the "corn" pattern), for the outline of FR-4 and G10.
    Diamond,
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
    pub const ALL: [Kind; 9] = [
        Kind::Drill,
        Kind::EndMill,
        Kind::DownCut,
        Kind::UpCut,
        Kind::Diamond,
        Kind::VBit,
        Kind::BallNose,
        Kind::Spot,
        Kind::Countersink,
    ];

    /// What the bit is used for, in a word the window translates.
    pub fn is_drill(self) -> bool {
        matches!(self, Kind::Drill | Kind::Spot | Kind::Countersink)
    }
}

/// One bit of the catalogue.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bit {
    pub kind: Kind,
    /// The diameter of the cut (of the shoulder of a V-bit or a countersink), in mm.
    pub diameter_mm: f64,
    /// The width of the tip of a V-bit, in mm (0 for the rest).
    pub tip_mm: f64,
    /// The included angle of a V-bit, a spot drill or a countersink, in degrees (0 for the rest).
    pub angle_deg: u32,
    pub flutes: u8,
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
        diameter_mm: d,
        tip_mm: tip,
        angle_deg: angle,
        flutes,
        rpm,
        feed_mm_min: feed,
        plunge_mm_min: plunge,
        pass_mm: pass,
    }
}

/// Every bit of the catalogue, kind by kind and from the smallest.
pub fn all() -> Vec<Bit> {
    let mut v = Vec::new();
    // drills: the small ones turn faster and go in slower
    for d in [
        0.3, 0.4, 0.5, 0.6, 0.7, 0.75, 0.8, 0.85, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8,
        2.0, 2.2, 2.5, 2.8, 3.0, 3.175,
    ] {
        let rpm = if d < 0.8 {
            60_000
        } else if d < 1.6 {
            50_000
        } else {
            35_000
        };
        let plunge = if d < 0.6 {
            300
        } else if d < 1.2 {
            600
        } else {
            900
        };
        v.push(bit(Kind::Drill, d, 0.0, 0, 2, rpm, 0, plunge, 1.8));
    }
    // square end mills
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
    // routers for the outline and slots
    for d in [0.8f64, 1.0, 1.2, 1.5, 1.6, 2.0, 2.4, 3.0, 3.175] {
        for kind in [Kind::DownCut, Kind::UpCut] {
            v.push(bit(
                kind,
                d,
                0.0,
                0,
                2,
                40_000,
                (d * 450.0).clamp(300.0, 1200.0) as u32,
                250,
                (d * 0.6).clamp(0.4, 1.0),
            ));
        }
    }
    for d in [1.0f64, 1.2, 1.5, 1.6, 2.0, 2.4, 3.0, 3.175] {
        v.push(bit(
            Kind::Diamond,
            d,
            0.0,
            0,
            0,
            40_000,
            (d * 400.0).clamp(300.0, 1100.0) as u32,
            250,
            (d * 0.6).clamp(0.4, 1.0),
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

/// The bits of one kind, or all of them.
pub fn of_kind(kind: Option<Kind>) -> Vec<Bit> {
    all()
        .into_iter()
        .filter(|b| kind.is_none_or(|k| b.kind == k))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalogue_is_wide_and_every_bit_fits_the_chuck() {
        let a = all();
        assert!(a.len() >= 100, "{}", a.len());
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
        }
        assert_eq!(of_kind(Some(Kind::Drill)).len(), 24);
        assert_eq!(of_kind(None).len(), a.len());
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
