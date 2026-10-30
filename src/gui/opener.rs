//! The Open window, made like the one of any Windows program: the places on the left (the examples, the desktop, the documents, the
//! downloads and the drives), the path above, the list of the folder with its name, date, type and size (a click on a heading sorts by
//! it), the file name and the type of file below, and Open and Cancel.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    Folder,
    Drill,
    Route,
    Zip,
    Other,
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub name: String,
    pub kind: EntryKind,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

/// The type of file the list shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Filter {
    Supported,
    Drill,
    Route,
    Zip,
    Any,
}

impl Filter {
    const ALL: [Filter; 5] = [
        Filter::Supported,
        Filter::Drill,
        Filter::Route,
        Filter::Zip,
        Filter::Any,
    ];

    fn next(self) -> Filter {
        let i = Filter::ALL.iter().position(|f| *f == self).unwrap_or(0);
        Filter::ALL[(i + 1) % Filter::ALL.len()]
    }

    fn shows(self, kind: EntryKind) -> bool {
        match self {
            Filter::Supported => kind != EntryKind::Other,
            Filter::Drill => kind == EntryKind::Drill,
            Filter::Route => kind == EntryKind::Route,
            Filter::Zip => kind == EntryKind::Zip,
            Filter::Any => true,
        }
    }

    fn text(self) -> Text {
        match self {
            Filter::Supported => Text::FilterSupported,
            Filter::Drill => Text::FilterDrill,
            Filter::Route => Text::FilterRoute,
            Filter::Zip => Text::FilterZip,
            Filter::Any => Text::FilterAny,
        }
    }
}

/// A place on the left of the window.
#[derive(Debug, Clone)]
pub(crate) struct Place {
    pub label: Option<Text>,
    pub name: String,
    pub path: PathBuf,
}

/// The folder being looked at and what is chosen in it.
pub(crate) struct Browser {
    /// `None` is the list of drives.
    pub dir: Option<PathBuf>,
    pub entries: Vec<Entry>,
    pub scroll: usize,
    pub sel: Option<usize>,
    /// The column sorted by (0 name, 1 date, 2 type, 3 size) and whether it goes up.
    pub sort: (usize, bool),
    pub filter: Filter,
    pub history: Vec<Option<PathBuf>>,
    pub path_text: String,
    pub name_text: String,
    pub places: Vec<Place>,
    /// The row and the moment of the last click, for the double click.
    pub last_click: Option<(usize, Instant)>,
}

/// The rows the list shows at once.
pub(crate) const ROWS: usize = 16;

fn kind_of(name: &str, dir: bool) -> EntryKind {
    if dir {
        return EntryKind::Folder;
    }
    if name.to_ascii_lowercase().ends_with(".zip") {
        return EntryKind::Zip;
    }
    match eh3d_archive::kind_of(name) {
        Kind::Drill => EntryKind::Drill,
        Kind::Route => EntryKind::Route,
        Kind::Other => EntryKind::Other,
    }
}

/// A date as Windows writes it, in universal time (the standard library has no time zones).
pub(crate) fn date_text(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // the civil date of a day count (the algorithm of Howard Hinnant)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        rem / 3600,
        rem % 3600 / 60
    )
}

pub(crate) fn size_text(n: u64) -> String {
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        format!("{:.1} KB", n as f64 / 1024.0)
    } else {
        format!("{:.1} MB", n as f64 / 1_048_576.0)
    }
}

