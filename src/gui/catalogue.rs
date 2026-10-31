//! The window of the catalogue of bits: a list of drills, slot drills, end mills, routers and V-bits (all with the 1/8 inch shank of a Bungard chuck), by kind, with the numbers a
//! job starts from. A bit that is clicked is drawn on the right, with its measures and what it is for. The list is in [`crate::catalogue`].

use super::extra::wrap;
use super::*;
use crate::catalogue as data;
use crate::defaults::{self, Note, Transfer};
use crate::spindle::{Fit, SpindleId, Why};
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

    /// The spindle the numbers of the catalogue are worked out for: the next one or the one before.
    pub(crate) fn cat_spindle_by(&mut self, steps: i32) {
        let all = SpindleId::ALL;
        let now = self.settings.spindle as i32;
        let next = (now + steps).rem_euclid(all.len() as i32) as usize;
        self.set_spindle(all[next]);
    }

    pub(crate) fn set_spindle(&mut self, s: SpindleId) {
        self.settings.spindle = s;
        self.remember_settings();
    }

    pub(crate) fn spindle_name(&self, s: SpindleId) -> &'static str {
        self.tr(match s {
            SpindleId::Original => Text::SpindleOriginal,
            SpindleId::Vfd => Text::SpindleVfd,
        })
    }

    /// Takes the bit that is selected into the tool the catalogue was opened for: the turns and the feeds that the spindle of the machine can give.
    pub(crate) fn cat_use(&mut self) {
        let Some((p, row)) = self.cat_target else {
            return;
        };
        let bits = self.cat_bits();
        let Some(b) = self.cat_sel.and_then(|i| bits.get(i)).copied() else {
            return;
        };
        if b.kind.is_drill() != (p == Program::Drill) {
            return;
        }
        let cell = |this: &Self, c: usize| -> Option<f64> {
            this.used_tools(p)
                .get(row)
                .and_then(|r| r.cells.get(c))
                .and_then(|s| s.trim().parse::<f64>().ok())
        };
        let Some(t) = self.cat_transfer(&b, cell(self, 0)) else {
            return;
        };
        match p {
            Program::Drill => {
                let feed = if t.plunge_mm_min > 0 {
                    t.plunge_mm_min
                } else {
                    t.feed_mm_min
                };
                self.tool_set(p, row, 1, feed as f64);
                self.tool_set(p, row, 2, t.rpm as f64);
            }
            Program::Route => {
                self.tool_set(p, row, 0, t.depth_mm);
                if let Some(passes) = t.passes {
                    self.tool_set(p, row, 1, passes as f64);
                }
                self.tool_set(p, row, 2, t.feed_mm_min as f64);
                self.tool_set(p, row, 3, t.plunge_mm_min as f64);
                self.tool_set(p, row, 4, t.rpm as f64);
            }
        }
        if let Some(tool) = self.used_tools(p).get(row).and_then(|r| r.name) {
            self.tool_bits.retain(|(q, n, _)| (*q, *n) != (p, tool));
            self.tool_bits.push((p, tool, b));
        }
        self.cat_target = None;
        self.show_modal(Modal::Tools);
    }

    /// The numbers a bit gives a tool for the spindle and the board of the job: `current` is the value now in the first cell of the tool (the extra stroke or the depth).
    pub(crate) fn cat_transfer(&self, b: &data::Bit, current: Option<f64>) -> Option<Transfer> {
        let current = current.unwrap_or(if b.kind.is_drill() {
            defaults::drill_extra(b.diameter_mm)
        } else {
            0.0
        });
        defaults::transfer(
            b,
            &self.settings.current_spindle(),
            self.board.material().core,
            self.board.thickness_mm,
            current,
        )
    }

    /// Whether the bit that a tool took from the catalogue is not suitable now (the spindle cannot work with it, the material asks for less than the spindle turns, or the
    /// cutting part does not go through the board and the extra).
    pub(crate) fn tool_bit_unsuitable(&self, p: Program, tool: u8) -> bool {
        let Some((_, _, b)) = self
            .tool_bits
            .iter()
            .find(|(q, n, _)| (*q, *n) == (p, tool))
        else {
            return false;
        };
        let current = self
            .used_tools(p)
            .iter()
            .find(|r| r.name == Some(tool))
            .and_then(|r| r.cells.first())
            .and_then(|s| s.trim().parse::<f64>().ok());
        self.cat_transfer(b, current)
            .is_none_or(|t| t.is_unsuitable())
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
        // the spindle the numbers are worked out for
        cv.text(
            x + 700.0,
            y + 136.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{}: {}",
                self.tr(Text::CatSpindleLbl),
                self.spindle_name(self.settings.spindle)
            ),
            16.0,
            col(t.text),
        );
        cv.icon_chip(
            x + w - 150.0,
            y + 108.0,
            56.0,
            38.0,
            Icon::Back,
            false,
            "",
            Act::CatSpindle(-1),
        );
        cv.icon_chip(
            x + w - 86.0,
            y + 108.0,
            56.0,
            38.0,
            Icon::Next,
            false,
            "",
            Act::CatSpindle(1),
        );
        // the columns
        let cols = [
            ("", 30.0),
            (self.tr(Text::CatSeries), 225.0),
            ("Ø mm", 295.0),
            ("L mm", 365.0),
            (&format!("{} mm", self.tr(Text::CatTip)), 430.0),
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
        paint_bit(cv, b, x, top, w, 230.0, self);
        let spindle = self.settings.current_spindle();
        let fit = spindle.fit(b);
        let mine = match fit {
            Fit::Adjusted { bit, .. } => bit,
            _ => *b,
        };
        // the numbers for the spindle and the board of the job
        let current = self
            .cat_target
            .filter(|(p, _)| b.kind.is_drill() == (*p == Program::Drill))
            .and_then(|(p, row)| {
                self.used_tools(p)
                    .get(row)
                    .and_then(|r| r.cells.first())
                    .and_then(|s| s.trim().parse::<f64>().ok())
            });
        let plan = self.cat_transfer(b, current);
        let mine = plan.as_ref().map_or(mine, |t| data::Bit {
            rpm: t.rpm,
            feed_mm_min: t.feed_mm_min,
            plunge_mm_min: t.plunge_mm_min,
            ..mine
        });
        let mut ly = top + 262.0;
        macro_rules! line {
            ($cv:expr, $s:expr, $c:expr $(,)?) => {{
                let s: String = $s;
                $cv.text(x, ly, Align2::LEFT_BOTTOM, &s, 15.0, $c);
                ly += 22.0;
            }};
        }
        line!(
            cv,
            format!(
                "{} 3.175   {} 38.1",
                self.tr(Text::CatShankLbl),
                self.tr(Text::CatTotal)
            ),
            col(t.text),
        );
        if b.angle_deg > 0 {
            line!(
                cv,
                format!("{} {}°", self.tr(Text::CatPoint), b.angle_deg),
                col(t.text),
            );
        }
        line!(
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
        if mine.rpm != b.rpm
            || mine.feed_mm_min != b.feed_mm_min
            || mine.plunge_mm_min != b.plunge_mm_min
        {
            line!(
                cv,
                format!("({} rpm: {} / {})", b.rpm, b.feed_mm_min, b.plunge_mm_min),
                col(t.dim),
            );
        }
        // the material of the board: the factors that were applied, the depth and the passes, and what to look at
        if let Some(p) = &plan {
            line!(
                cv,
                format!(
                    "{} {}  rpm x{:.2}  mm/min x{:.2}",
                    self.tr(Text::CatBoardApplied),
                    self.board.material().core.name(),
                    p.rpm_factor,
                    p.feed_factor
                ),
                col(t.text),
            );
            let mut depth = format!("{}: {:.2}", self.tr(Text::LblDepthMm), p.depth_mm);
            if let Some(n) = p.passes {
                depth.push_str(&format!("   {}: {n}", self.tr(Text::LblPasses)));
            }
            line!(cv, depth, col(t.text));
            for n in &p.notes {
                let text = match n {
                    Note::LowerRpm => Text::CatNoteLowerRpm,
                    Note::BelowSpindle => Text::CatNoteBelowSpindle,
                    Note::FluteShort => Text::CatNoteFluteShort,
                };
                for l in wrap(self.tr(text), 58) {
                    cv.text(x, ly, Align2::LEFT_BOTTOM, &l, 13.0, col(t.warn));
                    ly += 17.0;
                }
            }
        }
        // whether this spindle can work with the bit, and what it needs to
        let notice: Vec<(Text, Color32)> = match fit {
            Fit::Fits => vec![(Text::FitOk, col(t.ok))],
            Fit::Adjusted {
                lower_rpm,
                xy_slower,
                z_slower,
                ..
            } => {
                let mut v = vec![(Text::FitSlower, col(t.warn))];
                if lower_rpm {
                    v.push((Text::FitLowerRpm, col(t.warn)));
                }
                if xy_slower {
                    v.push((Text::FitSlowXy, col(t.warn)));
                }
                if z_slower {
                    v.push((Text::FitSlowZ, col(t.warn)));
                }
                v
            }
            Fit::Incompatible(Why::TooThick) => vec![(Text::FitNoThick, col(t.danger))],
            Fit::Incompatible(Why::TooSlow) => vec![(Text::FitNoSlow, col(t.danger))],
        };
        for (text, c) in notice {
            for l in wrap(self.tr(text), 52) {
                cv.text(x, ly, Align2::LEFT_BOTTOM, &l, 14.0, c);
                ly += 19.0;
            }
        }
        // taking the bit into the tool the catalogue was opened for
        if let Some((p, row)) = self.cat_target {
            let tool = self
                .used_tools(p)
                .get(row)
                .and_then(|r| r.name)
                .unwrap_or(0);
            let can =
                b.kind.is_drill() == (p == Program::Drill) && !matches!(fit, Fit::Incompatible(_));
            if can {
                cv.chip(
                    x,
                    ly + 6.0,
                    w.min(420.0),
                    42.0,
                    &format!("{} T{tool:02}", self.tr(Text::CatUseFor)),
                    true,
                    Act::CatUse,
                );
                ly += 56.0;
            }
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
