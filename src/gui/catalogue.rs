//! The window of the catalogue of bits: a list of drills, slot drills, end mills, routers and V-bits (all with the 1/8 inch shank of a Bungard chuck), by kind, with the numbers a
//! job starts from. A bit that is clicked is drawn on the right, with its measures and what it is for. The list is in [`crate::catalogue`].

use super::extra::wrap;
use super::*;
use crate::catalogue as data;
use data::Kind;

/// The kinds the list can be limited to, with the words that name them.
pub(super) const KINDS: [(Kind, Text); 12] = [
    (Kind::Drill, Text::KindDrill),
    (Kind::SlotDrill, Text::KindSlotDrill),
    (Kind::EndMill, Text::KindEndMill),
    (Kind::DownCut, Text::KindDownCut),
    (Kind::UpCut, Text::KindUpCut),
    (Kind::Diamond, Text::KindDiamond),
    (Kind::ChipBreaker, Text::KindChipBreaker),
    (Kind::Coated, Text::KindCoated),
    (Kind::VBit, Text::KindVBit),
    (Kind::BallNose, Text::KindBall),
    (Kind::Spot, Text::KindSpot),
    (Kind::Countersink, Text::KindCountersink),
];

/// How many rows the list shows at once.
const SHOWN: usize = 12;
const ROW_H: f32 = 40.0;
/// The width of the list, and where the drawing of the bit begins.
const LIST_W: f32 = 930.0;
const SPINDLE_STEP: i32 = 2_000;

impl App {
    /// The bits that the kind chosen shows.
    pub(crate) fn cat_bits(&self) -> Vec<data::Bit> {
        let kind = self
            .cat_kind
            .checked_sub(1)
            .and_then(|i| KINDS.get(i))
            .map(|k| k.0);
        data::of_kind(kind)
    }

    pub(crate) fn cat_scroll_by(&mut self, delta: i32) {
        let max = self.cat_bits().len().saturating_sub(SHOWN);
        self.cat_scroll = (self.cat_scroll as i64 + delta as i64).clamp(0, max as i64) as usize;
    }

    /// The top speed of the spindle that the catalogue works for moves by a step; 0 would mean "as the maker says".
    pub(crate) fn cat_spindle_by(&mut self, steps: i32) {
        let now = self.settings.spindle_rpm as i64;
        self.settings.spindle_rpm =
            (now + steps as i64 * SPINDLE_STEP as i64).clamp(4_000, 200_000) as u32;
        self.remember_settings();
    }

