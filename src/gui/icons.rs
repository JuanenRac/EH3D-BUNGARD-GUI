//! The pictures of the small buttons: drawn with lines and polygons, so they take the colour of the theme and the size of the button; a button
//! with a picture says its name in a note when the pointer stays over it.

use super::controls::Icon;
use super::*;

impl Cv<'_> {
    /// The pictures of the editing and the view buttons (the others are in `controls`).
    pub(crate) fn draw_more_icon(&self, cx: f32, cy: f32, size: f32, icon: Icon, c: Color32) {
        let h = size / 2.0;
        match icon {
            Icon::Select => {
                let (l, t, r, b) = (cx - h * 0.95, cy - h * 0.95, cx + h * 0.55, cy + h * 0.55);
                self.polyline(&[(l, t), (r, t), (r, b), (l, b), (l, t)], 2.2, c, true);
                // the pointer
                self.poly(
                    &[
                        (cx - h * 0.05, cy - h * 0.1),
                        (cx + h * 0.95, cy + h * 0.45),
                        (cx + h * 0.45, cy + h * 0.55),
                        (cx + h * 0.7, cy + h * 1.0),
                        (cx + h * 0.45, cy + h * 1.05),
                        (cx + h * 0.2, cy + h * 0.65),
                        (cx - h * 0.05, cy + h * 0.9),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::Copy => {
                let w = 2.4;
                self.polyline(
                    &[
                        (cx - h * 0.45, cy - h * 0.55),
                        (cx - h * 0.45, cy - h * 0.95),
                        (cx + h * 0.9, cy - h * 0.95),
                        (cx + h * 0.9, cy + h * 0.3),
                        (cx + h * 0.5, cy + h * 0.3),
                    ],
                    w,
                    c,
                    false,
                );
                self.polyline(
                    &[
                        (cx - h * 0.9, cy - h * 0.45),
                        (cx + h * 0.45, cy - h * 0.45),
                        (cx + h * 0.45, cy + h * 0.95),
                        (cx - h * 0.9, cy + h * 0.95),
                        (cx - h * 0.9, cy - h * 0.45),
                    ],
                    w,
                    c,
                    false,
                );
            }
            Icon::Paste => {
                let w = 2.4;
                self.polyline(
                    &[
                        (cx - h * 0.55, cy - h * 0.75),
                        (cx - h * 0.9, cy - h * 0.75),
                        (cx - h * 0.9, cy + h * 0.95),
                        (cx + h * 0.9, cy + h * 0.95),
                        (cx + h * 0.9, cy - h * 0.75),
                        (cx + h * 0.55, cy - h * 0.75),
                    ],
                    w,
                    c,
                    false,
                );
                self.poly(
                    &[
                        (cx - h * 0.55, cy - h * 1.0),
                        (cx + h * 0.55, cy - h * 1.0),
                        (cx + h * 0.55, cy - h * 0.45),
                        (cx - h * 0.55, cy - h * 0.45),
                    ],
                    c,
                    Stroke::NONE,
                );
                self.line(
                    (cx - h * 0.5, cy + h * 0.05),
                    (cx + h * 0.5, cy + h * 0.05),
                    2.0,
                    c,
                );
                self.line(
                    (cx - h * 0.5, cy + h * 0.5),
                    (cx + h * 0.2, cy + h * 0.5),
                    2.0,
                    c,
                );
            }
            Icon::Trash => {
                let w = 2.4;
                self.line(
                    (cx - h * 0.95, cy - h * 0.6),
                    (cx + h * 0.95, cy - h * 0.6),
                    w,
                    c,
                );
                self.polyline(
                    &[
                        (cx - h * 0.3, cy - h * 0.6),
                        (cx - h * 0.3, cy - h * 0.95),
                        (cx + h * 0.3, cy - h * 0.95),
                        (cx + h * 0.3, cy - h * 0.6),
                    ],
                    w,
                    c,
                    false,
                );
                self.polyline(
                    &[
                        (cx - h * 0.7, cy - h * 0.6),
                        (cx - h * 0.55, cy + h * 0.95),
                        (cx + h * 0.55, cy + h * 0.95),
                        (cx + h * 0.7, cy - h * 0.6),
                    ],
                    w,
                    c,
                    false,
                );
                for dx in [-0.22f32, 0.22] {
                    self.line(
                        (cx + h * dx, cy - h * 0.3),
                        (cx + h * dx, cy + h * 0.65),
                        2.0,
                        c,
                    );
                }
            }
            Icon::Rotate => {
                // three quarters of a circle and the head of an arrow at its end
                let r = h * 0.8;
                let pts: Vec<(f32, f32)> = (0..=18)
                    .map(|i| {
                        let a = (-60.0 + 270.0 * i as f32 / 18.0).to_radians();
                        (cx + r * a.cos(), cy - r * a.sin())
                    })
                    .collect();
                self.polyline(&pts, 2.6, c, false);
                let e = pts[18];
                self.poly(
                    &[
                        (e.0 - h * 0.5, e.1 - h * 0.35),
                        (e.0 + h * 0.5, e.1 - h * 0.45),
                        (e.0 + h * 0.05, e.1 + h * 0.5),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::TurnBoard => {
                // the board as a rectangle, standing on its short side, and the arrow that turns it a quarter
                let (a, b) = (h * 0.95, h * 0.55);
                let rect = [
                    (cx - b, cy - a * 0.55),
                    (cx + b, cy - a * 0.55),
                    (cx + b, cy + a * 0.55),
                    (cx - b, cy + a * 0.55),
                    (cx - b, cy - a * 0.55),
                ];
                self.polyline(&rect, 2.2, c, false);
                let r = h * 1.15;
                let arc: Vec<(f32, f32)> = (0..=10)
                    .map(|i| {
                        let ang = (200.0 - 100.0 * i as f32 / 10.0).to_radians();
                        (cx + r * ang.cos(), cy - r * ang.sin())
                    })
                    .collect();
                self.polyline(&arc, 2.2, c, false);
                let e = arc[10];
                self.poly(
                    &[
                        (e.0 - h * 0.45, e.1 - h * 0.05),
                        (e.0 + h * 0.2, e.1 - h * 0.5),
                        (e.0 + h * 0.3, e.1 + h * 0.35),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::FitBoard => {
                let (a, b) = (h * 0.95, h * 0.4);
                for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
                    self.polyline(
                        &[
                            (cx + sx * a, cy + sy * (a - b)),
                            (cx + sx * a, cy + sy * a),
                            (cx + sx * (a - b), cy + sy * a),
                        ],
                        2.6,
                        c,
                        false,
                    );
                }
                self.poly(
                    &[
                        (cx - h * 0.35, cy - h * 0.35),
                        (cx + h * 0.35, cy - h * 0.35),
                        (cx + h * 0.35, cy + h * 0.35),
                        (cx - h * 0.35, cy + h * 0.35),
                    ],
                    alpha(c, 150),
                    Stroke::NONE,
                );
            }
            Icon::FitTable => {
                self.polyline(
                    &[
                        (cx - h * 0.95, cy - h * 0.7),
                        (cx + h * 0.95, cy - h * 0.7),
                        (cx + h * 0.95, cy + h * 0.7),
                        (cx - h * 0.95, cy + h * 0.7),
                        (cx - h * 0.95, cy - h * 0.7),
                    ],
                    2.4,
                    c,
                    false,
                );
                self.poly(
                    &[
                        (cx - h * 0.4, cy - h * 0.25),
                        (cx + h * 0.4, cy - h * 0.25),
                        (cx + h * 0.4, cy + h * 0.25),
                        (cx - h * 0.4, cy + h * 0.25),
                    ],
                    alpha(c, 150),
                    Stroke::NONE,
                );
            }
            Icon::Target => {
                self.p.circle_stroke(
                    self.pt(cx, cy),
                    h * 0.65 * self.u,
                    Stroke::new(2.4 * self.u, c),
                );
                self.p.circle_filled(self.pt(cx, cy), h * 0.18 * self.u, c);
                for (dx, dy) in [(0.0f32, -1.0f32), (0.0, 1.0), (-1.0, 0.0), (1.0, 0.0)] {
                    self.line(
                        (cx + dx * h * 0.8, cy + dy * h * 0.8),
                        (cx + dx * h * 1.1, cy + dy * h * 1.1),
                        2.4,
                        c,
                    );
                }
            }
            Icon::Enlarge => {
                for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, 1.0)] {
                    self.line(
                        (cx + sx * h * 0.15, cy + sy * h * 0.15),
                        (cx + sx * h * 0.9, cy + sy * h * 0.9),
                        2.4,
                        c,
                    );
                    self.polyline(
                        &[
                            (cx + sx * h * 0.9, cy + sy * h * 0.35),
                            (cx + sx * h * 0.9, cy + sy * h * 0.9),
                            (cx + sx * h * 0.35, cy + sy * h * 0.9),
                        ],
                        2.4,
                        c,
                        false,
                    );
                }
            }
            Icon::Back => {
                self.polyline(
                    &[
                        (cx + h * 0.35, cy - h * 0.8),
                        (cx - h * 0.45, cy),
                        (cx + h * 0.35, cy + h * 0.8),
                    ],
                    3.2,
                    c,
                    false,
                );
            }
            Icon::Up => {
                self.polyline(
                    &[
                        (cx - h * 0.8, cy + h * 0.35),
                        (cx, cy - h * 0.45),
                        (cx + h * 0.8, cy + h * 0.35),
                    ],
                    3.2,
                    c,
                    false,
                );
            }
            Icon::Next => {
                self.polyline(
                    &[
                        (cx - h * 0.35, cy - h * 0.8),
                        (cx + h * 0.45, cy),
                        (cx - h * 0.35, cy + h * 0.8),
                    ],
                    3.2,
                    c,
                    false,
                );
            }
            Icon::Down => {
                self.polyline(
                    &[
                        (cx - h * 0.8, cy - h * 0.35),
                        (cx, cy + h * 0.45),
                        (cx + h * 0.8, cy - h * 0.35),
                    ],
                    3.2,
                    c,
                    false,
                );
            }
            Icon::Cut => {
                // two rings and the blades that cross
                for sx in [-1.0f32, 1.0] {
                    self.p.circle_stroke(
                        self.pt(cx + sx * h * 0.55, cy + h * 0.6),
                        h * 0.32 * self.u,
                        Stroke::new(2.2 * self.u, c),
                    );
                    self.line(
                        (cx + sx * h * 0.45, cy + h * 0.3),
                        (cx - sx * h * 0.7, cy - h * 0.95),
                        2.4,
                        c,
                    );
                }
            }
            Icon::Undo | Icon::Redo => {
                let s = if icon == Icon::Undo { 1.0f32 } else { -1.0 };
                let r = h * 0.62;
                let pts: Vec<(f32, f32)> = (0..=14)
                    .map(|i| {
                        let a = (-80.0 + 230.0 * i as f32 / 14.0).to_radians();
                        (
                            cx + s * (-h * 0.05 + r * a.cos()),
                            cy + h * 0.15 - r * a.sin() * 0.95,
                        )
                    })
                    .collect();
                self.polyline(&pts, 2.6, c, false);
                let e = pts[0];
                self.poly(
                    &[
                        (e.0 - s * h * 0.15, e.1 - h * 0.62),
                        (e.0 - s * h * 0.15, e.1 + h * 0.3),
                        (e.0 - s * h * 0.85, e.1 - h * 0.14),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::Folder => {
                self.polyline(
                    &[
                        (cx - h * 0.95, cy + h * 0.7),
                        (cx - h * 0.95, cy - h * 0.7),
                        (cx - h * 0.3, cy - h * 0.7),
                        (cx, cy - h * 0.4),
                        (cx + h * 0.95, cy - h * 0.4),
                        (cx + h * 0.95, cy + h * 0.7),
                        (cx - h * 0.95, cy + h * 0.7),
                    ],
                    2.4,
                    c,
                    false,
                );
                self.line(
                    (cx - h * 0.95, cy - h * 0.05),
                    (cx + h * 0.95, cy - h * 0.05),
                    2.0,
                    c,
                );
            }
            Icon::BoardWin => {
                self.polyline(
                    &[
                        (cx - h * 0.95, cy - h * 0.65),
                        (cx + h * 0.95, cy - h * 0.65),
                        (cx + h * 0.95, cy + h * 0.65),
                        (cx - h * 0.95, cy + h * 0.65),
                        (cx - h * 0.95, cy - h * 0.65),
                    ],
                    2.4,
                    c,
                    false,
                );
                for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    self.p.circle_filled(
                        self.pt(cx + sx * h * 0.6, cy + sy * h * 0.33),
                        1.7 * self.u,
                        c,
                    );
                }
            }
            Icon::DrillWin => {
                // the shank, the flutes that wind round the bit and its point
                self.line((cx, cy - h * 0.95), (cx, cy - h * 0.4), 3.4, c);
                self.polyline(
                    &[
                        (cx - h * 0.38, cy - h * 0.3),
                        (cx + h * 0.38, cy - h * 0.05),
                        (cx - h * 0.38, cy + h * 0.2),
                        (cx + h * 0.38, cy + h * 0.45),
                    ],
                    2.2,
                    c,
                    false,
                );
                self.poly(
                    &[
                        (cx - h * 0.38, cy + h * 0.42),
                        (cx + h * 0.38, cy + h * 0.42),
                        (cx, cy + h * 1.0),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::GridWin => {
                for (sx, sy) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let (x0, y0) = (cx + sx * h * 0.5, cy + sy * h * 0.5);
                    self.polyline(
                        &[
                            (x0 - h * 0.38, y0 - h * 0.38),
                            (x0 + h * 0.38, y0 - h * 0.38),
                            (x0 + h * 0.38, y0 + h * 0.38),
                            (x0 - h * 0.38, y0 + h * 0.38),
                            (x0 - h * 0.38, y0 - h * 0.38),
                        ],
                        2.2,
                        c,
                        false,
                    );
                }
            }
            Icon::Sliders => {
                for (i, k) in [0.3f32, 0.75, 0.45].into_iter().enumerate() {
                    let y = cy + (i as f32 - 1.0) * h * 0.65;
                    self.line((cx - h * 0.95, y), (cx + h * 0.95, y), 2.0, c);
                    self.p.circle_filled(
                        self.pt(cx + (k * 2.0 - 1.0) * h * 0.6, y),
                        3.2 * self.u,
                        c,
                    );
                }
            }
            Icon::Plug => {
                self.poly(
                    &[
                        (cx - h * 0.6, cy - h * 0.15),
                        (cx + h * 0.6, cy - h * 0.15),
                        (cx + h * 0.6, cy + h * 0.5),
                        (cx - h * 0.6, cy + h * 0.5),
                    ],
                    c,
                    Stroke::NONE,
                );
                for sx in [-1.0f32, 1.0] {
                    self.line(
                        (cx + sx * h * 0.3, cy - h * 0.2),
                        (cx + sx * h * 0.3, cy - h * 0.9),
                        2.6,
                        c,
                    );
                }
                self.line((cx, cy + h * 0.5), (cx, cy + h * 0.95), 2.6, c);
            }
            Icon::Nodes => {
                let pts = [(0.0f32, -0.7f32), (-0.85, 0.6), (0.85, 0.6)];
                for (px, py) in pts {
                    self.line((cx, cy), (cx + px * h, cy + py * h), 1.8, c);
                    self.p
                        .circle_filled(self.pt(cx + px * h, cy + py * h), 3.4 * self.u, c);
                }
                self.p
                    .circle_stroke(self.pt(cx, cy), 4.2 * self.u, Stroke::new(2.0 * self.u, c));
            }
            Icon::Info => {
                let ring: Vec<(f32, f32)> = (0..=20)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / 20.0;
                        (cx + h * 0.95 * a.cos(), cy + h * 0.95 * a.sin())
                    })
                    .collect();
                self.polyline(&ring, 2.2, c, false);
                self.p
                    .circle_filled(self.pt(cx, cy - h * 0.45), 1.9 * self.u, c);
                self.line((cx, cy - h * 0.1), (cx, cy + h * 0.55), 2.6, c);
            }
            Icon::Update => {
                // a circle drawn nearly all the way round with the head of an arrow at its end, and a short arrow pointing down inside
                let ring: Vec<(f32, f32)> = (0..=16)
                    .map(|i| {
                        let a = (-60.0_f32 + i as f32 * 300.0 / 16.0).to_radians();
                        (cx + h * 0.9 * a.cos(), cy + h * 0.9 * a.sin())
                    })
                    .collect();
                self.polyline(&ring, 2.2, c, false);
                let (ex, ey) = (
                    cx + h * 0.9 * (-60.0_f32).to_radians().cos(),
                    cy + h * 0.9 * (-60.0_f32).to_radians().sin(),
                );
                self.poly(
                    &[
                        (ex + h * 0.1, ey - h * 0.5),
                        (ex + h * 0.62, ey + h * 0.18),
                        (ex - h * 0.5, ey + h * 0.2),
                    ],
                    c,
                    Stroke::NONE,
                );
                self.line((cx, cy - h * 0.4), (cx, cy + h * 0.25), 2.4, c);
                self.poly(
                    &[
                        (cx - h * 0.34, cy + h * 0.1),
                        (cx + h * 0.34, cy + h * 0.1),
                        (cx, cy + h * 0.5),
                    ],
                    c,
                    Stroke::NONE,
                );
            }
            Icon::Play | Icon::Pause | Icon::Stop | Icon::Alert => {}
        }
    }

    /// A medium button of the top bar: its picture at the left and its name beside it, in the room that is left (a longer word, in another language, is drawn smaller).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn top_button(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        icon: Icon,
        label: &str,
        on: bool,
        act: Act,
    ) {
        self.chip(x, y, w, h, "", on, act);
        let rect = self.rect(x, y, w, h);
        let (_, down) = self.touch(rect);
        let ink = if on {
            col(self.t.background.0)
        } else {
            col(self.t.text)
        };
        let dy = if down { 1.8 } else { 0.0 };
        self.draw_icon(x + 15.0, y + h / 2.0 + dy, 20.0, icon, ink);
        self.text_fit(
            x + 28.0,
            y + h / 2.0 + 5.0 + dy,
            Align2::LEFT_BOTTOM,
            label,
            14.0,
            ink,
            w - 32.0,
        );
    }

    /// A small button with a picture and, when the pointer stays over it, a note with its name.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn icon_chip(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        icon: Icon,
        on: bool,
        tip: &str,
        act: Act,
    ) {
        self.chip(x, y, w, h, "", on, act);
        let rect = self.rect(x, y, w, h);
        let (hover, down) = self.touch(rect);
        let ink = if on {
            col(self.t.background.0)
        } else if hover {
            Color32::WHITE
        } else {
            col(self.t.text)
        };
        self.draw_icon(
            x + w / 2.0,
            y + h / 2.0 + if down { 1.8 } else { 0.0 },
            w.min(h) * 0.5,
            icon,
            ink,
        );
        if hover && !tip.is_empty() {
            let galley = self.p.layout_no_wrap(
                tip.to_string(),
                FontId::proportional(13.0 * self.u),
                col(self.t.text),
            );
            let size = galley.size() + Vec2::new(16.0 * self.u, 10.0 * self.u);
            let mut at = Pos2::new(rect.center().x - size.x / 2.0, rect.max.y + 8.0 * self.u);
            let right = self.o.x + DESIGN_W * self.u - 6.0;
            if at.x + size.x > right {
                at.x = right - size.x;
            }
            at.x = at.x.max(self.o.x + 4.0);
            let note = Rect::from_min_size(at, size);
            self.p.rect_filled(
                note.translate(Vec2::new(0.0, 2.0 * self.u)),
                6.0 * self.u,
                alpha(Color32::BLACK, 90),
            );
            self.p.rect_filled(note, 6.0 * self.u, col(self.t.panel));
            self.p.rect_stroke(
                note,
                6.0 * self.u,
                Stroke::new(self.u, col(self.t.accent)),
                StrokeKind::Inside,
            );
            self.p.galley(
                at + Vec2::new(8.0 * self.u, 5.0 * self.u),
                galley,
                col(self.t.text),
            );
        }
    }
}
