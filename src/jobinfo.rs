//! What the bar under the views says about the work: the steps done and the steps in all of each tool of the loaded program, the tool that works now, the one that
//! comes next, how many tools are finished and the time left. It is worked out from the scene alone, so it is the same for a real job and for a simulated one.

use eh3d_view::scene::{Hole, Path};

/// The steps of one tool: the holes it drills or the points of the paths it routes.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolStat {
    pub tool: u8,
    /// A drill (its steps are holes) or a cutter (its steps are the points of its paths).
    pub drill: bool,
    /// The diameter of its holes (0 for a cutter: the file does not say it).
    pub diameter_mm: f64,
    pub done: usize,
    pub total: usize,
}

impl ToolStat {
    pub fn finished(&self) -> bool {
        self.total > 0 && self.done >= self.total
    }
}

/// The steps of each tool, in the order the file uses the tools (the drills first, then the cutters).
pub fn tool_stats(holes: &[Hole], paths: &[Path]) -> Vec<ToolStat> {
    let mut out: Vec<ToolStat> = Vec::new();
    for h in holes {
        let at = match out.iter().position(|s| s.drill && s.tool == h.tool) {
            Some(i) => i,
            None => {
                out.push(ToolStat {
                    tool: h.tool,
                    drill: true,
                    diameter_mm: h.diameter_mm,
                    done: 0,
                    total: 0,
                });
                out.len() - 1
            }
        };
        out[at].total += 1;
        if h.done {
            out[at].done += 1;
        }
    }
    for p in paths {
        let at = match out.iter().position(|s| !s.drill && s.tool == p.tool) {
            Some(i) => i,
            None => {
                out.push(ToolStat {
                    tool: p.tool,
                    drill: false,
                    diameter_mm: 0.0,
                    done: 0,
                    total: 0,
                });
                out.len() - 1
            }
        };
        out[at].total += p.points.len();
        out[at].done += p.reached.min(p.points.len());
    }
    out
}

/// Where the work is among its tools.
#[derive(Debug, Clone, PartialEq)]
pub struct Where {
    /// The tool that works now (the one the machine says, or the first that is not finished when it says none).
    pub current: Option<ToolStat>,
    /// The next tool that still has steps to do after the current one.
    pub next: Option<ToolStat>,
    /// The tools that are finished and the tools in all.
    pub tools_done: usize,
    pub tools_total: usize,
}

/// Finds the current and the next tool. `tool` is the number the machine works with and `drilling` says whether the running program is the drilling one.
pub fn locate(stats: &[ToolStat], tool: Option<u8>, drilling: bool) -> Where {
    let current_at = tool
        .and_then(|t| {
            stats
                .iter()
                .position(|s| s.tool == t && s.drill == drilling)
        })
        .or_else(|| stats.iter().position(|s| !s.finished()));
    let next = current_at.and_then(|i| stats.iter().skip(i + 1).find(|s| !s.finished()).cloned());
    Where {
        current: current_at.map(|i| stats[i].clone()),
        next,
        tools_done: stats.iter().filter(|s| s.finished()).count(),
        tools_total: stats.len(),
    }
}

/// The seconds that are left, from the seconds spent and the part done (nothing until there is enough done to tell, and nothing when it is all done).
pub fn seconds_left(spent_s: f64, progress: f64) -> Option<u64> {
    if !(0.02..1.0).contains(&progress) || !spent_s.is_finite() || spent_s <= 0.0 {
        return None;
    }
    Some((spent_s * (1.0 - progress) / progress).round() as u64)
}

