//! The blocks under the bar of progress that say what the work is doing: the tool in use and its step, the times, the position and the depth, and the counters.
//! The numbers come from [`crate::jobinfo`].

use super::*;
use crate::jobinfo::{clock, clock_offset_min, end_clock, locate, seconds_left, tool_stats};

impl App {
    /// The state of the machine in a word: working with the kind of tool, holding, or waiting for a tool change.
    fn hud_state(&self) -> Option<&'static str> {
        let p = self.running?;
        if self.paused {
            return Some(self.tr(Text::Pause));
        }
        let side = if p == Program::Drill {
            &self.drill
        } else {
            &self.route
        };
        Some(match side.engine.question() {
            Some(Question::ToolChange(_)) => self.tr(Text::HudChanging),
            Some(Question::SpindleStart) => self.tr(Text::AskSpindle),
            None => match p {
                Program::Drill => self.tr(Text::Drilling),
                Program::Route => self.tr(Text::Routing),
            },
        })
    }

    /// The name of a tool for the line: its number, its diameter when the file says it, and what it is.
    fn hud_tool_name(&self, s: &crate::jobinfo::ToolStat) -> String {
        let p = if s.drill {
            Program::Drill
        } else {
            Program::Route
        };
        let mut out = format!("T{:02}", s.tool);
        if s.diameter_mm > 0.0 {
            let d = format!("{:.2}", s.diameter_mm);
            out.push_str(&format!(
                " \u{d8}{}",
                d.trim_end_matches('0').trim_end_matches('.')
            ));
        }
        out.push(' ');
        out.push_str(self.tr(if s.drill {
            Text::HudKindDrill
        } else {
            Text::HudKindCutter
        }));
        if let Some((_, _, b)) = self
            .tool_bits
            .iter()
            .find(|(q, t, _)| *q == p && *t == s.tool)
        {
            if !b.series.is_empty() {
                out.push_str(&format!(" ({})", b.series));
            }
        }
        out
    }

    /// The blocks under the bar of progress: the tool and the step, the times, the position and the depth, and the counters. Each has a small label, a value in big
    /// letters and a line of detail; lengths are in the units of the settings. Nothing while the program has no steps.
    pub(crate) fn hud_blocks(&self) -> Option<Vec<HudBlock>> {
        let stats = tool_stats(&self.drill.scene.holes, &self.route.scene.paths);
        if stats.is_empty() {
            return None;
        }
        let drilling = self.running != Some(Program::Route);
        let view = self.running_view();
        let at = locate(&stats, view.as_ref().and_then(|v| v.tool), drilling);
        let unit = self.len_unit();

        // 1. the tool and the step
        let (tool, step) = match &at.current {
            Some(c) => {
                let mut detail = format!(
                    "{} {}/{}  \u{b7}  {}",
                    self.tr(if c.drill {
                        Text::HudHoleOne
                    } else {
                        Text::HudSegmentOne
                    }),
                    (c.done + 1).min(c.total),
                    c.total,
                    self.tr(if c.drill {
                        Text::Drilling
                    } else {
                        Text::Routing
                    })
                );
                if let Some((slot, _, _)) = self
                    .rack_slot_tools()
                    .into_iter()
                    .find(|(_, p, t)| (*p == Program::Drill) == c.drill && *t == c.tool)
                {
                    detail.push_str(&format!("  \u{b7}  {} {slot}", self.tr(Text::HudRackSlot)));
                }
                (self.hud_tool_name(c), detail)
            }
            None => ("--".to_string(), String::new()),
        };
        let step = match self.hud_state() {
            Some(st) if !step.is_empty() => format!("{step}  \u{b7}  {st}"),
            _ => step,
        };

        // 2. the times
        let mut left_text = "--:--".to_string();
        let mut spent_text = String::new();
        if let (Some(t0), Some(_)) = (self.job_started, self.running) {
            let spent = t0.elapsed().as_secs_f64();
            spent_text = format!("{} {}", clock(spent as u64), self.tr(Text::HudElapsed));
            if let Some(s) = seconds_left(spent, self.progress()) {
                left_text = format!("{} {}", clock(s), self.tr(Text::HudLeft));
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                spent_text.push_str(&format!(
                    "  \u{b7}  {} {}",
                    end_clock(
                        now,
                        s,
                        clock_offset_min(self.settings.clock_utc_offset_min, now)
                    ),
                    self.tr(Text::HudEndsAt)
                ));
            }
        }

        // 3. the position and the depth
        let head = self.head();
        let num = |v: Option<f64>| v.map_or("--".to_string(), |x| self.len_text(x));
        let place = format!(
            "X {}  Y {}  Z {} {unit}",
            num(head.map(|h| h.0)),
            num(head.map(|h| h.1)),
            num(self.running.map(|_| self.head_lift_mm()))
        );
        let deep = format!(
            "{} {} {unit}  \u{b7}  {} {}  \u{b7}  {} rpm",
            self.tr(Text::HudDepth),
            num(view.as_ref().map(|v| v.stroke_mm)),
            self.tr(Text::HudFeed),
            view.as_ref()
                .map_or("--".to_string(), |v| v.z_feed.to_string()),
            view.as_ref()
                .map_or("--".to_string(), |v| v.rpm.to_string())
        );

        // 4. the counters
        let (hd, ht) = stats
            .iter()
            .filter(|s| s.drill)
            .fold((0, 0), |(d, t), s| (d + s.done, t + s.total));
        let (cut, all) = crate::jobinfo::route_lengths(&self.route.scene.paths);
        let (made, changes) = crate::jobinfo::tool_changes(&stats);
        let holes = if ht > 0 {
            format!("{hd}/{ht} {}", self.tr(Text::HudHolesLbl))
        } else {
            "--".to_string()
        };
        let routed = if all > 0.0 {
            format!(
                "{} {}/{} {unit}  \u{b7}  ",
                self.tr(Text::Routing),
                self.len_text(cut),
                self.len_text(all)
            )
        } else {
            String::new()
        };
        let counts = format!("{routed}{made}/{changes} {}", self.tr(Text::HudChanges));
        Some(vec![
            HudBlock {
                label: self.tr(Text::HudBlkTool),
                value: tool,
                detail: step,
            },
            HudBlock {
                label: self.tr(Text::HudBlkTime),
                value: left_text,
                detail: spent_text,
            },
            HudBlock {
                label: self.tr(Text::HudBlkPos),
                value: place,
                detail: deep,
            },
            HudBlock {
                label: self.tr(Text::HudBlkCount),
                value: holes,
                detail: counts,
            },
        ])
    }
}

