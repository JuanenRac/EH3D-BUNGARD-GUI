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
}
