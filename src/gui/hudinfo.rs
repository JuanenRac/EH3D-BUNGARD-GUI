//! The line of the bar under the views that says what the work is doing: the tool in use (its number, diameter, kind, name in the catalogue and place in the rack),
//! the steps of that tool, the next tool, the tools finished, the time left and the state. The numbers come from [`crate::jobinfo`].

use super::*;
use crate::jobinfo::{clock, locate, seconds_left, tool_stats};

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

    /// The text of the line: what is being done now and what is left. Nothing while the program has no steps.
    pub(crate) fn hud_info_line(&self) -> Option<String> {
        let stats = tool_stats(&self.drill.scene.holes, &self.route.scene.paths);
        if stats.is_empty() {
            return None;
        }
        let drilling = self.running != Some(Program::Route);
        let tool = self.running_view().and_then(|v| v.tool);
        let at = locate(&stats, tool, drilling);
        let mut parts: Vec<String> = Vec::new();
        if let Some(c) = &at.current {
            let mut now = self.hud_tool_name(c);
            if let Some((slot, _, _)) = self
                .rack_slot_tools()
                .into_iter()
                .find(|(_, p, t)| (*p == Program::Drill) == c.drill && *t == c.tool)
            {
                now.push_str(&format!(" | {} {slot}", self.tr(Text::HudRackSlot)));
            }
            parts.push(now);
            parts.push(format!(
                "{}/{} {}",
                c.done,
                c.total,
                self.tr(if c.drill {
                    Text::HudHolesLbl
                } else {
                    Text::HudSegmentsLbl
                })
            ));
        }
        if let Some(n) = &at.next {
            parts.push(format!(
                "{} {}",
                self.tr(Text::HudNext),
                self.hud_tool_name(n)
            ));
        }
        parts.push(format!(
            "{}/{} {}",
            at.tools_done,
            at.tools_total,
            self.tr(Text::HudToolsLbl)
        ));
        if let (Some(t0), Some(_)) = (self.job_started, self.running) {
            if let Some(s) = seconds_left(t0.elapsed().as_secs_f64(), self.progress()) {
                parts.push(format!("{} {}", clock(s), self.tr(Text::HudLeft)));
            }
        }
        if let Some(state) = self.hud_state() {
            parts.push(state.to_string());
        }
        Some(parts.join("   |   "))
    }
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

    #[test]
    fn before_a_job_the_line_gives_the_first_tool_its_steps_and_the_tools_to_do() {
        let a = app();
        let line = a.hud_info_line().expect("a program with holes has a line");
        // the steps are those of the first tool, not of all the holes
        let total = tool_stats(&a.drill.scene.holes, &[])[0].total;
        assert!(line.contains(&format!("0/{total} ")), "{line}");
        assert!(line.contains('T'));
        let tools = tool_stats(&a.drill.scene.holes, &a.route.scene.paths).len();
        assert!(line.contains(&format!("0/{tools} ")), "{line}");
        // nothing is running: no time left and no state
        assert!(!line.contains(a.tr(Text::HudLeft)));
    }

    #[test]
    fn the_steps_of_the_tool_follow_the_holes_that_are_done_and_the_line_names_the_next_tool() {
        let mut a = app();
        let stats = tool_stats(&a.drill.scene.holes, &[]);
        let first = stats[0].tool;
        let n = stats[0].total;
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
        let line = a.hud_info_line().unwrap();
        assert!(line.contains(&format!("2/{n}")) || n < 2, "{line}");
        if stats.len() > 1 {
            assert!(line.contains(a.tr(Text::HudNext)), "{line}");
        }
    }

    #[test]
    fn nothing_is_said_when_the_program_has_no_steps() {
        let mut a = app();
        a.drill.scene.holes.clear();
        a.route.scene.paths.clear();
        assert_eq!(a.hud_info_line(), None);
    }
}
