//! The drop-down lists (combo boxes): where a choice has many values, as the theme, the language, the model of the machine or the sound of an event, the
//! window shows one box with the value that is chosen and, when it is clicked, a list of a few rows that scrolls with the wheel; a click on a row chooses it and
//! a click anywhere else closes the list. This replaces rows and rows of buttons, one for each value.

use super::board::{lang_names, BoardPick};
use super::*;
use eh3d_drillpro::job::TabPlacement;
use eh3d_drillpro::protocol::sound::{self, Cue, SoundSetting};

/// The lists of the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ComboId {
    Theme,
    Lang,
    Model,
    Sound(Cue),
    /// Where the tabs of the panel go.
    TabPlace,
    /// The board: the maker of the material, its thickness and its size.
    Maker,
    Thickness,
    BoardSize,
    /// The kind of bit the catalogue shows.
    CatKind,
}

/// A list that is open, and the first row it shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Combo {
    pub id: ComboId,
    pub scroll: usize,
}

/// How many rows the open list shows at once, and how high each one is.
pub(crate) const ROWS: usize = 8;
const ROW_H: f32 = 36.0;
/// What comes first in the list of a sound: no sound, the standard tone and a file of the person's; then the spoken voices that are recorded, then the sounds of the catalogue.
const SOUND_FIXED: usize = 3;

impl App {
    /// The spoken voices that are recorded in the folder of the voices: (language, female).
    pub(crate) fn voices(&self) -> Vec<(&'static str, bool)> {
        sound::voices_dir()
            .map(|d| sound::available_voices(&d))
            .unwrap_or_default()
    }

    /// The name of a voice in the list: `Voice - Español (mujer)`.
    fn voice_label(&self, lang: &str, female: bool) -> String {
        let name = lang_names()
            .iter()
            .find(|(l, _)| l.code() == lang)
            .map_or(lang, |(_, n)| *n);
        let who = if female {
            Text::SndFemale
        } else {
            Text::SndMale
        };
        format!("{} - {} ({})", self.tr(Text::SndVoice), name, self.tr(who))
    }

    /// The rows of a list.
    pub(crate) fn combo_items(&self, id: ComboId) -> Vec<String> {
        match id {
            ComboId::Theme => ThemeId::ALL
                .into_iter()
                .map(|t| self.tr(t.text()).to_string())
                .collect(),
            ComboId::Lang => lang_names().iter().map(|(_, n)| n.to_string()).collect(),
            ComboId::Model => crate::machines::models().iter().map(|m| m.name()).collect(),
            ComboId::Maker => crate::boards::makers()
                .iter()
                .map(|m| m.to_string())
                .collect(),
            ComboId::Thickness => self
                .board
                .material()
                .thickness_mm
                .iter()
                .map(|v| format!("{v} mm"))
                .collect(),
            ComboId::BoardSize => {
                let mut v = vec![self.tr(Text::LblFromFiles).to_string()];
                v.extend(
                    self.board
                        .material()
                        .formats_mm
                        .iter()
                        .map(|(w, d)| format!("{w:.0} x {d:.0} mm")),
                );
                v
            }
            ComboId::CatKind => {
                let mut v = vec![self.tr(Text::CatAll).to_string()];
                v.extend(
                    super::catalogue::KINDS
                        .iter()
                        .map(|k| self.tr(k.1).to_string()),
                );
                v
            }
            ComboId::TabPlace => [Text::TabEven, Text::TabEnds, Text::TabMiddle]
                .into_iter()
                .map(|t| self.tr(t).to_string())
                .collect(),
            ComboId::Sound(cue) => {
                let mut v = vec![
                    self.tr(Text::SndOff).to_string(),
                    self.tr(Text::SndDefault).to_string(),
                    self.tr(Text::SndFile).to_string(),
                ];
                v.extend(
                    self.voices()
                        .into_iter()
                        .map(|(l, f)| self.voice_label(l, f)),
                );
                v.extend((0..sound::TONES).map(|k| self.tone_label(cue, k)));
                v
            }
        }
    }

