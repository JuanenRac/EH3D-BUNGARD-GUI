//! The tool bar of the ATC models: the bar at the end of the machine where the tools wait, drawn in the plan view and in the 3D view, and
//! the window where the places of its first and its last tool are set. The places between them are spaced evenly; they are saved in the
//! machine file with the other places of the changer.
//!
//! What is drawn: the bar, its places in a line with their numbers, the tools in them (the colour of their size in the tool table and the
//! number), the tool in the head, the one the head goes to take next and the free place where the one it holds is left.

use super::*;
use eh3d_bungard_formats::config::{rack_ends, rack_line, AtcSlot, RackEnds, RackSetup};

/// The bar as it is now: the places of the machine file, or the ones the model starts from when the file has none yet (`proposal`).
pub(crate) struct BarView {
    pub slots: Vec<AtcSlot>,
    pub proposal: bool,
    pub setup: RackSetup,
}

impl BarView {
    /// The first and the last tool of the bar and how many tools it holds.
    pub fn ends(&self, fallback: RackEnds) -> RackEnds {
        rack_ends(&self.slots).unwrap_or(fallback)
    }
}

/// Everything the drawing needs: the bar, the tool of each place, and who is where.
pub(crate) struct BarDraw {
    pub view: BarView,
    /// For each place with a tool in it: the place, the tool, and the colour of its size.
    pub tools: Vec<(u8, u8, usize)>,
    /// The tool in the head (its number and the colour of its size); none while the head holds the test tool.
    pub head: Option<(u8, usize)>,
    /// The place of the tool the head takes next and the free place where the one it holds is left.
    pub take: Option<u8>,
    pub leave: Option<u8>,
}

/// The rectangle of the bar in mm (x0, y0, x1, y1): the places with room round them.
pub(crate) fn bar_bounds(slots: &[AtcSlot]) -> Option<(f64, f64, f64, f64)> {
    if slots.is_empty() {
        return None;
    }
    let pitch = bar_pitch(slots);
    let fold = |f: fn(f64, f64) -> f64, init: f64, g: fn(&AtcSlot) -> f64| {
        slots.iter().map(g).fold(init, f)
    };
    Some((
        fold(f64::min, f64::MAX, |s| s.x_mm) - 13.0,
        fold(f64::min, f64::MAX, |s| s.y_mm) - pitch * 0.6,
        fold(f64::max, f64::MIN, |s| s.x_mm) + 13.0,
        fold(f64::max, f64::MIN, |s| s.y_mm) + pitch * 0.6,
    ))
}

/// Whether a point of the table (mm) is at the bar: in the rectangle of its places with 20 mm round it. The head is there while it takes or leaves a tool.
pub(crate) fn at_bar(slots: &[AtcSlot], p: (f64, f64)) -> bool {
    bar_bounds(slots).is_some_and(|(x0, y0, x1, y1)| {
        p.0 >= x0 - 20.0 && p.0 <= x1 + 20.0 && p.1 >= y0 - 20.0 && p.1 <= y1 + 20.0
    })
}

/// The distance between two neighbouring places, in mm (20 when the bar has one place only).
pub(crate) fn bar_pitch(slots: &[AtcSlot]) -> f64 {
    let mut s: Vec<&AtcSlot> = slots.iter().collect();
    s.sort_by_key(|a| a.index);
    s.windows(2)
        .map(|w| (w[1].y_mm - w[0].y_mm).hypot(w[1].x_mm - w[0].x_mm))
        .filter(|d| *d > 0.5)
        .fold(f64::MAX, f64::min)
        .clamp(4.0, 40.0)
}

fn ellipse(c: (f32, f32), rx: f32, ry: f32) -> Vec<(f32, f32)> {
    (0..=20)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 20.0;
            (c.0 + rx * a.cos(), c.1 + ry * a.sin())
        })
        .collect()
}

/// A cylinder standing on the table in the 3D view, as the eye sees it: the front half of its foot, the back half of its top, a stripe of light on one side and the top face. `r` is in
/// millimetres; `bottom` and `top` are the centres of its two ellipses on the screen.
#[allow(clippy::too_many_arguments)]
fn cylinder(
    cv: &mut Cv,
    bottom: (f32, f32),
    top: (f32, f32),
    r_px: f32,
    fill: Color32,
    edge: Color32,
    lit: Color32,
) {
    let (rx, ry) = (r_px * 1.15, r_px * 0.62);
    let half = |c: (f32, f32), from: f32| -> Vec<(f32, f32)> {
        (0..=8)
            .map(|i| {
                let a = from + i as f32 * std::f32::consts::PI / 8.0;
                (c.0 + rx * a.cos(), c.1 + ry * a.sin())
            })
            .collect()
    };
    // the front half of the foot (right to left) and the back half of the top (left to right)
    let mut body = half(bottom, 0.0);
    body.extend(half(top, std::f32::consts::PI));
    cv.poly(&body, fill, Stroke::new(1.0 * cv.u, edge));
    // the light on the left side makes it round
    let x = -0.5 * rx;
    cv.polyline(
        &[
            (top.0 + x, top.1 + ry * 0.85),
            (bottom.0 + x, bottom.1 + ry * 0.85),
        ],
        1.4,
        lit,
        false,
    );
    cv.poly(&ellipse(top, rx, ry), lit, Stroke::new(1.0 * cv.u, edge));
}

