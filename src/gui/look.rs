//! The relief of the controls: a body with a gradient (lighter on top, darker below), an edge of light, a shade under it and a glow when the pointer is
//! over it; the boxes where the person types and the tracks of the bars are sunk into the panel instead; and the X of the corner of a window,
//! which is red, turns when the pointer is over it and sinks when it is pressed.

use egui::epaint::{Mesh, Vertex, WHITE_UV};

use super::*;

/// The points round a rectangle with rounded corners, clockwise from the top left.
pub(super) fn round_points(rect: Rect, r: f32) -> Vec<Pos2> {
    let r = r.min(rect.width() / 2.0).min(rect.height() / 2.0).max(0.0);
    if r < 0.5 {
        return vec![
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
        ];
    }
    let steps = 6;
    let mut v = Vec::with_capacity(4 * (steps + 1));
    // each corner: its centre and the angle where its arc starts
    let corners = [
        (Pos2::new(rect.min.x + r, rect.min.y + r), 180.0f32),
        (Pos2::new(rect.max.x - r, rect.min.y + r), 270.0),
        (Pos2::new(rect.max.x - r, rect.max.y - r), 0.0),
        (Pos2::new(rect.min.x + r, rect.max.y - r), 90.0),
    ];
    for (c, start) in corners {
        for i in 0..=steps {
            let a = (start + 90.0 * i as f32 / steps as f32).to_radians();
            v.push(Pos2::new(c.x + r * a.cos(), c.y + r * a.sin()));
        }
    }
    v
}

fn mix(a: Color32, b: Color32, k: f32) -> Color32 {
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}

impl Cv<'_> {
    /// A convex shape filled with a gradient from `top` to `bottom` (the colour of each vertex follows its height, so the fan of triangles
    /// from the middle gives a true gradient).
    pub(crate) fn gradient_poly(&self, pts: &[Pos2], top: Color32, bottom: Color32) {
        if pts.len() < 3 {
            return;
        }
        let (y0, y1) = pts
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(p.y), b.max(p.y)));
        let at = |y: f32| {
            mix(
                top,
                bottom,
                ((y - y0) / (y1 - y0).max(0.001)).clamp(0.0, 1.0),
            )
        };
        let mid = Pos2::new(
            pts.iter().map(|p| p.x).sum::<f32>() / pts.len() as f32,
            (y0 + y1) / 2.0,
        );
        let mut mesh = Mesh::default();
        mesh.vertices.push(Vertex {
            pos: mid,
            uv: WHITE_UV,
            color: at(mid.y),
        });
        for p in pts {
            mesh.vertices.push(Vertex {
                pos: *p,
                uv: WHITE_UV,
                color: at(p.y),
            });
        }
        let n = pts.len() as u32;
        for i in 0..n {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        self.p.add(Shape::mesh(mesh));
    }

    /// A rounded rectangle with a gradient.
    pub(crate) fn gradient_rrect(&self, rect: Rect, r: f32, top: Color32, bottom: Color32) {
        self.gradient_poly(&round_points(rect, r), top, bottom);
    }

    /// A body in relief: the shadow under it, the glow if the pointer is over it, the gradient, the edge, the line of light on top and the
    /// line of shade at the bottom. Pressed, it sinks and darkens.
    pub(crate) fn raised(
        &self,
        rect: Rect,
        r: f32,
        base: Color32,
        hover: bool,
        down: bool,
    ) -> Rect {
        let u = self.u;
        let accent = col(self.t.accent);
        if !down {
            self.p.rect_filled(
                rect.translate(Vec2::new(0.0, 2.5 * u)),
                r,
                alpha(Color32::BLACK, if hover { 80 } else { 46 }),
            );
        }
        if hover && !down {
            for (g, a) in [(5.0, 18u8), (2.5, 36)] {
                self.p
                    .rect_filled(rect.expand(g * u), r + g * u, alpha(accent, a));
            }
        }
        let body = if down {
            rect.translate(Vec2::new(0.0, 1.8 * u))
        } else {
            rect
        };
        let (top, bottom) = if down {
            (shade(base, 0.74), shade(base, 0.9))
        } else if hover {
            (shade(base, 1.34), shade(base, 0.92))
        } else {
            (shade(base, 1.2), shade(base, 0.8))
        };
        self.gradient_rrect(body, r, top, bottom);
        self.p.rect_stroke(
            body,
            r,
            Stroke::new(
                if hover { 1.6 } else { 1.0 } * u,
                alpha(accent, if hover { 255 } else { 160 }),
            ),
            StrokeKind::Inside,
        );
        let inset = (r * 0.6).max(5.0 * u);
        let ty = body.min.y + 2.4 * u;
        self.p.line_segment(
            [
                Pos2::new(body.min.x + inset, ty),
                Pos2::new(body.max.x - inset, ty),
            ],
            Stroke::new(1.4 * u, alpha(Color32::WHITE, if down { 10 } else { 72 })),
        );
        let by = body.max.y - 2.0 * u;
        self.p.line_segment(
            [
                Pos2::new(body.min.x + inset, by),
                Pos2::new(body.max.x - inset, by),
            ],
            Stroke::new(1.2 * u, alpha(Color32::BLACK, if down { 18 } else { 60 })),
        );
        body
    }

    /// A body sunk into the panel (the track of a bar, a box to type in): dark on top where the edge shades it, lighter below, with an edge of
    /// light along the bottom lip. Under the pointer its edge takes the accent colour.
    pub(crate) fn sunk(&self, rect: Rect, r: f32, base: Color32, hover: bool) {
        let u = self.u;
        self.gradient_rrect(rect, r, shade(base, 0.62), shade(base, 0.9));
        self.p.rect_stroke(
            rect,
            r,
            Stroke::new(
                if hover { 1.5 } else { 1.0 } * u,
                if hover {
                    col(self.t.accent)
                } else {
                    alpha(Color32::BLACK, 120)
                },
            ),
            StrokeKind::Inside,
        );
        let inset = (r * 0.6).max(4.0 * u);
        self.p.line_segment(
            [
                Pos2::new(rect.min.x + inset, rect.max.y + 0.8 * u),
                Pos2::new(rect.max.x - inset, rect.max.y + 0.8 * u),
            ],
            Stroke::new(1.0 * u, alpha(Color32::WHITE, 22)),
        );
    }

    /// The X of the corner of a window: red, with a glow; under the pointer it turns a quarter of a turn and the glow beats; pressed, it sinks.
    pub(crate) fn close_button(&mut self, x: f32, y: f32, w: f32, h: f32, act: Act) {
        let rect = self.rect(x, y, w, h);
        let (hover, down) = self.touch(rect);
        let ctx = self.p.ctx().clone();
        let k =
            ctx.animate_bool_with_time(egui::Id::unique(("close-x", act_key(&act))), hover, 0.18);
        let u = self.u;
        let red = Color32::from_rgb(214, 62, 66);
        let r = if self.t.corner == Corner::Square {
            2.0 * u
        } else {
            9.0 * u
        };
        if hover && !down {
            let pulse = 0.5 + 0.5 * (self.time * 6.0).sin();
            for (g, a) in [(7.0, 20u8), (4.0, 42), (2.0, 70)] {
                self.p.rect_filled(
                    rect.expand((g + 1.5 * pulse) * u),
                    r + g * u,
                    alpha(Color32::from_rgb(255, 90, 90), a),
                );
            }
        }
        let body = self.raised(
            rect,
            r,
            mix(red, Color32::from_rgb(255, 96, 96), k * 0.5),
            false,
            down,
        );
        // the cross, turning a quarter of a turn as the pointer arrives
        let c = body.center();
        let arm = body.width().min(body.height()) * 0.24 * (1.0 + 0.1 * k);
        let ang = std::f32::consts::FRAC_PI_4 + k * std::f32::consts::FRAC_PI_2;
        let (s, co) = ang.sin_cos();
        let ink = Color32::WHITE;
        for dir in [(co, s), (-s, co)] {
            let d = Vec2::new(dir.0, dir.1) * arm;
            self.p.line_segment(
                [c - d + Vec2::new(0.0, u), c + d + Vec2::new(0.0, u)],
                Stroke::new(4.4 * u, alpha(Color32::BLACK, 70)),
            );
            self.p
                .line_segment([c - d, c + d], Stroke::new(3.4 * u, ink));
        }
        self.hits.push((rect, act));
    }
}