    /// The name of a sound of the catalogue: its number, its pitch and its pattern (`07 · C7 · rising notes`).
    pub(crate) fn tone_label(&self, cue: Cue, k: usize) -> String {
        let (pitch, pattern) = sound::tone_parts(k);
        let words = match cue {
            Cue::Finished | Cue::CheckOk => [
                Text::SndChime,
                Text::SndRise,
                Text::SndFall,
                Text::SndFanfare,
                Text::SndTriple,
                Text::SndBell,
            ],
            Cue::ToolChange | Cue::CheckNotOk => [
                Text::SndBeep,
                Text::SndDouble,
                Text::SndTripleBeep,
                Text::SndAlarm,
                Text::SndKnock,
                Text::SndSiren,
            ],
        };
        format!(
            "{:02} \u{b7} {} \u{b7} {}",
            k + 1,
            sound::PITCH_NAMES[pitch],
            self.tr(words[pattern])
        )
    }

    /// The row that is chosen in a list.
    pub(crate) fn combo_current(&self, id: ComboId) -> usize {
        match id {
            ComboId::Theme => ThemeId::ALL
                .iter()
                .position(|t| *t == self.theme.id)
                .unwrap_or(0),
            ComboId::Lang => lang_names()
                .iter()
                .position(|(l, _)| *l == self.lang)
                .unwrap_or(0),
            ComboId::Model => self.model(),
            ComboId::Maker => self.maker,
            ComboId::CatKind => self.cat_kind,
            ComboId::Thickness => self
                .board
                .material()
                .thickness_mm
                .iter()
                .position(|v| (self.board.thickness_mm - v).abs() < 1e-9)
                .unwrap_or(0),
            ComboId::BoardSize => self.board.size_mm.map_or(0, |s| {
                self.board
                    .material()
                    .formats_mm
                    .iter()
                    .position(|f| *f == s)
                    .map_or(0, |i| i + 1)
            }),
            ComboId::TabPlace => match self.panel.tab_place {
                TabPlacement::Even => 0,
                TabPlacement::Ends => 1,
                TabPlacement::Middle => 2,
            },
            ComboId::Sound(cue) => match self.sound_of(cue) {
                SoundSetting::Off => 0,
                SoundSetting::Default => 1,
                SoundSetting::File(_) => 2,
                SoundSetting::Voice { lang, female } => self
                    .voices()
                    .iter()
                    .position(|(l, f)| *l == lang && f == female)
                    .map_or(0, |i| SOUND_FIXED + i),
                SoundSetting::Tone(k) => SOUND_FIXED + self.voices().len() + *k as usize,
            },
        }
    }

    pub(crate) fn sound_of(&self, cue: Cue) -> &SoundSetting {
        match cue {
            Cue::Finished => &self.conn.sound_finish,
            Cue::ToolChange => &self.conn.sound_tool_change,
            Cue::CheckOk => &self.conn.sound_check_ok,
            Cue::CheckNotOk => &self.conn.sound_check_bad,
        }
    }

    pub(crate) fn combo_open(&mut self, id: ComboId) {
        if self.combo.is_some_and(|c| c.id == id) {
            self.combo = None;
            return;
        }
        let n = self.combo_items(id).len();
        let current = self.combo_current(id);
        // the list opens with the row that is chosen in view
        let scroll = current.saturating_sub(ROWS / 2).min(n.saturating_sub(ROWS));
        self.combo = Some(Combo { id, scroll });
    }

    pub(crate) fn combo_scroll(&mut self, delta: i32) {
        if let Some(c) = self.combo {
            let max = self.combo_items(c.id).len().saturating_sub(ROWS);
            let scroll = (c.scroll as i64 + delta as i64).clamp(0, max as i64) as usize;
            self.combo = Some(Combo { scroll, ..c });
        }
    }

