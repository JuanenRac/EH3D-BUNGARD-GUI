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

impl App {
    /// The machine in the settings has a tool changer (an ATC model) and so a bar.
    pub(crate) fn bar_shown(&self) -> bool {
        self.has_changer() && self.layers[2]
    }

    /// The profile of the machine in use.
    fn bar_profile(&self) -> &'static machines::Profile {
        machines::models()[self.model().min(machines::models().len() - 1)].profile()
    }

    /// Where the bar starts from for this machine: the model's own first and last tool, with the depth inside the travel of the file.
    fn bar_default(&self) -> RackEnds {
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
        let tools = loaded
            .iter()
            .filter(|(s, _, _)| Some(*s) != at)
            .map(|(s, p, t)| (*s, *t, color(*p, *t)))
            .collect();
        let head = at.and_then(|s| {
            loaded
                .iter()
                .find(|(i, _, _)| *i == s)
                .map(|(_, p, t)| (*t, color(*p, *t)))
        });
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
                Stroke::new(1.0 * cv.u, edge),
            );
            if let Some((_, tool, color)) = d.tools.iter().find(|(i, _, _)| *i == s.index) {
                let w = rr * 0.8;
                draw_boxes(
                    cv,
                    iso,
                    o,
                    shift,
                    &[(
                        s.x_mm - w,
                        table_depth - s.y_mm - w,
                        top,
                        2.0 * w,
                        2.0 * w,
                        shank,
                        col(t.size_color(*color)),
                    )],
                    edge,
                );
                let (px, py) = p(s.x_mm, table_depth - s.y_mm, top + shank + 3.0);
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
                    let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(DESIGN_W, DESIGN_H));
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
}
