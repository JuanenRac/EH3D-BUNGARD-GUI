//! What the viewer adds around the three views: opening files (a browser in the window, drag and drop), panelization, the settings of the
//! table, the About window and moving around the 2D and 3D views.

use super::*;
use eh3d_drillpro::job::{TabPlacement, Tabs};

/// The two views that can be moved around.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Zone {
    Plan,
    Machine,
    /// The emergency button, which is dragged to where the operator wants it.
    Estop,
}

/// A number that a stepper changes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Field {
    Columns,
    Rows,
    SpacingX,
    SpacingY,
    TableW,
    TableD,
    TabCount,
    TabWidth,
    LevCols,
    LevRows,
}

/// A window on top of the viewer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Modal {
    About,
    Open,
    Panel,
    Settings,
    Connection,
    Board,
    Tools,
    /// The machines and stations of the server, followed at once.
    Fleet,
    /// A job that was left in the middle: go on or leave it.
    Resume,
    /// The catalogue of drills and mills.
    Catalogue,
    /// The node of the drill head: its link, its camera and what it reads.
    Head,
}

/// A text box of the Connection window.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnField {
    Rs232,
    Usb,
    NodeAddr,
    NodeToken,
    ServerAddr,
    ServerToken,
    ServerMachine,
}

/// How the copies of a panel are placed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PanelMode {
    /// The spacing is the distance from the start of one board to the start of the next.
    Pitch,
    /// The spacing is the empty space between the boards.
    Gap,
    /// As many copies as fit in the table, with the empty space between them.
    Fit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PanelCfg {
    pub columns: u32,
    pub rows: u32,
    pub spacing: (f64, f64),
    pub mode: PanelMode,
    /// The outside of the panel is routed with tabs, how many on each side, how wide, and where.
    pub tabs_on: bool,
    pub tab_count: u32,
    pub tab_width: f64,
    pub tab_place: TabPlacement,
}

impl PanelCfg {
    /// The tabs of the panel, when they are asked for.
    pub(crate) fn tabs(&self) -> Option<Tabs> {
        self.tabs_on.then_some(Tabs {
            per_side: self.tab_count,
            width_mm: self.tab_width,
            placement: self.tab_place,
        })
    }
}

impl Default for PanelCfg {
    fn default() -> Self {
        PanelCfg {
            columns: 1,
            rows: 1,
            spacing: (5.0, 5.0),
            mode: PanelMode::Gap,
            tabs_on: false,
            tab_count: 2,
            tab_width: 3.0,
            tab_place: TabPlacement::Even,
        }
    }
}

/// Zoom and shift of the plan view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Nav2 {
    pub zoom: f32,
    pub pan: (f32, f32),
    /// A point of the table the view is centred on (the tool that works, or the work of a tool); the pan is added to it.
    pub focus: Option<(f64, f64)>,
}

/// Turn, zoom and shift of the 3D view.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Nav3 {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub pan: (f32, f32),
}

impl Default for Nav2 {
    fn default() -> Self {
        Nav2 {
            zoom: 1.0,
            pan: (0.0, 0.0),
            focus: None,
        }
    }
}

impl Default for Nav3 {
    fn default() -> Self {
        Nav3 {
            yaw: 0.0,
            pitch: 30.0,
            zoom: 1.0,
            pan: (0.0, 0.0),
        }
    }
}

pub(super) fn is_job(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".zip") || eh3d_archive::kind_of(name) != Kind::Other
}

