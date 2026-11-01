//! How the width of the main screen is shared between the list of the job (left), the plan view (middle) and the machine view (right).
//! The three widths are in the units of the drawing board of the window (the same at every window size); the middle one is what the other two leave.
//! Every pane keeps a least width so that none of them disappears and what is drawn inside never overlaps.

/// The width of the drawing board the window is drawn on.
pub const BOARD_W: f32 = 1600.0;
/// What the margins and the gaps between the panes take: 20 + 16 + 16 + 20.
pub const FRAME: f32 = 72.0;
/// What the three panes share.
pub const TOTAL: f32 = BOARD_W - FRAME;
/// The least width of the left, the middle and the right pane.
pub const MIN: [f32; 3] = [220.0, 560.0, 380.0];
/// The width the left pane has when nothing was dragged, and the one of the right pane.
pub const DEFAULT_LEFT: f32 = 280.0;
pub const DEFAULT_RIGHT: f32 = 608.0;

/// The three widths for the wanted left and right ones: each is brought inside its limits and the middle one is the rest.
pub fn split(left: f32, right: f32) -> (f32, f32, f32) {
    let left_max = TOTAL - MIN[1] - MIN[2];
    let left = if left.is_finite() { left } else { DEFAULT_LEFT }.clamp(MIN[0], left_max);
    let right_max = TOTAL - left - MIN[1];
    let right = if right.is_finite() {
        right
    } else {
        DEFAULT_RIGHT
    }
    .clamp(MIN[2], right_max);
    (left, TOTAL - left - right, right)
}

/// The bar between the left and the middle pane is dragged by `delta`: the left pane grows or shrinks, the right one stays.
pub fn drag_left(left: f32, right: f32, delta: f32) -> (f32, f32) {
    let (l, _, r) = split(left + delta, right);
    (l, r)
}

/// The bar between the middle and the right pane is dragged by `delta` (to the right shrinks the right pane); the left one stays.
pub fn drag_right(left: f32, right: f32, delta: f32) -> (f32, f32) {
    let (l, _, r) = split(left, right - delta);
    (l, r)
}

/// The x of the left bar and of the right bar (the middle of the gap each one sits in), on the drawing board.
pub fn bars(left: f32, right: f32) -> (f32, f32) {
    let (l, m, _) = split(left, right);
    (20.0 + l + 8.0, 20.0 + l + 16.0 + m + 8.0)
}

/// Reads `LEFT,RIGHT`.
pub fn parse(text: &str) -> Option<(f32, f32)> {
    let (a, b) = text.split_once(',')?;
    Some((a.trim().parse().ok()?, b.trim().parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_split_is_what_the_screen_always_had() {
        let (l, m, r) = split(DEFAULT_LEFT, DEFAULT_RIGHT);
        assert_eq!((l, r), (280.0, 608.0));
        assert_eq!(m, 640.0);
        assert_eq!(20.0 + l + 16.0, 316.0);
    }

    #[test]
    fn the_three_widths_always_fill_the_room() {
        for (l, r) in [(0.0, 0.0), (1000.0, 1000.0), (300.0, 900.0), (-5.0, 5000.0)] {
            let (a, b, c) = split(l, r);
            assert!((a + b + c - TOTAL).abs() < 1e-3);
            assert!(a >= MIN[0] && b >= MIN[1] - 1e-3 && c >= MIN[2]);
        }
    }

    #[test]
    fn no_pane_goes_under_its_least_width() {
        assert_eq!(drag_left(280.0, 608.0, -1000.0).0, MIN[0]);
        let (l, r) = drag_left(280.0, 608.0, 5000.0);
        assert!((TOTAL - l - r - MIN[1]).abs() < 1e-3);
        assert_eq!(drag_right(280.0, 608.0, 5000.0).1, MIN[2]);
        let (l, r) = drag_right(280.0, 608.0, -5000.0);
        assert!((TOTAL - l - r - MIN[1]).abs() < 1e-3);
    }

    #[test]
    fn dragging_one_bar_leaves_the_other_pane_where_it_was() {
        let (l, r) = drag_left(280.0, 608.0, 40.0);
        assert_eq!((l, r), (320.0, 608.0));
        let (l, r) = drag_right(280.0, 608.0, 40.0);
        assert_eq!((l, r), (280.0, 568.0));
    }

    #[test]
    fn a_left_pane_that_grows_pushes_the_right_one_before_the_middle_gets_too_small() {
        let (l, m, r) = split(900.0, 608.0);
        assert_eq!(m, MIN[1]);
        assert!(r >= MIN[2] && l + r + m - TOTAL < 1e-3);
    }

    #[test]
    fn a_bad_number_gives_the_default() {
        assert_eq!(
            split(f32::NAN, f32::NAN),
            split(DEFAULT_LEFT, DEFAULT_RIGHT)
        );
    }

    #[test]
    fn the_bars_sit_in_the_gaps() {
        let (a, b) = bars(DEFAULT_LEFT, DEFAULT_RIGHT);
        assert_eq!(a, 308.0);
        assert_eq!(b, 20.0 + 280.0 + 16.0 + 640.0 + 8.0);
    }

    #[test]
    fn the_command_line_value_is_read() {
        assert_eq!(parse("300, 500"), Some((300.0, 500.0)));
        assert_eq!(parse("300"), None);
        assert_eq!(parse("a,b"), None);
    }
}