    /// A row of the open list is chosen: the list closes and the choice is made.
    pub(crate) fn combo_pick(&mut self, row: usize) {
        let Some(c) = self.combo.take() else { return };
        if row >= self.combo_items(c.id).len() {
            return;
        }
        match c.id {
            ComboId::Theme => self.set_theme(ThemeId::ALL[row]),
            ComboId::Lang => self.set_lang(lang_names()[row].0),
            ComboId::Model => self.apply_model(row),
            ComboId::Maker => self.board_pick(BoardPick::Maker(row)),
            ComboId::CatKind => {
                self.cat_kind = row;
                self.cat_scroll = 0;
                self.cat_sel = None;
            }
            ComboId::Thickness => self.board_pick(BoardPick::Thickness(row)),
            ComboId::BoardSize => self.board_pick(BoardPick::Size(row.checked_sub(1))),
            ComboId::TabPlace => {
                self.panel.tab_place =
                    [TabPlacement::Even, TabPlacement::Ends, TabPlacement::Middle][row];
                self.saved = false;
                self.apply_panel();
            }
            ComboId::Sound(cue) => {
                let setting = match row {
                    0 => SoundSetting::Off,
                    1 => SoundSetting::Default,
                    2 => SoundSetting::File(PathBuf::new()),
                    k => {
                        let voices = self.voices();
                        match k.checked_sub(SOUND_FIXED).filter(|i| *i < voices.len()) {
                            Some(i) => SoundSetting::Voice {
                                lang: voices[i].0.to_string(),
                                female: voices[i].1,
                            },
                            None => SoundSetting::Tone((k - SOUND_FIXED - voices.len()) as u8),
                        }
                    }
                };
                self.sound_choose(cue, setting);
            }
        }
    }

    /// The list that is open is drawn over everything, with a sheet over the rest of the window that closes it when it is clicked.
    pub(super) fn paint_combo(&self, cv: &mut Cv) {
        let Some(c) = self.combo else { return };
        let Some(&(_, ax, ay, aw)) = cv.combos.iter().find(|a| a.0 == c.id) else {
            return;
        };
        let items = self.combo_items(c.id);
        let current = self.combo_current(c.id);
        let shown = items.len().min(ROWS);
        let (w, h) = (aw.max(300.0), shown as f32 * ROW_H + 12.0);
        // under the box, or over it when it would leave the window
        let y = if ay + h > DESIGN_H - 10.0 {
            (ay - 42.0 - h).max(10.0)
        } else {
            ay
        };
        let t = cv.t.clone();
        cv.modal_from = cv.hits.len();
        cv.hits
            .push((cv.rect(0.0, 0.0, DESIGN_W, DESIGN_H), Act::ComboClose));
        cv.p.rect_filled(
            cv.rect(ax, y, w, h).translate(Vec2::new(0.0, 4.0 * cv.u)),
            4.0 * cv.u,
            alpha(Color32::BLACK, 90),
        );
        cv.card(ax, y, w, h, true, None);
        for r in 0..shown {
            let i = c.scroll + r;
            let ry = y + 6.0 + r as f32 * ROW_H;
            let rect = cv.rect(ax + 6.0, ry, w - 22.0, ROW_H - 2.0);
            let (hover, _) = cv.touch(rect);
            if i == current {
                cv.p.rect_filled(rect, 3.0 * cv.u, alpha(col(t.accent), 70));
            } else if hover {
                cv.p.rect_filled(rect, 3.0 * cv.u, alpha(col(t.accent), 28));
            }
            cv.text(
                ax + 16.0,
                ry + ROW_H - 10.0,
                Align2::LEFT_BOTTOM,
                &items[i],
                16.0,
                col(t.text),
            );
            cv.hits.push((rect, Act::ComboPick(i)));
        }
        // the bar at the side shows where the list is, and a click or a drag on it moves the list
        cv.scrollbar(
            Bar::Combo,
            ax + w - 16.0,
            y + 6.0,
            h - 12.0,
            (ax, y, w, h),
            items.len(),
            shown,
            c.scroll,
        );
    }
}

impl Cv<'_> {
    /// The box of a list: the value that is chosen, an arrow, and a click that opens the list (the list itself is drawn last, over the rest).
    pub(crate) fn combo(&mut self, x: f32, y: f32, w: f32, id: ComboId, current: &str, open: bool) {
        self.chip(x, y, w, 38.0, current, open, Act::ComboOpen(id));
        let (cx, cy) = (x + w - 22.0, y + 19.0);
        self.poly(
            &[(cx - 6.0, cy - 3.0), (cx + 6.0, cy - 3.0), (cx, cy + 5.0)],
            col(self.t.text),
            Stroke::NONE,
        );
        self.combos.push((id, x, y + 40.0, w));
    }
}
