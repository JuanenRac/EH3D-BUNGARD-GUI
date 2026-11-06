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

// ------------------------------------------------------------------------------------------------ the card under the views
// Every height of the card is decided here, in design units counted from the top of the card; what paints it (`paint_viewer`) and the tests read [`hud_layout`].

/// The height of the card of the numbers, the bar of progress, the blocks, the notices and the buttons of the job: the views take the rest of the height.
pub(super) const HUD_H: f32 = 200.0;
/// The width of each of the four blocks of text of the card.
pub(super) const HUD_BLOCK_W: f32 = 246.0;
/// The space between two blocks.
const HUD_BLOCK_GAP: f32 = 10.0;
/// Where the contents of the card begin along it (the card itself begins at 20).
const HUD_LEFT: f32 = 44.0;
/// The numbers of the head: how many, how wide each is, how far apart they start, and the top of the row with its two heights (label and value).
const HUD_VALS: usize = 6;
const HUD_VAL_W: f32 = 148.0;
const HUD_VAL_STEP: f32 = 156.0;
const HUD_VALS_Y: f32 = 8.0;
const HUD_VAL_LABEL_H: f32 = 13.0;
const HUD_VAL_H: f32 = 32.0;
/// The row of the progress: the top, the height of its label, the bar and where the percent goes.
const HUD_PROG_Y: f32 = 60.0;
const HUD_PROG_LABEL_H: f32 = 14.0;
const HUD_BAR_W: f32 = 900.0;
const HUD_BAR_H: f32 = 10.0;
const HUD_PERCENT_X: f32 = 956.0;
const HUD_PERCENT_W: f32 = 90.0;
const HUD_PERCENT_H: f32 = 20.0;
/// The note of a simulated job ends here, on the line of the label of the progress.
const HUD_NOTE_RIGHT: f32 = 1064.0;
const HUD_NOTE_W: f32 = 364.0;
/// The four blocks: the top of the row and the heights of the label, the value and the detail.
const HUD_BLOCKS_Y: f32 = 100.0;
const HUD_BLOCK_LABEL_H: f32 = 14.0;
const HUD_BLOCK_VALUE_H: f32 = 24.0;
const HUD_BLOCK_DETAIL_H: f32 = 18.0;
/// The zone of the notices in red: below the blocks, as wide as they are, with room for this many lines of this height.
const HUD_WARN_Y: f32 = 160.0;
pub(super) const HUD_WARN_LINES: usize = 2;
pub(super) const HUD_WARN_LINE_H: f32 = 16.0;
pub(super) const HUD_WARN_SIZE: f32 = 13.0;
/// The buttons of the job at the right: where they start, their sizes and the space between them.
const HUD_BTN_X: f32 = 1084.0;
const HUD_BTN_W: f32 = 150.0;
const HUD_BTN_STEP: f32 = 166.0;
const HUD_BTN_H: f32 = 76.0;
const HUD_CHIP_H: f32 = 40.0;
const HUD_BTN_GAP: f32 = 10.0;
const HUD_BTN_RIGHT: f32 = 1566.0;

/// The label, the value and the detail of one text of the card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct HudCell {
    pub label: Rect,
    pub value: Rect,
    pub detail: Rect,
}

/// Where everything in the card goes, in design units from the top left corner of the card at its left margin of 20 (x) and its top (y). The text is drawn on the bottom
/// edge of its rectangle.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct HudLayout {
    /// The numbers of the head (the label and the value of each).
    pub vals: Vec<HudCell>,
    pub progress_label: Rect,
    pub bar: Rect,
    pub percent: Rect,
    pub note: Rect,
    pub blocks: Vec<HudCell>,
    pub warn: Rect,
    /// Pause, Start and Stop, and the button with text under them.
    pub buttons: [Rect; 3],
    pub jog: Rect,
}