/// What is in each place of the bar: `loaded` is the tool that belongs in each place (the place, the program and the tool), `held` the tool the head holds now (if any).
/// The place of the tool the head holds is empty; when the head leaves it, its place has it again. The plan view and the 3D view read this one state.
pub(crate) fn rack_occupancy(
    loaded: &[(u8, Program, u8)],
    held: Option<(Program, u8)>,
) -> Vec<(u8, Option<u8>)> {
    loaded
        .iter()
        .map(|&(slot, p, t)| (slot, if held == Some((p, t)) { None } else { Some(t) }))
        .collect()
}

/// How near the head has to come to the spot of a place of the bar to take a tool from it or leave one in it, in mm.
const GRIP_MM: f64 = 1.5;

/// The tool the head holds after it went in a straight line from `from` to `to`. `spots` are the places of the bar with the tool that belongs in each,
/// `wanted` is the tool the job works with (or is about to). The head leaves the tool it holds at its own place when the job wants another one, and
/// takes the tool the job wants at the place that has it; it does neither by passing near a place of another tool.
pub(crate) fn carried_after(
    held: Option<(Program, u8)>,
    spots: &[(f64, f64, Program, u8)],
    from: (f64, f64),
    to: (f64, f64),
    wanted: Option<(Program, u8)>,
) -> Option<(Program, u8)> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let len2 = dx * dx + dy * dy;
    // the places the segment passes within reach of, in the order the head reaches them
    let mut near: Vec<(f64, (Program, u8))> = spots
        .iter()
        .filter_map(|&(x, y, p, t)| {
            let k = if len2 > 1e-12 {
                (((x - from.0) * dx + (y - from.1) * dy) / len2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            let d = (from.0 + k * dx - x).hypot(from.1 + k * dy - y);
            (d <= GRIP_MM).then_some((k, (p, t)))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut held = held;
    for (_, place) in near {
        if held == Some(place) && wanted != held {
            held = None;
        } else if held.is_none() && wanted == Some(place) {
            held = Some(place);
        }
    }
    held
}

impl App {
    /// Follows the head along the bar: which tool it holds now, from the places it went to (not from the tool the job names, which changes before the head
    /// gets to the bar). The plan view and the 3D view draw the bar from this.
    pub(crate) fn carry_follow(&mut self) {
        let (Some(p), Some(h)) = (self.running, self.running_head_mm()) else {
            self.rack_carried = None;
            self.rack_head_prev = None;
            return;
        };
        let here = (h.0, h.1);
        let from = self.rack_head_prev.replace(here).unwrap_or(here);
        let Some(view) = self.bar_view() else {
            self.rack_carried = None;
            return;
        };
        let wanted = self.running_view().and_then(|v| v.tool).map(|t| (p, t));
        let spots: Vec<(f64, f64, Program, u8)> = self
            .rack_slot_tools()
            .iter()
            .filter_map(|&(i, lp, lt)| {
                let s = view.slots.iter().find(|s| s.index == i)?;
                Some((s.x_mm, s.y_mm, lp, lt))
            })
            .collect();
        self.rack_carried = carried_after(self.rack_carried, &spots, from, here, wanted);
    }

    /// The machine in the settings has a tool changer (an ATC model) and so a bar.
    pub(crate) fn bar_shown(&self) -> bool {
        self.has_changer() && self.layers[2]
    }

    /// The profile of the machine in use.
    fn bar_profile(&self) -> &'static machines::Profile {
        machines::models()[self.model().min(machines::models().len() - 1)].profile()
    }

    /// Where the bar starts from for this machine: the model's own first and last tool, with the depth inside the travel of the file.
    pub(crate) fn bar_default(&self) -> RackEnds {
        let (mut a, mut b, n) = self.bar_profile().default_bar();
        if let Some(s) = self.drill.engine.settings() {
            let z = a.2.min(s.z_max_depth_mm);
            a.2 = z;
            b.2 = z;
        }
        (a, b, n)
    }

    /// The bar now: the places of the machine file, or those of the model while the file has none.
    pub(crate) fn bar_view(&self) -> Option<BarView> {
        if !self.has_changer() {
            return None;
        }
        let (atc, setup) = self
            .drill
            .engine
            .settings()
            .map_or((Vec::new(), RackSetup::default()), |s| {
                (s.atc.clone(), s.rack)
            });
        if atc.is_empty() {
            let (a, b, n) = self.bar_default();
            return Some(BarView {
                slots: rack_line(a, b, n),
                proposal: true,
                setup,
            });
        }
        Some(BarView {
            slots: atc,
            proposal: false,
            setup,
        })
    }

    /// How far the bar may be: past the table along X by what the machine file allows, across the table, and down the travel of Z.
    pub(crate) fn bar_reach(&self) -> (f64, f64, f64) {
        match self.drill.engine.settings() {
            Some(s) => (
                s.clip_mm.0 + s.rack.reach_x_mm,
                s.clip_mm.1,
                s.z_max_depth_mm,
            ),
            None => (
                self.table.width_mm + RackSetup::default().reach_x_mm,
                self.table.depth_mm,
                self.bar_profile().work_mm.2,
            ),
        }
    }

    /// The first place of the bar that is not inside what the machine reaches, if there is one.
    pub(crate) fn bar_outside(&self) -> Option<u8> {
        let v = self.bar_view()?;
        let (rx, ry, rz) = self.bar_reach();
        v.slots
            .iter()
            .find(|s| {
                !(0.0..=rx).contains(&s.x_mm)
                    || !(0.0..=ry).contains(&s.y_mm)
                    || !(0.0..=rz).contains(&s.z_mm)
            })
            .map(|s| s.index)
    }

    /// The bar with the tools in it and the three roles of the changer at this moment.
    pub(crate) fn bar_draw(&self) -> Option<BarDraw> {
        let view = self.bar_view()?;
        let lines = self.tool_lines();
        let color = |p: Program, t: u8| {
            lines
                .iter()
                .find(|l| l.prog == p && l.tool == t)
                .map_or(0, |l| l.color)
        };
        let loaded = self.rack_slot_tools();
        // the tool of the job that is working now, if a job is running
        let working = self
            .running
            .and_then(|p| self.running_view().and_then(|v| v.tool).map(|t| (p, t)));
        let at = working.and_then(|(p, t)| {
            loaded
                .iter()
                .find(|(_, lp, lt)| *lp == p && *lt == t)
                .map(|(s, _, _)| *s)
        });
        let count = loaded.len() as u8;
        let (leave, take) = match at {
            // the tool in the head goes back to its own place, and the next one of the load is taken (the test tool after the last)
            Some(k) => (Some(k), Some(if k < count { k + 1 } else { 0 })),
            None if count > 0 => (Some(0), Some(1)),
            None => (None, None),
        };
        // while a job runs the two tags are shown during the change only: when the head is at the bar
        let (leave, take) = match (self.running, self.running_head_mm()) {
            (Some(_), Some(h)) if !at_bar(&view.slots, (h.0, h.1)) => (None, None),
            _ => (leave, take),
        };
        // what is in each place comes from the tool in the head alone: the one the head holds is not in its place, and every other is
        let held = self.rack_carried.filter(|_| self.running.is_some());
        let tools = rack_occupancy(&loaded, held)
            .into_iter()
            .filter_map(|(s, t)| {
                let t = t?;
                let p = loaded
                    .iter()
                    .find(|(i, _, _)| *i == s)
                    .map(|(_, p, _)| *p)?;
                Some((s, t, color(p, t)))
            })
            .collect();
        let head = held.map(|(p, t)| (t, color(p, t)));
        Some(BarDraw {
            view,
            tools,
            head,
            take,
            leave,
        })
    }

    /// The bar fitted into the plan view with the table: the bounds in mm of what has to be in view, and the room of the tags to its right.
    pub(crate) fn bar_fit(&self, size: (f64, f64)) -> Option<eh3d_view::scene::View2D> {
        if !self.bar_shown() {
            return None;
        }
        let v = self.bar_view()?;
        let b = bar_bounds(&v.slots)?;
        Some(machines::fit_plan(
            &self.table,
            b,
            size,
            (10.0, 96.0, 10.0, 10.0),
        ))
    }

    // ------------------------------------------------------------------------------------------ setting the places

    /// The value of a number of the window of the bar, in mm (or the number of places).
    pub(crate) fn bar_value(&self, f: Field) -> Option<f64> {
        let v = self.bar_view()?;
        let (first, last, n) = v.ends(self.bar_default());
        Some(match f {
            Field::BarFx => first.0,
            Field::BarFy => first.1,
            Field::BarFz => first.2,
            Field::BarLx => last.0,
            Field::BarLy => last.1,
            Field::BarLz => last.2,
            Field::BarN => n as f64,
            Field::BarToolLen => v.setup.tool_length_mm,
            Field::BarRing => v.setup.ring_to_tip_mm,
            Field::BarClear => v.setup.clearance_mm,
            Field::BarApproach => v.setup.approach_mm,
            _ => return None,
        })
    }

    /// Changes a number of the window of the bar by `delta` (mm, or places) and puts the places between the first and the last tool in a line
    /// again. The first and the last tool stay inside what the machine reaches.
    pub(crate) fn bar_adjust(&mut self, f: Field, delta: f64) {
        let Some(v) = self.bar_view() else {
            return;
        };
        let (mut first, mut last, mut n) = v.ends(self.bar_default());
        let mut setup = v.setup;
        match f {
            Field::BarFx => first.0 += delta,
            Field::BarFy => first.1 += delta,
            Field::BarFz => first.2 += delta,
            Field::BarLx => last.0 += delta,
            Field::BarLy => last.1 += delta,
            Field::BarLz => last.2 += delta,
            Field::BarN => n = (n as f64 + delta).round().clamp(1.0, 30.0) as usize,
            Field::BarToolLen => {
                setup.tool_length_mm = (setup.tool_length_mm + delta).clamp(5.0, 100.0)
            }
            Field::BarRing => {
                setup.ring_to_tip_mm =
                    (setup.ring_to_tip_mm + delta).clamp(0.0, setup.tool_length_mm)
            }
            Field::BarClear => setup.clearance_mm = (setup.clearance_mm + delta).clamp(0.0, 100.0),
            Field::BarApproach => {
                setup.approach_mm = (setup.approach_mm + delta).clamp(-60.0, 60.0)
            }
            _ => return,
        }
        let (rx, ry, rz) = self.bar_reach();
        let tidy = |p: (f64, f64, f64)| {
            let r = |v: f64, hi: f64| (v.clamp(0.0, hi) * 100.0).round() / 100.0;
            (r(p.0, rx), r(p.1, ry), r(p.2, rz))
        };
        let (first, last) = (tidy(first), tidy(last));
        if let Some(s) = self.drill.engine.settings_mut() {
            s.atc = rack_line(first, last, n);
            s.rack = setup;
            s.auto_change = true;
        }
        self.drill.engine.settings_changed();
        self.saved = false;
    }

    // ------------------------------------------------------------------------------------------ drawing

    /// The bar in the plan view: `screen` takes a point of the table in mm to the canvas, `scale` is the pixels of the canvas for each mm.
    pub(super) fn paint_bar_plan(
        &self,
        cv: &mut Cv,
        screen: &dyn Fn(f64, f64) -> (f32, f32),
        scale: f64,
    ) {
        if !self.bar_shown() {
            return;
        }
        let Some(d) = self.bar_draw() else {
            return;
        };
        let Some((x0, y0, x1, y1)) = bar_bounds(&d.view.slots) else {
            return;
        };
        let t = cv.t.clone();
        let u = cv.u;
        let (ax, ay) = screen(x0, y1);
        let (bx, by) = screen(x1, y0);
        let body = cv.rect(ax, ay, bx - ax, by - ay);
        let base = col(t.panel_raised);
        cv.raised(
            body,
            6.0 * u,
            if d.view.proposal {
                alpha(base, 150)
            } else {
                base
            },
            false,
            false,
        );
        if d.view.proposal {
            // the places are the ones the model starts from, not the ones of the machine file
            let r = [
                body.left_top(),
                body.right_top(),
                body.right_bottom(),
                body.left_bottom(),
                body.left_top(),
            ];
            cv.p.extend(Shape::dashed_line(
                &r,
                Stroke::new(1.4 * u, col(t.accent2)),
                6.0 * u,
                4.0 * u,
            ));
        }
        let mut order: Vec<&AtcSlot> = d.view.slots.iter().collect();
        order.sort_by_key(|s| s.index);
        // the groove the tools stand in
        let groove: Vec<(f32, f32)> = order.iter().map(|s| screen(s.x_mm, s.y_mm)).collect();
        cv.polyline(&groove, 5.0, alpha(Color32::BLACK, 70), false);
        let pitch = bar_pitch(&d.view.slots);
        let r = ((pitch * 0.36 * scale) as f32).clamp(4.0, 13.0);
        let last_index = order.last().map_or(0, |s| s.index);
        for s in &order {
            let (px, py) = screen(s.x_mm, s.y_mm);
            let c = cv.pt(px, py);
            let ring = alpha(col(t.line), 220);
            if s.index == 0 {
                // the place of the test tool
                cv.p.rect_filled(
                    Rect::from_center_size(c, Vec2::splat(2.0 * r * u)),
                    2.0 * u,
                    alpha(Color32::BLACK, 110),
                );
                cv.p.rect_stroke(
                    Rect::from_center_size(c, Vec2::splat(2.0 * r * u)),
                    2.0 * u,
                    Stroke::new(1.4 * u, col(t.accent2)),
                    StrokeKind::Inside,
                );
            } else {
                cv.p.circle_filled(c, r * u, alpha(Color32::BLACK, 110));
                let edge = if s.index == last_index && order.len() > 2 {
                    // the drill break detector
                    Color32::from_rgb(90, 200, 190)
                } else {
                    ring
                };
                cv.p.circle_stroke(c, r * u, Stroke::new(1.4 * u, edge));
            }
            if let Some((_, tool, color)) = d.tools.iter().find(|(i, _, _)| *i == s.index) {
                cv.p.circle_filled(c, (r - 1.8) * u, col(t.size_color(*color)));
                cv.p.circle_filled(
                    c + Vec2::new(-0.25, -0.3) * r * u,
                    0.35 * r * u,
                    alpha(Color32::WHITE, 70),
                );
                cv.text(
                    px,
                    py + r * 0.38,
                    Align2::CENTER_BOTTOM,
                    &tool.to_string(),
                    (r * 1.05).clamp(8.0, 12.0),
                    Color32::BLACK,
                );
            }
            if pitch * scale >= 9.0 {
                // the number of the place, outside the bar on the side of the table
                cv.text(
                    ax - 3.0,
                    py + 4.0,
                    Align2::RIGHT_BOTTOM,
                    &s.index.to_string(),
                    9.0,
                    col(t.dim),
                );
            }
        }
        // the tool the head goes to take, and the free place where it leaves the one it holds
        let tag = |cv: &mut Cv, slot: u8, word: Text, color: Color32, dashed: bool| {
            let Some(s) = order.iter().find(|s| s.index == slot) else {
                return;
            };
            let (px, py) = screen(s.x_mm, s.y_mm);
            let ring = ellipse((px, py), r + 3.0, r + 3.0);
            cv.polyline(&ring, 2.2, color, dashed);
            let name = d
                .tools
                .iter()
                .find(|(i, _, _)| *i == slot)
                .map(|(_, t, _)| format!(" T{t:02}"))
                .unwrap_or_default();
            cv.text(
                bx + 6.0,
                py + 4.0,
                Align2::LEFT_BOTTOM,
                &format!("{}{name}", self.tr(word)),
                11.0,
                color,
            );
        };
        if let Some(s) = d.take {
            tag(cv, s, Text::RackToTake, col(t.ok), false);
        }
        if let Some(s) = d.leave {
            tag(cv, s, Text::RackToLeave, col(t.warn), true);
        }
        // the tool in the head, beside the head
        if let (Some((tool, color)), Some((hx, hy))) = (d.head, self.head()) {
            let (px, py) = screen(hx, hy);
            let c = cv.pt(px + 24.0, py - 24.0);
            cv.p.circle_filled(c, 9.0 * u, col(t.size_color(color)));
            cv.p.circle_stroke(c, 9.0 * u, Stroke::new(1.6 * u, col(t.ok)));
            cv.text(
                px + 24.0,
                py - 20.0,
                Align2::CENTER_BOTTOM,
                &tool.to_string(),
                11.0,
                Color32::BLACK,
            );
            cv.text(
                px + 38.0,
                py - 20.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::RackInHead),
                11.0,
                col(t.ok),
            );
        }
    }

    /// The bar in the 3D view: the bar in relief at the short edge, the tools standing in it, and the three roles marked.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn paint_bar_3d(
        &self,
        cv: &mut Cv,
        iso: &Iso,
        o: (f32, f32),
        shift: (f64, f64),
        edge: Color32,
        board_z: f64,
        table_depth: f64,
        tip: (f32, f32),
    ) {
        if !self.bar_shown() {
            return;
        }
        let Some(d) = self.bar_draw() else {
            return;
        };
        let Some((x0, y0, x1, y1)) = bar_bounds(&d.view.slots) else {
            return;
        };
        let t = cv.t.clone();
        let p = |x: f64, y: f64, z: f64| {
            let (px, py) = iso.project(x + shift.0, y + shift.1, z);
            (o.0 + px as f32, o.1 + py as f32)
        };
        let alu = Color32::from_rgb(196, 200, 206);
        let top = board_z + 10.0;
        // the bar: a body in relief with the places cut in its top
        draw_boxes(
            cv,
            iso,
            o,
            shift,
            &[(x0, table_depth - y1, board_z, x1 - x0, y1 - y0, 10.0, alu)],
            edge,
        );
        let setup = d.view.setup;
        let shank = (setup.tool_length_mm - setup.ring_to_tip_mm).max(4.0);
        let mut order: Vec<&AtcSlot> = d.view.slots.iter().collect();
        order.sort_by_key(|s| s.index);
        // far places first, so that the near tools are in front
        order.sort_by(|a, b| a.y_mm.total_cmp(&b.y_mm).reverse());
        let pitch = bar_pitch(&d.view.slots);
        let rr = (pitch * 0.3).clamp(2.0, 6.0);
        for s in &order {
            let hole: Vec<(f32, f32)> = (0..10)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 10.0;
                    let (px, py) = p(s.x_mm, table_depth - s.y_mm, top + 0.2);
                    let k = (rr * iso.k) as f32;
                    (px + k * 1.15 * a.cos(), py + k * 0.62 * a.sin())
                })
                .collect();
            cv.poly(
                &hole,
                alpha(Color32::BLACK, 190),
                Stroke::new(1.6 * cv.u, alpha(Color32::WHITE, 90)),
            );
            if let Some((_, tool, color)) = d.tools.iter().find(|(i, _, _)| *i == s.index) {
                // a drill as it stands in the bar: the ring that the changer holds (in the colour of its size), the shank above it and the flat end of the shank
                let y = table_depth - s.y_mm;
                let k = iso.k as f32;
                let ring = (rr as f32 * 0.85) * k;
                let stem = (rr as f32 * 0.38) * k;
                let ring_h = (shank * 0.22).clamp(2.0, 6.0);
                let size = col(t.size_color(*color));
                let steel = Color32::from_rgb(168, 174, 182);
                cylinder(
                    cv,
                    p(s.x_mm, y, top),
                    p(s.x_mm, y, top + ring_h),
                    ring,
                    size,
                    edge,
                    alpha(Color32::WHITE, 90),
                );
                cylinder(
                    cv,
                    p(s.x_mm, y, top + ring_h),
                    p(s.x_mm, y, top + shank),
                    stem,
                    steel,
                    edge,
                    alpha(Color32::WHITE, 150),
                );
                let (px, py) = p(s.x_mm, y, top + shank + 3.0);
                cv.text(
                    px,
                    py,
                    Align2::CENTER_BOTTOM,
                    &tool.to_string(),
                    10.0,
                    col(t.text),
                );
            }
        }
        let mark = |cv: &mut Cv, slot: u8, word: Text, color: Color32, dashed: bool, dy: f32| {
            let Some(s) = order.iter().find(|s| s.index == slot) else {
                return;
            };
            let c = p(s.x_mm, table_depth - s.y_mm, top + 0.4);
            let k = ((rr + 2.5) * iso.k) as f32;
            cv.polyline(&ellipse(c, k * 1.15, k * 0.62), 2.0, color, dashed);
            // the word goes beside the bar (towards the table), above or below the other one
            let label = p(s.x_mm - 30.0, table_depth - s.y_mm, top + 2.0);
            cv.text(
                label.0 - 4.0,
                label.1 + dy,
                Align2::RIGHT_CENTER,
                self.tr(word),
                11.0,
                color,
            );
        };
        if let Some(s) = d.take {
            mark(cv, s, Text::RackToTake, col(t.ok), false, -9.0);
        }
        if let Some(s) = d.leave {
            mark(cv, s, Text::RackToLeave, col(t.warn), true, 9.0);
        }
        // the tool in the head, at the tip of the spindle
        if let Some((tool, color)) = d.head {
            cv.p.circle_filled(cv.pt(tip.0, tip.1), 7.0 * cv.u, col(t.size_color(color)));
            cv.p.circle_stroke(
                cv.pt(tip.0, tip.1),
                7.0 * cv.u,
                Stroke::new(1.6 * cv.u, col(t.ok)),
            );
            cv.text(
                tip.0 + 12.0,
                tip.1 + 4.0,
                Align2::LEFT_BOTTOM,
                &format!("{} T{tool:02}", self.tr(Text::RackInHead)),
                11.0,
                col(t.ok),
            );
        }
    }

    /// The window of the bar.
    pub(super) fn paint_bar_modal(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y) = self.modal_card(cv, self.tr(Text::RackSetupTitle), 720.0);
        let Some(view) = self.bar_view() else {
            return;
        };
        let u = self.len_unit();
        let first = self.tr(Text::RackFirst);
        let last = self.tr(Text::RackLast);
        let big = [-10.0, -1.0, 1.0, 10.0];
        let fine = [-1.0, -0.1, 0.1, 1.0];
        let rows: Vec<(String, String, Field, [f32; 4])> = vec![
            (format!("{first}  X"), Field::BarFx, big),
            (format!("{first}  Y"), Field::BarFy, big),
            (format!("{first}  Z"), Field::BarFz, fine),
            (format!("{last}  X"), Field::BarLx, big),
            (format!("{last}  Y"), Field::BarLy, big),
            (format!("{last}  Z"), Field::BarLz, fine),
            (
                {
                    let w = self.tr(Text::RackPlaces);
                    let mut c = w.chars();
                    c.next()
                        .map_or(String::new(), |f| f.to_uppercase().chain(c).collect())
                },
                Field::BarN,
                [-5.0, -1.0, 1.0, 5.0],
            ),
            (
                self.tr(Text::RackToolLength).to_string(),
                Field::BarToolLen,
                fine,
            ),
            (
                self.tr(Text::RackRingToTip).to_string(),
                Field::BarRing,
                fine,
            ),
            (
                self.tr(Text::RackClearance).to_string(),
                Field::BarClear,
                fine,
            ),
            (
                self.tr(Text::RackApproach).to_string(),
                Field::BarApproach,
                fine,
            ),
        ]
        .into_iter()
        .map(|(l, f, s)| {
            let v = self.bar_value(f).unwrap_or(0.0);
            let text = if f == Field::BarN {
                format!("{v:.0}")
            } else if self.settings.display_mil {
                format!("{:.1} {u}", self.len_to_display(v))
            } else {
                format!("{v:.2} {u}")
            };
            (l, text, f, s)
        })
        .collect();
        for (i, (label, value, f, steps)) in rows.iter().enumerate() {
            self.stepper(
                cv,
                y + 70.0 + i as f32 * 46.0,
                label,
                value,
                *f,
                *steps,
                true,
            );
        }
        let mut ny = y + 70.0 + rows.len() as f32 * 46.0 + 18.0;
        if let Some(i) = self.bar_outside() {
            cv.text(
                x + 30.0,
                ny,
                Align2::LEFT_BOTTOM,
                &format!("{}: {i}", self.tr(Text::RackOutside)),
                14.0,
                col(t.danger),
            );
            ny += 22.0;
        }
        if view.proposal {
            cv.text(
                x + 30.0,
                ny,
                Align2::LEFT_BOTTOM,
                self.tr(Text::LblNoRack),
                14.0,
                col(t.warn),
            );
            ny += 22.0;
        }
        for line in extra::wrap(self.tr(Text::RackSetupNote), 92).iter().take(3) {
            cv.text(x + 30.0, ny, Align2::LEFT_BOTTOM, line, 13.0, col(t.dim));
            ny += 18.0;
        }
        let by = y + 720.0 - 64.0;
        cv.chip(
            x + 30.0,
            by,
            200.0,
            44.0,
            self.tr(Text::Save),
            !self.saved,
            Act::SaveSettings,
        );
        cv.chip(
            x + 250.0,
            by,
            170.0,
            44.0,
            self.tr(Text::Close),
            false,
            Act::CloseModal,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bounds_and_the_pitch_of_a_line_of_places() {
        let line = rack_line((540.0, 40.0, 37.4), (540.0, 320.0, 37.4), 15);
        let (x0, y0, x1, y1) = bar_bounds(&line).unwrap();
        assert!(x0 < 540.0 && x1 > 540.0);
        assert!(y0 < line[0].y_mm && y1 > line[16].y_mm);
        assert!((bar_pitch(&line) - 20.0).abs() < 1e-6);
        assert_eq!(bar_bounds(&[]), None);
    }

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("eh3d-bg-bar-{}-{n}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        for f in [
            "DRILLPRO.DCF",
            "DRILLPRO.DTD",
            "ROUTEPRO.RCF",
            "ROUTEPRO.RTD",
            "05_small_board.ncd",
            "05_board_outline.plt",
        ] {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("testdata")
                    .join(f),
                dir.join(f),
            )
            .unwrap();
        }
        let mut a = App::new(&crate::cli::Options {
            data: Some(dir.clone()),
            screen: Some(Screen::Viewer),
            ..Default::default()
        });
        a.open_path(&dir.join("05_small_board.ncd"));
        a.open_path(&dir.join("05_board_outline.plt"));
        a
    }

    #[test]
    fn only_the_models_with_the_changer_have_a_bar_and_it_starts_from_the_model() {
        let mut a = app();
        a.settings.model = "CCD MTC".to_string();
        assert!(a.bar_view().is_none() && a.bar_fit((900.0, 500.0)).is_none() && !a.bar_shown());
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        let v = a.bar_view().unwrap();
        assert!(v.proposal, "the file has no places yet");
        assert_eq!(v.slots.len(), 17);
        assert!(a.bar_outside().is_none());
        // the layer of the machine hides it
        a.layers[2] = false;
        assert!(!a.bar_shown());
    }

    #[test]
    fn the_first_and_the_last_tool_are_set_and_the_places_between_them_follow_in_a_line() {
        let mut a = app();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        a.bar_adjust(Field::BarFy, 5.0);
        let v = a.bar_view().unwrap();
        assert!(!v.proposal, "a change makes the places the machine's");
        let (first, last, n) = v.ends(a.bar_default());
        assert_eq!(n, 15);
        let (d_first, _, _) = a.bar_default();
        assert!((first.1 - (d_first.1 + 5.0)).abs() < 0.006);
        // evenly spaced from the first to the last, the test tool before and the detector after
        let step = (last.1 - first.1) / 14.0;
        for s in &v.slots {
            let want = first.1 + step * (s.index as f64 - 1.0);
            assert!((s.y_mm - want).abs() < 1e-9, "place {}", s.index);
        }
        // fewer places, the last one moves up the line
        a.bar_adjust(Field::BarN, -5.0);
        let v = a.bar_view().unwrap();
        assert_eq!(v.slots.len(), 12);
        assert_eq!(v.ends(a.bar_default()).2, 10);
        // the numbers outside what the machine reaches stop at its edge
        a.bar_adjust(Field::BarLx, 99_999.0);
        a.bar_adjust(Field::BarFy, -99_999.0);
        let (rx, _, _) = a.bar_reach();
        let (first, last, _) = a.bar_view().unwrap().ends(a.bar_default());
        assert_eq!(last.0, rx);
        assert_eq!(first.1, 0.0);
    }

    #[test]
    fn the_bar_is_saved_in_the_machine_file_and_read_back_with_the_lengths() {
        let mut a = app();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        a.bar_adjust(Field::BarFx, 3.0);
        a.bar_adjust(Field::BarClear, 2.0);
        a.bar_adjust(Field::BarApproach, 4.0);
        let mine = a.bar_view().unwrap();
        a.save_settings();
        a.drill.engine.reload_config().unwrap();
        let back = a.drill.engine.settings().unwrap().clone();
        assert!(back.auto_change);
        assert_eq!(back.atc.len(), mine.slots.len());
        assert_eq!(back.rack, mine.setup);
        assert_eq!(back.rack.tool_length_mm, 38.0);
        assert_eq!(back.rack.ring_to_tip_mm, 21.0);
        for (x, y) in back.atc.iter().zip(&mine.slots) {
            assert!(
                (x.x_mm - y.x_mm).abs() < 0.006
                    && (x.y_mm - y.y_mm).abs() < 0.006
                    && (x.z_mm - y.z_mm).abs() < 0.006
            );
        }
        // the file of the routing program has nothing of the bar
        assert!(a.route.engine.settings().unwrap().atc.is_empty());
    }

    #[test]
    fn the_bar_is_in_view_with_the_table_and_its_tags_have_room() {
        let mut a = app();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        let size = (760.0, 420.0);
        let fit = a.bar_fit(size).unwrap();
        let v = a.bar_view().unwrap();
        let (x0, y0, x1, y1) = bar_bounds(&v.slots).unwrap();
        for (x, y) in [
            (x0, y0),
            (x1, y1),
            (0.0, 0.0),
            (a.table.width_mm, a.table.depth_mm),
        ] {
            let (px, py) = fit.to_screen(&a.table, x, y);
            assert!(px >= 9.9 && px <= size.0 - 96.0 + 0.1, "{px}");
            assert!(py >= 9.9 && py <= size.1 - 10.0 + 0.1, "{py}");
        }
    }

    #[test]
    fn the_roles_of_the_exchange_follow_the_tool_that_works() {
        let mut a = app();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        let d = a.bar_draw().unwrap();
        let loaded = a.rack_slot_tools();
        assert!(!loaded.is_empty());
        // before the job the head holds the test tool: it is left in the place 0 and the first tool is taken
        assert_eq!((d.leave, d.take, d.head), (Some(0), Some(1), None));
        assert_eq!(d.tools.len(), loaded.len());
    }

    #[test]
    fn the_point_is_at_the_bar_when_it_is_in_its_rectangle_with_a_margin() {
        let mut a = app();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        let v = a.bar_view().unwrap();
        let (x0, y0, x1, y1) = bar_bounds(&v.slots).unwrap();
        assert!(at_bar(&v.slots, ((x0 + x1) / 2.0, (y0 + y1) / 2.0)));
        assert!(at_bar(&v.slots, (x0 - 19.0, y0 - 19.0)));
        assert!(!at_bar(&v.slots, (x0 - 25.0, y0)));
        assert!(!at_bar(&v.slots, (x1 + 25.0, y1)));
        assert!(!at_bar(&[], (0.0, 0.0)));
    }

    #[test]
    fn the_simulated_head_is_the_head_of_the_plan_and_the_exchange_tags_show_while_it_is_at_the_bar(
    ) {
        let mut a = app();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        a.auto_ok = true;
        a.fixed_dt = Some(0.05);
        a.act(Act::Start);
        // the window of the rack comes first on these models
        if a.modal == Some(Modal::Rack) {
            a.act(Act::RackGo);
        }
        let slots = a.bar_view().unwrap().slots;
        let (mut at, mut away) = (0, 0);
        for _ in 0..600_000 {
            a.tick();
            if a.modal == Some(Modal::Rack) {
                a.act(Act::RackGo);
            }
            if a.running.is_none() && a.modal.is_none() {
                break;
            }
            let Some(h) = a.running_head_mm() else {
                continue;
            };
            // the marker of the plan view is the head of the steps, not the hole that is next
            assert_eq!(a.head(), Some((h.0, h.1)));
            let d = a.bar_draw().unwrap();
            if at_bar(&slots, (h.0, h.1)) {
                at += 1;
                assert!(d.take.is_some() || d.leave.is_some(), "no tag at the bar");
            } else {
                away += 1;
                assert!(
                    d.take.is_none() && d.leave.is_none(),
                    "a tag away from the bar"
                );
            }
            if at > 0 && away > 0 && a.m3d.phase() == super::super::machine3d::Phase::Working {
                break;
            }
        }
        assert!(at > 0, "the head went to the bar");
        assert!(away > 0, "the head went away from the bar");
        a.act(Act::Stop);
    }

    #[test]
    fn the_bar_and_its_window_paint_in_every_theme_and_language_on_a_model_with_the_changer() {
        let ctx = egui::Context::default();
        let mut a = app();
        a.animate = false;
        a.screen = Screen::Viewer;
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        for lang in [Lang::En, Lang::Ja, Lang::Ru] {
            a.set_lang(lang);
            for id in ThemeId::ALL {
                a.set_theme(id);
                for modal in [None, Some(Modal::Bar)] {
                    a.modal = None;
                    if let Some(m) = modal {
                        a.show_modal(m);
                    }
                    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(design_w(), DESIGN_H));
                    let mut out = ctx.run_ui(
                        egui::RawInput {
                            screen_rect: Some(rect),
                            ..Default::default()
                        },
                        |ui| {
                            let p = ui.ctx().layer_painter(egui::LayerId::background());
                            a.paint(&p, rect);
                        },
                    );
                    out.textures_delta.clear();
                }
            }
        }
    }

    #[test]
    fn a_tool_leaves_its_place_when_the_head_takes_it_and_is_back_when_it_is_left() {
        let loaded = [
            (1u8, Program::Drill, 1u8),
            (2, Program::Drill, 3),
            (3, Program::Drill, 5),
        ];
        let at = |held: Option<(Program, u8)>| rack_occupancy(&loaded, held);
        // nothing held: every place has its tool
        assert_eq!(at(None), vec![(1, Some(1)), (2, Some(3)), (3, Some(5))]);
        // T3 taken: its place is empty and the others are not
        assert_eq!(
            at(Some((Program::Drill, 3))),
            vec![(1, Some(1)), (2, None), (3, Some(5))]
        );
        // T3 left: the place has it again and the head holds nothing
        assert_eq!(at(None)[1], (2, Some(3)));
        // a change from T3 to T5: T3 is back and T5 is out
        assert_eq!(
            at(Some((Program::Drill, 5))),
            vec![(1, Some(1)), (2, Some(3)), (3, None)]
        );
        // a tool of the other program with the same number is not the one in the head
        assert_eq!(at(Some((Program::Route, 3)))[1], (2, Some(3)));
        // exactly one place is empty while the head holds one of the tools, and none otherwise
        assert_eq!(
            at(Some((Program::Drill, 1)))
                .iter()
                .filter(|s| s.1.is_none())
                .count(),
            1
        );
        assert_eq!(at(None).iter().filter(|s| s.1.is_none()).count(), 0);
    }

    #[test]
    fn a_tool_leaves_its_place_when_the_head_gets_to_it_and_not_when_the_job_names_it() {
        let mut a = app();
        // a job of two tools, on the changer model
        let dir = std::env::temp_dir().join(format!("eh3d-bg-carry-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("two.ncd");
        std::fs::write(
            &file,
            "%
T06
X669Y2953
X1063Y2953
T08
X1181Y984
X1481Y984
",
        )
        .unwrap();
        a.apply_model_with(machines::Model::from_name("CCD/2 ATC").unwrap(), false);
        a.open_path(&file);
        a.auto_ok = true;
        a.fixed_dt = Some(0.02);
        let loaded = a.rack_slot_tools();
        let all = loaded.len();
        assert!(all >= 2, "{loaded:?}");
        a.act(Act::Start);
        // the changer models ask for the rack to be loaded first
        if a.modal == Some(Modal::Rack) {
            a.act(Act::RackGo);
        }
        assert_eq!(a.running, Some(Program::Drill));
        let (mut takes, mut leaves) = (0, 0);
        let mut named_before_taken = false;
        let mut prev = None;
        for _ in 0..200_000 {
            if a.running.is_none() {
                break;
            }
            a.tick();
            let carried = a.rack_carried;
            let draw = a.bar_draw().unwrap();
            // one place is empty exactly while the head holds a tool
            assert_eq!(
                draw.tools.len(),
                all - loaded
                    .iter()
                    .filter(|(_, p, t)| a.running.is_some() && carried == Some((*p, *t)))
                    .count(),
                "{carried:?}"
            );
            assert_eq!(draw.head.map(|h| h.0), carried.map(|c| c.1));
            if carried != prev {
                // the head holds a tool only where it is at the place of that tool
                let head = a.running_head_mm().unwrap_or_default();
                let view = a.bar_view().unwrap();
                let place = carried.or(prev).unwrap();
                let slot = loaded
                    .iter()
                    .find(|(_, p, t)| (*p, *t) == place)
                    .map(|l| l.0)
                    .unwrap();
                let spot = view.slots.iter().find(|s| s.index == slot).unwrap();
                assert!(
                    (head.0 - spot.x_mm).hypot(head.1 - spot.y_mm) < 6.0,
                    "{carried:?} {prev:?} head {head:?} place {spot:?}"
                );
                if carried.is_some() {
                    takes += 1;
                } else {
                    leaves += 1;
                }
                prev = carried;
            }
            // the job names the second tool while the head still holds the first one
            if let (Some(c), Some(v)) = (carried, a.running_view()) {
                if v.tool.is_some_and(|t| t != c.1) {
                    named_before_taken = true;
                    assert!(draw.tools.iter().all(|t| t.0 != 0));
                }
            }
        }
        assert!(takes >= 2 && leaves >= 1, "takes {takes} leaves {leaves}");
        assert!(
            named_before_taken,
            "the job named the next tool before the head left the first"
        );
    }
}