impl Browser {
    /// Reads the folder again with the filter and the sort of now.
    pub fn refresh(&mut self) {
        self.scroll = 0;
        self.sel = None;
        let mut all = Vec::new();
        match &self.dir {
            None => {
                for l in b'A'..=b'Z' {
                    let d = format!("{}:\\", l as char);
                    if Path::new(&d).exists() {
                        all.push(Entry {
                            name: d,
                            kind: EntryKind::Folder,
                            size: 0,
                            modified: None,
                        });
                    }
                }
            }
            Some(d) => {
                if let Ok(rd) = std::fs::read_dir(d) {
                    for e in rd.flatten() {
                        let name = e.file_name().to_string_lossy().to_string();
                        let meta = e.metadata().ok();
                        let dir = e.path().is_dir();
                        if name.starts_with('.') && dir {
                            continue;
                        }
                        let kind = kind_of(&name, dir);
                        if kind != EntryKind::Folder && !self.filter.shows(kind) {
                            continue;
                        }
                        all.push(Entry {
                            name,
                            kind,
                            size: meta.as_ref().map_or(0, |m| m.len()),
                            modified: meta.and_then(|m| m.modified().ok()),
                        });
                    }
                }
            }
        }
        let (col, up) = self.sort;
        all.sort_by(|a, b| {
            // the folders come first, whatever the sort
            let by = (b.kind == EntryKind::Folder).cmp(&(a.kind == EntryKind::Folder));
            by.then_with(|| {
                let o = match col {
                    1 => a.modified.cmp(&b.modified),
                    2 => (a.kind as u8).cmp(&(b.kind as u8)),
                    3 => a.size.cmp(&b.size),
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                };
                if up {
                    o
                } else {
                    o.reverse()
                }
            })
        });
        self.entries = all;
        self.path_text = self
            .dir
            .as_ref()
            .map_or_else(String::new, |d| d.display().to_string());
    }
}

/// The places of the window: the examples next to the program, the folders of the user and the drives.
fn places(base: &Path, last: &Path) -> Vec<Place> {
    let mut v = Vec::new();
    let examples = base.join("examples");
    if examples.is_dir() {
        v.push(Place {
            label: Some(Text::PlaceExamples),
            name: String::new(),
            path: examples,
        });
    } else if last.is_dir() {
        v.push(Place {
            label: Some(Text::PlaceExamples),
            name: String::new(),
            path: last.to_path_buf(),
        });
    }
    if let Some(home) = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")) {
        let home = PathBuf::from(home);
        for (label, sub) in [
            (Text::PlaceDesktop, "Desktop"),
            (Text::PlaceDocuments, "Documents"),
            (Text::PlaceDownloads, "Downloads"),
        ] {
            let p = home.join(sub);
            if p.is_dir() {
                v.push(Place {
                    label: Some(label),
                    name: String::new(),
                    path: p,
                });
            }
        }
    }
    for l in b'A'..=b'Z' {
        let d = format!("{}:\\", l as char);
        if Path::new(&d).exists() {
            v.push(Place {
                label: None,
                name: format!("{}:", l as char),
                path: PathBuf::from(d),
            });
        }
    }
    v
}

impl App {
    /// The browser of a fresh Open window: it starts where the last file was.
    pub(super) fn new_browser(&self) -> Browser {
        let mut b = Browser {
            dir: Some(self.last_dir.clone()),
            entries: Vec::new(),
            scroll: 0,
            sel: None,
            sort: (0, true),
            filter: Filter::Supported,
            history: Vec::new(),
            path_text: String::new(),
            name_text: String::new(),
            places: places(&self.base_dir, &self.last_dir),
            last_click: None,
        };
        b.refresh();
        b
    }

    fn go(&mut self, dir: Option<PathBuf>) {
        if let Some(b) = self.browser.as_mut() {
            if let Some(d) = &dir {
                if !d.is_dir() {
                    return;
                }
            }
            b.history.push(b.dir.clone());
            b.dir = dir;
            b.name_text.clear();
            b.refresh();
        }
    }

    pub(super) fn browse_up(&mut self) {
        let next = self.browser.as_ref().and_then(|b| match &b.dir {
            Some(d) => {
                Some(d.parent().map(Path::to_path_buf)).filter(|p| p.is_some() || cfg!(windows))
            }
            None => None,
        });
        if let Some(n) = next {
            self.go(n);
        }
    }

    pub(super) fn browse_back(&mut self) {
        if let Some(b) = self.browser.as_mut() {
            if let Some(prev) = b.history.pop() {
                b.dir = prev;
                b.name_text.clear();
                b.refresh();
            }
        }
    }

    pub(super) fn browse_place(&mut self, i: usize) {
        let p = self
            .browser
            .as_ref()
            .and_then(|b| b.places.get(i))
            .map(|p| p.path.clone());
        if let Some(p) = p {
            self.go(Some(p));
        }
    }