    fn kind_name(&self, k: Kind) -> &'static str {
        let text = KINDS
            .iter()
            .find(|x| x.0 == k)
            .map_or(Text::CatAll, |x| x.1);
        self.tr(text)
    }

    /// What a kind of bit is for, in words.
    fn kind_note(&self, k: Kind) -> &'static str {
        self.tr(match k {
            Kind::Drill => Text::NoteDrill,
            Kind::SlotDrill => Text::NoteSlot,
            Kind::EndMill => Text::NoteEndMill,
            Kind::DownCut => Text::NoteDownCut,
            Kind::UpCut => Text::NoteUpCut,
            Kind::Diamond => Text::NoteDiamond,
            Kind::ChipBreaker => Text::NoteChip,
            Kind::Coated => Text::NoteCoated,
            Kind::VBit => Text::NoteVBit,
            Kind::BallNose => Text::NoteBall,
            Kind::Spot => Text::NoteSpot,
            Kind::Countersink => Text::NoteCountersink,
        })
    }

    pub(super) fn paint_catalogue(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let (x, y, w, h) = (110.0, 50.0, 1380.0, 800.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 44.0, self.tr(Text::CatTitle));
        for (i, l) in wrap(self.tr(Text::CatShank), 150).iter().enumerate() {
            cv.text(
                x + 30.0,
                y + 78.0 + i as f32 * 20.0,
                Align2::LEFT_BOTTOM,
                l,
                14.0,
                col(t.dim),
            );
        }
        let items = self.combo_items(ComboId::CatKind);
        cv.combo(
            x + 30.0,
            y + 110.0,
            460.0,
            ComboId::CatKind,
            &items[self.cat_kind.min(items.len() - 1)],
            self.combo.is_some_and(|c| c.id == ComboId::CatKind),
        );
        let bits = self.cat_bits();
        cv.text(
            x + 520.0,
            y + 136.0,
            Align2::LEFT_BOTTOM,
            &format!("{} {}", bits.len(), self.tr(Text::CatCount)),
            16.0,
            col(t.dim),
        );
        // the top speed of the spindle the numbers are worked out for
        cv.text(
            x + 700.0,
            y + 136.0,
            Align2::LEFT_BOTTOM,
            &format!("max {} rpm", self.settings.spindle_rpm),
            16.0,
            col(t.text),
        );
        cv.chip(
            x + 860.0,
            y + 108.0,
            40.0,
            38.0,
            "-",
            false,
            Act::CatSpindle(-1),
        );
        cv.chip(
            x + 906.0,
            y + 108.0,
            40.0,
            38.0,
            "+",
            false,
            Act::CatSpindle(1),
        );
        // the columns
        let cols = [
            ("", 30.0),
            ("serie", 225.0),
            ("Ø mm", 295.0),
            ("L mm", 365.0),
            ("tip mm", 430.0),
            ("°", 505.0),
            ("z", 550.0),
            ("rpm", 590.0),
            ("mm/min", 680.0),
            ("Z mm/min", 775.0),
            ("mm/pass", 860.0),
        ];
        let top = y + 170.0;
        for (name, cx) in cols {
            cv.text(
                x + cx,
                top + 16.0,
                Align2::LEFT_BOTTOM,
                name,
                13.0,
                col(t.dim),
            );
        }
        let first = self.cat_scroll.min(bits.len().saturating_sub(SHOWN));
        for (r, b) in bits.iter().skip(first).take(SHOWN).enumerate() {
            let ry = top + 26.0 + r as f32 * ROW_H;
            let chosen = self.cat_sel == Some(first + r);
            let rect = cv.rect(x + 20.0, ry, LIST_W - 30.0, ROW_H - 4.0);
            if r % 2 == 0 || chosen {
                cv.p.rect_filled(
                    rect,
                    3.0 * cv.u,
                    if chosen {
                        alpha(col(t.accent), 70)
                    } else {
                        alpha(col(t.panel_raised), 255)
                    },
                );
            }
            if chosen {
                cv.p.rect_stroke(
                    rect,
                    3.0 * cv.u,
                    Stroke::new(1.6 * cv.u, col(t.accent)),
                    StrokeKind::Inside,
                );
            }
            cv.hits.push((rect, Act::CatSelect(first + r)));
            let base = ry + ROW_H - 14.0;
            let put = |cv: &mut Cv, cx: f32, s: &str| {
                cv.text(x + cx, base, Align2::LEFT_BOTTOM, s, 15.0, col(t.text));
            };
            put(cv, 30.0, self.kind_name(b.kind));
            put(cv, 225.0, if b.series.is_empty() { "-" } else { b.series });
            put(
                cv,
                295.0,
                format!("{:.3}", b.diameter_mm)
                    .trim_end_matches('0')
                    .trim_end_matches('.'),
            );
            put(
                cv,
                365.0,
                &if b.flute_mm > 0.0 {
                    format!("{:.1}", b.flute_mm)
                } else {
                    "-".into()
                },
            );
            put(
                cv,
                430.0,
                &if b.tip_mm > 0.0 {
                    format!("{:.2}", b.tip_mm)
                } else {
                    "-".into()
                },
            );
            put(
                cv,
                505.0,
                &if b.angle_deg > 0 {
                    format!("{}", b.angle_deg)
                } else {
                    "-".into()
                },
            );
            put(
                cv,
                550.0,
                &if b.flutes > 0 {
                    format!("{}", b.flutes)
                } else {
                    "-".into()
                },
            );
            put(cv, 590.0, &format!("{}", b.rpm));
            put(
                cv,
                680.0,
                &if b.feed_mm_min > 0 {
                    format!("{}", b.feed_mm_min)
                } else {
                    "-".into()
                },
            );
            put(cv, 775.0, &format!("{}", b.plunge_mm_min));
            put(cv, 860.0, &format!("{:.2}", b.pass_mm));
        }
        // the scroll of the list: the bar at its side (the wheel works over the whole list)
        cv.scrollbar(
            Bar::Catalogue,
            x + LIST_W + 2.0,
            top + 26.0,
            SHOWN as f32 * ROW_H - 4.0,
            (x + 20.0, top + 26.0, LIST_W - 10.0, SHOWN as f32 * ROW_H),
            bits.len(),
            SHOWN,
            first,
        );
        // the bit that was clicked: its drawing, its measures and what it is for
        let (px, py, pw) = (x + LIST_W + 30.0, y + 170.0, w - LIST_W - 60.0);
        match self.cat_sel.and_then(|i| bits.get(i)) {
            None => {
                for (i, l) in wrap(self.tr(Text::CatPick), 40).iter().enumerate() {
                    cv.text(
                        px,
                        py + 40.0 + i as f32 * 22.0,
                        Align2::LEFT_BOTTOM,
                        l,
                        16.0,
                        col(t.dim),
                    );
                }
            }
            Some(b) => self.paint_bit_detail(cv, b, px, py, pw),
        }
        for (i, l) in wrap(self.tr(Text::CatSpindle), 150)
            .iter()
            .take(3)
            .enumerate()
        {
            cv.text(
                x + 30.0,
                y + h - 58.0 + i as f32 * 18.0,
                Align2::LEFT_BOTTOM,
                l,
                13.0,
                col(t.dim),
            );
        }
        cv.chip(
            x + w - 200.0,
            y + h - 60.0,
            170.0,
            42.0,
            self.tr(Text::Close),
            true,
            Act::CloseModal,
        );
    }

    /// The name, the drawing, the measures and the speeds of one bit.
    fn paint_bit_detail(&self, cv: &mut Cv, b: &data::Bit, x: f32, y: f32, w: f32) {
        let t = cv.t.clone();
        let name = if b.series.is_empty() {
            self.kind_name(b.kind).to_string()
        } else {
            format!("{} {}", b.series, self.kind_name(b.kind))
        };
        for (i, l) in wrap(&name, 34).iter().enumerate() {
            cv.text(
                x,
                y + 6.0 + i as f32 * 20.0,
                Align2::LEFT_BOTTOM,
                l,
                16.0,
                col(t.accent),
            );
        }
        let top = y + 32.0;
        paint_bit(cv, b, x, top, w, 300.0, self);
        let mine = b.for_spindle(self.settings.spindle_rpm);
        let mut ly = top + 330.0;
        let mut line = |cv: &mut Cv, s: String, c: Color32| {
            cv.text(x, ly, Align2::LEFT_BOTTOM, &s, 15.0, c);
            ly += 22.0;
        };
        line(
            cv,
            format!(
                "{} 3.175   {} 38.1",
                self.tr(Text::CatShankLbl),
                self.tr(Text::CatTotal)
            ),
            col(t.text),
        );
        if b.angle_deg > 0 {
            line(
                cv,
                format!("{} {}°", self.tr(Text::CatPoint), b.angle_deg),
                col(t.text),
            );
        }
        line(
            cv,
            format!(
                "{} rpm  {} mm/min  Z {} mm/min",
                mine.rpm,
                if mine.feed_mm_min > 0 {
                    mine.feed_mm_min.to_string()
                } else {
                    "-".into()
                },
                mine.plunge_mm_min
            ),
            col(t.ok),
        );
        if mine.rpm != b.rpm {
            line(
                cv,
                format!("({} rpm: {} / {})", b.rpm, b.feed_mm_min, b.plunge_mm_min),
                col(t.dim),
            );
        }
        ly += 4.0;
        for l in wrap(self.kind_note(b.kind), 52) {
            cv.text(x, ly, Align2::LEFT_BOTTOM, &l, 13.0, col(t.dim));
            ly += 18.0;
        }
        cv.text(
            x,
            ly + 4.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::CatSchematic),
            12.0,
            col(t.dim),
        );
    }
}