/// Breaks a text into lines of about `max` characters (at spaces; a text without spaces, as the Chinese and the Japanese, at any letter).
pub(crate) fn wrap(s: &str, max: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    if s.contains(' ') {
        for w in s.split(' ') {
            if !cur.is_empty() && cur.chars().count() + 1 + w.chars().count() > max {
                lines.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(w);
        }
    } else {
        for c in s.chars() {
            if cur.chars().count() >= max / 2 {
                lines.push(std::mem::take(&mut cur));
            }
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

impl<'a> Cv<'a> {
    /// A small button with a border: filled when it is the one in use. It brightens under the pointer, with a line of light on its top edge,
    /// and sinks a little when it is pressed.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn chip(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str, on: bool, act: Act) {
        let rect = self.rect(x, y, w, h);
        let (hover, down) = self.touch(rect);
        let r = if self.t.corner == Corner::Square {
            0.0
        } else {
            8.0 * self.u
        };
        let base = if on {
            col(self.t.accent)
        } else {
            col(self.t.panel_raised)
        };
        let fill = if down {
            shade(base, 0.8)
        } else if hover {
            if on {
                shade(base, 1.12)
            } else {
                // a button that is not in use takes a trace of the accent colour under the pointer
                let a = col(self.t.accent);
                Color32::from_rgb(
                    ((base.r() as u16 * 3 + a.r() as u16) / 4) as u8,
                    ((base.g() as u16 * 3 + a.g() as u16) / 4) as u8,
                    ((base.b() as u16 * 3 + a.b() as u16) / 4) as u8,
                )
            }
        } else {
            base
        };
        let drop = if down { 2.0 * self.u } else { 0.0 };
        let body = rect.translate(Vec2::new(0.0, drop));
        if !down {
            self.p.rect_filled(
                rect.translate(Vec2::new(0.0, 2.0 * self.u)),
                r,
                alpha(Color32::BLACK, if hover { 70 } else { 40 }),
            );
        }
        if hover && !down {
            self.p.rect_filled(
                rect.expand(3.0 * self.u),
                r + 3.0 * self.u,
                alpha(col(self.t.accent), 40),
            );
        }
        self.p.rect_filled(body, r, fill);
        self.p.rect_stroke(
            body,
            r,
            Stroke::new(
                if hover { 1.6 } else { 1.0 } * self.u,
                alpha(col(self.t.accent), if hover { 255 } else { 170 }),
            ),
            StrokeKind::Inside,
        );
        // the shine along the top edge
        let top = body.min.y + 2.5 * self.u;
        self.p.line_segment(
            [
                Pos2::new(body.min.x + 7.0 * self.u, top),
                Pos2::new(body.max.x - 7.0 * self.u, top),
            ],
            Stroke::new(
                1.3 * self.u,
                alpha(
                    Color32::WHITE,
                    if down {
                        8
                    } else if on {
                        60
                    } else {
                        28
                    },
                ),
            ),
        );
        let ink = if on {
            col(self.t.background.0)
        } else {
            col(self.t.text)
        };
        self.text(
            x + w / 2.0,
            y + h / 2.0 + 6.0 + if down { 2.0 } else { 0.0 },
            Align2::CENTER_BOTTOM,
            label,
            14.0,
            ink,
        );
        self.hits.push((rect, act));
    }

    /// The same canvas on another painter (one that cuts at the edge of a view).
    pub(crate) fn with_painter<'b>(&self, p: &'b egui::Painter) -> Cv<'b>
    where
        'a: 'b,
    {
        Cv {
            p,
            o: self.o,
            u: self.u,
            t: self.t,
            hits: Vec::new(),
            zones: Vec::new(),
            combos: Vec::new(),
            modal_from: 0,
            pointer: self.pointer,
            down: self.down,
        }
    }
}

impl App {
    // ------------------------------------------------------------------------------------------ opening files

    /// Opens what the extension says it is: a job archive, a drilling file or a routing file.
    pub(crate) fn open_path(&mut self, path: &Path) {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.to_ascii_lowercase().ends_with(".zip") {
            self.open_zip(path);
        } else {
            match eh3d_archive::kind_of(&name) {
                Kind::Drill => {
                    self.archive = None;
                    self.load_drill(path);
                }
                Kind::Route => {
                    self.archive = None;
                    self.load_route(path);
                }
                Kind::Other => {
                    self.message = Some(format!("{name}: {}", self.tr(Text::DropHint)));
                    return;
                }
            }
        }
        if let Some(d) = path.parent() {
            self.last_dir = d.to_path_buf();
        }
        self.settings.remember_recent(&path.to_string_lossy());
        self.remember_settings();
        self.refit_table();
        self.apply_panel();
    }

    /// A file the person chose (not one named on the command line): it is opened and, when it holds work and the setting says so, the window of the tools
    /// opens over it, as the classic programs show the tools of a file after loading it.
    pub(crate) fn open_path_asking(&mut self, path: &Path) {
        self.open_path(path);
        let holds_work = !self.drill.scene.holes.is_empty() || !self.route.scene.paths.is_empty();
        if self.settings.tools_on_open && holds_work && self.message.is_none() {
            self.show_modal(Modal::Tools);
        }
    }

    pub(crate) fn show_modal(&mut self, m: Modal) {
        self.saved = false;
        if m == Modal::Open {
            self.browser = Some(self.new_browser());
        }
        self.modal = Some(m);
    }

    // ------------------------------------------------------------------------------------------ panelization

    /// The size of the board (both files together), where it starts on the table.
    fn board(&self) -> Option<(f64, f64, f64, f64)> {
        let mut all = self.drill.base.clone();
        all.overlay(&self.route.base);
        all.bounds()
    }

    /// The columns, the rows and the distance between copies, as the mode and the spacing say.
    pub(crate) fn panel_plan(&self) -> (u32, u32, (f64, f64)) {
        let c = self.panel;
        let (x0, y0, x1, y1) = self.board().unwrap_or((0.0, 0.0, 0.0, 0.0));
        let size = (x1 - x0, y1 - y0);
        match c.mode {
            PanelMode::Pitch => (c.columns, c.rows, c.spacing),
            PanelMode::Gap => (
                c.columns,
                c.rows,
                (size.0 + c.spacing.0, size.1 + c.spacing.1),
            ),
            PanelMode::Fit => {
                let pitch = (size.0 + c.spacing.0, size.1 + c.spacing.1);
                let (cols, rows) = Scene::panel_fit(
                    (x0, y0),
                    size,
                    (self.table.width_mm, self.table.depth_mm),
                    pitch,
                );
                (cols, rows, pitch)
            }
        }
    }

    /// The copies of the panel are drawn and the job will make them.
    pub(crate) fn apply_panel(&mut self) {
        let (c, r, pitch) = self.panel_plan();
        let many = c * r > 1;
        self.panel_applied = many.then_some((c, r, pitch));
        let table = self.table;
        for side in [&mut self.drill, &mut self.route] {
            side.scene = if many {
                side.base.panelized(c, r, pitch)
            } else {
                side.base.clone()
            };
            side.scene.table = table;
            side.last_done = 0;
        }
        // the outside of the panel, routed with tabs: the paths are cut into the pieces between them
        if let (true, Some(tabs)) = (many, self.panel.tabs()) {
            let cut: Vec<eh3d_view::scene::Path> = self
                .route
                .scene
                .paths
                .iter()
                .flat_map(|p| {
                    eh3d_drillpro::job::cut_tabs(&p.points, &tabs)
                        .into_iter()
                        .map(|points| eh3d_view::scene::Path {
                            points,
                            ..p.clone()
                        })
                })
                .collect();
            self.route.scene.paths = cut;
        }
    }

    pub(crate) fn adjust(&mut self, f: Field, d: f32) {
        let d = d as f64;
        // the steps are in the units on the screen
        let dl = self.len_from_display(d);
        self.saved = false;
        match f {
            Field::Columns => {
                self.panel.columns = (self.panel.columns as f64 + d).clamp(1.0, 99.0) as u32
            }
            Field::Rows => self.panel.rows = (self.panel.rows as f64 + d).clamp(1.0, 99.0) as u32,
            Field::SpacingX => {
                self.panel.spacing.0 = (self.panel.spacing.0 + dl).clamp(0.0, 5000.0)
            }
            Field::SpacingY => {
                self.panel.spacing.1 = (self.panel.spacing.1 + dl).clamp(0.0, 5000.0)
            }
            Field::LevCols => {
                self.level_reset(
                    (self.level.cols as f64 + d).max(2.0) as usize,
                    self.level.rows,
                );
                return;
            }
            Field::LevRows => {
                self.level_reset(
                    self.level.cols,
                    (self.level.rows as f64 + d).max(2.0) as usize,
                );
                return;
            }
            Field::TabCount => {
                self.panel.tab_count = (self.panel.tab_count as f64 + d).clamp(1.0, 20.0) as u32
            }
            Field::TabWidth => self.panel.tab_width = (self.panel.tab_width + dl).clamp(0.5, 50.0),
            Field::TableW | Field::TableD => {
                let (w, h) = (self.table.width_mm, self.table.depth_mm);
                let new = if f == Field::TableW {
                    ((w + dl).clamp(50.0, 3000.0), h)
                } else {
                    (w, (h + dl).clamp(50.0, 3000.0))
                };
                for side in [&mut self.drill, &mut self.route] {
                    if let Some(s) = side.engine.settings_mut() {
                        s.clip_mm = new;
                    }
                    side.engine.settings_changed();
                }
                self.table = Table::new(new.0, new.1);
                self.drill.scene.table = self.table;
                self.route.scene.table = self.table;
            }
        }
        self.apply_panel();
    }

    // ------------------------------------------------------------------------------------------ the connection

    /// The text boxes of the Connection window, over the card (the canvas has no text boxes of its own).
    pub(super) fn connection_fields(&mut self, ctx: &egui::Context) {
        let u = self.ui_scale;
        let rows: Vec<(Text, ConnField)> = match self.conn.kind {
            LinkKind::Demo => vec![],
            LinkKind::Rs232 => vec![(Text::ConnPort, ConnField::Rs232)],
            LinkKind::Usb => vec![(Text::ConnPort, ConnField::Usb)],
            LinkKind::Node => vec![
                (Text::ConnAddress, ConnField::NodeAddr),
                (Text::ConnToken, ConnField::NodeToken),
            ],
            LinkKind::Server => vec![
                (Text::ConnAddress, ConnField::ServerAddr),
                (Text::ConnToken, ConnField::ServerToken),
                (Text::ConnMachine, ConnField::ServerMachine),
            ],
        };
        for (i, (_, f)) in rows.into_iter().enumerate() {
            let pos = Pos2::new(
                self.ui_origin.x + 700.0 * u,
                self.ui_origin.y + (130.0 + 214.0 + i as f32 * 62.0) * u,
            );
            let secret = matches!(f, ConnField::NodeToken | ConnField::ServerToken);
            let buf = match f {
                ConnField::Rs232 => &mut self.conn.rs232_port,
                ConnField::Usb => &mut self.conn.usb_port,
                ConnField::NodeAddr => &mut self.conn.node_address,
                ConnField::NodeToken => &mut self.conn.node_token,
                ConnField::ServerAddr => &mut self.conn.server_address,
                ConnField::ServerToken => &mut self.conn.server_token,
                ConnField::ServerMachine => &mut self.conn.server_machine,
            };
            egui::Area::new(egui::Id::unique(("conn-field", i)))
                .fixed_pos(pos)
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(buf)
                            .desired_width(430.0 * u)
                            .font(FontId::proportional(18.0 * u))
                            .password(secret),
                    );
                    if r.changed() {
                        self.saved = false;
                        self.conn_result = None;
                    }
                });
        }
    }

    /// Tries to open the link with the values of the window (in another thread: a node that does not answer takes its time).
    pub(super) fn conn_test(&mut self) {
        let c = self.conn.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let r = c.open().map(|_| ()).map_err(|e| e.to_string());
            let _ = tx.send(r);
        });
        self.conn_rx = Some(rx);
        self.conn_result = None;
    }

    /// The machine files get the connection of the window; both programs use it from now on.
    pub(super) fn conn_save(&mut self) {
        if self.running.is_some() {
            self.message = Some(self.tr(Text::Stop).to_string());
            return;
        }
        self.drill.engine.set_connection(self.conn.clone());
        self.route.engine.set_connection(self.conn.clone());
        self.connection = self.conn.describe();
        let old = std::fs::read_to_string(&self.conn_path).unwrap_or_default();
        let text = crate::link::update(&old, &crate::link::pairs(&self.conn));
        if let Some(dir) = self.conn_path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        match std::fs::write(&self.conn_path, text) {
            Ok(()) => {
                self.saved = true;
                self.message = None;
            }
            Err(e) => self.message = Some(format!("{}: {e}", self.conn_path.display())),
        }
    }

    pub(crate) fn save_settings(&mut self) {
        let a = self.drill.engine.save_config();
        let b = self.route.engine.save_config();
        match (a, b) {
            (Ok(_), Ok(_)) => {
                self.saved = true;
                self.message = None;
            }
            (Err(e), _) | (_, Err(e)) => self.message = Some(e),
        }
        // the sounds and the connection go to the machine file
        if self.running.is_none() {
            self.conn_save();
        }
    }

    // ------------------------------------------------------------------------------------------ moving around the views

    pub(crate) fn zone_at(&self, pos: Pos2) -> Option<Zone> {
        if self.modal.is_some() {
            return None;
        }
        self.zones
            .iter()
            .rev()
            .find(|(r, _)| r.contains(pos))
            .map(|(_, z)| *z)
    }

    /// A drag in design units: the plan view is moved, the 3D view is turned (or moved with Shift or the right button).
    pub(crate) fn drag(&mut self, z: Zone, d: (f32, f32), pan: bool) {
        match z {
            Zone::Plan => {
                let scale = self.plan_scale.get().max(1e-6);
                if self.move_board {
                    // the board goes where it is dragged: millimetres, with Y going up
                    self.shift_board(d.0 as f64 / scale, -(d.1 as f64) / scale);
                } else {
                    self.view2d.pan.0 += d.0;
                    self.view2d.pan.1 += d.1;
                }
            }
            Zone::Estop => {
                let (x, y) = self.settings.estop_pos;
                self.settings.estop_pos =
                    ((x + d.0).clamp(50.0, 1550.0), (y + d.1).clamp(100.0, 850.0));
            }
            Zone::Machine => {
                if pan {
                    self.view3d.pan.0 += d.0;
                    self.view3d.pan.1 += d.1;
                } else {
                    self.view3d.yaw = (self.view3d.yaw + d.0 * 0.35).clamp(-40.0, 40.0);
                    self.view3d.pitch = (self.view3d.pitch + d.1 * 0.25).clamp(15.0, 60.0);
                }
            }
        }
    }

    pub(crate) fn zoom(&mut self, z: Zone, scroll: f32) {
        let k = (scroll * 0.002).exp();
        match z {
            Zone::Plan => self.view2d.zoom = (self.view2d.zoom * k).clamp(0.3, 20.0),
            Zone::Machine => self.view3d.zoom = (self.view3d.zoom * k).clamp(0.3, 8.0),
            Zone::Estop => {}
        }
    }

    pub(crate) fn reset_view(&mut self, z: Zone) {
        match z {
            Zone::Plan => self.view2d = Nav2::default(),
            Zone::Machine => self.view3d = Nav3::default(),
            Zone::Estop => self.settings.estop_pos = crate::launcher::Settings::default().estop_pos,
        }
    }

    // ------------------------------------------------------------------------------------------ the windows

    pub(super) fn modal_card(&self, cv: &mut Cv, title: &str, h: f32) -> (f32, f32) {
        let t = cv.t.clone();
        let (x, y, w) = (430.0, 130.0, 740.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 170),
        );
        cv.card(x, y, w, h, true, None);
        cv.title(x + 30.0, y + 44.0, title);
        cv.line(
            (x + 30.0, y + 56.0),
            (x + w - 30.0, y + 56.0),
            1.0,
            alpha(col(t.line), 120),
        );
        (x, y)
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn stepper(
        &self,
        cv: &mut Cv,
        y: f32,
        label: &str,
        value: &str,
        f: Field,
        steps: [f32; 4],
        live: bool,
    ) {
        let t = cv.t.clone();
        let ink = if live { col(t.text) } else { col(t.dim) };
        cv.text(460.0, y + 32.0, Align2::LEFT_BOTTOM, label, 17.0, ink);
        let xs = [790.0, 850.0, 1020.0, 1080.0];
        for (x, s) in xs.into_iter().zip(steps) {
            cv.chip(
                x,
                y,
                54.0,
                42.0,
                &format!("{s:+}"),
                false,
                Act::Adjust(f, s),
            );
        }
        cv.text(960.0, y + 32.0, Align2::CENTER_BOTTOM, value, 22.0, ink);
        if live {
            // the value can be clicked and typed
            let r = cv.rect(900.0, y, 120.0, 42.0);
            cv.p.rect_stroke(
                r,
                6.0 * cv.u,
                Stroke::new(1.0 * cv.u, alpha(col(t.line), 70)),
                StrokeKind::Inside,
            );
            cv.hits
                .push((r, Act::Edit(EditTarget::Field(f), 905.0, y + 4.0, 110.0)));
        }
    }

    pub(super) fn paint_modal(&self, cv: &mut Cv, m: Modal) {
        let t = cv.t.clone();
        match m {
            Modal::Board => self.paint_board_modal(cv),
            Modal::Tools => self.paint_tools_modal(cv),
            Modal::Fleet => self.paint_fleet_modal(cv),
            Modal::Resume => self.paint_resume(cv),
            Modal::Catalogue => self.paint_catalogue(cv),
            Modal::Head => self.paint_head(cv),
            Modal::About => {
                let (x, y) = self.modal_card(cv, self.tr(Text::About), 420.0);
                cv.text(
                    x + 30.0,
                    y + 120.0,
                    Align2::LEFT_BOTTOM,
                    "EH3D-BUNGARD-GUI",
                    30.0,
                    col(t.text),
                );
                cv.text(
                    x + 30.0,
                    y + 156.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{} {}", self.tr(Text::Version), env!("CARGO_PKG_VERSION")),
                    18.0,
                    col(t.accent),
                );
                for (i, l) in wrap(self.tr(Text::AboutBody), 62).iter().enumerate() {
                    cv.text(
                        x + 30.0,
                        y + 206.0 + i as f32 * 26.0,
                        Align2::LEFT_BOTTOM,
                        l,
                        18.0,
                        col(t.text),
                    );
                }
                cv.text(
                    x + 30.0,
                    y + 290.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{}: GPL-3.0-or-later", self.tr(Text::Licence)),
                    16.0,
                    col(t.dim),
                );
                cv.text(
                    x + 30.0,
                    y + 318.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{}: {}", self.tr(Text::Connection), self.connection),
                    16.0,
                    col(t.dim),
                );
                cv.text(
                    x + 30.0,
                    y + 346.0,
                    Align2::LEFT_BOTTOM,
                    "github.com/JuanenRac/EH3D-BUNGARD-GUI",
                    16.0,
                    col(t.dim),
                );
                cv.chip(
                    x + 540.0,
                    y + 350.0,
                    170.0,
                    44.0,
                    self.tr(Text::Close),
                    true,
                    Act::CloseModal,
                );
            }
            Modal::Open => self.paint_open_modal(cv),
            Modal::Panel => {
                let (x, y) = self.modal_card(cv, self.tr(Text::PanelTitle), 750.0);
                let (c, r, pitch) = self.panel_plan();
                let (x0, y0, x1, y1) = self.board().unwrap_or((0.0, 0.0, 0.0, 0.0));
                let size = (x1 - x0, y1 - y0);
                cv.text(
                    x + 30.0,
                    y + 96.0,
                    Align2::LEFT_BOTTOM,
                    &format!(
                        "{c} x {r} = {} {}   -   {:.1} x {:.1} mm",
                        c * r,
                        self.tr(Text::PanelCopies),
                        size.0,
                        size.1
                    ),
                    18.0,
                    col(t.text),
                );
                let modes = [
                    (PanelMode::Gap, Text::PanelModeGap),
                    (PanelMode::Pitch, Text::PanelModePitch),
                    (PanelMode::Fit, Text::PanelModeFit),
                ];
                for (i, (m2, n)) in modes.into_iter().enumerate() {
                    cv.chip(
                        x + 30.0 + i as f32 * 228.0,
                        y + 116.0,
                        216.0,
                        44.0,
                        self.tr(n),
                        self.panel.mode == m2,
                        Act::PanelMode(m2),
                    );
                }
                let live = self.panel.mode != PanelMode::Fit;
                self.stepper(
                    cv,
                    y + 196.0,
                    self.tr(Text::Columns),
                    &format!("{c}"),
                    Field::Columns,
                    [-5.0, -1.0, 1.0, 5.0],
                    live,
                );
                self.stepper(
                    cv,
                    y + 256.0,
                    self.tr(Text::Rows),
                    &format!("{r}"),
                    Field::Rows,
                    [-5.0, -1.0, 1.0, 5.0],
                    live,
                );
                self.stepper(
                    cv,
                    y + 316.0,
                    self.tr(Text::SpacingX),
                    &format!("{:.1}", self.panel.spacing.0),
                    Field::SpacingX,
                    [-10.0, -1.0, 1.0, 10.0],
                    true,
                );
                self.stepper(
                    cv,
                    y + 376.0,
                    self.tr(Text::SpacingY),
                    &format!("{:.1}", self.panel.spacing.1),
                    Field::SpacingY,
                    [-10.0, -1.0, 1.0, 10.0],
                    true,
                );
                let ext = (
                    x0 + (c.saturating_sub(1)) as f64 * pitch.0 + size.0,
                    y0 + (r.saturating_sub(1)) as f64 * pitch.1 + size.1,
                );
                cv.text(
                    x + 30.0,
                    y + 460.0,
                    Align2::LEFT_BOTTOM,
                    &format!(
                        "{:.1} x {:.1} mm  /  {:.0} x {:.0} mm",
                        ext.0 - x0,
                        ext.1 - y0,
                        self.table.width_mm,
                        self.table.depth_mm
                    ),
                    16.0,
                    col(t.dim),
                );
                if ext.0 > self.table.width_mm + 0.01 || ext.1 > self.table.depth_mm + 0.01 {
                    cv.text(
                        x + 30.0,
                        y + 490.0,
                        Align2::LEFT_BOTTOM,
                        self.tr(Text::OutsideTable),
                        18.0,
                        col(t.danger),
                    );
                }
                // the outside of the panel: routed through, or with tabs that hold the boards
                cv.chip(
                    x + 30.0,
                    y + 506.0,
                    330.0,
                    44.0,
                    self.tr(Text::TabsOn),
                    self.panel.tabs_on,
                    Act::ToggleTabs,
                );
                if self.panel.tabs_on {
                    cv.text(
                        x + 380.0,
                        y + 514.0,
                        Align2::LEFT_BOTTOM,
                        &self.tr(Text::TabPlace).to_uppercase(),
                        12.0,
                        col(t.dim),
                    );
                    let place = self.combo_items(ComboId::TabPlace);
                    cv.combo(
                        x + 380.0,
                        y + 520.0,
                        330.0,
                        ComboId::TabPlace,
                        &place[self.combo_current(ComboId::TabPlace)],
                        self.combo.is_some_and(|c| c.id == ComboId::TabPlace),
                    );
                    self.stepper(
                        cv,
                        y + 566.0,
                        self.tr(Text::TabsPerSide),
                        &format!("{}", self.panel.tab_count),
                        Field::TabCount,
                        [-5.0, -1.0, 1.0, 5.0],
                        true,
                    );
                    self.stepper(
                        cv,
                        y + 626.0,
                        self.tr(Text::TabWidth),
                        &format!("{:.1}", self.panel.tab_width),
                        Field::TabWidth,
                        [-5.0, -1.0, 1.0, 5.0],
                        true,
                    );
                }
                cv.chip(
                    x + 540.0,
                    y + 690.0,
                    170.0,
                    44.0,
                    self.tr(Text::Close),
                    true,
                    Act::CloseModal,
                );
            }
            Modal::Connection => {
                let (x, y) = self.modal_card(cv, self.tr(Text::Connection), 560.0);
                let kinds = [
                    (LinkKind::Demo, "Demo"),
                    (LinkKind::Rs232, "RS-232"),
                    (LinkKind::Usb, "USB-RS232"),
                    (LinkKind::Node, "EH3D node"),
                    (LinkKind::Server, "EH3D server"),
                ];
                for (i, (k, n)) in kinds.into_iter().enumerate() {
                    cv.chip(
                        x + 30.0 + i as f32 * 140.0,
                        y + 90.0,
                        130.0,
                        44.0,
                        n,
                        self.conn.kind == k,
                        Act::ConnKind(k),
                    );
                }
                let labels: Vec<Text> = match self.conn.kind {
                    LinkKind::Demo => vec![],
                    LinkKind::Rs232 | LinkKind::Usb => vec![Text::ConnPort],
                    LinkKind::Node => vec![Text::ConnAddress, Text::ConnToken],
                    LinkKind::Server => {
                        vec![Text::ConnAddress, Text::ConnToken, Text::ConnMachine]
                    }
                };
                for (i, n) in labels.into_iter().enumerate() {
                    cv.text(
                        x + 30.0,
                        y + 214.0 + 30.0 + i as f32 * 62.0,
                        Align2::LEFT_BOTTOM,
                        self.tr(n),
                        17.0,
                        col(t.text),
                    );
                }
                cv.text(
                    x + 30.0,
                    y + 174.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{}: {}", self.tr(Text::Connection), self.conn.describe()),
                    15.0,
                    col(t.dim),
                );
                cv.chip(
                    x + 530.0,
                    y + 134.0,
                    180.0,
                    34.0,
                    self.tr(Text::HeadOpen),
                    false,
                    Act::Show(Modal::Head),
                );
                match &self.conn_result {
                    Some(Ok(())) => {
                        cv.text(
                            x + 30.0,
                            y + 440.0,
                            Align2::LEFT_BOTTOM,
                            self.tr(Text::ConnWorks),
                            18.0,
                            col(t.ok),
                        );
                    }
                    Some(Err(e)) => {
                        let e: String = e.chars().take(80).collect();
                        cv.text(
                            x + 30.0,
                            y + 440.0,
                            Align2::LEFT_BOTTOM,
                            &e,
                            15.0,
                            col(t.danger),
                        );
                    }
                    None => {
                        if self.saved {
                            cv.text(
                                x + 30.0,
                                y + 440.0,
                                Align2::LEFT_BOTTOM,
                                self.tr(Text::Saved),
                                18.0,
                                col(t.ok),
                            );
                        }
                    }
                }
                cv.chip(
                    x + 30.0,
                    y + 480.0,
                    230.0,
                    44.0,
                    self.tr(Text::ConnTest),
                    false,
                    Act::ConnTest,
                );
                cv.chip(
                    x + 330.0,
                    y + 480.0,
                    180.0,
                    44.0,
                    self.tr(Text::Save),
                    false,
                    Act::ConnSave,
                );
                cv.chip(
                    x + 530.0,
                    y + 480.0,
                    180.0,
                    44.0,
                    self.tr(Text::Close),
                    true,
                    Act::CloseModal,
                );
            }
            Modal::Settings => self.paint_settings_modal(cv),
        }
    }
}

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
    fn a_file_is_opened_as_what_its_extension_says() {
        let mut a = app();
        a.open_path(&testdata().join("05_small_board.ncd"));
        assert!(!a.drill.scene.holes.is_empty());
        a.open_path(&testdata().join("05_board_outline.plt"));
        assert!(!a.route.scene.paths.is_empty());
        a.open_path(&testdata().join("DRILLPRO.DCF"));
        assert!(a.message.is_some());
    }

    #[test]
    fn the_panel_makes_the_copies_of_the_board() {
        let mut a = app();
        a.open_path(&testdata().join("05_small_board.ncd"));
        let one = a.drill.scene.holes.len();
        a.adjust(Field::Columns, 2.0);
        a.adjust(Field::Rows, 1.0);
        assert_eq!(a.drill.scene.holes.len(), one * 6);
        assert_eq!(a.panel_applied.map(|p| (p.0, p.1)), Some((3, 2)));
        // the mode that fills the table gives as many copies as fit
        a.panel.mode = PanelMode::Fit;
        a.apply_panel();
        let (c, r, _) = a.panel_plan();
        assert!(c * r >= 1);
        assert_eq!(a.drill.scene.holes.len(), one * (c * r) as usize);
    }

    #[test]
    fn the_outside_of_the_panel_is_routed_with_tabs_that_the_person_chooses() {
        use super::super::combo::ComboId;
        let mut a = app();
        a.open_path(&testdata().join("05_board_outline.plt"));
        a.adjust(Field::Columns, 1.0);
        let whole = a.route.scene.paths.len();
        let points: usize = a.route.scene.paths.iter().map(|p| p.points.len()).sum();
        assert!(whole >= 2);
        // the tabs are off: the paths stay whole
        assert!(a.panel.tabs().is_none());
        a.act(Act::ToggleTabs);
        assert!(a.panel.tabs().is_some());
        let with = a.route.scene.paths.len();
        assert!(with > whole, "{with} > {whole}");
        // more tabs on each side, more pieces; the places and the width are chosen
        a.adjust(Field::TabCount, 2.0);
        assert!(a.route.scene.paths.len() > with);
        a.adjust(Field::TabWidth, 2.0);
        assert_eq!(a.panel.tab_width, 5.0);
        a.combo_open(ComboId::TabPlace);
        a.combo_pick(2);
        assert_eq!(
            a.panel.tabs().map(|t| t.placement),
            Some(eh3d_drillpro::job::TabPlacement::Middle)
        );
        // and off again: as it was
        a.act(Act::ToggleTabs);
        assert_eq!(a.route.scene.paths.len(), whole);
        assert_eq!(
            a.route
                .scene
                .paths
                .iter()
                .map(|p| p.points.len())
                .sum::<usize>(),
            points
        );
    }

    #[test]
    fn the_views_move_and_come_back() {
        let mut a = app();
        a.drag(Zone::Plan, (10.0, -5.0), false);
        a.zoom(Zone::Plan, 200.0);
        assert!(a.view2d.zoom > 1.0 && a.view2d.pan == (10.0, -5.0));
        a.drag(Zone::Machine, (1000.0, 1000.0), false);
        assert_eq!((a.view3d.yaw, a.view3d.pitch), (40.0, 60.0));
        a.reset_view(Zone::Plan);
        a.reset_view(Zone::Machine);
        assert_eq!((a.view2d, a.view3d), (Nav2::default(), Nav3::default()));
    }

    #[test]
    fn the_table_of_the_settings_follows_the_steppers() {
        let mut a = app();
        let w = a.table.width_mm;
        a.adjust(Field::TableW, 10.0);
        assert_eq!(a.table.width_mm, w + 10.0);
        assert_eq!(a.drill.scene.table.width_mm, w + 10.0);
    }

    #[test]
    fn a_paused_job_holds_and_goes_on() {
        let mut a = app();
        a.auto_ok = true;
        a.fixed_dt = Some(0.05);
        a.open_path(&testdata().join("05_small_board.ncd"));
        a.start_job();
        for _ in 0..30 {
            a.tick();
        }
        a.act(Act::Pause);
        let (done, progress) = (a.drill.engine.done(), a.progress());
        for _ in 0..200 {
            a.tick();
        }
        assert_eq!((a.drill.engine.done(), a.progress()), (done, progress));
        a.act(Act::Pause);
        for _ in 0..200_000 {
            if a.running.is_none() {
                break;
            }
            a.tick();
        }
        assert!(
            a.running.is_none(),
            "still running after the loop: done {} was {done}",
            a.drill.engine.done()
        );
        assert!(a.progress() > progress);
    }

    #[test]
    fn long_texts_are_wrapped() {
        assert_eq!(wrap("one two three", 7), vec!["one two", "three"]);
        assert!(wrap("这是一个很长的中文句子需要换行显示", 10).len() > 1);
    }

    #[test]
    fn the_tools_window_opens_over_a_file_the_person_chose_and_not_over_one_of_the_command_line() {
        let mut a = app();
        // the command line and the tests open files without asking
        a.modal = None;
        a.open_path(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("05_small_board.ncd"),
        );
        assert_eq!(a.modal, None);
        // a file the person chose: the window of the tools opens over it
        a.open_path_asking(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("05_small_board.ncd"),
        );
        assert_eq!(a.modal, Some(Modal::Tools));
        // not when the setting is off, and not for a file that cannot be read
        a.modal = None;
        a.settings.tools_on_open = false;
        a.open_path_asking(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("05_small_board.ncd"),
        );
        assert_eq!(a.modal, None);
        a.settings.tools_on_open = true;
        a.open_path_asking(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("testdata")
                .join("DRILLPRO.DCF"),
        );
        assert_eq!(a.modal, None);
        // the check box of the settings turns it off and on, and it is remembered
        a.toggle_tools_on_open();
        assert!(!a.settings.tools_on_open);
        a.toggle_tools_on_open();
        assert!(a.settings.tools_on_open);
    }

    #[test]
    fn the_hole_that_was_drilled_is_marked_and_not_the_one_the_head_goes_to_next() {
        let mut a = app();
        a.auto_ok = true;
        a.fixed_dt = Some(0.016);
        a.open_path(&testdata().join("05_small_board.ncd"));
        a.start_job();
        let order = a.drill.engine.hole_order();
        assert!(order.len() > 3);
        // until the first hole is finished no hole is marked, and when it is, it is the first of the plan and only that one
        let mut seen = 0;
        for _ in 0..400_000 {
            a.tick();
            let done = a.drill.engine.done();
            let marked: Vec<usize> = a
                .drill
                .scene
                .holes
                .iter()
                .enumerate()
                .filter(|(_, h)| h.done)
                .map(|(i, _)| i)
                .collect();
            assert_eq!(marked.len(), done.min(order.len()), "done {done}");
            for (k, &(x, y)) in order.iter().enumerate().take(done) {
                let h = a
                    .drill
                    .scene
                    .holes
                    .iter()
                    .find(|h| (h.x_mm - x).abs() < 1e-6 && (h.y_mm - y).abs() < 1e-6 && h.done);
                assert!(
                    h.is_some(),
                    "the hole {k} of the plan at ({x}, {y}) is not marked at done {done}"
                );
            }
            if done > seen {
                seen = done;
            }
            if seen >= 4 || a.running.is_none() {
                break;
            }
        }
        assert!(seen >= 4);
    }
}