    pub(super) fn browse_sort(&mut self, col: usize) {
        if let Some(b) = self.browser.as_mut() {
            b.sort = if b.sort.0 == col {
                (col, !b.sort.1)
            } else {
                (col, true)
            };
            b.refresh();
        }
    }

    pub(super) fn browse_filter(&mut self) {
        if let Some(b) = self.browser.as_mut() {
            b.filter = b.filter.next();
            b.refresh();
        }
    }

    pub(crate) fn browse_scroll(&mut self, delta: i32) {
        if let Some(b) = self.browser.as_mut() {
            let max = b.entries.len().saturating_sub(ROWS);
            b.scroll = (b.scroll as i64 + delta as i64).clamp(0, max as i64) as usize;
        }
    }

    /// What is done with an entry: a folder (or a drive) is entered, a file is opened.
    fn browse_enter(&mut self, e: &Entry) {
        let Some(dir) = self.browser.as_ref().map(|b| b.dir.clone()) else {
            return;
        };
        let path = match &dir {
            Some(d) => d.join(&e.name),
            None => PathBuf::from(&e.name),
        };
        if e.kind == EntryKind::Folder {
            self.go(Some(path));
        } else {
            self.modal = None;
            self.open_path_asking(&path);
        }
    }

    /// A click on a row: it is chosen and its name goes to the name box; a second click on the same one opens it.
    pub(crate) fn browse(&mut self, i: usize) {
        let now = Instant::now();
        let Some(b) = self.browser.as_mut() else {
            return;
        };
        let Some(e) = b.entries.get(i).cloned() else {
            return;
        };
        let twice = b
            .last_click
            .is_some_and(|(row, at)| row == i && now.duration_since(at).as_millis() < 500);
        b.sel = Some(i);
        b.name_text = e.name.clone();
        b.last_click = Some((i, now));
        if twice {
            self.browse_enter(&e);
        }
    }

    /// The Open button, or Enter in the name box: the chosen file, the name typed, or a folder typed.
    pub(super) fn browse_open(&mut self) {
        let Some(b) = self.browser.as_ref() else {
            return;
        };
        let typed = b.name_text.trim().trim_matches('"').to_string();
        if typed.is_empty() {
            if let Some(e) = b.sel.and_then(|i| b.entries.get(i)).cloned() {
                self.browse_enter(&e);
            }
            return;
        }
        let base = b.dir.clone().unwrap_or_default();
        let path = if Path::new(&typed).is_absolute() {
            PathBuf::from(&typed)
        } else {
            base.join(&typed)
        };
        if path.is_dir() {
            self.go(Some(path));
        } else if path.is_file() {
            self.modal = None;
            self.open_path_asking(&path);
        } else {
            self.message = Some(format!("{typed}: {}", self.tr(Text::NoFile)));
        }
    }

    /// Enter in the path box: the folder typed there.
    fn browse_path(&mut self) {
        let typed = self
            .browser
            .as_ref()
            .map(|b| b.path_text.trim().trim_matches('"').to_string());
        if let Some(t) = typed {
            let p = PathBuf::from(&t);
            if p.is_dir() {
                self.go(Some(p));
            } else if let Some(b) = self.browser.as_mut() {
                // a path that is not a folder is put back as it was
                b.path_text = b
                    .dir
                    .as_ref()
                    .map_or_else(String::new, |d| d.display().to_string());
            }
        }
    }

