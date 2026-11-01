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
    /// The starting values: prudent ones, to be tuned on the machine (they are kept in the settings file, one line each).
    pub fn default_for(id: SpindleId) -> Spindle {
        match id {
            SpindleId::Original => Spindle {
                id,
                min_rpm: 3_000,
                max_rpm: 63_000,
                power_w: 150,
                max_xy_mm_min: 3_000,
                max_z_mm_min: 4_000,
                max_bit_mm: 3.2,
            },
            SpindleId::Vfd => Spindle {
                id,
                min_rpm: 6_000,
                max_rpm: 24_000,
                power_w: 2_200,
                max_xy_mm_min: 1_500,
                max_z_mm_min: 1_200,
                max_bit_mm: 6.0,
            },
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
        // the control unit of the original spindle only has 16 levels, 30000 to 60000 rpm in steps of 2000: what the job asks for becomes
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
        for (asked, gets) in [(18_000, 30_000), (31_000, 32_000), (45_000, 46_000), (61_500, 60_000)] {
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
}