/// Draws one bit upright, tip down, with the shank above it: a schematic of the shape of its cutting part (not to scale across: a thin drill is drawn wider than it is).
fn paint_bit(cv: &mut Cv, b: &data::Bit, x0: f32, y0: f32, _w: f32, h: f32, app: &App) {
    let t = cv.t.clone();
    let s = (h - 20.0) / data::TOTAL_MM as f32;
    let cx = x0 + 150.0;
    let tip_y = y0 + h - 6.0;
    let top_y = tip_y - data::TOTAL_MM as f32 * s;
    let ws = data::SHANK_MM as f32 * s;
    let dia = b.diameter_mm as f32;
    let wd = match b.kind {
        Kind::VBit | Kind::Countersink | Kind::Spot => (dia * s).clamp(10.0, 60.0),
        _ => (dia * s).clamp(5.0, 60.0),
    };
    let mm = |v: f64| v as f32 * s;
    let half_tan = |deg: u32| ((deg.max(10) as f32) / 2.0).to_radians().tan();
    // the length of the pointed or rounded end, and of the part with the flutes
    let cone = match b.kind {
        Kind::Drill | Kind::SlotDrill => wd * 0.30,
        Kind::Spot | Kind::Countersink => {
            ((wd - 2.0) / 2.0 / half_tan(if b.angle_deg > 0 { b.angle_deg } else { 90 }))
                .min(mm(14.0))
        }
        Kind::VBit => (((dia - b.tip_mm as f32) * s / 2.0) / half_tan(b.angle_deg)).min(mm(24.0)),
        Kind::BallNose => wd / 2.0,
        _ => 0.0,
    };
    let flute = if matches!(b.kind, Kind::VBit | Kind::Spot | Kind::Countersink) {
        cone
    } else if b.flute_mm > 0.0 {
        mm(b.flute_mm)
    } else {
        mm((4.0 * b.diameter_mm).clamp(2.0, 12.0))
    };
    let tip_w = if b.kind == Kind::VBit {
        (b.tip_mm as f32 * s).max(2.0)
    } else {
        0.0
    };
    let body_w = if b.kind == Kind::VBit { ws } else { wd };
    // where the flutes end and the neck (the 30 degree cone to the shank) begins
    let flute_top = tip_y - flute.max(cone);
    let neck = ((ws - body_w).abs() / 2.0 / 15f32.to_radians().tan()).min(40.0);
    let shank_bottom = flute_top - neck;
    let line_c = col(t.dim);
    let fill = if b.kind == Kind::Coated {
        Color32::from_rgb(214, 168, 60)
    } else {
        alpha(col(t.accent), 110)
    };
    let stroke = Stroke::new(1.4 * cv.u, col(t.text));
    // the shank
    cv.poly(
        &[
            (cx - ws / 2.0, top_y),
            (cx + ws / 2.0, top_y),
            (cx + ws / 2.0, shank_bottom),
            (cx - ws / 2.0, shank_bottom),
        ],
        alpha(col(t.panel_raised), 255),
        stroke,
    );
    // the neck
    cv.poly(
        &[
            (cx - ws / 2.0, shank_bottom),
            (cx + ws / 2.0, shank_bottom),
            (cx + body_w / 2.0, flute_top),
            (cx - body_w / 2.0, flute_top),
        ],
        alpha(col(t.panel_raised), 255),
        stroke,
    );
    // the cutting part
    let body_bottom = tip_y - cone;
    // the shape of the cutting part as a polygon: down the right side, round the end, up the left side (convex for every shape here)
    let mut shape = vec![
        (cx - body_w / 2.0, flute_top),
        (cx + body_w / 2.0, flute_top),
        (cx + body_w / 2.0, body_bottom),
    ];
    match b.kind {
        Kind::BallNose => {
            for i in 1..8 {
                let a = std::f32::consts::PI * i as f32 / 8.0;
                shape.push((
                    cx + body_w / 2.0 * a.cos(),
                    body_bottom + body_w / 2.0 * a.sin(),
                ));
            }
        }
        Kind::VBit => {
            shape.push((cx + tip_w / 2.0, tip_y));
            shape.push((cx - tip_w / 2.0, tip_y));
        }
        _ if cone > 0.0 => {
            let half_tip = if matches!(b.kind, Kind::Spot | Kind::Countersink) {
                1.0
            } else {
                0.0
            };
            shape.push((cx + half_tip, tip_y));
            shape.push((cx - half_tip, tip_y));
        }
        _ => {}
    }
    shape.push((cx - body_w / 2.0, body_bottom));
    cv.poly(&shape, fill, stroke);
    // the marks of the flutes, inside the straight part of the body
    let (y0f, y1f) = (flute_top, body_bottom);
    let (xl, xr) = (cx - body_w / 2.0, cx + body_w / 2.0);
    let step = (body_w * 0.9).max(9.0);
    let rise = body_w * 0.5;
    let mut yy = y1f - 3.0;
    let mark = |cv: &mut Cv, y: f32, right_hand: bool, broken: bool| {
        let (a, c) = if right_hand {
            ((xl, y), (xr, y - rise))
        } else {
            ((xl, y - rise), (xr, y))
        };
        if y - rise < y0f {
            return;
        }
        if broken {
            let m = ((a.0 + c.0) / 2.0, (a.1 + c.1) / 2.0);
            let g = (c.0 - a.0) * 0.12;
            let k = (c.1 - a.1) * 0.12;
            cv.line(a, (m.0 - g, m.1 - k), 1.4, line_c);
            cv.line((m.0 + g, m.1 + k), c, 1.4, line_c);
        } else {
            cv.line(a, c, 1.4, line_c);
        }
    };
    match b.kind {
        Kind::Drill | Kind::SlotDrill | Kind::UpCut | Kind::Diamond | Kind::ChipBreaker => {
            while yy - rise > y0f {
                mark(cv, yy, true, b.kind == Kind::ChipBreaker);
                if b.kind == Kind::Diamond {
                    mark(cv, yy, false, false);
                }
                yy -= step;
            }
        }
        Kind::DownCut => {
            while yy - rise > y0f {
                mark(cv, yy, false, false);
                yy -= step;
            }
        }
        Kind::EndMill | Kind::Coated => {
            cv.line(
                (cx - body_w / 4.0, y0f + 2.0),
                (cx - body_w / 4.0, y1f - 2.0),
                1.4,
                line_c,
            );
            cv.line(
                (cx + body_w / 4.0, y0f + 2.0),
                (cx + body_w / 4.0, y1f - 2.0),
                1.4,
                line_c,
            );
        }
        _ => {}
    }
    // the measures: the cutting length on the right, the diameter on the left of the body
    let mx = cx + body_w.max(ws) / 2.0 + 28.0;
    if b.flute_mm > 0.0 {
        cv.line(
            (mx, flute_top),
            (mx, tip_y - if cone > 0.0 { cone } else { 0.0 }),
            1.2,
            col(t.dim),
        );
        cv.line(
            (mx - 5.0, flute_top),
            (mx + 5.0, flute_top),
            1.2,
            col(t.dim),
        );
        cv.line(
            (mx - 5.0, tip_y - if cone > 0.0 { cone } else { 0.0 }),
            (mx + 5.0, tip_y - if cone > 0.0 { cone } else { 0.0 }),
            1.2,
            col(t.dim),
        );
        cv.text(
            mx + 10.0,
            (flute_top + body_bottom) / 2.0 + 6.0,
            Align2::LEFT_BOTTOM,
            &format!("{} {:.1}", app.tr(Text::CatFlute), b.flute_mm),
            14.0,
            col(t.text),
        );
    }
    cv.text(
        cx - body_w.max(ws) / 2.0 - 12.0,
        (flute_top + body_bottom) / 2.0 + 6.0,
        Align2::RIGHT_BOTTOM,
        &format!(
            "Ø {}",
            format!("{:.3}", b.diameter_mm)
                .trim_end_matches('0')
                .trim_end_matches('.')
        ),
        15.0,
        col(t.text),
    );
    if b.kind == Kind::VBit && b.tip_mm > 0.0 {
        cv.text(
            cx + 14.0,
            tip_y - 2.0,
            Align2::LEFT_BOTTOM,
            &format!("{:.2}", b.tip_mm),
            12.0,
            col(t.dim),
        );
    }
}