    /// The path box and the name box, over the window (the canvas has no text boxes of its own).
    pub(super) fn open_fields(&mut self, ctx: &egui::Context) {
        let u = self.ui_scale;
        let at = |x: f32, y: f32| Pos2::new(self.ui_origin.x + x * u, self.ui_origin.y + y * u);
        let (x, y) = (WIN.0, WIN.1);
        let mut enter_path = false;
        let mut enter_name = false;
        if let Some(b) = self.browser.as_mut() {
            egui::Area::new(egui::Id::unique("open-path"))
                .fixed_pos(at(x + 150.0, y + 66.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut b.path_text)
                            .desired_width(1090.0 * u)
                            .font(FontId::proportional(17.0 * u)),
                    );
                    enter_path = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                });
            egui::Area::new(egui::Id::unique("open-name"))
                .fixed_pos(at(x + 440.0, y + 686.0))
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut b.name_text)
                            .desired_width(360.0 * u)
                            .font(FontId::proportional(17.0 * u)),
                    );
                    enter_name = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                });
        }
        if enter_path {
            self.browse_path();
        }
        if enter_name {
            self.browse_open();
        }
    }

    fn type_text(&self, k: EntryKind) -> &'static str {
        match k {
            EntryKind::Folder => self.tr(Text::TypeFolder),
            EntryKind::Drill => self.tr(Text::Drilling),
            EntryKind::Route => self.tr(Text::Routing),
            EntryKind::Zip => self.tr(Text::TypeZip),
            EntryKind::Other => "",
        }
    }

    pub(super) fn paint_open_modal(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        let Some(b) = &self.browser else {
            return;
        };
        let (x, y, w, h) = (WIN.0, WIN.1, 1360.0, 780.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::ZERO, Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 40.0, self.tr(Text::OpenFile));
        // the toolbar: back, up and the path
        cv.chip(x + 30.0, y + 62.0, 54.0, 38.0, "<", false, Act::BrowseBack);
        cv.chip(x + 90.0, y + 62.0, 54.0, 38.0, "^", false, Act::BrowseUp);
        // the places
        cv.text(
            x + 30.0,
            y + 132.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::Places).to_uppercase(),
            12.0,
            col(t.dim),
        );
        for (i, p) in b.places.iter().take(12).enumerate() {
            let ry = y + 144.0 + i as f32 * 38.0;
            let on = b.dir.as_ref() == Some(&p.path);
            let r = cv.rect(x + 30.0, ry, 200.0, 32.0);
            cv.p.rect_filled(
                r,
                6.0 * cv.u,
                if on {
                    alpha(col(t.accent), 90)
                } else {
                    col(t.panel_raised)
                },
            );
            let label = p.label.map_or_else(
                || format!("{}  {}", self.tr(Text::ThisPc), p.name),
                |l| self.tr(l).to_string(),
            );
            let label = if p.label.is_none() {
                p.name.clone()
            } else {
                label
            };
            cv.text(
                x + 44.0,
                ry + 23.0,
                Align2::LEFT_BOTTOM,
                &label,
                15.0,
                col(t.text),
            );
            cv.hits.push((r, Act::BrowsePlace(i)));
        }
        // the files opened last
        let recent: Vec<(usize, &String)> = self
            .settings
            .recent
            .iter()
            .enumerate()
            .filter(|(_, p)| Path::new(p.as_str()).exists())
            .collect();
        if !recent.is_empty() {
            let top = y + 144.0 + b.places.len().min(12) as f32 * 38.0 + 22.0;
            cv.text(
                x + 30.0,
                top + 12.0,
                Align2::LEFT_BOTTOM,
                &self.tr(Text::Recent).to_uppercase(),
                12.0,
                col(t.dim),
            );
            let room = (((y + h - 110.0) - (top + 24.0)) / 38.0).floor().max(0.0) as usize;
            for (row, (i, p)) in recent.iter().take(room).enumerate() {
                let ry = top + 24.0 + row as f32 * 38.0;
                let r = cv.rect(x + 30.0, ry, 200.0, 32.0);
                cv.p.rect_filled(r, 6.0 * cv.u, col(t.panel_raised));
                let name = Path::new(p.as_str())
                    .file_name()
                    .map_or_else(|| (*p).clone(), |n| n.to_string_lossy().to_string());
                let shown: String = if name.chars().count() > 22 {
                    format!("{}...", name.chars().take(20).collect::<String>())
                } else {
                    name
                };
                cv.text(
                    x + 44.0,
                    ry + 23.0,
                    Align2::LEFT_BOTTOM,
                    &shown,
                    15.0,
                    col(t.text),
                );
                cv.hits.push((r, Act::OpenRecent(*i)));
            }
        }
        // the headings
        let cols = [
            (250.0, 480.0, Text::ColName),
            (740.0, 210.0, Text::ColDate),
            (960.0, 160.0, Text::ColType),
            (1130.0, 120.0, Text::ColSize),
        ];
        let hy = y + 120.0;
        cv.p.rect_filled(
            cv.rect(x + 250.0, hy, 1050.0, 30.0),
            4.0 * cv.u,
            col(t.panel_raised),
        );
        for (i, (cx, cw, text)) in cols.into_iter().enumerate() {
            let arrow = if b.sort.0 == i {
                if b.sort.1 {
                    " ^"
                } else {
                    " v"
                }
            } else {
                ""
            };
            cv.text(
                x + cx + 10.0,
                hy + 22.0,
                Align2::LEFT_BOTTOM,
                &format!("{}{arrow}", self.tr(text)),
                14.0,
                col(t.accent),
            );
            cv.hits
                .push((cv.rect(x + cx, hy, cw, 30.0), Act::BrowseSort(i)));
        }
        // the rows
        for k in 0..ROWS {
            let i = b.scroll + k;
            let Some(e) = b.entries.get(i) else { break };
            let ry = hy + 36.0 + k as f32 * 32.0;
            let on = b.sel == Some(i);
            let r = cv.rect(x + 250.0, ry, 1050.0, 30.0);
            if on {
                cv.p.rect_filled(r, 4.0 * cv.u, alpha(col(t.accent), 90));
            } else if k % 2 == 1 {
                cv.p.rect_filled(r, 4.0 * cv.u, alpha(Color32::WHITE, 8));
            }
            let (tag, c) = match e.kind {
                EntryKind::Folder => (">", col(t.accent)),
                EntryKind::Drill => ("D", col(t.ok)),
                EntryKind::Route => ("R", col(t.accent2)),
                EntryKind::Zip => ("Z", col(t.text)),
                EntryKind::Other => ("-", col(t.dim)),
            };
            cv.text(x + 262.0, ry + 22.0, Align2::LEFT_BOTTOM, tag, 15.0, c);
            let shown: String = e.name.chars().take(52).collect();
            cv.text(
                x + 290.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                &shown,
                15.0,
                col(t.text),
            );
            if let Some(m) = e.modified {
                cv.text(
                    x + 750.0,
                    ry + 22.0,
                    Align2::LEFT_BOTTOM,
                    &date_text(m),
                    14.0,
                    col(t.dim),
                );
            }
            cv.text(
                x + 970.0,
                ry + 22.0,
                Align2::LEFT_BOTTOM,
                self.type_text(e.kind),
                14.0,
                col(t.dim),
            );
            if e.kind != EntryKind::Folder {
                cv.text(
                    x + 1240.0,
                    ry + 22.0,
                    Align2::RIGHT_BOTTOM,
                    &size_text(e.size),
                    14.0,
                    col(t.dim),
                );
            }
            cv.hits.push((r, Act::Browse(i)));
        }
        if b.entries.is_empty() {
            cv.text(
                x + 270.0,
                hy + 70.0,
                Align2::LEFT_BOTTOM,
                self.tr(Text::NoFile),
                16.0,
                col(t.dim),
            );
        }
        cv.chip(
            x + 1308.0,
            hy,
            36.0,
            36.0,
            "^",
            false,
            Act::BrowseScroll(-3),
        );
        cv.chip(
            x + 1308.0,
            hy + 40.0,
            36.0,
            36.0,
            "v",
            false,
            Act::BrowseScroll(3),
        );
        // the bottom: name, type of file, Open and Cancel
        cv.text(
            x + 250.0,
            y + 712.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::FileNameLabel),
            15.0,
            col(t.text),
        );
        cv.chip(
            x + 830.0,
            y + 680.0,
            320.0,
            38.0,
            self.tr(b.filter.text()),
            false,
            Act::BrowseFilter,
        );
        cv.chip(
            x + 1170.0,
            y + 680.0,
            160.0,
            38.0,
            self.tr(Text::OpenFile),
            true,
            Act::BrowseOpen,
        );
        cv.chip(
            x + 1170.0,
            y + 726.0,
            160.0,
            38.0,
            self.tr(Text::Close),
            false,
            Act::CloseModal,
        );
        if let Some(e) = b.sel.and_then(|i| b.entries.get(i)) {
            let info = match e.kind {
                EntryKind::Folder => self.tr(Text::TypeFolder).to_string(),
                k => format!("{} - {}", self.type_text(k), size_text(e.size)),
            };
            cv.text(
                x + 250.0,
                y + 756.0,
                Align2::LEFT_BOTTOM,
                &info,
                14.0,
                col(t.dim),
            );
        }
        cv.text(
            x + 30.0,
            y + 764.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::DropHint),
            13.0,
            col(t.dim),
        );
    }
}

