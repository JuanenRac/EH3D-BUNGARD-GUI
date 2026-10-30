//! The window of the catalogue of bits: a list of drills, end mills, routers and V-bits (all with the 1/8 inch shank of a Bungard chuck), by kind, with the numbers a job
//! starts from. The list is in [`crate::catalogue`].

use super::extra::wrap;
use super::*;
use crate::catalogue as data;
use data::Kind;

/// The kinds the list can be limited to, with the words that name them.
pub(super) const KINDS: [(Kind, Text); 9] = [
    (Kind::Drill, Text::KindDrill),
    (Kind::EndMill, Text::KindEndMill),
    (Kind::DownCut, Text::KindDownCut),
    (Kind::UpCut, Text::KindUpCut),
    (Kind::Diamond, Text::KindDiamond),
    (Kind::VBit, Text::KindVBit),
    (Kind::BallNose, Text::KindBall),
    (Kind::Spot, Text::KindSpot),
    (Kind::Countersink, Text::KindCountersink),
];

/// How many rows the list shows at once.
const SHOWN: usize = 12;
const ROW_H: f32 = 40.0;

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

    fn kind_name(&self, k: Kind) -> &'static str {
        let text = KINDS
            .iter()
            .find(|x| x.0 == k)
            .map_or(Text::CatAll, |x| x.1);
        self.tr(text)
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
        // the columns
        let cols = [
            ("", 30.0),
            ("Ø mm", 520.0),
            ("tip mm", 620.0),
            ("°", 730.0),
            ("z", 800.0),
            ("rpm", 880.0),
            ("mm/min", 990.0),
            ("Z mm/min", 1110.0),
            ("mm / pass", 1240.0),
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
            if r % 2 == 0 {
                cv.p.rect_filled(
                    cv.rect(x + 20.0, ry, w - 40.0, ROW_H - 4.0),
                    3.0 * cv.u,
                    alpha(col(t.panel_raised), 255),
                );
            }
            let base = ry + ROW_H - 14.0;
            let put = |cv: &mut Cv, cx: f32, s: &str| {
                cv.text(x + cx, base, Align2::LEFT_BOTTOM, s, 16.0, col(t.text));
            };
            put(cv, 30.0, self.kind_name(b.kind));
            put(cv, 520.0, &format!("{:.2}", b.diameter_mm));
            put(
                cv,
                620.0,
                &if b.tip_mm > 0.0 {
                    format!("{:.2}", b.tip_mm)
                } else {
                    "-".into()
                },
            );
            put(
                cv,
                730.0,
                &if b.angle_deg > 0 {
                    format!("{}", b.angle_deg)
                } else {
                    "-".into()
                },
            );
            put(
                cv,
                800.0,
                &if b.flutes > 0 {
                    format!("{}", b.flutes)
                } else {
                    "-".into()
                },
            );
            put(cv, 880.0, &format!("{}", b.rpm));
            put(
                cv,
                990.0,
                &if b.feed_mm_min > 0 {
                    format!("{}", b.feed_mm_min)
                } else {
                    "-".into()
                },
            );
            put(cv, 1110.0, &format!("{}", b.plunge_mm_min));
            put(cv, 1240.0, &format!("{:.2}", b.pass_mm));
        }
        // the scroll of the list
        if bits.len() > SHOWN {
            cv.chip(
                x + w - 70.0,
                top + 30.0,
                40.0,
                40.0,
                "^",
                false,
                Act::CatScroll(-(SHOWN as i32)),
            );
            cv.chip(
                x + w - 70.0,
                top + 30.0 + SHOWN as f32 * ROW_H - 44.0,
                40.0,
                40.0,
                "v",
                false,
                Act::CatScroll(SHOWN as i32),
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
}
