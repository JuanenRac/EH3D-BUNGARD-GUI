//! The spindles the machine can carry, and whether a bit of the catalogue can work on each of them. A spindle has its own limits (the turns, the feeds the machine can
//! hold with it and the thickest bit it takes); a bit asks for the maker's turns and feeds. Where the spindle cannot give what the bit asks for, the numbers that do work
//! are worked out (the same bite for each turn) so the person can take them; where nothing can work, the bit is said to be not compatible.

use crate::catalogue::Bit;

/// Below this part of the turns a bit asks for, a spindle cannot cut with it: a small drill breaks and a router burns the board.
const LEAST_SPEED_PART: f64 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpindleId {
    /// The spindle the machine leaves the factory with.
    Original,
    /// The spindle with its variable-frequency drive, driven by the node of the machine.
    Vfd,
}

impl SpindleId {
    pub const ALL: [SpindleId; 2] = [SpindleId::Original, SpindleId::Vfd];

    pub fn code(self) -> &'static str {
        match self {
            SpindleId::Original => "original",
            SpindleId::Vfd => "vfd",
        }
    }

    pub fn from_code(s: &str) -> Option<SpindleId> {
        SpindleId::ALL.into_iter().find(|i| i.code() == s.trim())
    }
}

/// The limits of one spindle on this machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spindle {
    pub id: SpindleId,
    /// The slowest and the fastest turns, per minute.
    pub min_rpm: u32,
    pub max_rpm: u32,
    /// The power in watts.
    pub power_w: u32,
    /// The fastest feed along the cut (XY) and of the plunge (Z) that the machine holds with this spindle, in mm per minute.
    pub max_xy_mm_min: u32,
    pub max_z_mm_min: u32,
    /// The thickest bit the spindle takes, in mm.
    pub max_bit_mm: f64,
}

impl Spindle {
    /// The starting values: prudent ones, to be tuned on the machine (they are kept in the settings file, one line each). They come from the table of the spindles
    /// shared with the server (`spindles.cfg` of the shared crate), so both say the same.
    pub fn default_for(id: SpindleId) -> Spindle {
        let l = eh3d_materials::spindle::defaults(id.code())
            .expect("the table of the spindles has every spindle");
        Spindle {
            id,
            min_rpm: l.min_rpm,
            max_rpm: l.max_rpm,
            power_w: l.power_w,
            max_xy_mm_min: l.max_xy_mm_min,
            max_z_mm_min: l.max_z_mm_min,
            max_bit_mm: l.max_bit_mm,
        }
    }