/// One block of the area under the bar of progress.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct HudBlock {
    /// The small label, the value in big letters and the line of detail under it.
    pub label: &'static str,
    pub value: String,
    pub detail: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn app() -> App {
        let mut a = App::new(&Options {
            data: Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")),
            screen: Some(Screen::Viewer),
            ..Default::default()
        });
        a.open_path(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("05_small_board.ncd"),
        );
        a
    }

    fn app_with_both() -> App {
        let mut a = app();
        a.open_path(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("05_board_outline.plt"),
        );
        a
    }

    #[test]
    fn before_a_job_the_blocks_give_the_first_tool_its_step_and_the_counters() {
        let a = app();
        let blocks = a.hud_blocks().expect("a program with holes has blocks");
        assert_eq!(blocks.len(), 4);
        // each block has its own label, in the language of the window
        let labels: Vec<&str> = blocks.iter().map(|b| b.label).collect();
        assert_eq!(
            labels,
            vec![
                a.tr(Text::HudBlkTool),
                a.tr(Text::HudBlkTime),
                a.tr(Text::HudBlkPos),
                a.tr(Text::HudBlkCount)
            ]
        );
        // the steps are those of the first tool, not of all the holes
        let stats = tool_stats(&a.drill.scene.holes, &[]);
        assert!(blocks[0].value.starts_with('T'), "{:?}", blocks[0]);
        assert!(
            blocks[0].detail.contains(&format!("1/{}", stats[0].total)),
            "{:?}",
            blocks[0]
        );
        // the holes done in all and the tool changes
        let holes: usize = stats.iter().map(|s| s.total).sum();
        assert!(
            blocks[3].value.starts_with(&format!("0/{holes} ")),
            "{:?}",
            blocks[3]
        );
        assert!(
            blocks[3]
                .detail
                .contains(&format!("0/{} ", stats.len() - 1)),
            "{:?}",
            blocks[3]
        );
        // nothing runs: no time left
        assert_eq!(blocks[1].value, "--:--");
        assert!(blocks[1].detail.is_empty());
    }

    #[test]
    fn the_blocks_follow_the_holes_that_are_done_and_the_lengths_use_the_units_of_the_settings() {
        let mut a = app_with_both();
        let stats = tool_stats(&a.drill.scene.holes, &[]);
        let first = stats[0].tool;
        for h in a
            .drill
            .scene
            .holes
            .iter_mut()
            .filter(|h| h.tool == first)
            .take(2)
        {
            h.done = true;
        }
        let blocks = a.hud_blocks().unwrap();
        let total: usize = stats.iter().map(|s| s.total).sum();
        let done = a.drill.scene.holes.iter().filter(|h| h.done).count();
        assert!(
            blocks[3].value.starts_with(&format!("{done}/{total} ")),
            "{:?}",
            blocks[3]
        );
        // the routing length is told in the unit of the settings
        assert!(blocks[3].detail.contains(" mm"), "{:?}", blocks[3]);
        a.settings.display_mil = true;
        let blocks = a.hud_blocks().unwrap();
        assert!(blocks[3].detail.contains(" mil"), "{:?}", blocks[3]);
        assert!(blocks[2].value.contains(" mil"), "{:?}", blocks[2]);
        assert!(!blocks[3].detail.contains(" mm"), "{:?}", blocks[3]);
    }

    #[test]
    fn while_a_job_runs_the_blocks_give_the_times_the_position_and_the_depth() {
        let mut a = app();
        a.auto_ok = true;
        a.fixed_dt = Some(0.05);
        a.act(Act::Start);
        for _ in 0..400_000 {
            a.tick();
            if a.drill.engine.done() > 0 {
                break;
            }
        }
        assert!(a.running.is_some());
        // the time spent takes a moment of the clock of the computer to show
        a.job_started = Some(std::time::Instant::now() - std::time::Duration::from_secs(30));
        let blocks = a.hud_blocks().unwrap();
        assert!(
            blocks[1].detail.contains(a.tr(Text::HudElapsed)),
            "{:?}",
            blocks[1]
        );
        // the position has the three axes and the detail has the depth, the feed and the speed of the spindle
        assert!(blocks[2].value.starts_with("X "), "{:?}", blocks[2]);
        assert!(
            blocks[2].value.contains(" Y ") && blocks[2].value.contains(" Z "),
            "{:?}",
            blocks[2]
        );
        assert!(
            blocks[2].detail.contains(a.tr(Text::HudDepth)) && blocks[2].detail.contains(" rpm"),
            "{:?}",
            blocks[2]
        );
        // and the step is the one of a hole
        assert!(
            blocks[0].detail.contains(a.tr(Text::HudHoleOne)),
            "{:?}",
            blocks[0]
        );
    }

    #[test]
    fn nothing_is_said_when_the_program_has_no_steps() {
        let mut a = app();
        a.drill.scene.holes.clear();
        a.route.scene.paths.clear();
        assert_eq!(a.hud_blocks(), None);
    }
}