impl HudLayout {
    #[cfg(test)]
    /// The rectangles that must not touch each other (the parts of the cells, not the cells).
    pub(super) fn leaves(&self) -> Vec<Rect> {
        let mut out = vec![
            self.progress_label,
            self.bar,
            self.percent,
            self.note,
            self.warn,
            self.jog,
        ];
        out.extend(self.buttons);
        for c in self.vals.iter().chain(&self.blocks) {
            out.extend([c.label, c.value]);
            if c.detail.height() > 0.0 {
                out.push(c.detail);
            }
        }
        out
    }
}

/// The height of the group of the three round buttons and the button with text under them, and where it starts so that it is centred in the card.
fn hud_buttons_y() -> f32 {
    (HUD_H - (HUD_BTN_H + HUD_BTN_GAP + HUD_CHIP_H)) / 2.0
}

/// The places of everything in the card (see [`HudLayout`]).
pub(super) fn hud_layout() -> HudLayout {
    let r = |x: f32, y: f32, w: f32, h: f32| Rect::from_min_size(Pos2::new(x, y), Vec2::new(w, h));
    let vals = (0..HUD_VALS)
        .map(|i| {
            let x = HUD_LEFT + i as f32 * HUD_VAL_STEP;
            HudCell {
                label: r(x, HUD_VALS_Y, HUD_VAL_W, HUD_VAL_LABEL_H),
                value: r(x, HUD_VALS_Y + HUD_VAL_LABEL_H + 1.0, HUD_VAL_W, HUD_VAL_H),
                detail: r(x, 0.0, 0.0, 0.0),
            }
        })
        .collect();
    let blocks = (0..4)
        .map(|i| {
            let x = HUD_LEFT + i as f32 * (HUD_BLOCK_W + HUD_BLOCK_GAP);
            let value_y = HUD_BLOCKS_Y + HUD_BLOCK_LABEL_H;
            HudCell {
                label: r(x, HUD_BLOCKS_Y, HUD_BLOCK_W, HUD_BLOCK_LABEL_H),
                value: r(x, value_y, HUD_BLOCK_W, HUD_BLOCK_VALUE_H),
                detail: r(
                    x,
                    value_y + HUD_BLOCK_VALUE_H,
                    HUD_BLOCK_W,
                    HUD_BLOCK_DETAIL_H,
                ),
            }
        })
        .collect();
    let bar_y = HUD_PROG_Y + HUD_PROG_LABEL_H + 1.0 + (HUD_PERCENT_H - HUD_BAR_H) / 2.0;
    let by = hud_buttons_y();
    let button = |i: usize| {
        r(
            HUD_BTN_X + i as f32 * HUD_BTN_STEP,
            by,
            HUD_BTN_W,
            HUD_BTN_H,
        )
    };
    HudLayout {
        vals,
        progress_label: r(HUD_LEFT, HUD_PROG_Y, 300.0, HUD_PROG_LABEL_H),
        bar: r(HUD_LEFT, bar_y, HUD_BAR_W, HUD_BAR_H),
        percent: r(
            HUD_PERCENT_X,
            HUD_PROG_Y + HUD_PROG_LABEL_H + 1.0,
            HUD_PERCENT_W,
            HUD_PERCENT_H,
        ),
        note: r(
            HUD_NOTE_RIGHT - HUD_NOTE_W,
            HUD_PROG_Y,
            HUD_NOTE_W,
            HUD_PROG_LABEL_H,
        ),
        blocks,
        warn: r(
            HUD_LEFT,
            HUD_WARN_Y,
            4.0 * HUD_BLOCK_W + 3.0 * HUD_BLOCK_GAP,
            HUD_WARN_LINES as f32 * HUD_WARN_LINE_H + 2.0,
        ),
        buttons: [button(0), button(1), button(2)],
        jog: r(
            HUD_BTN_X,
            by + HUD_BTN_H + HUD_BTN_GAP,
            HUD_BTN_RIGHT - HUD_BTN_X,
            HUD_CHIP_H,
        ),
    }
}