    /// The line of the settings file: the six limits, separated by commas.
    pub fn to_text(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.min_rpm,
            self.max_rpm,
            self.power_w,
            self.max_xy_mm_min,
            self.max_z_mm_min,
            self.max_bit_mm
        )
    }

    /// The spindle of a line of the settings file; what is not a valid number keeps the starting value.
    pub fn from_text(id: SpindleId, v: &str) -> Spindle {
        let mut s = Spindle::default_for(id);
        let f: Vec<&str> = v.split(',').map(str::trim).collect();
        let num = |i: usize| {
            f.get(i)
                .and_then(|x| x.parse::<f64>().ok())
                .filter(|n| *n > 0.0)
        };
        if let Some(n) = num(0) {
            s.min_rpm = n as u32;
        }
        if let Some(n) = num(1) {
            s.max_rpm = (n as u32).max(1_000);
        }
        if let Some(n) = num(2) {
            s.power_w = n as u32;
        }
        if let Some(n) = num(3) {
            s.max_xy_mm_min = n as u32;
        }
        if let Some(n) = num(4) {
            s.max_z_mm_min = n as u32;
        }
        if let Some(n) = num(5) {
            s.max_bit_mm = n;
        }
        if s.min_rpm > s.max_rpm {
            s.min_rpm = Spindle::default_for(id).min_rpm.min(s.max_rpm);
        }
        s
    }

    /// Whether `bit` can work on this spindle, and with which numbers.
    pub fn fit(&self, bit: &Bit) -> Fit {
        if bit.diameter_mm > self.max_bit_mm + 1e-9 {
            return Fit::Incompatible(Why::TooThick);
        }
        if bit.rpm > 0 && (self.max_rpm as f64) < bit.rpm as f64 * LEAST_SPEED_PART {
            return Fit::Incompatible(Why::TooSlow);
        }
        let mut mine = bit.for_spindle(self.max_rpm);
        let lower_rpm = mine.rpm != bit.rpm;
        if mine.rpm < self.min_rpm {
            mine.rpm = self.min_rpm;
        }
        // the old control unit of the original spindle (63000 rpm is the maker's figure for the spindle itself) only has 16 levels, 30000 to 60000 rpm in steps of 2000: what the job asks for becomes
        // the nearest of them, and the bit is shown with the speed it will really get
        if self.id == SpindleId::Original {
            let level = (mine.rpm.clamp(30_000, 60_000) - 30_000 + 1_000) / 2_000;
            mine.rpm = 30_000 + level.min(15) * 2_000;
        }
        let lower_rpm = lower_rpm || mine.rpm != bit.rpm;
        let xy_slower = mine.feed_mm_min > self.max_xy_mm_min;
        if xy_slower {
            mine.feed_mm_min = self.max_xy_mm_min;
        }
        let z_slower = mine.plunge_mm_min > self.max_z_mm_min;
        if z_slower {
            mine.plunge_mm_min = self.max_z_mm_min;
        }
        // a drill has no feed along the cut: its plunge is the feed of the hole
        if lower_rpm || xy_slower || z_slower || mine != *bit {
            Fit::Adjusted {
                bit: mine,
                lower_rpm,
                xy_slower,
                z_slower,
            }
        } else {
            Fit::Fits
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    TooThick,
    TooSlow,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fit {
    /// The spindle takes the maker's values as they are.
    Fits,
    /// It works with other numbers: lower turns (and the feeds in proportion), or a slower XY or Z.
    Adjusted {
        bit: Bit,
        lower_rpm: bool,
        xy_slower: bool,
        z_slower: bool,
    },
    /// Nothing the machine can do makes it work.
    Incompatible(Why),
}

/// The sizes of the original spindle as the 3D view draws it (a 174 mm long, 33 mm round chrome body with an O20 nose that has an O27 collar), from its drawing.
/// They are kept in the settings file (`spindle_drawing`, eight numbers in mm separated by commas); what is not a valid number keeps the value of the drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeadDims {
    pub total_mm: f64,
    pub nose_mm: f64,
    pub nose_d: f64,
    pub collar_d: f64,
    pub collar_mm: f64,
    pub body_mm: f64,
    pub body_d: f64,
    /// Height over the axis of the connector at the rear, in mm (the overall height of the drawing).
    pub connector_h: f64,
}

impl Default for HeadDims {
    fn default() -> HeadDims {
        HeadDims {
            total_mm: 174.0,
            nose_mm: 33.0,
            nose_d: 20.0,
            collar_d: 27.0,
            collar_mm: 14.0,
            body_mm: 88.0,
            body_d: 33.0,
            connector_h: 59.0,
        }
    }
}

impl HeadDims {
    /// The line of the settings file: the eight sizes in the order of the fields.
    pub fn to_text(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{}",
            self.total_mm,
            self.nose_mm,
            self.nose_d,
            self.collar_d,
            self.collar_mm,
            self.body_mm,
            self.body_d,
            self.connector_h
        )
    }

    /// The sizes of a line of the settings file. A number that is missing, is not a number or is not a sensible size (zero, negative, over 1000 mm)
    /// keeps the size of the drawing; the collar cannot be longer than the nose.
    pub fn from_text(v: &str) -> HeadDims {
        let mut d = HeadDims::default();
        let f: Vec<&str> = v.split(',').map(str::trim).collect();
        let num = |i: usize, keep: f64| {
            f.get(i)
                .and_then(|x| x.parse::<f64>().ok())
                .filter(|n| n.is_finite() && *n > 0.0 && *n <= 1000.0)
                .unwrap_or(keep)
        };
        d.total_mm = num(0, d.total_mm);
        d.nose_mm = num(1, d.nose_mm);
        d.nose_d = num(2, d.nose_d);
        d.collar_d = num(3, d.collar_d);
        d.collar_mm = num(4, d.collar_mm);
        d.body_mm = num(5, d.body_mm);
        d.body_d = num(6, d.body_d);
        d.connector_h = num(7, d.connector_h);
        d.collar_mm = d.collar_mm.min(d.nose_mm);
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::{all, Kind};

    fn first(kind: Kind) -> Bit {
        *all()
            .iter()
            .find(|b| b.kind == kind)
            .expect("a bit of the kind")
    }

    #[test]
    fn the_spindles_are_the_ones_of_the_shared_table_with_its_numbers() {
        let ids: Vec<&str> = SpindleId::ALL.iter().map(|i| i.code()).collect();
        assert_eq!(ids, eh3d_materials::spindle::IDS);
        for id in SpindleId::ALL {
            let shared = eh3d_materials::spindle::defaults(id.code()).unwrap();
            let own = Spindle::default_for(id);
            assert_eq!(
                (own.min_rpm, own.max_rpm, own.power_w),
                (shared.min_rpm, shared.max_rpm, shared.power_w)
            );
            assert_eq!(
                (own.max_xy_mm_min, own.max_z_mm_min, own.max_bit_mm),
                (shared.max_xy_mm_min, shared.max_z_mm_min, shared.max_bit_mm)
            );
        }
    }

    #[test]
    fn the_two_spindles_have_the_turns_of_the_machines() {
        assert_eq!(Spindle::default_for(SpindleId::Original).max_rpm, 63_000);
        assert_eq!(Spindle::default_for(SpindleId::Vfd).max_rpm, 24_000);
        assert_eq!(SpindleId::from_code("vfd"), Some(SpindleId::Vfd));
        assert_eq!(
            SpindleId::from_code(" original "),
            Some(SpindleId::Original)
        );
        assert_eq!(SpindleId::from_code("x"), None);
    }

    #[test]
    fn the_line_of_the_settings_comes_back_and_a_bad_one_keeps_the_start() {
        for id in SpindleId::ALL {
            let s = Spindle::default_for(id);
            assert_eq!(Spindle::from_text(id, &s.to_text()), s);
        }
        let mut s = Spindle::default_for(SpindleId::Vfd);
        s.max_xy_mm_min = 900;
        assert_eq!(
            Spindle::from_text(SpindleId::Vfd, &s.to_text()).max_xy_mm_min,
            900
        );
        assert_eq!(
            Spindle::from_text(SpindleId::Vfd, "x,y,,-3"),
            Spindle::default_for(SpindleId::Vfd)
        );
    }

    #[test]
    fn a_bit_that_the_spindle_can_give_takes_the_makers_values_or_the_ones_that_it_holds() {
        let slow = Spindle {
            max_rpm: 200_000,
            ..Spindle::default_for(SpindleId::Vfd)
        };
        let mut fits = 0;
        for b in all().iter().filter(|b| b.diameter_mm <= slow.max_bit_mm) {
            match slow.fit(b) {
                Fit::Fits => fits += 1,
                Fit::Adjusted { bit, lower_rpm, .. } => {
                    assert!(!lower_rpm);
                    assert!(bit.feed_mm_min <= slow.max_xy_mm_min);
                    assert!(bit.plunge_mm_min <= slow.max_z_mm_min);
                }
                Fit::Incompatible(_) => {}
            }
        }
        assert!(fits > 0, "some bits need nothing changed");
    }

    #[test]
    fn a_slower_spindle_lowers_the_turns_and_the_feeds_in_proportion_and_says_so() {
        let vfd = Spindle::default_for(SpindleId::Vfd);
        let b = all()
            .iter()
            .copied()
            .find(|b| {
                b.rpm > 24_000
                    && b.rpm as f64 * LEAST_SPEED_PART <= 24_000.0
                    && b.diameter_mm <= vfd.max_bit_mm
            })
            .expect("a fast bit that the VFD spindle can still work with");
        match vfd.fit(&b) {
            Fit::Adjusted { bit, lower_rpm, .. } => {
                assert!(lower_rpm);
                assert_eq!(bit.rpm, 24_000);
                assert!(bit.feed_mm_min <= b.feed_mm_min && bit.plunge_mm_min <= b.plunge_mm_min);
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_bit_that_asks_for_far_more_turns_than_the_spindle_has_is_not_compatible() {
        let mut b = first(Kind::Drill);
        b.rpm = 100_000;
        let vfd = Spindle::default_for(SpindleId::Vfd);
        assert_eq!(vfd.fit(&b), Fit::Incompatible(Why::TooSlow));
        // the same bit on the fast spindle is fine
        assert!(!matches!(
            Spindle::default_for(SpindleId::Original).fit(&b),
            Fit::Incompatible(_)
        ));
    }

    #[test]
    fn the_original_spindle_gets_the_nearest_level_of_its_control_unit() {
        let original = Spindle::default_for(SpindleId::Original);
        for (asked, gets) in [
            (18_000, 30_000),
            (31_000, 32_000),
            (45_000, 46_000),
            (61_500, 60_000),
        ] {
            let mut b = first(Kind::Drill);
            b.rpm = asked;
            b.diameter_mm = b.diameter_mm.min(original.max_bit_mm);
            match original.fit(&b) {
                Fit::Adjusted { bit, .. } => assert_eq!(bit.rpm, gets, "asked {asked}"),
                other => panic!("asked {asked}: {other:?}"),
            }
        }
        let mut b = first(Kind::Drill);
        b.rpm = 36_000;
        b.diameter_mm = b.diameter_mm.min(original.max_bit_mm);
        assert!(!matches!(original.fit(&b), Fit::Incompatible(_)));
    }

    #[test]
    fn a_bit_thicker_than_the_spindle_takes_is_not_compatible() {
        let mut b = first(Kind::Drill);
        b.diameter_mm = 5.0;
        b.rpm = 20_000;
        assert_eq!(
            Spindle::default_for(SpindleId::Original).fit(&b),
            Fit::Incompatible(Why::TooThick)
        );
        assert!(!matches!(
            Spindle::default_for(SpindleId::Vfd).fit(&b),
            Fit::Incompatible(_)
        ));
    }

    #[test]
    fn the_xy_and_the_z_of_a_heavy_spindle_are_held_back() {
        let mut b = first(Kind::Drill);
        b.rpm = 20_000;
        b.feed_mm_min = 5_000;
        b.plunge_mm_min = 6_000;
        let vfd = Spindle::default_for(SpindleId::Vfd);
        match vfd.fit(&b) {
            Fit::Adjusted {
                bit,
                lower_rpm,
                xy_slower,
                z_slower,
            } => {
                assert!(!lower_rpm && xy_slower && z_slower);
                assert_eq!((bit.feed_mm_min, bit.plunge_mm_min), (1_500, 1_200));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn a_bit_that_asks_for_less_than_the_spindle_can_turn_slowly_gets_the_least_turns() {
        let mut b = first(Kind::Drill);
        b.rpm = 2_000;
        b.feed_mm_min = 300;
        b.plunge_mm_min = 300;
        let vfd = Spindle::default_for(SpindleId::Vfd);
        match vfd.fit(&b) {
            Fit::Adjusted { bit, .. } => assert_eq!(bit.rpm, vfd.min_rpm),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_drawing_sizes_round_trip_and_bad_numbers_keep_the_drawing() {
        let d = HeadDims::default();
        assert_eq!(HeadDims::from_text(&d.to_text()), d);
        let odd = HeadDims::from_text("200, x, -3, 0, 99, 88, 40");
        assert_eq!(odd.total_mm, 200.0);
        assert_eq!(odd.nose_mm, d.nose_mm);
        assert_eq!(odd.nose_d, d.nose_d);
        assert_eq!(odd.collar_d, d.collar_d);
        // the collar is cut to the nose
        assert_eq!(odd.collar_mm, 33.0);
        assert_eq!(odd.body_d, 40.0);
        assert_eq!(odd.connector_h, d.connector_h);
    }
}