/// Where the window sits on the canvas.
pub(crate) const WIN: (f32, f32) = (120.0, 60.0);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn testdata() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    fn app() -> App {
        App::new(&Options {
            data: Some(testdata()),
            screen: Some(Screen::Viewer),
            ..Default::default()
        })
    }

    #[test]
    fn a_date_is_written_as_windows_does() {
        assert_eq!(date_text(UNIX_EPOCH), "1970-01-01 00:00");
        assert_eq!(
            date_text(UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000)),
            "2023-11-14 22:13"
        );
        assert_eq!(size_text(500), "500 B");
        assert_eq!(size_text(2048), "2.0 KB");
    }

    #[test]
    fn the_list_has_the_folders_first_and_follows_the_filter_and_the_sort() {
        let mut a = app();
        a.last_dir = testdata();
        a.show_modal(Modal::Open);
        let b = a.browser.as_ref().unwrap();
        assert!(b.entries.iter().any(|e| e.name == "05_small_board.ncd"));
        assert!(!b.entries.iter().any(|e| e.name == "DRILLPRO.DCF"));
        a.browse_filter();
        a.browse_filter();
        a.browse_filter();
        a.browse_filter();
        let b = a.browser.as_ref().unwrap();
        assert_eq!(b.filter, Filter::Any);
        assert!(b.entries.iter().any(|e| e.name == "DRILLPRO.DCF"));
        a.browse_sort(3);
        let b = a.browser.as_ref().unwrap();
        let sizes: Vec<u64> = b
            .entries
            .iter()
            .filter(|e| e.kind != EntryKind::Folder)
            .map(|e| e.size)
            .collect();
        assert!(sizes.windows(2).all(|w| w[0] <= w[1]));
        a.browse_sort(3);
        let sizes: Vec<u64> = a
            .browser
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .map(|e| e.size)
            .collect();
        assert!(sizes.windows(2).all(|w| w[0] >= w[1]));
    }

    #[test]
    fn a_click_chooses_and_a_second_click_opens() {
        let mut a = app();
        a.last_dir = testdata();
        a.show_modal(Modal::Open);
        let i = a
            .browser
            .as_ref()
            .unwrap()
            .entries
            .iter()
            .position(|e| e.name == "05_small_board.ncd")
            .unwrap();
        a.browse(i);
        assert_eq!(a.browser.as_ref().unwrap().name_text, "05_small_board.ncd");
        assert!(a.modal.is_some());
        a.browse(i);
        // the Open window is gone and the window of the tools is over the file
        assert_eq!(a.modal, Some(Modal::Tools));
        assert!(!a.drill.scene.holes.is_empty());
    }

    #[test]
    fn the_name_box_opens_a_file_and_up_and_back_move_between_folders() {
        let mut a = app();
        a.last_dir = testdata();
        a.show_modal(Modal::Open);
        a.browser.as_mut().unwrap().name_text = "05_board_outline.plt".into();
        a.browse_open();
        assert_eq!(a.modal, Some(Modal::Tools));
        assert!(!a.route.scene.paths.is_empty());
        a.show_modal(Modal::Open);
        a.browse_up();
        assert_ne!(a.browser.as_ref().unwrap().dir, Some(testdata()));
        a.browse_back();
        assert_eq!(a.browser.as_ref().unwrap().dir, Some(testdata()));
        // a place goes to its folder
        let n = a.browser.as_ref().unwrap().places.len();
        assert!(n >= 1);
    }
}
