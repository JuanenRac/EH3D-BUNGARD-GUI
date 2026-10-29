//! Starting values for the tools of a job: what to take when nothing better is known. They are the usual ones for solid carbide drills and
//! two millimetre routers on a spindle that goes up to 60000 rpm; the material of the board changes them (paper boards are softer, the
//! PTFE and ceramic filled laminates wear the tools and want less speed, aluminium wants much less). They are a start, to be tuned on the
//! machine and then typed in: the values of the tool table are always the operator's to change.

use crate::boards::Core;

/// How the material changes the speed and the feed: (rpm factor, feed factor).
fn material_factors(core: Core) -> (f64, f64) {
    match core {
        Core::Fr2 | Core::Fr3 | Core::Cem1 => (1.0, 1.3),
        Core::Cem3 | Core::Fr4 | Core::Polyimide => (1.0, 1.0),
        Core::Hydrocarbon | Core::Ptfe => (0.8, 0.8),
        Core::Flex => (1.0, 1.2),
        Core::Aluminium => (0.5, 0.5),
        Core::Ceramic => (0.3, 0.2),
    }
}

fn round_to(v: f64, step: f64) -> f64 {
    (v / step).round() * step
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
    let passes = (thickness_mm / 1.0).ceil().max(1.0) as u32;
    let (kr, kf) = material_factors(core);
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
}