/// A time as `m:ss`, or `h:mm:ss` from an hour.
pub fn clock(s: u64) -> String {
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// The distance in mm from a point to the segment `a`-`b`.
pub fn point_segment_dist(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 {
        (((p.0 - a.0) * dx + (p.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p.0 - (a.0 + t * dx)).hypot(p.1 - (a.1 + t * dy))
}

/// Marks how far a path has been cut when the cutting tool is at `at`: the path advances to the first segment, from the one it has reached, that the tool is within `tol` mm
/// of. The points before the tool are passed, and the end of the segment only when the tool is at it. It never goes back, and a closed path whose last point is its first is not
/// finished when the tool is at the start, because the search begins where the path left off.
pub fn advance_reached(path: &mut Path, at: (f64, f64), tol: f64) {
    let n = path.points.len();
    if n == 0 || path.reached >= n {
        return;
    }
    if n == 1 {
        let p = path.points[0];
        if (at.0 - p.0).hypot(at.1 - p.1) <= tol {
            path.reached = 1;
        }
        return;
    }
    for i in path.reached.saturating_sub(1)..n - 1 {
        let (a, b) = (path.points[i], path.points[i + 1]);
        if point_segment_dist(at, a, b) <= tol {
            let at_end = (at.0 - b.0).hypot(at.1 - b.1) <= tol;
            let count = if at_end { i + 2 } else { i + 1 };
            path.reached = path.reached.max(count);
            return;
        }
    }
}

/// The hour of the day (`HH:MM`) that comes `left_s` seconds after `now_unix_s` (seconds since 1970), on a clock that is `offset_min` minutes from universal time.
pub fn end_clock(now_unix_s: u64, left_s: u64, offset_min: i32) -> String {
    let t = now_unix_s.saturating_add(left_s) as i64 + i64::from(offset_min) * 60;
    let day = t.rem_euclid(86_400);
    format!("{:02}:{:02}", day / 3600, day % 3600 / 60)
}

/// Minutes the time zone of the system is from universal time at the given moment (seconds since 1970), daylight saving included.
pub fn system_utc_offset_min(now_unix_s: u64) -> i32 {
    use chrono::{Offset, TimeZone};
    i64::try_from(now_unix_s)
        .ok()
        .and_then(|t| chrono::Local.timestamp_opt(t, 0).single())
        .map_or(0, |d| d.offset().fix().local_minus_utc() / 60)
}

/// The offset the end hour is shown with: the one of the settings when there is one, the one of the system when not.
pub fn clock_offset_min(setting: Option<i32>, now_unix_s: u64) -> i32 {
    setting.unwrap_or_else(|| system_utc_offset_min(now_unix_s))
}

/// The length of the paths in mm, as (cut so far, in all): a segment is cut when the point it ends at has been reached.
pub fn route_lengths(paths: &[Path]) -> (f64, f64) {
    let (mut done, mut total) = (0.0, 0.0);
    for p in paths {
        let reached = p.reached.min(p.points.len());
        for (i, w) in p.points.windows(2).enumerate() {
            let l = (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1);
            total += l;
            if i + 1 < reached {
                done += l;
            }
        }
    }
    (done, total)
}

/// The tool changes made so far and in all: every tool that has begun but the first came after a change.
pub fn tool_changes(stats: &[ToolStat]) -> (usize, usize) {
    let begun = stats.iter().filter(|s| s.done > 0).count();
    (begun.saturating_sub(1), stats.len().saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hole(tool: u8, d: f64, done: bool) -> Hole {
        Hole {
            x_mm: 0.0,
            y_mm: 0.0,
            diameter_mm: d,
            tool,
            done,
        }
    }

    fn path(tool: u8, n: usize, reached: usize) -> Path {
        Path {
            tool,
            points: vec![(0.0, 0.0); n],
            reached,
        }
    }

    #[test]
    fn the_holes_are_counted_by_tool_in_the_order_of_the_file() {
        let holes = [
            hole(2, 1.0, true),
            hole(1, 0.8, true),
            hole(1, 0.8, false),
            hole(2, 1.0, false),
            hole(1, 0.8, false),
        ];
        let s = tool_stats(&holes, &[]);
        assert_eq!(s.len(), 2);
        assert_eq!((s[0].tool, s[0].done, s[0].total), (2, 1, 2));
        assert_eq!((s[1].tool, s[1].done, s[1].total), (1, 1, 3));
        assert!(s[1].drill && (s[1].diameter_mm - 0.8).abs() < 1e-12);
    }

    #[test]
    fn the_points_of_the_paths_are_counted_by_cutter_and_a_cutter_is_not_a_drill_of_its_number() {
        let s = tool_stats(
            &[hole(1, 0.8, false)],
            &[path(1, 10, 4), path(1, 5, 5), path(2, 8, 0)],
        );
        assert_eq!(s.len(), 3);
        assert!(s[0].drill && !s[1].drill);
        assert_eq!((s[1].tool, s[1].done, s[1].total), (1, 9, 15));
        assert_eq!((s[2].tool, s[2].done, s[2].total), (2, 0, 8));
        // more points reached than the path has cannot count more than it has
        let s = tool_stats(&[], &[path(3, 4, 9)]);
        assert_eq!((s[0].done, s[0].total), (4, 4));
    }

    #[test]
    fn the_current_tool_is_the_one_the_machine_says_and_the_next_is_the_following_one_that_is_not_finished(
    ) {
        let holes = [
            hole(1, 0.8, true),
            hole(2, 1.0, false),
            hole(3, 1.2, true),
            hole(4, 1.5, false),
        ];
        let s = tool_stats(&holes, &[]);
        let w = locate(&s, Some(2), true);
        assert_eq!(w.current.as_ref().map(|c| c.tool), Some(2));
        // tool 3 is finished: the next one that has work is 4
        assert_eq!(w.next.as_ref().map(|c| c.tool), Some(4));
        assert_eq!((w.tools_done, w.tools_total), (2, 4));
        // the last tool has no next one
        assert_eq!(locate(&s, Some(4), true).next, None);
        // with no tool said, the first that is not finished is the current one
        assert_eq!(locate(&s, None, true).current.map(|c| c.tool), Some(2));
        // a tool that the program does not have falls back to the first that is not finished
        assert_eq!(locate(&s, Some(9), true).current.map(|c| c.tool), Some(2));
        // the number of a drill is not the number of a cutter
        assert_eq!(locate(&s, Some(2), false).current.map(|c| c.tool), Some(2));
    }

    #[test]
    fn nothing_is_said_about_a_job_without_steps_and_a_finished_job_has_no_current_tool() {
        let w = locate(&[], Some(1), true);
        assert_eq!(w.current, None);
        assert_eq!((w.tools_done, w.tools_total), (0, 0));
        let s = tool_stats(&[hole(1, 0.8, true)], &[]);
        let w = locate(&s, None, true);
        assert_eq!(w.current, None);
        assert_eq!((w.tools_done, w.tools_total), (1, 1));
    }

    #[test]
    fn the_time_left_follows_the_time_spent_and_waits_for_enough_work_done() {
        assert_eq!(seconds_left(60.0, 0.5), Some(60));
        assert_eq!(seconds_left(30.0, 0.25), Some(90));
        assert_eq!(seconds_left(10.0, 0.0), None);
        assert_eq!(seconds_left(10.0, 0.01), None);
        assert_eq!(seconds_left(10.0, 1.0), None);
        assert_eq!(seconds_left(0.0, 0.5), None);
        assert_eq!(seconds_left(f64::NAN, 0.5), None);
        assert_eq!(clock(65), "1:05");
        assert_eq!(clock(3 * 3600 + 7 * 60 + 9), "3:07:09");
        assert_eq!(clock(0), "0:00");
    }

    #[test]
    fn the_hour_the_job_ends_follows_the_time_left_and_the_offset_of_the_clock() {
        // 1970-01-01 10:00 universal time, an hour and a half left
        let ten = 10 * 3600;
        assert_eq!(end_clock(ten, 5400, 0), "11:30");
        // a clock two hours ahead, and one that is behind midnight
        assert_eq!(end_clock(ten, 5400, 120), "13:30");
        assert_eq!(end_clock(ten, 0, -11 * 60), "23:00");
        // past midnight it starts again
        // a setting wins over the system; without one the zone of the system is used (a sane value, whatever the machine)
        assert_eq!(clock_offset_min(Some(90), ten), 90);
        assert!((-14 * 60..=14 * 60).contains(&clock_offset_min(None, ten)));
        assert_eq!(end_clock(23 * 3600 + 50 * 60, 20 * 60, 0), "00:10");
    }

    #[test]
    fn the_length_cut_counts_the_segments_whose_end_has_been_reached() {
        let line = |reached| Path {
            tool: 1,
            points: vec![(0.0, 0.0), (3.0, 4.0), (3.0, 10.0)],
            reached,
        };
        assert_eq!(route_lengths(&[line(0)]), (0.0, 11.0));
        assert_eq!(route_lengths(&[line(1)]), (0.0, 11.0));
        assert_eq!(route_lengths(&[line(2)]), (5.0, 11.0));
        assert_eq!(route_lengths(&[line(3)]), (11.0, 11.0));
        // more points reached than the path has cannot cut more than it has
        assert_eq!(route_lengths(&[line(9)]), (11.0, 11.0));
        assert_eq!(route_lengths(&[line(3), line(0)]), (11.0, 22.0));
        assert_eq!(route_lengths(&[]), (0.0, 0.0));
    }

    #[test]
    fn the_tool_changes_are_the_tools_begun_but_the_first_out_of_the_tools_but_the_first() {
        let holes = [hole(1, 0.8, true), hole(2, 1.0, true), hole(3, 1.2, false)];
        let s = tool_stats(&holes, &[]);
        assert_eq!(tool_changes(&s), (1, 2));
        assert_eq!(tool_changes(&[]), (0, 0));
        let none = tool_stats(&[hole(1, 0.8, false)], &[]);
        assert_eq!(tool_changes(&none), (0, 0));
    }

    #[test]
    fn the_distance_to_a_segment_is_to_its_nearest_point() {
        let (a, b) = ((0.0, 0.0), (10.0, 0.0));
        assert_eq!(point_segment_dist((5.0, 3.0), a, b), 3.0);
        assert_eq!(point_segment_dist((13.0, 4.0), a, b), 5.0);
        assert_eq!(point_segment_dist((-3.0, 4.0), a, b), 5.0);
        // a segment that is a point
        assert_eq!(point_segment_dist((3.0, 4.0), a, a), 5.0);
    }

    fn square() -> Path {
        Path {
            tool: 1,
            points: vec![
                (0.0, 0.0),
                (10.0, 0.0),
                (10.0, 10.0),
                (0.0, 10.0),
                (0.0, 0.0),
            ],
            reached: 0,
        }
    }

    #[test]
    fn a_path_is_marked_as_the_cutting_tool_goes_along_it_and_a_closed_one_is_not_done_at_its_start(
    ) {
        let mut q = square();
        // the tool plunges at the start: the first point is reached, and a closed path is not finished by it
        advance_reached(&mut q, (0.0, 0.0), 0.3);
        assert_eq!(q.reached, 1);
        advance_reached(&mut q, (5.0, 0.1), 0.3);
        assert_eq!(q.reached, 1);
        advance_reached(&mut q, (10.0, 0.0), 0.3);
        assert_eq!(q.reached, 2);
        advance_reached(&mut q, (10.0, 6.0), 0.3);
        assert_eq!(q.reached, 2);
        advance_reached(&mut q, (10.0, 10.0), 0.3);
        assert_eq!(q.reached, 3);
        // the same position again marks nothing new, and the search goes on from the segment the path is at
        advance_reached(&mut q, (10.0, 10.0), 0.3);
        assert_eq!(q.reached, 3);
        advance_reached(&mut q, (0.0, 5.0), 0.3);
        assert_eq!(q.reached, 4);
        advance_reached(&mut q, (0.0, 0.0), 0.3);
        assert_eq!(q.reached, 5);
        // done: nothing changes
        advance_reached(&mut q, (10.0, 0.0), 0.3);
        assert_eq!(q.reached, 5);
    }

    #[test]
    fn a_tool_that_is_not_near_a_path_marks_nothing_and_one_that_skipped_a_stretch_catches_up() {
        let mut q = square();
        advance_reached(&mut q, (50.0, 50.0), 0.3);
        assert_eq!(q.reached, 0);
        // the tool is on the first path from the start and then on the third side: the first two are passed
        advance_reached(&mut q, (0.0, 0.0), 0.3);
        advance_reached(&mut q, (5.0, 10.0), 0.3);
        assert_eq!(q.reached, 3);
        // a path of one point is reached when the tool is at it
        let mut dot = Path {
            tool: 2,
            points: vec![(4.0, 4.0)],
            reached: 0,
        };
        advance_reached(&mut dot, (4.1, 4.0), 0.3);
        assert_eq!(dot.reached, 1);
        let mut empty = Path {
            tool: 2,
            points: vec![],
            reached: 0,
        };
        advance_reached(&mut empty, (0.0, 0.0), 0.3);
        assert_eq!(empty.reached, 0);
    }

    #[test]
    fn the_build_names_the_commit_of_the_program_and_of_each_sibling() {
        let text = env!("EH3D_BUILD_COMMITS");
        let parts: Vec<&str> = text.split(" / ").collect();
        let names: Vec<&str> = parts
            .iter()
            .map(|p| p.split(' ').next().unwrap_or(""))
            .collect();
        assert_eq!(names, ["BUNGARD-GUI", "COMMON", "DRILLPRO", "ROUTEPRO"]);
        for p in parts {
            let commit = p.split(' ').nth(1).unwrap_or("");
            assert!(
                commit == "unknown"
                    || (commit.len() == 7 && commit.chars().all(|c| c.is_ascii_hexdigit())),
                "{p}"
            );
        }
    }
}