/// A text broken into lines no wider than `max_w` by `width` (the same measure the painter uses): at the spaces, and inside a word that is wider than a line alone.
pub(super) fn wrap_to_width(text: &str, max_w: f32, width: &dyn Fn(&str) -> f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let joined = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if width(&joined) <= max_w {
            cur = joined;
            continue;
        }
        if !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
        }
        let mut piece = String::new();
        for ch in word.chars() {
            piece.push(ch);
            if width(&piece) > max_w && piece.chars().count() > 1 {
                piece.pop();
                lines.push(std::mem::take(&mut piece));
                piece.push(ch);
            }
        }
        cur = piece;
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

/// The text in at most `max_lines` lines of at most `max_w`; the last one ends in an ellipsis when there is more. The flag says that something was left out.
pub(super) fn fit_lines(
    text: &str,
    max_lines: usize,
    max_w: f32,
    width: &dyn Fn(&str) -> f32,
) -> (Vec<String>, bool) {
    let mut lines = wrap_to_width(text, max_w, width);
    let cut = lines.len() > max_lines;
    if cut {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            while last.chars().count() > 1 && width(&format!("{last}\u{2026}")) > max_w {
                last.pop();
            }
            last.push('\u{2026}');
        }
    }
    (lines, cut)
}

impl Cv<'_> {
    /// The notices of the zone `zone` (in the units of the design, an absolute place), one over the other without touching: the lines are made with the measure of the
    /// painter, the zone has room for `HUD_WARN_LINES` of them and a notice that does not fit ends in an ellipsis. With `centred` they go in the middle of the zone.
    pub(super) fn notices(&self, zone: Rect, notes: &[(String, Color32)], centred: bool) {
        if notes.is_empty() {
            return;
        }
        let font = FontId::proportional(HUD_WARN_SIZE * self.u);
        let width = |s: &str| {
            self.p
                .layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE)
                .size()
                .x
                / self.u
        };
        let each = (HUD_WARN_LINES / notes.len()).max(1);
        let mut y = zone.min.y;
        for (text, color) in notes {
            let (lines, cut) = fit_lines(text, each, zone.width(), &width);
            #[cfg(test)]
            if cut {
                CLIPPED.with(|c| {
                    c.borrow_mut()
                        .push(format!("{text} (notice, {each} lines)"))
                });
            }
            #[cfg(not(test))]
            let _ = cut;
            for line in lines {
                if y + HUD_WARN_LINE_H > zone.max.y + 1.0 {
                    return;
                }
                if centred {
                    self.text(
                        (zone.min.x + zone.max.x) / 2.0,
                        y,
                        Align2::CENTER_TOP,
                        &line,
                        HUD_WARN_SIZE,
                        *color,
                    );
                } else {
                    self.text(
                        zone.min.x,
                        y,
                        Align2::LEFT_TOP,
                        &line,
                        HUD_WARN_SIZE,
                        *color,
                    );
                }
                y += HUD_WARN_LINE_H;
            }
        }
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

    /// Whether two rectangles share some area (touching at an edge is not sharing).
    fn overlap(a: Rect, b: Rect) -> bool {
        a.min.x < b.max.x && b.min.x < a.max.x && a.min.y < b.max.y && b.min.y < a.max.y
    }

    #[test]
    fn the_places_of_the_card_do_not_overlap_fit_in_it_and_the_buttons_are_centred() {
        let lay = hud_layout();
        // the card goes from the margin of 20 to the margin of the window, and from its top to its height
        let card = Rect::from_min_size(
            Pos2::new(20.0, 0.0),
            Vec2::new(crate::panes::BOARD_W - 40.0, HUD_H),
        );
        let leaves = lay.leaves();
        assert!(leaves.len() > 20);
        for (i, a) in leaves.iter().enumerate() {
            assert!(
                a.width() > 0.0 && a.height() > 0.0,
                "empty place {i}: {a:?}"
            );
            assert!(
                a.min.x >= card.min.x && a.max.x <= card.max.x,
                "{a:?} leaves the card along it"
            );
            assert!(
                a.min.y >= card.min.y && a.max.y <= card.max.y,
                "{a:?} leaves the card in height {}",
                card.height()
            );
            for b in &leaves[i + 1..] {
                assert!(!overlap(*a, *b), "{a:?} overlaps {b:?}");
            }
        }
        // the card is back to about what it was before the blocks, and the notices are the last thing in it
        assert!(card.height() <= 200.0, "the card is {} high", card.height());
        assert!(lay.warn.max.y <= card.height());
        assert!(
            lay.warn.height() >= HUD_WARN_LINES as f32 * HUD_WARN_LINE_H,
            "the zone of the notices has room for its lines"
        );
        // the notices have the width of the four blocks and are under all of them
        let last = lay.blocks.last().unwrap();
        assert!((lay.warn.max.x - last.value.max.x).abs() < 1e-3);
        for b in &lay.blocks {
            assert!(b.detail.max.y <= lay.warn.min.y);
        }
        // Pause, Start and Stop are in one line, and they and the button under them are centred in the height of the card
        assert!(lay.buttons.iter().all(|b| b.min.y == lay.buttons[0].min.y));
        assert!(lay.jog.min.y >= lay.buttons[0].max.y);
        let (top, bottom) = (lay.buttons[0].min.y, lay.jog.max.y);
        let middle = (top + bottom) / 2.0;
        assert!(
            (middle - card.height() / 2.0).abs() <= 1.0,
            "the buttons are centred at {middle}, the card at {}",
            card.height() / 2.0
        );
        assert!((top - (card.height() - bottom)).abs() <= 1.0);
        // the buttons end where the button with text ends, and that is inside the card at its narrowest
        assert!((lay.buttons[2].max.x - lay.jog.max.x).abs() <= 1.0);
        // the bar of progress is centred on the line of its percent
        assert!((lay.bar.center().y - lay.percent.center().y).abs() <= 1.0);
    }

    #[test]
    fn the_notices_are_broken_into_lines_that_fit_with_an_ellipsis_at_the_end_of_the_last() {
        let width = |s: &str| s.chars().count() as f32 * 7.0;
        // a short one is one line, as it is
        let (lines, cut) = fit_lines("Does not fit", 2, 300.0, &width);
        assert_eq!(lines, vec!["Does not fit".to_string()]);
        assert!(!cut);
        // nothing gives no lines
        assert!(fit_lines("   ", 2, 300.0, &width).0.is_empty());
        // a long one goes to the second line at a space
        let text = "The board does not fit on the table of this machine and the job cannot start";
        let (lines, cut) = fit_lines(text, 2, 300.0, &width);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(!cut);
        assert!(lines.iter().all(|l| width(l) <= 300.0));
        assert_eq!(lines.join(" "), text);
        // a longer one is cut with an ellipsis and still fits
        let long = text.repeat(4);
        let (lines, cut) = fit_lines(&long, 2, 300.0, &width);
        assert!(cut);
        assert_eq!(lines.len(), 2);
        assert!(lines[1].ends_with('\u{2026}'));
        assert!(lines.iter().all(|l| width(l) <= 300.0), "{lines:?}");
        // a word wider than the line (a path, or a language written without spaces) is broken inside
        let (lines, cut) = fit_lines(&"a".repeat(100), 6, 140.0, &width);
        assert!(!cut);
        assert_eq!(lines.len(), 5);
        assert!(lines.iter().all(|l| width(l) <= 140.0), "{lines:?}");
        assert_eq!(lines.concat(), "a".repeat(100));
        let (lines, cut) = fit_lines(&"\u{5de5}".repeat(60), 2, 140.0, &width);
        assert!(cut);
        assert!(lines.iter().all(|l| width(l) <= 140.0), "{lines:?}");
    }

    #[test]
    fn nothing_is_said_when_the_program_has_no_steps() {
        let mut a = app();
        a.drill.scene.holes.clear();
        a.route.scene.paths.clear();
        assert_eq!(a.hud_blocks(), None);
    }
}