impl Cv<'_> {
    /// A small check box (the layers, the box of the launcher): in relief with its mark when it is on, sunk when it is off, and answering to the pointer.
    pub(crate) fn check_box_small(&self, r: Rect, on: bool, hover: bool, down: bool) {
        let u = self.u;
        let radius = 4.0 * u;
        if on {
            let body = self.raised(r, radius, col(self.t.accent), hover, down);
            let c = body.center();
            let k = body.width() / 16.0;
            let ink = col(self.t.background.0);
            let pts = [
                Pos2::new(c.x - 4.2 * k, c.y + 0.2 * k),
                Pos2::new(c.x - 1.2 * k, c.y + 3.2 * k),
                Pos2::new(c.x + 4.4 * k, c.y - 3.4 * k),
            ];
            self.p.add(Shape::line(
                pts.to_vec(),
                Stroke::new(2.0 * u * k.max(0.8), ink),
            ));
        } else {
            self.sunk(r, radius, col(self.t.panel_raised), hover);
        }
    }
}

/// A key that tells the buttons apart for their animations.
fn act_key(act: &Act) -> String {
    format!("{act:?}")
}

/// A text box with relief: sunk into the panel, its edge lighting up under the pointer and when it has the focus. The box of egui is drawn over it
/// with the colour of the theme; this is the shade round it.
pub(crate) fn text_box(ui: &mut egui::Ui, edit: egui::TextEdit<'_>) -> egui::Response {
    text_box_show(ui, edit).response.response
}

/// The same box, giving back the whole output of egui (the state of the cursor too).
pub(crate) fn text_box_show(
    ui: &mut egui::Ui,
    edit: egui::TextEdit<'_>,
) -> egui::text_edit::TextEditOutput {
    let accent = ui.visuals().selection.stroke.color;
    let slot = ui.painter().add(Shape::Noop);
    let out = edit.show(ui);
    let r = out.response.clone();
    let hot = r.has_focus();
    let hover = r.hovered();
    let rect = r.rect;
    let mut shapes = vec![
        // the shade at the top where the box is sunk
        Shape::rect_filled(
            rect.expand(1.0),
            7.0,
            Color32::from_black_alpha(if hot { 110 } else { 80 }),
        ),
    ];
    if hot || hover {
        shapes.push(Shape::rect_filled(
            rect.expand(4.0),
            9.0,
            Color32::from_rgba_unmultiplied(
                accent.r(),
                accent.g(),
                accent.b(),
                if hot { 60 } else { 28 },
            ),
        ));
    }
    ui.painter().set(slot, Shape::Vec(shapes));
    // the lip of light along the bottom edge, over the box
    ui.painter().line_segment(
        [
            Pos2::new(rect.min.x + 6.0, rect.max.y + 1.0),
            Pos2::new(rect.max.x - 6.0, rect.max.y + 1.0),
        ],
        Stroke::new(1.0, Color32::from_white_alpha(26)),
    );
    out
}
