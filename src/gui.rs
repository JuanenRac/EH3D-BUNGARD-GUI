//! The window of the modern interface: the launcher and the viewer, drawn on one canvas with the colours of the theme.
//!
//! The machine is driven by the engines of the classic programs (`ClassicEngine` of DRILLPRO and of ROUTEPRO) with the same machine
//! file, so the computer's RS-232, a USB-RS232 adapter and the EH3D node (directly or through the EH3D server) work as they do there.
//! What the views show of a running job comes from the engine: how many holes are done and where the head is.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Align2, Color32, FontData, FontDefinitions, FontFamily, FontId, Pos2, Rect, Sense, Shape,
    Stroke, StrokeKind, Vec2,
};
use eh3d_archive::{Archive, Kind};
use eh3d_bungard_formats::limits::ReaderLimits;
use eh3d_drillpro::classic::engine::{ClassicEngine, Question};
use eh3d_drillpro::job::{JobOptions, Panel};
use eh3d_drillpro::protocol::connection::{ConnectionConfig, Kind as LinkKind};
use eh3d_view::scene::{Iso, Scene, Table};
use eh3d_view::texts::{tr, Lang, Text};
use eh3d_view::theme::{Corner, Rgb, Theme, ThemeId};

use crate::cli::{Options, Screen};
use crate::jobs;
use crate::launcher::{self, Choice, Machine, Program, Settings};
use crate::machines;

mod bar;
mod board;
mod catalogue;
mod combo;
mod controls;
mod diag;
mod edit;
mod extra;
mod fleet;
mod frame;
mod head;
mod hudinfo;
mod icons;
mod inspect;
mod jogpanel;
mod levelling;
mod logpane;
mod look;
mod machine3d;
mod numedit;
mod opener;
mod rack;
mod reportpane;
mod resume;
mod slotswin;
mod spindle3d;
mod update;
mod view2d;
use board::BoardPick;
use combo::{Combo, ComboId};
use controls::{apply_visuals, Icon};
use extra::{Field, Modal, Nav2, Nav3, PanelCfg, PanelMode, Zone};
use jogpanel::{JogAct, JogUi};
use numedit::{EditTarget, NumEdit};
use opener::Browser;
use std::sync::mpsc::Receiver;

/// The design size of the canvas; everything is laid out in these units and scaled to the window.
fn design_w() -> f32 {
    crate::panes::board_width()
}
const DESIGN_H: f32 = 900.0;
// The height of the card under the views (all its places are in `hudinfo`).
use hudinfo::HUD_H;
/// The buttons of the top bar: where they start along the bar and how wide each is.
const TOP_BAR_FROM: f32 = 280.0;
const TOP_BUTTON_W: f32 = 112.0;
/// The button of the nodes and stations has the longest word in most languages.
const TOP_BUTTON_WIDE: f32 = 150.0;

fn col(c: Rgb) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

fn alpha(c: Color32, a: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

fn shade(c: Color32, k: f32) -> Color32 {
    let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0) as u8;
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

/// The classic engines speak the language type of DRILLPRO.
fn drill_lang(l: Lang) -> eh3d_drillpro::i18n::Lang {
    eh3d_drillpro::i18n::Lang::from_code(l.code()).unwrap_or(eh3d_drillpro::i18n::Lang::En)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Act {
    Choose(Choice),
    Remember,
    NextTheme,
    /// Chooses the machine of the list that the jobs are sent to.
    FleetUse(usize),
    FleetRefresh,
    NextLang,
    Layer(usize),
    Start,
    Pause,
    Stop,
    Back,
    Answer(bool),
    /// The answer to the summary of the job: start it, or leave it.
    Summary(bool),
    /// Copies the report of the job as text.
    ReportCopy,
    /// A file of the ZIP archive of the job (by its place among the drilling and routing files).
    Entry(usize),
    Show(Modal),
    CloseModal,
    Board(BoardPick),
    BoardAdj(bool, f32),
    ToolAdj(Program, usize, usize, f32),
    ToolsSave,
    Emergency,
    SoundTest(eh3d_drillpro::protocol::sound::Cue),
    ToggleFollow,
    ToggleMoveBoard,
    /// A drag in the plan moves only the design (the holes and the paths) over the board and the table, which stay where they are.
    ToggleMoveDesign,
    ToggleBig,
    Rotate(i8),
    BoardSwap,
    FitPlan(bool),
    CenterTool(Program, u8),
    ToggleEmergency,
    /// A button of the panel that moves the machine by hand.
    Jog(JogAct),
    /// A tab of the settings window.
    Tab(u8),
    FileUnit(u8),
    DisplayMil(bool),
    /// The window fills the screen, or not.
    ToggleFullscreen,
    /// The log of the program: the switch, the level, the largest size of a file (KiB), open its folder, copy its path.
    LogToggle,
    LogLevel(crate::applog::Level),
    LogSize(u32),
    LogOpenFolder,
    LogCopyPath,
    /// A value to type: what it is and where its box goes (x, y, width).
    Edit(EditTarget, f32, f32, f32),
    ToolDefault(Program, usize),
    ToolsDefaults,
    /// Starts the counts of the holes of every drill again from zero.
    ResetHoles,
    SetSpeed(f32),
    ToggleAsk,
    /// The window of the updates: look for a new version, install it, put the earlier one back, restart, and the two switches.
    UpdCheck,
    UpdInstall,
    UpdUndo,
    UpdRestart,
    UpdAllow,
    UpdAtStart,
    /// The node of the drill head: how it is reached, its camera and a measure of the table.
    HeadMode(eh3d_drillpro::protocol::connection::HeadNode),
    HeadCamera,
    HeadMeasure,
    /// The ring of LEDs around the camera: off (`None`) or on in the white of the camera at a level.
    HeadLight(Option<u32>),
    /// The quality check of the work through the camera: its checks, its profile, the light for the picture, the hole in the centre, the scale, the run, the tolerances.
    InspOpen,
    InspToggleHoles,
    InspToggleMilling,
    InspPreset(usize),
    InspLight(usize),
    InspHole(i32),
    InspRun,
    InspCalibrate,
    InspScale(f32),
    InspSave,
    InspTune(crate::inspection::Param, i32),
    CatSelect(usize),
    /// The spindle of the catalogue: the next one (1) or the one before (-1).
    CatSpindle(i32),
    /// The spindle the machine carries, chosen in the settings.
    SetSpindle(crate::spindle::SpindleId),
    /// The catalogue opens to take a bit for this tool of the window of tools; the values of the bit go to the tool.
    ToolFromCat(Program, usize),
    CatUse,
    /// The drilling or the routing of the job is switched on or off.
    JobPart(Program),
    /// A tool of the job is switched on or off.
    ToolOff(Program, u8),
    /// The probe measures the table, or stops; the map of the table is cleared.
    LevelToggle,
    LevelApply,
    LevelClear,
    /// The job that was left in the middle goes on, or is left.
    ResumeContinue,
    ResumeCancel,
    /// The outside of the panel is routed with tabs, or through.
    ToggleTabs,
    /// A drop-down list opens (or closes) and one of its rows is chosen.
    ComboOpen(ComboId),
    ComboPick(usize),
    ComboClose,
    /// The window of the places of the tools in the bar: give each tool a place in order, or take every place away.
    SlotsAuto,
    SlotsClear,
    ToggleToolsOnOpen,
    ToggleSkip,
    Adjust(Field, f32),
    PanelMode(PanelMode),
    SaveSettings,
    ConnKind(LinkKind),
    ConnTest,
    ConnSave,
    /// A row of the browser of the Open window.
    Browse(usize),
    BrowseSort(usize),
    BrowseFilter,
    ToggleSelect,
    RackGo,
    RackCancel,
    StartAt(Program, u8),
    CutSel,
    UndoEdit,
    RedoEdit,
    ConnPort(usize),
    PortsRefresh,
    CopySel,
    PasteSel,
    DeleteSel,
    RemoveTool(Program, u8),
    TogglePreview,
    BrowsePlace(usize),
    /// Opens one of the recent files.
    OpenRecent(usize),
    BrowseUp,
    MatScroll(i32),
    BrowseBack,
    BrowseOpen,
    ResetView(Zone),
}

/// One of the two programs inside the window: its engine and what the views show of its file.
struct Side {
    engine: Box<dyn ClassicEngine>,
    scene: Scene,
    /// The file as it is on the table, before the panel makes its copies.
    base: Scene,
    last_done: usize,
    /// Where the work starts on the table (the offset of the configuration), added to every point of the file.
    offset: (f64, f64),
}

impl Side {
    fn new(engine: Box<dyn ClassicEngine>) -> Side {
        Side {
            scene: Scene::new(jobs::DEFAULT_TABLE),
            base: Scene::new(jobs::DEFAULT_TABLE),
            engine,
            last_done: 0,
            offset: (0.0, 0.0),
        }
    }

    fn table(&self) -> Table {
        self.engine
            .settings()
            .filter(|s| s.clip_mm.0 > 0.0 && s.clip_mm.1 > 0.0)
            .map_or(jobs::DEFAULT_TABLE, |s| {
                Table::new(s.clip_mm.0, s.clip_mm.1)
            })
    }
}

/// The machine file of the tests: one for each run of them, made (as the simulator, `connection = demo`) when it is not there yet.
#[cfg(test)]
fn test_machine_file() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "eh3d-bungard-gui-test-{}.machine.cfg",
        std::process::id()
    ));
    if !p.exists() {
        let _ = std::fs::write(
            &p,
            "connection = demo
",
        );
    }
    p
}

#[cfg(not(test))]
fn test_machine_file() -> PathBuf {
    eh3d_drillpro::protocol::connection::default_path()
}

struct Shot {
    file: PathBuf,
    wait: u32,
    done: Arc<AtomicBool>,
}

pub struct App {
    settings: Settings,
    settings_path: PathBuf,
    lang: Lang,
    theme: Theme,
    screen: Screen,
    drill: Side,
    route: Side,
    table: Table,
    layers: [bool; 4],
    remember: bool,
    running: Option<Program>,
    /// The job is held between two moves: the engine is not asked for the next one.
    paused: bool,
    message: Option<String>,
    /// Text for the clipboard, which the window hands over at the start of the next frame.
    pending_copy: Option<String>,
    /// The lines of the report of the job, worked out when its window opens.
    report_lines: Vec<String>,
    connection: String,
    hits: Vec<(Rect, Act)>,
    frame: u32,
    start_at_frame: Option<u32>,
    /// The frame at which the quality check starts by itself (`--check`).
    check_at_frame: Option<u32>,
    shot: Option<Shot>,
    close: bool,
    auto_ok: bool,
    archive: Option<Archive>,
    archive_dir: PathBuf,
    base_dir: PathBuf,
    last_dir: PathBuf,
    modal: Option<Modal>,
    /// Where the hits of the window on top begin: a click goes only to those.
    modal_from: usize,
    browser: Option<Browser>,
    panel: PanelCfg,
    panel_applied: Option<(u32, u32, (f64, f64))>,
    saved: bool,
    view2d: Nav2,
    view3d: Nav3,
    /// The machine of the 3D view: its own place (home), its start and its travel to the first point of the work.
    m3d: machine3d::Sim3d,
    m3d_first: Option<(f64, f64)>,
    zones: Vec<(Rect, Zone)>,
    drag_zone: Option<Zone>,
    ui_scale: f32,
    ui_origin: Pos2,
    /// How deep the head is below its home, in mm, while a simulated job runs (the steps of the plan say it).
    head_z: Option<f64>,
    pointer: Option<Pos2>,
    pointer_down: bool,
    /// The value being typed, and the seconds the last frame took (the simulated job follows the clock of the machine).
    edit: Option<NumEdit>,
    /// The emergency stop is on: nothing starts until the button is pressed again.
    estop: bool,
    /// The first material of the list that the Board window shows.
    mat_scroll: usize,
    /// The type the list of materials is narrowed to (0: all types, then the families in order), and the words typed to search it.
    mat_family: usize,
    mat_query: String,
    /// The plan view follows the tool that works; a drag in it moves the board instead of the view; it takes the room of the 3D view;
    /// the work is turned by this many quarter turns.
    follow: bool,
    move_board: bool,
    move_design: bool,
    /// The choice of holes and paths in the plan view (indices into the scenes), the box being dragged (mm), whether the choosing tool is on, what was
    /// copied, how many times it was pasted, the counter of the files written by edits, and how the plan view was drawn (scale, origin, place) to
    /// turn the pointer into millimetres.
    /// The serial ports the system reports, for the Connection window.
    com_ports: Vec<String>,
    /// The tool each program starts with (the ones before it are left out of the job), if the person chose one.
    start_at: Vec<(Program, u8)>,
    sel_holes: std::collections::BTreeSet<usize>,
    /// The edits that can be undone and redone, and the move of the choice that is being dragged (mm).
    undo: Vec<edit::Snapshot>,
    redo: Vec<edit::Snapshot>,
    sel_move: Option<(f64, f64)>,
    sel_move_from: Option<(f64, f64)>,
    sel_paths: std::collections::BTreeSet<usize>,
    band: Option<((f64, f64), (f64, f64))>,
    select_mode: bool,
    clip: edit::Clip,
    paste_n: u32,
    edit_serial: u32,
    plan_xf: std::cell::Cell<(f64, f64, f64, f64, f64)>,
    /// The board where it was left when the design started to move alone: (x0, y0, x1, y1) in mm. `None`: the board follows the design.
    board_frozen: Option<(f64, f64, f64, f64)>,
    plan_big: bool,
    rotation: u8,
    plan_scale: std::cell::Cell<f64>,
    settings_tab: u8,
    /// The update from GitHub: what its thread shows, the program it would replace, the file of its history, and whether the end of a look was told to the settings.
    upd: crate::updates::State,
    upd_cancel: Arc<AtomicBool>,
    upd_exe: Option<PathBuf>,
    upd_history: PathBuf,
    upd_seen: bool,
    visuals_theme: Option<ThemeId>,
    /// The files of the job as they were opened (they are read again when the unit of the coordinates changes).
    drill_path: Option<PathBuf>,
    route_path: Option<PathBuf>,
    /// When the simulation was last advanced: the job follows the clock of the machine, so it goes on (and catches up) even when the
    /// window is behind another one or has not been drawn for a while. A test sets `fixed_dt` instead.
    last_tick: Instant,
    fixed_dt: Option<f64>,
    /// The board of the job and the maker whose materials the board window lists.
    board: crate::boards::Board,
    maker: usize,
    /// The connection being edited in its window, the file it is saved to and the answer of the last test.
    conn: ConnectionConfig,
    conn_path: PathBuf,
    conn_result: Option<Result<(), String>>,
    conn_rx: Option<Receiver<Result<(), String>>>,
    fleet: fleet::FleetView,
    /// The drop-down list that is open.
    combo: Option<Combo>,
    /// The file of the job that is running, so that a job that is left in the middle can go on; the job that is offered; how many holes (or paths) it had done.
    recovery_path: Option<PathBuf>,
    recovery_at: Option<Instant>,
    recovery_done: usize,
    resume: Option<crate::recovery::Recovery>,
    resume_skip: usize,
    /// The calibration of the table.
    level: levelling::LevelState,
    /// The node of the drill head: what it said last, its camera.
    head: head::HeadState,
    /// The quality check through the camera of the drill head.
    insp: inspect::InspectState,
    /// The catalogue of bits: the kind that is shown (0 is all of them) and the first row.
    cat_kind: usize,
    cat_scroll: usize,
    cat_sel: Option<usize>,
    /// How far the description of the bit is scrolled (in units of the canvas), and what the last drawing found: how far it can go and where it is on the screen.
    cat_detail_scroll: f32,
    /// When the job that runs now began (for the time left).
    job_started: Option<std::time::Instant>,
    cat_detail_max: std::cell::Cell<f32>,
    cat_detail_area: std::cell::Cell<Rect>,
    /// The tool of the window of tools that the catalogue is picking a bit for.
    cat_target: Option<(Program, usize)>,
    /// The bit of the catalogue that each tool of the job took its numbers from (kept to warn when it is not suitable for the board).
    tool_bits: Vec<(Program, u8, crate::catalogue::Bit)>,
    /// Whether Ctrl and Shift are down at the click being handled (to choose several files).
    key_mods: (bool, bool),
    /// The widths of the left and of the right pane of the main screen (the middle one is what is left), and whether the window was set to full screen.
    panes: (f32, f32),
    fullscreen_applied: bool,
    /// What a move of the design left over when the offset of the machine (kept to 0.01 mm) took its part: it goes with the next move.
    move_rest: (f64, f64),
    /// How far the window on top has been dragged from its place (design units), the place of its card as it was drawn (design units, without that move) and the strip of the top
    /// that is taken to drag it (screen units).
    modal_off: (f32, f32),
    modal_card: Option<(f32, f32, f32, f32)>,
    modal_drag: Option<Rect>,
    /// Which programs of the job are on (the drilling, the routing) and the tools that are switched off; the programs that are on run one after the other.
    job_drill: bool,
    job_route: bool,
    off_tools: Vec<(Program, u8)>,
    /// The programs that wait for the one that runs.
    job_queue: Vec<Program>,
    /// The summary of a job that waits for the person's answer before a machine that moves is started: the program, the holes already done and the lines.
    summary_ask: Option<(Program, usize, Vec<String>)>,
    /// The tool changer (the ATC models): the loads of the rack still to come (the tools of each, by program), the one being shown, the pieces of the
    /// load still to run, the piece that runs and the holes to skip in the first one.
    rack_loads: Vec<Vec<(Program, Vec<u8>)>>,
    rack_show: Vec<(Program, Vec<u8>)>,
    rack_pieces: Vec<(Program, Vec<u8>)>,
    /// The tool the head holds now at the tool bar, worked out from where the head goes (see `carry_follow`), and where the head was at the last look.
    rack_carried: Option<(Program, u8)>,
    rack_head_prev: Option<(f64, f64)>,
    rack_only: Option<(Program, Vec<u8>)>,
    rack_active: bool,
    rack_skip: usize,
    rack_no: (usize, usize),
    /// How many places of the load the pieces that already ran take (the next piece takes its tools from the places after them).
    rack_first: usize,
    /// The program that ran last in this load left its last tool in the chuck for the next one.
    rack_chain: bool,
    /// The last tool of the program that ran last in this load (the one the chuck holds when it is handed on), and the tools of the whole load, whatever has run.
    rack_last: Option<(Program, u8)>,
    rack_load: Vec<(Program, Vec<u8>)>,
    /// The place the person gave to each tool of the job in the bar, and the tools of the job when the places were last matched with what was remembered.
    slot_assign: crate::slots::Assignment,
    slot_seen: Vec<crate::slots::JobTool>,
    /// The drilling is done and the routing waits for the person to accept (the window that follows the process); the pieces still to run wait in the same lists.
    stage_wait: bool,
    /// The clock of the animations, and when the screen and the window that are on show began to show (in seconds of that clock).
    anim0: Instant,
    anim_screen: (Option<Screen>, f32),
    anim_modal: (Option<Modal>, f32),
    /// Animations are off when a picture is taken: it must be the finished picture.
    animate: bool,
    /// The first tool shown in the list of the job when there are more than fit, the scroll bars of this frame and the one that is being dragged.
    tool_scroll: usize,
    bars: Vec<BarInfo>,
    bar_drag: Option<Bar>,
    /// The panel that moves the machine by hand.
    jog_ui: JogUi,
    /// What is logged of the window and of the state of the program (see `diag`).
    diag: diag::Diag,
}

impl App {
    pub fn new(o: &Options) -> App {
        // the tests each have their own settings, so what one counts or remembers is not read by another
        let settings_path = if cfg!(test) {
            static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let n = N.fetch_add(1, Ordering::SeqCst);
            std::env::temp_dir().join(format!("eh3d-bg-settings-{}-{n}.cfg", std::process::id()))
        } else {
            launcher::settings_path()
        };
        let settings = Settings::load(&settings_path);
        let jog_path = settings_path.with_extension("jog.cfg");
        let settings_panes = (settings.pane_left, settings.pane_right);
        let settings_fullscreen = settings.fullscreen;
        let settings_log_beat = settings.log.heartbeat_s;
        let lang = o.lang.unwrap_or(settings.lang);
        let theme_id = o.theme.unwrap_or(settings.theme);
        let dlang = drill_lang(lang);
        let data = |file: &str| {
            o.data
                .clone()
                .unwrap_or_else(|| eh3d_drillpro::config::find_data_dir(file))
        };
        let mut drill = eh3d_drillpro::app::App::new(dlang, &data("DRILLPRO.DCF"));
        let mut route = eh3d_routepro::app::App::new(dlang, &data("ROUTEPRO.RCF"));
        let mut connection = String::from("demo");
        let mut conn = ConnectionConfig::default();
        // the tests never read or write the machine file of the person: each run of them has its own
        let conn_file: Option<PathBuf> = o
            .connection
            .clone()
            .or_else(|| cfg!(test).then(test_machine_file));
        match eh3d_drillpro::protocol::connection::load(conn_file.as_deref()) {
            Ok(c) => {
                connection = c.describe();
                conn = c.clone();
                drill.set_connection(c.clone());
                route.set_connection(c);
            }
            Err(e) => {
                drill.set_error(e.clone());
                route.set_error(e);
            }
        }
        let board = settings.board;
        // the list of makers starts with "all makers" (place 0); the view the person left is the one that comes back
        let maker = if settings.all_makers {
            0
        } else {
            crate::boards::makers()
                .iter()
                .position(|m| *m == board.material().maker)
                .map_or(0, |i| i + 1)
        };
        let upd_history = settings_path.with_file_name("updates-history.log");
        let mut app = App {
            screen: o.screen.unwrap_or(match settings.choice {
                Some(Choice::Modern) => Screen::Viewer,
                _ => Screen::Launcher,
            }),
            settings,
            settings_path,
            lang,
            theme: Theme::for_id(theme_id),
            drill: Side::new(Box::new(drill)),
            route: Side::new(Box::new(route)),
            table: jobs::DEFAULT_TABLE,
            layers: [true; 4],
            remember: false,
            running: None,
            paused: false,
            message: None,
            pending_copy: None,
            report_lines: Vec::new(),
            connection,
            hits: Vec::new(),
            frame: 0,
            start_at_frame: o.start.then_some(4),
            check_at_frame: o.check.then_some(40),
            shot: o.shot.clone().map(|file| Shot {
                file,
                wait: o.wait,
                done: Arc::new(AtomicBool::new(false)),
            }),
            close: false,
            auto_ok: o.auto_ok,
            archive: None,
            archive_dir: std::env::temp_dir()
                .join(format!("eh3d-bungard-gui-{}", std::process::id())),
            base_dir: std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(Path::to_path_buf))
                .unwrap_or_else(|| PathBuf::from(".")),
            last_dir: PathBuf::new(),
            modal: None,
            modal_from: 0,
            browser: None,
            panel: PanelCfg::default(),
            panel_applied: None,
            saved: false,
            view2d: Nav2::default(),
            view3d: Nav3::default(),
            m3d: machine3d::Sim3d::new((0.0, 0.0)),
            m3d_first: None,
            zones: Vec::new(),
            drag_zone: None,
            ui_scale: 1.0,
            ui_origin: Pos2::ZERO,
            head_z: None,
            pointer: None,
            pointer_down: false,
            edit: None,
            estop: false,
            mat_scroll: 0,
            mat_family: 0,
            mat_query: String::new(),
            follow: false,
            move_board: false,
            move_design: false,
            com_ports: Vec::new(),
            start_at: Vec::new(),
            sel_holes: Default::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            sel_move: None,
            sel_move_from: None,
            sel_paths: Default::default(),
            band: None,
            select_mode: false,
            clip: edit::Clip::default(),
            paste_n: 0,
            edit_serial: 0,
            plan_xf: std::cell::Cell::new((0.0, 0.0, 0.0, 0.0, 0.0)),
            board_frozen: None,
            plan_big: false,
            rotation: 0,
            plan_scale: std::cell::Cell::new(1.0),
            settings_tab: 0,
            upd: Arc::default(),
            upd_cancel: Arc::new(AtomicBool::new(false)),
            upd_exe: update::program_to_replace(),
            upd_history,
            upd_seen: false,
            visuals_theme: None,
            drill_path: None,
            route_path: None,
            last_tick: Instant::now(),
            fixed_dt: None,
            board,
            maker,
            conn,
            conn_path: conn_file.unwrap_or_else(eh3d_drillpro::protocol::connection::default_path),
            conn_result: None,
            conn_rx: None,
            fleet: fleet::FleetView::default(),
            combo: None,
            recovery_path: App::recovery_file(),
            recovery_at: None,
            recovery_done: usize::MAX,
            resume: None,
            resume_skip: 0,
            level: levelling::LevelState::new(jobs::DEFAULT_TABLE),
            head: head::HeadState::default(),
            insp: inspect::InspectState::default(),
            cat_kind: 0,
            cat_scroll: 0,
            cat_sel: None,
            cat_detail_scroll: 0.0,
            job_started: None,
            cat_detail_max: std::cell::Cell::new(0.0),
            cat_detail_area: std::cell::Cell::new(Rect::NOTHING),
            cat_target: None,
            tool_bits: Vec::new(),
            key_mods: (false, false),
            panes: o
                .panes
                .map(|(l, r)| {
                    let (l, _, r) = crate::panes::split(l, r);
                    (l, r)
                })
                .unwrap_or_else(|| {
                    let (l, _, r) = crate::panes::split(settings_panes.0, settings_panes.1);
                    (l, r)
                }),
            fullscreen_applied: settings_fullscreen && o.shot.is_none(),
            move_rest: (0.0, 0.0),
            modal_off: (0.0, 0.0),
            modal_card: None,
            modal_drag: None,
            job_drill: true,
            job_route: true,
            off_tools: Vec::new(),
            job_queue: Vec::new(),
            summary_ask: None,
            rack_loads: Vec::new(),
            rack_show: Vec::new(),
            rack_pieces: Vec::new(),
            rack_carried: None,
            rack_head_prev: None,
            rack_only: None,
            rack_active: false,
            rack_skip: 0,
            rack_no: (0, 0),
            rack_first: 0,
            rack_chain: false,
            rack_last: None,
            rack_load: Vec::new(),
            slot_assign: Vec::new(),
            slot_seen: Vec::new(),
            stage_wait: false,
            anim0: Instant::now(),
            anim_screen: (None, 0.0),
            anim_modal: (None, 0.0),
            animate: o.shot.is_none(),
            tool_scroll: 0,
            bars: Vec::new(),
            bar_drag: None,
            jog_ui: JogUi::new(jog_path),
            diag: diag::Diag::new(settings_log_beat),
        };
        let examples = app.base_dir.join("examples");
        let has_jobs = std::fs::read_dir(&examples).is_ok_and(|rd| {
            rd.flatten()
                .any(|e| extra::is_job(&e.file_name().to_string_lossy()))
        });
        app.last_dir = if has_jobs {
            examples
        } else {
            app.drill.engine.dir()
        };
        if let Some(f) = &o.drill {
            app.load_drill(f);
        }
        if let Some(f) = &o.route {
            app.load_route(f);
        }
        if let Some(z) = &o.zip {
            app.open_zip(z);
        }
        for f in &o.open {
            app.open_path(f);
        }
        if let Some((c, r)) = o.panel {
            app.panel.columns = c.max(1);
            app.panel.rows = r.max(1);
            app.apply_panel();
        }
        if o.measure {
            app.level_reset(9, 6);
            app.level.measuring = true;
            while app.level.measuring {
                app.level_tick(Instant::now(), true);
            }
        }
        if let Some(n) = o.tabs {
            app.panel.tabs_on = true;
            app.panel.tab_count = n.clamp(1, 20);
            app.apply_panel();
        }
        if let Some(z) = o.zoom {
            app.view2d.zoom = z.clamp(0.3, 20.0);
        }
        if let Some(y) = o.yaw {
            app.view3d.yaw = y.clamp(-40.0, 40.0);
        }
        if let Some(t) = o.tab {
            app.settings_tab = t.min(3);
        }
        if let Some(c) = o.combo.as_deref() {
            use eh3d_drillpro::protocol::sound::Cue;
            let id = match c {
                "theme" => Some(ComboId::Theme),
                "lang" => Some(ComboId::Lang),
                "model" => Some(ComboId::Model),
                "finish" => Some(ComboId::Sound(Cue::Finished)),
                "tool" => Some(ComboId::Sound(Cue::ToolChange)),
                "maker" => Some(ComboId::Maker),
                "type" => Some(ComboId::MatType),
                "thickness" => Some(ComboId::Thickness),
                "size" => Some(ComboId::BoardSize),
                "tabs" => Some(ComboId::TabPlace),
                _ => None,
            };
            if let Some(id) = id {
                app.combo_open(id);
            }
        }
        if let Some(f) = &o.fleet_file {
            match std::fs::read_to_string(f)
                .map_err(|e| e.to_string())
                .and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string()))
                .and_then(|v| eh3d_fleet::Fleet::from_json(&v))
            {
                Ok(fl) => app.fleet.fleet = Some(fl),
                Err(e) => app.fleet.error = Some(format!("{}: {e}", f.display())),
            }
        }
        // a model asked for on the command line is shown, and not remembered
        if let Some(i) = o.machine.as_deref().and_then(machines::Model::from_name) {
            app.apply_model_with(i, false);
        }
        match o.modal.as_deref() {
            Some("about") => app.show_modal(Modal::About),
            Some("update") => app.show_modal(Modal::Update),
            Some("open") => app.show_modal(Modal::Open),
            Some("panel") => app.show_modal(Modal::Panel),
            Some("settings") => app.show_modal(Modal::Settings),
            Some("fleet") => app.show_modal(Modal::Fleet),
            Some("connection") => app.show_modal(Modal::Connection),
            Some("board") => app.show_modal(Modal::Board),
            Some("tools") => app.show_modal(Modal::Tools),
            Some("report") => app.show_modal(Modal::Report),
            Some("bar") => app.show_modal(Modal::Bar),
            Some("slots") => app.show_modal(Modal::Slots),
            Some("jog") => app.jog_act(JogAct::Toggle),
            Some("catalogue") => app.show_modal(Modal::Catalogue),
            Some("head") => app.show_modal(Modal::Head),
            Some("inspect") => {
                app.insp_open();
                if let Some(n) = o.hole {
                    app.insp.hole = n.saturating_sub(1);
                }
            }
            Some("inspect-tune") => app.show_modal(Modal::InspectTune),
            _ => {}
        }
        app.refit_table();
        if o.shot.is_some()
            && !matches!(o.modal.as_deref(), Some(m) if m != "jog")
            && !o.start
            && o.machine.is_none()
        {
            // a picture shows the job, not the corner of the table where it starts
            let mut all = app.drill.base.clone();
            all.overlay(&app.route.base);
            if let Some((x0, y0, x1, y1)) = all.bounds() {
                app.follow = false;
                app.view2d.pan = (0.0, 0.0);
                app.view2d.focus = Some(((x0 + x1) / 2.0, (y0 + y1) / 2.0));
                let k = (app.table.width_mm / (x1 - x0 + 14.0))
                    .min(app.table.depth_mm / (y1 - y0 + 14.0));
                app.view2d.zoom = (k * 0.8).clamp(0.5, 20.0) as f32;
            }
        }
        if o.select && o.modal.as_deref() == Some("open") {
            // the first file of the folder is the one chosen in the Open window, with its preview
            let first = app.browser.as_ref().and_then(|b| {
                b.entries
                    .iter()
                    .position(|e| e.kind == opener::EntryKind::Drill)
            });
            if let Some(i) = first {
                app.browse(i);
            }
        } else if o.select {
            app.select_mode = true;
            if let Some((x0, y0, x1, y1)) = app.board_rect() {
                app.select_in_box((x0, y0), (x0 + (x1 - x0) * 0.45, y1), false, false);
            }
        }
        let nothing_asked = o.shot.is_none()
            && o.drill.is_none()
            && o.route.is_none()
            && o.zip.is_none()
            && o.open.is_empty()
            && o.modal.is_none();
        if nothing_asked {
            app.offer_resume();
        }
        if o.shot.is_none() {
            app.upd_start();
        }
        app
    }

    fn tr(&self, t: Text) -> &'static str {
        tr(self.lang, t)
    }

    fn refit_table(&mut self) {
        self.table = self.drill.table();
        self.drill.scene.table = self.table;
        self.route.scene.table = self.table;
    }

    fn load_drill(&mut self, path: &Path) {
        crate::applog::info(
            "file",
            format!("loading the drilling file {}", path.display()),
        );
        self.drill_path = Some(path.to_path_buf());
        let turned = self.source_for(path, true);
        let path = turned.as_path();
        self.drill.engine.load(path);
        // the size limits of the readers (reader_limits.cfg of the shared crate): a file past a hard limit is not drawn, one past a warning limit is drawn and said
        let (file, big) = match jobs::read_latin1(path)
            .ok()
            .and_then(|t| eh3d_drillpro::ncd::parse_limited(&t, &ReaderLimits::default()).ok())
        {
            Some((f, verdict)) => (
                Some(f),
                eh3d_drillpro::summary::size_warning(verdict, self.lang),
            ),
            None => (None, None),
        };
        let (unit, offset, scale) = self
            .drill
            .engine
            .settings()
            .map_or((0.0254, (0.0, 0.0), (1.0, 1.0)), |s| {
                (s.unit.mm(), s.offset_mm, s.scale)
            });
        match file {
            Some(f) => {
                self.drill.scene = jobs::scene_from_drill(&f, self.table, unit);
                self.drill.offset = offset;
                // the job takes the scale of the machine and then the offset: the drawing does the same, in the same order
                for h in &mut self.drill.scene.holes {
                    h.x_mm = h.x_mm * scale.0 + offset.0;
                    h.y_mm = h.y_mm * scale.1 + offset.1;
                }
                self.drill.base = self.drill.scene.clone();
                self.message = big;
                crate::applog::info(
                    "file",
                    format!(
                        "drilling file loaded: {} holes, {} tools, unit {} mm",
                        self.drill.scene.holes.len(),
                        self.drill.scene.sizes().len(),
                        unit
                    ),
                );
            }
            None => {
                let why = self.drill.engine.error().unwrap_or("?").to_string();
                crate::applog::error("file", format!("the drilling file was not loaded: {why}"));
                self.message = Some(why)
            }
        }
        self.drill.last_done = 0;
    }

    /// The chords of a whole circle of a plotter file: the chord angle of the routing configuration says it.
    pub(crate) fn route_segments(&self) -> u32 {
        self.route
            .engine
            .settings()
            .map_or(eh3d_routepro::hpgl::SEGMENTS_PER_CIRCLE, |s| {
                eh3d_routepro::hpgl::segments_for_chord(s.chord_angle)
            })
    }

    /// The outline of a board in Gerber, or the drawing of it in DXF, written as a plotter file in the work folder so that the routing
    /// engine can take it as it takes any other. A Gerber file that is not an outline (a layer of copper) is said, not loaded.
    fn drawing_as_plot(&mut self, path: &Path) -> Option<PathBuf> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let text = jobs::read_latin1(path).ok()?;
        let segments = self.route_segments();
        let is_dxf = name.to_ascii_lowercase().ends_with(".dxf");
        if !is_dxf && !eh3d_bungard_formats::gerber::looks_like_outline(&name, &text) {
            self.message = Some(format!("{name}: {}", self.tr(Text::NotAnOutline)));
            return None;
        }
        let plot = if is_dxf {
            eh3d_bungard_formats::dxf::outline(&text, segments)
        } else {
            eh3d_bungard_formats::gerber::outline(&text, segments)
        };
        match plot {
            Ok(plot) => {
                let _ = std::fs::create_dir_all(&self.archive_dir);
                let out = self
                    .archive_dir
                    .join(format!("{}.plt", path.file_stem()?.to_string_lossy()));
                std::fs::write(&out, plot.to_hpgl()).ok()?;
                Some(out)
            }
            Err(e) => {
                self.message = Some(format!("{name}: {e}"));
                None
            }
        }
    }

    fn load_route(&mut self, path: &Path) {
        let is_drawing = path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .is_some_and(|e| {
                e == eh3d_bungard_formats::dxf::EXTENSION
                    || eh3d_bungard_formats::gerber::OUTLINE_EXTENSIONS.contains(&e.as_str())
            });
        let converted;
        let path = if is_drawing {
            match self.drawing_as_plot(path) {
                Some(p) => {
                    converted = p;
                    converted.as_path()
                }
                None => return,
            }
        } else {
            path
        };
        crate::applog::info(
            "file",
            format!("loading the routing file {}", path.display()),
        );
        self.route_path = Some(path.to_path_buf());
        let turned = self.source_for(path, false);
        let path = turned.as_path();
        self.route.engine.load(path);
        let segments = self.route_segments();
        let (plot, big) = match jobs::read_latin1(path).ok().and_then(|t| {
            eh3d_routepro::hpgl::parse_limited(&t, segments, &ReaderLimits::default()).ok()
        }) {
            Some((p, verdict)) => (
                Some(p),
                eh3d_drillpro::summary::size_warning(verdict, self.lang),
            ),
            None => (None, None),
        };
        let offset = self
            .route
            .engine
            .settings()
            .map_or((0.0, 0.0), |s| s.offset_mm);
        if let Some(p) = plot {
            self.route.scene = jobs::scene_from_plot(&p, self.table);
            self.route.offset = offset;
            for path in &mut self.route.scene.paths {
                for pt in &mut path.points {
                    pt.0 += offset.0;
                    pt.1 += offset.1;
                }
            }
            self.route.base = self.route.scene.clone();
            if big.is_some() {
                self.message = big;
            }
            crate::applog::info(
                "file",
                format!(
                    "routing file loaded: {} paths",
                    self.route.scene.paths.len()
                ),
            );
        } else {
            crate::applog::error("file", "the routing file could not be read");
        }
        self.route.last_done = 0;
    }

    /// Opens a job archive: its first drilling file and its first routing file are loaded and the others can be chosen from the list.
    fn open_zip(&mut self, path: &Path) {
        crate::applog::info("file", format!("opening the archive {}", path.display()));
        match Archive::open(path) {
            Ok(a) => {
                let first_drill = a.first(Kind::Drill).map(|e| e.name.clone());
                let first_route = a.first(Kind::Route).map(|e| e.name.clone());
                self.archive = Some(a);
                for name in [first_drill, first_route].into_iter().flatten() {
                    self.load_named(&name);
                }
                self.refit_table();
            }
            Err(e) => {
                crate::applog::error("file", format!("the archive was not opened: {e}"));
                self.message = Some(e.to_string())
            }
        }
    }

    /// Takes a file of the archive out into the work folder and loads it as what its extension says.
    fn load_named(&mut self, name: &str) {
        let Some(a) = self.archive.as_mut() else {
            return;
        };
        match a.extract(name, &self.archive_dir) {
            Ok(p) => match eh3d_archive::kind_of(name) {
                Kind::Drill => self.load_drill(&p),
                Kind::Route => self.load_route(&p),
                Kind::Other => {}
            },
            Err(e) => {
                crate::applog::error(
                    "file",
                    format!("{name}: it was not taken out of the archive: {e}"),
                );
                self.message = Some(e.to_string())
            }
        }
    }

    fn load_entry(&mut self, i: usize) {
        let name = self
            .archive
            .as_ref()
            .and_then(|a| a.entries.iter().filter(|e| e.kind != Kind::Other).nth(i))
            .map(|e| e.name.clone());
        if let Some(n) = name {
            self.load_named(&n);
            self.refit_table();
        }
    }

    fn remember_settings(&mut self) {
        self.settings.lang = self.lang;
        self.settings.theme = self.theme.id;
        if let Err(e) = self.settings.save(&self.settings_path) {
            crate::applog::error(
                "settings",
                format!("cannot save {}: {e}", self.settings_path.display()),
            );
            self.message = Some(format!("{}: {e}", self.settings_path.display()));
        }
    }

    /// The sizes of the holes of both files, from the smallest.
    fn sizes(&self) -> Vec<f64> {
        let mut all = self.drill.scene.clone();
        all.overlay(&self.route.scene);
        all.sizes()
    }

    fn active(&mut self) -> Option<(Program, &mut Side)> {
        if self.drill.engine.file_loaded() {
            Some((Program::Drill, &mut self.drill))
        } else if self.route.engine.file_loaded() {
            Some((Program::Route, &mut self.route))
        } else {
            None
        }
    }

    fn start_job(&mut self) {
        self.start_job_from(0);
    }

    /// The tools of a program that are switched off in the job.
    pub(crate) fn tools_off(&self, p: Program) -> Vec<u8> {
        let mut off: Vec<u8> = self
            .off_tools
            .iter()
            .filter(|(q, _)| *q == p)
            .map(|(_, t)| *t)
            .collect();
        // the tool changer: only the tools of the load that is in the rack
        if let Some((q, keep)) = &self.rack_only {
            if *q == p {
                for t in self.job_tool_order(p) {
                    if !keep.contains(&t) && !off.contains(&t) {
                        off.push(t);
                    }
                }
            }
        }
        // a program that starts at a tool leaves out the ones that come before it
        if let Some(&(_, first)) = self.start_at.iter().find(|(q, _)| *q == p) {
            for t in self.job_tool_order(p) {
                if t == first {
                    break;
                }
                if !off.contains(&t) {
                    off.push(t);
                }
            }
        }
        off
    }

    /// The tools of a program in the order the job does them: the one that the file uses first comes first.
    pub(crate) fn job_tool_order(&self, p: Program) -> Vec<u8> {
        let mut order: Vec<u8> = Vec::new();
        let tools: Box<dyn Iterator<Item = u8> + '_> = match p {
            Program::Drill => Box::new(self.drill.base.holes.iter().map(|h| h.tool)),
            Program::Route => Box::new(self.route.base.paths.iter().map(|q| q.tool)),
        };
        for t in tools {
            if !order.contains(&t) {
                order.push(t);
            }
        }
        order
    }

    pub(crate) fn toggle_start_at(&mut self, p: Program, tool: u8) {
        if let Some(i) = self.start_at.iter().position(|x| *x == (p, tool)) {
            self.start_at.remove(i);
        } else {
            self.start_at.retain(|(q, _)| *q != p);
            self.start_at.push((p, tool));
        }
    }

    pub(crate) fn tool_is_on(&self, p: Program, tool: u8) -> bool {
        !self.off_tools.contains(&(p, tool))
    }

    /// Whether a program of the job (drilling or routing) is on, and has a file and a tool that is on.
    pub(crate) fn program_runs(&self, p: Program) -> bool {
        let (on, loaded) = match p {
            Program::Drill => (self.job_drill, self.drill.engine.file_loaded()),
            Program::Route => (self.job_route, self.route.engine.file_loaded()),
        };
        on && loaded
            && match p {
                Program::Drill => self
                    .drill
                    .scene
                    .holes
                    .iter()
                    .any(|h| self.tool_is_on(p, h.tool)),
                Program::Route => self
                    .route
                    .scene
                    .paths
                    .iter()
                    .any(|q| self.tool_is_on(p, q.tool)),
            }
    }

    /// The programs a job does, in order: the drilling and then the routing, the ones that are on.
    fn job_programs(&self) -> Vec<Program> {
        [Program::Drill, Program::Route]
            .into_iter()
            .filter(|p| self.program_runs(*p))
            .collect()
    }

    pub(crate) fn toggle_job_part(&mut self, p: Program) {
        match p {
            Program::Drill => self.job_drill = !self.job_drill,
            Program::Route => self.job_route = !self.job_route,
        }
    }

    pub(crate) fn toggle_tool(&mut self, p: Program, tool: u8) {
        if let Some(i) = self.off_tools.iter().position(|x| *x == (p, tool)) {
            self.off_tools.remove(i);
        } else {
            self.off_tools.push((p, tool));
        }
    }

    /// Starts the job leaving out the first `skip` holes (or paths) of it, which are already done. Without a skip it does the drilling and the routing that are on, one after the other.
    fn start_job_from(&mut self, skip: usize) {
        // the machine is the job's now: the panel lets go of it
        self.jog_yield();
        let mut programs = self.job_programs();
        if skip > 0 {
            programs = self.active().map(|(p, _)| p).into_iter().collect();
        }
        self.start_programs(programs, skip);
    }

    /// Starts the programs one after the other; the first leaves out its first `skip` holes (or paths).
    pub(crate) fn start_programs(&mut self, programs: Vec<Program>, skip: usize) {
        crate::applog::info("job", format!("start requested: {programs:?}, skipping {skip}; connection {}, model {}, emergency stop {}", self.connection, self.settings.model, self.estop));
        if programs.is_empty() {
            if self.drill.engine.file_loaded() || self.route.engine.file_loaded() {
                self.message = Some(self.tr(Text::JobNothingOn).to_string());
            }
            return;
        }
        // a board bigger than the table (turned too) is said before the job starts; the job is not stopped by it
        if let Some((head, detail)) = self.board_fit_warning() {
            self.message = Some(format!("{head}: {detail}"));
        }
        // a job that does not fit on the table is refused before it starts, not found out on the machine
        let outside = programs.iter().any(|p| {
            let (holes, paths) = match p {
                Program::Drill => self.drill.scene.outside_table(),
                Program::Route => self.route.scene.outside_table(),
            };
            !holes.is_empty() || !paths.is_empty()
        });
        if outside {
            self.message = Some(self.tr(Text::JobOutsideTable).to_string());
            return;
        }
        // the models with the tool changer need every tool of the job in a place of the bar of its own, and said so, before anything moves
        if self.has_changer() && !self.slots_ready() {
            return;
        }
        // the models with the tool changer load the rack first (and again when the tools do not fit all at once)
        if self.has_changer() {
            let loads = self.rack_plan(&programs);
            if !loads.is_empty() {
                self.rack_active = true;
                self.rack_skip = skip;
                self.rack_no = (0, loads.len());
                self.rack_loads = loads;
                self.job_queue.clear();
                self.rack_next();
                return;
            }
        }
        self.rack_active = false;
        self.job_queue = programs[1..].to_vec();
        self.begin_program(programs[0], skip);
    }

    /// Starts one program of the job (the rest wait in the queue).
    fn begin_program(&mut self, program: Program, skip: usize) {
        self.begin_program_with(program, skip, false);
    }

    /// The same; with `started` the engine already started the job (the person accepted its summary) and only the rest is done.
    fn begin_program_with(&mut self, program: Program, skip: usize, started: bool) {
        let ask_operator = self.settings.ask_operator;
        let levelled = self.level_for_jobs();
        let skip_tools = self.tools_off(program);
        // a single board with tabs is a panel of one: the outside of it is routed with the tabs too
        let panel = match (self.panel_applied, self.panel.tabs()) {
            (Some((columns, rows, pitch_mm)), tabs) => Some(Panel {
                columns,
                rows,
                pitch_mm,
                tabs,
            }),
            (None, Some(tabs)) => Some(Panel {
                columns: 1,
                rows: 1,
                pitch_mm: (0.0, 0.0),
                tabs: Some(tabs),
            }),
            (None, None) => None,
        };
        // a program whose machine file has no places for the bar of the changer is planned with the places the bar is drawn in
        let lent_bar = self
            .bar_for_job(program)
            .map(|line| self.lend_bar(program, &line));
        // the end of a program that another one follows is not the end of the job: the engine does not sound it (the notice between the parts is the GUI's)
        let more_follows = self.next_program().is_some();
        let (rack_first, rack_carry_in, rack_carry_out) = if self.rack_active {
            (
                self.rack_first,
                self.rack_first > 0 && self.rack_chain,
                // the last tool stays in the chuck only for a program that changes its tools by itself
                self.rack_pieces
                    .first()
                    .is_some_and(|piece| self.program_has_rack(piece.0)),
            )
        } else {
            (0, false, false)
        };
        self.rack_chain = rack_carry_out;
        // the places the person gave to the tools of this program, and the one of the tool the chuck comes holding
        let (rack_slots, rack_carry_slot) = self.slots_for_program(program, rack_carry_in);
        self.set_end_sound(program, more_follows);
        if !started {
            let side = self.side_mut(program);
            // nothing outside the table of the machine is sent to it: those vectors are left out and counted
            let area = side.table();
            side.engine.set_job_options(JobOptions {
                ask_operator,
                panel,
                skip,
                skip_tools,
                table: levelled,
                clip_mm: Some((area.width_mm, area.depth_mm)),
                rack_first,
                rack_carry_in,
                rack_carry_out,
                rack_slots,
                rack_carry_slot,
                ..Default::default()
            });
            side.engine.start();
        }
        if let Some(before) = lent_bar {
            self.restore_bar(program, before);
        }
        {
            let p = program;
            let side = self.side_mut(p);
            // the engine refuses a job that would run on made-up data (a tool missing from the table, a gap in the rack, a height map through the server): the person is told and nothing runs
            if let Some(why) = side.engine.error() {
                let why = why.to_string();
                self.job_queue.clear();
                self.message = Some(why);
                return;
            }
            // a machine that moves: the engine shows the summary of the job (what it does, how long, how far the axes go, what looks wrong) and waits
            let pending = side.engine.pending_summary().map(|l| l.to_vec());
            if let Some(lines) = pending {
                self.summary_ask = Some((program, skip, lines));
                return;
            }
            // what the person should know and does not stop the job (tools that run with the cautious values)
            let warning = side.engine.warning().map(str::to_string);
            // the first point of the work, for the machine of the 3D view to travel to (the engine says where the job begins)
            // (a job whose steps the engine follows starts from home like the machine does: the head goes where the plan says)
            let first = if side.engine.head_mm().is_some() {
                None
            } else {
                side.engine
                    .job_view()
                    .map(|v| (v.x_mm, v.y_mm))
                    .or_else(|| side.scene.holes.first().map(|h| (h.x_mm, h.y_mm)))
                    .or_else(|| {
                        side.scene
                            .paths
                            .first()
                            .and_then(|q| q.points.first().copied())
                    })
            };
            side.last_done = 0;
            if skip == 0 {
                for h in &mut side.scene.holes {
                    h.done = false;
                }
                for pth in &mut side.scene.paths {
                    pth.reached = 0;
                }
            }
            if warning.is_some() {
                self.message = warning;
            }
            self.m3d_first = first;
            self.running = Some(p);
            self.job_started = Some(std::time::Instant::now());
            self.m3d.begin(self.m3d_first);
            self.paused = false;
            self.resume_skip = skip;
            self.recovery_done = usize::MAX;
            self.recovery_at = None;
            self.save_recovery();
        }
    }

    /// The program that runs after the one that runs now, if the job has one: the next piece of the load of the rack, the first of the next load, or the next of
    /// the queue.
    pub(crate) fn next_program(&self) -> Option<Program> {
        if self.rack_active {
            self.rack_pieces
                .first()
                .or_else(|| self.rack_loads.first().and_then(|l| l.first()))
                .map(|x| x.0)
        } else {
            self.job_queue.first().copied()
        }
    }

    /// Whether a program of the job is planned with the tool changer: its machine file has the places of the bar and asks for the automatic change, or the places are lent to it.
    fn program_has_rack(&self, p: Program) -> bool {
        let side = match p {
            Program::Drill => &self.drill,
            Program::Route => &self.route,
        };
        self.bar_for_job(p).is_some()
            || side
                .engine
                .settings()
                .is_some_and(|s| s.auto_change && !s.atc.is_empty())
    }

    /// Whether the end of this program is a part of the job being done and not the end of the job: the drilling is done and the routing comes next.
    fn stage_follows(&self, finished: Program) -> bool {
        finished == Program::Drill && self.next_program() == Some(Program::Route)
    }

    /// Gives the engine of a program the sounds of the window, but with no sound of the end of the job when another program follows it.
    fn set_end_sound(&mut self, program: Program, quiet: bool) {
        let mut conn = self.conn.clone();
        if quiet {
            conn.sound_finish = eh3d_drillpro::protocol::sound::SoundSetting::Off;
        }
        self.side_mut(program).engine.set_connection(conn);
    }

    /// Marks the paths the head is cutting at this moment: the ones of the tools that are on, along the stretch the head has reached. The head cuts when the tool is into
    /// the board by more than the margin of the settings; above that it travels from one path to the next and marks nothing.
    pub(crate) fn mark_route(&mut self, off: &[u8]) {
        let cutting = self
            .route
            .engine
            .depth_below_surface_mm()
            .is_some_and(|d| d >= self.settings.route_mark_min_depth_mm);
        let Some(head) = self.route.engine.head_mm() else {
            return;
        };
        if !cutting {
            return;
        }
        let tol = self.settings.route_mark_tolerance_mm;
        for q in self
            .route
            .scene
            .paths
            .iter_mut()
            .filter(|q| !off.contains(&q.tool))
        {
            crate::jobinfo::advance_reached(q, (head.0, head.1), tol);
        }
    }

    /// Advances the routing job by `dt` seconds of machine time in steps of at most the length the settings say, marking after each one the paths the head cuts.
    fn route_step(&mut self, dt: f64, off: &[u8]) -> bool {
        let step = self.settings.route_mark_step_s.max(0.005);
        let mut left = dt;
        let mut more = true;
        // a very fast simulation does the rest in one go
        for _ in 0..400 {
            if left <= 0.0 || !more {
                break;
            }
            let d = left.min(step);
            left -= d;
            more = self.route.engine.tick_for(d);
            self.mark_route(off);
            if self.route.engine.question().is_some() {
                return more;
            }
        }
        if left > 0.0 && more {
            more = self.route.engine.tick_for(left);
            self.mark_route(off);
        }
        more
    }

    /// Adds the holes of the drilling job that are done to the counts of their drills, which the settings keep.
    fn count_holes(&mut self) {
        let mut any = false;
        for h in self.drill.scene.holes.iter().filter(|h| h.done) {
            *self.settings.holes_drilled.entry(h.tool).or_insert(0) += 1;
            any = true;
        }
        if any {
            self.remember_settings();
        }
    }

    fn stop_job(&mut self) {
        crate::applog::info("job", format!("stop requested; running {:?}", self.running));
        self.paused = false;
        self.stage_wait = false;
        self.job_queue.clear();
        self.rack_clear();
        self.clear_recovery();
        if self.running == Some(Program::Drill) {
            self.count_holes();
        }
        if let Some(p) = self.running.take() {
            self.side_mut(p).engine.stop();
        }
        self.m3d.finish();
    }

    fn side_mut(&mut self, p: Program) -> &mut Side {
        match p {
            Program::Drill => &mut self.drill,
            Program::Route => &mut self.route,
        }
    }

    /// Advances the engine and takes from it what the views show.
    fn tick(&mut self) {
        self.slots_sync();
        self.carry_follow();
        self.fleet_tick();
        if let Some(r) = self.conn_rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            self.conn_result = Some(r);
            self.conn_rx = None;
        }
        let now = Instant::now();
        self.level_tick(now, self.fixed_dt.is_some());
        let real = self
            .fixed_dt
            .unwrap_or_else(|| now.duration_since(self.last_tick).as_secs_f64().min(3600.0));
        self.last_tick = now;
        let dt = real * self.settings.sim_speed as f64;
        let Some(p) = self.running else {
            // no job: the machine finishes what it was doing (it goes home) and rests
            self.m3d.advance(dt);
            return;
        };
        if self.paused {
            return;
        }
        // the machine powers up, goes home and travels to the first point before the job moves; the engine waits for it
        if self.m3d.phase() == machine3d::Phase::Parked {
            self.m3d.begin(self.m3d_first);
        }
        if !self.m3d.advance(dt) {
            return;
        }
        let auto_ok = self.auto_ok;
        let skip = self.resume_skip;
        let off = self.tools_off(p);
        let side = self.side_mut(p);
        if side.engine.question().is_some() {
            if auto_ok {
                side.engine.answer(true);
            }
            return;
        }
        // the routing goes on in short steps, looking at where the head is after each one, so that the paths are marked as they are cut
        let more = if p == Program::Route {
            self.route_step(dt, &off)
        } else {
            side.engine.tick_for(dt)
        };
        let side = self.side_mut(p);
        let done = side.engine.done();
        let view = side.engine.job_view();
        // the head goes where the steps of the plan take it (the trips to the tool bar included); an engine that does not follow them says where the next hole is
        let track = side.engine.head_mm();
        let head = track
            .map(|h| (h.0, h.1))
            .or_else(|| view.as_ref().map(|v| (v.x_mm, v.y_mm)));
        let head_z = track.map(|h| h.2);
        let percent = view.as_ref().map_or(0, |v| v.percent);
        if p == Program::Drill {
            // the hole that was drilled is the one nearest to the head (the engine drills them in its own order)
            // the engine says in which order it drills: the hole number k is the k-th of its plan (its view shows the next hole, not the one drilled)
            let order = if side.last_done < done {
                side.engine.hole_order()
            } else {
                Vec::new()
            };
            while side.last_done < done {
                let (hx, hy) = order
                    .get(side.last_done)
                    .copied()
                    .or(head)
                    .unwrap_or((0.0, 0.0));
                let best = side
                    .scene
                    .holes
                    .iter()
                    .enumerate()
                    .filter(|(_, h)| !h.done && !off.contains(&h.tool))
                    .min_by(|a, b| {
                        let da = (a.1.x_mm - hx).hypot(a.1.y_mm - hy);
                        let db = (b.1.x_mm - hx).hypot(b.1.y_mm - hy);
                        da.total_cmp(&db)
                    })
                    .map(|(i, _)| i);
                if let Some(i) = best {
                    side.scene.holes[i].done = true;
                }
                side.last_done += 1;
            }
        } else if track.is_none() {
            // an engine that does not follow the steps of the plan: a job that went on leaves out the paths that were done, and the rest share the percent
            let total: usize = side
                .scene
                .paths
                .iter()
                .skip(skip)
                .filter(|q| !off.contains(&q.tool))
                .map(|q| q.points.len())
                .sum();
            let mut left = total * percent as usize / 100;
            for q in side
                .scene
                .paths
                .iter_mut()
                .skip(skip)
                .filter(|q| !off.contains(&q.tool))
            {
                q.reached = left.min(q.points.len());
                left = left.saturating_sub(q.points.len());
            }
        }
        if !more && side.engine.outcome() != eh3d_drillpro::classic::engine::Outcome::Completed {
            // a stopped or failed job is not a finished one: nothing is marked as done, the file to resume from stays and the queue stops
            let why = match side.engine.outcome() {
                eh3d_drillpro::classic::engine::Outcome::Failed(why) => why,
                _ => String::new(),
            };
            self.running = None;
            self.rack_pieces.clear();
            self.rack_loads.clear();
            self.job_queue.clear();
            self.m3d.finish();
            if !why.is_empty() {
                self.message = Some(why);
            }
        } else if !more {
            if p == Program::Drill {
                for h in side
                    .scene
                    .holes
                    .iter_mut()
                    .filter(|h| !off.contains(&h.tool))
                {
                    h.done = true;
                }
            } else {
                for q in side
                    .scene
                    .paths
                    .iter_mut()
                    .filter(|q| !off.contains(&q.tool))
                {
                    q.reached = q.points.len();
                }
            }
            crate::applog::info(
                "job",
                format!(
                    "{p:?} finished: {} holes done, {} paths done",
                    self.drill.last_done, self.route.last_done
                ),
            );
            self.running = None;
            self.clear_recovery();
            if p == Program::Drill {
                self.count_holes();
            }
            if self.stage_follows(p) {
                // the drilling is done and the routing comes next: the person is told (by sound and in a window) and goes on when ready
                self.stage_wait = true;
                crate::applog::info("sound", "notice between the stages");
                eh3d_drillpro::protocol::sound::play_stage(&self.conn.sound_finish);
                self.message = Some(self.tr(Text::StageDoneTitle).to_string());
            } else {
                self.continue_job();
            }
        } else {
            self.save_recovery();
        }
        self.head_z = head_z;
        // the head of the machine goes where the job says, once the machine is working
        if let Some(h) = head {
            self.m3d.follow(h);
        }
    }

    /// Goes on with the next program of the job (the routing after the drilling) without the machine going home; when there is none the job is over, and the
    /// person is told that the board can be taken out.
    fn continue_job(&mut self) {
        self.stage_wait = false;
        match crate::jobflow::after_program(
            self.rack_active,
            self.rack_pieces.len(),
            self.rack_loads.len(),
            self.job_queue.len(),
        ) {
            crate::jobflow::Next::Piece => {
                // the next program takes its tools from the places after the ones of the program that ran, and the chuck is handed on to it
                let (first, last) =
                    crate::jobflow::hand_over(self.rack_first, self.rack_only.as_ref());
                self.rack_first = first;
                self.rack_last = last;
                let next = self.rack_pieces.remove(0);
                self.rack_only = Some(next.clone());
                self.begin_program(next.0, 0);
            }
            crate::jobflow::Next::Reload => {
                // the rack has to be loaded again
                self.rack_only = None;
                self.m3d.finish();
                self.rack_next();
            }
            crate::jobflow::Next::Over if self.rack_active => {
                self.rack_clear();
                self.m3d.finish();
                self.job_over();
            }
            crate::jobflow::Next::Over => {
                self.m3d.finish();
                self.job_over();
            }
            crate::jobflow::Next::Program => {
                let next = self.job_queue.remove(0);
                self.begin_program(next, 0);
            }
        }
    }

    /// The whole job is done: the text that goes with the sound of the end.
    fn job_over(&mut self) {
        self.message = Some(self.tr(Text::JobFinishedRemove).to_string());
    }

    fn choose(&mut self, c: Choice) {
        crate::applog::info(
            "launcher",
            format!("choice {c:?}, remembered {}", self.remember),
        );
        if self.remember {
            self.settings.choice = Some(c);
            self.remember_settings();
        }
        match c {
            Choice::Modern => self.screen = Screen::Viewer,
            classic => match launcher::classic_launch(classic, &self.base_dir) {
                Some(l) => match std::process::Command::new(&l.exe).args(&l.args).spawn() {
                    Ok(_) => {
                        crate::applog::info(
                            "launcher",
                            format!("classic program started: {}", l.exe.display()),
                        );
                        self.close = true
                    }
                    Err(e) => {
                        crate::applog::error(
                            "launcher",
                            format!("cannot start {}: {e}", l.exe.display()),
                        );
                        self.message = Some(format!("{}: {e}", l.exe.display()))
                    }
                },
                None => self.message = Some(self.tr(Text::ClassicMissing).to_string()),
            },
        }
    }

    fn act(&mut self, a: Act) {
        crate::applog::info("action", format!("{a:?}"));
        // the design is locked while the machine works: what would change it is refused (the views still move and zoom)
        if self.design_locked()
            && matches!(
                a,
                Act::ToggleTabs
                    | Act::PanelMode(_)
                    | Act::BoardSwap
                    | Act::Rotate(_)
                    | Act::BoardAdj(..)
                    | Act::Board(_)
                    | Act::CutSel
                    | Act::PasteSel
                    | Act::DeleteSel
                    | Act::UndoEdit
                    | Act::RedoEdit
                    | Act::RemoveTool(..)
            )
        {
            return;
        }
        match a {
            Act::Choose(c) => self.choose(c),
            Act::Remember => self.remember = !self.remember,
            Act::NextTheme => {
                let i = ThemeId::ALL
                    .iter()
                    .position(|t| *t == self.theme.id)
                    .unwrap_or(0);
                self.theme = Theme::for_id(ThemeId::ALL[(i + 1) % ThemeId::ALL.len()]);
                self.remember_settings();
            }
            Act::NextLang => {
                let i = Lang::ALL.iter().position(|l| *l == self.lang).unwrap_or(0);
                self.lang = Lang::ALL[(i + 1) % Lang::ALL.len()];
                let dl = drill_lang(self.lang);
                self.drill.engine.set_lang(dl);
                self.route.engine.set_lang(dl);
                self.remember_settings();
            }
            Act::Layer(i) => self.layers[i] = !self.layers[i],
            Act::Entry(i) => {
                self.load_entry(i);
                self.apply_panel();
            }
            Act::Show(m) => self.show_modal(m),
            Act::CloseModal => self.modal = None,
            Act::Board(p) => self.board_pick(p),
            Act::BoardAdj(width, d) => self.board_adjust(width, d),
            Act::ToolAdj(p, row, c, d) => self.tool_adjust(p, row, c, d),
            Act::ToolsSave => self.tools_save(),
            Act::Emergency => self.emergency(),
            Act::SoundTest(c) => self.sound_test(c),
            Act::ToggleFollow => {
                self.follow = !self.follow;
                if self.follow {
                    self.view2d.focus = None;
                    self.view2d.pan = (0.0, 0.0);
                    self.view2d.zoom = self.view2d.zoom.max(3.0);
                }
            }
            Act::ToggleMoveBoard => {
                self.move_board = !self.move_board;
                if self.move_board {
                    self.move_design = false;
                }
            }
            Act::ToggleMoveDesign => self.toggle_move_design(),
            Act::ToggleBig => self.plan_big = !self.plan_big,
            Act::Rotate(q) => self.rotate_work(q),
            Act::BoardSwap => self.board_swap(),
            Act::FitPlan(b) => self.fit_plan(b),
            Act::CenterTool(p, t) => self.center_on_tool(p, t),
            Act::Jog(a) => self.jog_act(a),
            Act::ToggleEmergency => {
                self.settings.emergency = !self.settings.emergency;
                self.remember_settings();
            }
            Act::Tab(i) => self.settings_tab = i,
            Act::FileUnit(i) => self.set_file_unit(i),
            Act::ToggleFullscreen => {
                self.settings.fullscreen = !self.settings.fullscreen;
                self.remember_settings();
            }
            Act::LogToggle => self.log_toggle(),
            Act::LogLevel(l) => self.log_set_level(l),
            Act::LogSize(kb) => self.log_set_size(kb),
            Act::LogOpenFolder => self.log_open_folder(),
            Act::LogCopyPath => self.log_copy_path(),
            Act::ReportCopy => self.report_copy(),
            Act::DisplayMil(v) => {
                self.settings.display_mil = v;
                self.remember_settings();
            }
            Act::Edit(t, x, y, w) => self.start_edit(t, x, y, w),
            Act::ToolDefault(p, row) => self.tool_default(p, row),
            Act::ToolsDefaults => self.tools_defaults(),
            Act::ResetHoles => {
                self.settings.holes_drilled.clear();
                self.remember_settings();
            }
            Act::SetSpeed(v) => {
                self.settings.sim_speed = v;
                self.remember_settings();
            }
            Act::ToggleAsk => self.toggle_ask(),
            Act::UpdCheck => self.upd_check(),
            Act::UpdInstall => self.upd_install(),
            Act::UpdUndo => self.upd_undo(),
            Act::UpdRestart => self.upd_restart(),
            Act::UpdAllow => self.upd_toggle_allow(),
            Act::UpdAtStart => self.upd_toggle_start(),
            Act::HeadMode(m) => {
                self.conn.head_node = m;
                self.head.status = None;
                self.saved = false;
            }
            Act::HeadCamera => {
                self.conn.head_camera = !self.conn.head_camera;
                self.saved = false;
            }
            Act::HeadMeasure => self.head_measure(),
            Act::HeadLight(_)
            | Act::InspOpen
            | Act::InspToggleHoles
            | Act::InspToggleMilling
            | Act::InspPreset(_)
            | Act::InspLight(_)
            | Act::InspHole(_)
            | Act::InspRun
            | Act::InspCalibrate
            | Act::InspScale(_)
            | Act::InspSave
            | Act::InspTune(..) => self.insp_act(a),
            Act::CatSelect(i) => {
                self.cat_sel = Some(i);
                self.cat_detail_scroll = 0.0;
            }
            Act::CatSpindle(d) => self.cat_spindle_by(d),
            Act::SetSpindle(s) => self.set_spindle(s),
            Act::ToolFromCat(p, row) => {
                self.show_modal(Modal::Catalogue);
                self.cat_target = Some((p, row));
                self.cat_sel = None;
                self.cat_detail_scroll = 0.0;
            }
            Act::CatUse => self.cat_use(),
            Act::JobPart(p) => self.toggle_job_part(p),
            Act::ToolOff(p, t) => self.toggle_tool(p, t),
            Act::LevelToggle => self.level_toggle(),
            Act::LevelApply => self.level.apply = !self.level.apply,
            Act::LevelClear => self.level_clear(),
            Act::ResumeContinue => self.resume_job(),
            Act::ResumeCancel => self.resume_cancel(),
            Act::ToggleTabs => {
                self.panel.tabs_on = !self.panel.tabs_on;
                self.saved = false;
                self.apply_panel();
            }
            Act::ComboOpen(id) => self.combo_open(id),
            Act::ComboPick(i) => self.combo_pick(i),
            Act::ComboClose => self.combo = None,
            Act::ToggleToolsOnOpen => self.toggle_tools_on_open(),
            Act::ToggleSkip => self.toggle_skip(),
            Act::Adjust(f, d) => self.adjust(f, d),
            Act::PanelMode(m) => {
                self.panel.mode = m;
                self.apply_panel();
            }
            Act::SaveSettings => self.save_settings(),
            Act::ConnKind(k) => {
                self.conn.kind = k;
                self.refresh_ports();
                self.conn_result = None;
                self.saved = false;
            }
            Act::ConnTest => self.conn_test(),
            Act::FleetUse(i) => self.fleet_use(i),
            Act::FleetRefresh => self.fleet_ask(),
            Act::ConnSave => self.conn_save(),
            Act::Browse(i) => self.browse(i),
            Act::BrowseSort(c) => self.browse_sort(c),
            Act::BrowseFilter => self.browse_filter(),
            Act::ToggleSelect => self.toggle_select_mode(),
            Act::SlotsAuto => self.slots_auto(),
            Act::SlotsClear => self.slots_clear(),
            Act::RackGo => self.rack_go(),
            Act::RackCancel => self.rack_cancel(),
            Act::StartAt(p, t) => self.toggle_start_at(p, t),
            Act::CutSel => self.cut_selection(),
            Act::UndoEdit => self.undo_edit(),
            Act::RedoEdit => self.redo_edit(),
            Act::ConnPort(i) => self.pick_port(i),
            Act::PortsRefresh => self.refresh_ports(),
            Act::CopySel => self.copy_selection(),
            Act::PasteSel => self.paste_clip(),
            Act::DeleteSel => self.delete_selection(),
            Act::RemoveTool(p, t) => self.remove_tool(p, t),
            Act::TogglePreview => self.toggle_preview(),
            Act::BrowsePlace(i) => self.browse_place(i),
            Act::OpenRecent(i) => {
                if let Some(p) = self.settings.recent.get(i).map(PathBuf::from) {
                    self.modal = None;
                    self.browser = None;
                    self.open_path_asking(&p);
                }
            }
            Act::BrowseUp => self.browse_up(),
            Act::MatScroll(d) => self.mat_scroll_by(d),
            Act::BrowseBack => self.browse_back(),
            Act::BrowseOpen => self.browse_open(),
            Act::ResetView(z) => self.reset_view(z),
            Act::Start if self.stage_wait && !self.estop => self.continue_job(),
            Act::Start => {
                if self.paused {
                    self.paused = false;
                } else if !self.estop {
                    self.start_job();
                }
            }
            Act::Pause => self.paused = self.running.is_some() && !self.paused,
            Act::Stop => self.stop_job(),
            Act::Back => {
                self.stop_job();
                self.screen = Screen::Launcher;
            }
            Act::Summary(start) => {
                if let Some((p, skip, _)) = self.summary_ask.take() {
                    if start {
                        self.side_mut(p).engine.accept_summary();
                        self.begin_program_with(p, skip, true);
                    } else {
                        self.side_mut(p).engine.decline_summary();
                        self.job_queue.clear();
                    }
                }
            }
            Act::Answer(ok) => {
                if let Some(p) = self.running {
                    self.side_mut(p).engine.answer(ok);
                    if !ok {
                        self.running = None;
                        self.m3d.finish();
                    }
                }
            }
        }
    }
}

/// The lists that have a scroll bar: the bar is drawn next to the list and moves with a click or a drag on it, and the wheel over the list moves it too.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Bar {
    Combo,
    Tools,
    Catalogue,
    Browse,
}

/// One bar of this frame: where its track is, the area of the list that the wheel works on, and how many rows there are and are shown.
#[derive(Debug, Clone, Copy)]
struct BarInfo {
    bar: Bar,
    track: Rect,
    /// The little arrows at the two ends of the bar (a click moves the list one row); none when the bar is too short for them.
    up: Option<Rect>,
    down: Option<Rect>,
    area: Rect,
    total: usize,
    shown: usize,
}

// ------------------------------------------------------------------------------------------------ drawing

/// A canvas of the design size scaled into the window: every coordinate is in design units.
struct Cv<'a> {
    p: &'a egui::Painter,
    o: Pos2,
    u: f32,
    t: &'a Theme,
    hits: Vec<(Rect, Act)>,
    zones: Vec<(Rect, Zone)>,
    /// The boxes of the drop-down lists drawn in this frame, with the place under each one: the list that is open hangs from its box.
    combos: Vec<(ComboId, f32, f32, f32)>,
    bars: Vec<BarInfo>,
    modal_from: usize,
    /// The first card drawn on this canvas (the window, when this is the canvas of a window), in design units.
    first_card: Option<(f32, f32, f32, f32)>,
    /// Where the pointer is and whether its button is held down: the buttons answer to both.
    pointer: Option<Pos2>,
    down: bool,
    /// Seconds on the clock of the animations (0 when they are off), and whether a job is running.
    time: f32,
    running: bool,
}

#[cfg(test)]
thread_local! {
    /// The texts that did not fit their room even when made smaller, in the order they were drawn (the test of the widths reads them).
    static CLIPPED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Where the note "design locked" is drawn in the plan view: its right end, at the top right (the corner of the board and the origin are at the bottom left, where the
/// design is, and the warning of a board that does not fit is at the top left).
pub(crate) fn lock_label_anchor(vx: f32, vy: f32, vw: f32) -> (f32, f32) {
    (vx + vw - 14.0, vy + 28.0)
}

/// A slow ease: fast at first, calm at the end (0 to 1).
fn ease_out(k: f32) -> f32 {
    1.0 - (1.0 - k.clamp(0.0, 1.0)).powi(3)
}

/// A smooth wave between 0 and 1 with this period in seconds.
fn wave(time: f32, period: f32) -> f32 {
    0.5 - 0.5 * (std::f32::consts::TAU * time / period).cos()
}

impl<'a> Cv<'a> {
    /// A vertical scroll bar next to a list: the track and the thumb, drawn where the list says; `area` is the list itself, for the wheel. Nothing is drawn when the list fits.
    #[allow(clippy::too_many_arguments)]
    fn scrollbar(
        &mut self,
        bar: Bar,
        x: f32,
        y: f32,
        h: f32,
        area: (f32, f32, f32, f32),
        total: usize,
        shown: usize,
        first: usize,
    ) {
        if total <= shown || shown == 0 || h < 30.0 {
            return;
        }
        let w = 12.0;
        // the arrows at the ends, as the bars of a list have them; a short bar has only its thumb
        let arrow = if h >= 70.0 { 14.0 } else { 0.0 };
        let (ty, th) = (y + arrow, h - 2.0 * arrow);
        let track = self.rect(x, ty, w, th);
        self.sunk(
            track,
            4.0 * self.u,
            col(self.t.panel_raised),
            self.touch(track.expand(3.0 * self.u)).0,
        );
        let thumb_h = (th * shown as f32 / total as f32).clamp(28.0, th);
        let top = first.min(total - shown) as f32 / (total - shown) as f32 * (th - thumb_h);
        let thumb = self.rect(x, ty + top, w, thumb_h);
        let (thumb_hover, thumb_down) = self.touch(thumb);
        self.raised(
            thumb,
            4.0 * self.u,
            col(self.t.accent),
            thumb_hover,
            thumb_down,
        );
        let (mut up, mut down) = (None, None);
        if arrow > 0.0 {
            let ur = self.rect(x, y, w, arrow - 1.0);
            let dr = self.rect(x, y + h - arrow + 1.0, w, arrow - 1.0);
            for (r, pointing_up) in [(ur, true), (dr, false)] {
                let (hover, down) = self.touch(r);
                self.raised(r, 3.0 * self.u, col(self.t.panel_raised), hover, down);
                let c = r.center() + Vec2::new(0.0, if down { 1.5 * self.u } else { 0.0 });
                let (hw, hh) = (3.5 * self.u, 2.5 * self.u);
                let (tip, base) = if pointing_up {
                    (c.y - hh, c.y + hh)
                } else {
                    (c.y + hh, c.y - hh)
                };
                self.p.add(Shape::convex_polygon(
                    vec![
                        Pos2::new(c.x, tip),
                        Pos2::new(c.x - hw, base),
                        Pos2::new(c.x + hw, base),
                    ],
                    if hover {
                        Color32::WHITE
                    } else {
                        col(self.t.accent)
                    },
                    Stroke::NONE,
                ));
            }
            up = Some(ur);
            down = Some(dr);
        }
        let (ax, ay, aw, ah) = area;
        self.bars.push(BarInfo {
            bar,
            track,
            up,
            down,
            area: self.rect(ax, ay, aw, ah),
            total,
            shown,
        });
    }

    fn pt(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.o.x + x * self.u, self.o.y + y * self.u)
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.pt(x, y), Vec2::new(w * self.u, h * self.u))
    }

    fn text(&self, x: f32, y: f32, al: Align2, s: &str, size: f32, c: Color32) -> Rect {
        self.p
            .text(self.pt(x, y), al, s, FontId::proportional(size * self.u), c)
    }

    /// The text of a button or a switch in the room it has: a longer one (another language) is drawn smaller, to a limit, and then cut with an ellipsis.
    #[allow(clippy::too_many_arguments)]
    fn text_fit(
        &self,
        x: f32,
        y: f32,
        al: Align2,
        s: &str,
        size: f32,
        c: Color32,
        max_w: f32,
    ) -> Rect {
        let width = |text: &str, sz: f32| {
            self.p
                .layout_no_wrap(text.to_string(), FontId::proportional(sz * self.u), c)
                .size()
                .x
                / self.u
        };
        let mut size = size;
        let w0 = width(s, size);
        if w0 > max_w + 0.05 {
            size = (size * max_w / w0).max(size * 0.72);
        }
        let mut shown = s.to_string();
        if width(&shown, size) > max_w + 0.05 {
            #[cfg(test)]
            CLIPPED.with(|c| {
                c.borrow_mut().push(format!(
                    "{s} ({max_w:.0}, needs {:.0} at size {size:.1})",
                    width(&shown, size)
                ))
            });
            while shown.chars().count() > 1 && width(&format!("{shown}…"), size) > max_w {
                shown.pop();
            }
            shown.push('…');
        }
        self.text(x, y, al, &shown, size, c)
    }

    fn line(&self, a: (f32, f32), b: (f32, f32), w: f32, c: Color32) {
        self.p.line_segment(
            [self.pt(a.0, a.1), self.pt(b.0, b.1)],
            Stroke::new(w * self.u, c),
        );
    }

    fn poly(&self, pts: &[(f32, f32)], fill: Color32, stroke: Stroke) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        self.p.add(Shape::convex_polygon(v, fill, stroke));
    }

    fn polyline(&self, pts: &[(f32, f32)], w: f32, c: Color32, dashed: bool) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        if v.len() < 2 {
            return;
        }
        if dashed {
            self.p.extend(Shape::dashed_line(
                &v,
                Stroke::new(w * self.u, c),
                7.0 * self.u,
                5.0 * self.u,
            ));
        } else {
            self.p.add(Shape::line(v, Stroke::new(w * self.u, c)));
        }
    }

    /// A card in the corner style of the theme.
    fn card(&mut self, x: f32, y: f32, w: f32, h: f32, hot: bool, act: Option<Act>) {
        if self.first_card.is_none() {
            self.first_card = Some((x, y, w, h));
        }
        let t = self.t;
        let rect = self.rect(x, y, w, h);
        // a card that answers (the launcher choices) lights up and lifts under the pointer
        let hover = act.is_some() && self.touch(rect).0;
        let fill = col(t.panel);
        let (top_fill, bottom_fill) = if hover {
            (shade(fill, 1.28), shade(fill, 1.0))
        } else {
            (shade(fill, 1.12), shade(fill, 0.9))
        };
        let line = if hover {
            col(t.accent)
        } else if hot {
            col(t.line)
        } else {
            alpha(Color32::WHITE, 34)
        };
        let stroke = Stroke::new(if hot || hover { 1.6 } else { 1.0 } * self.u, line);
        match t.corner {
            Corner::Cut => {
                let c = 16.0;
                let shape = [
                    (x + c, y),
                    (x + w, y),
                    (x + w, y + h - c),
                    (x + w - c, y + h),
                    (x, y + h),
                    (x, y + c),
                ];
                let lift = if hover { 7.0 } else { 4.0 };
                for (g, a) in [(lift, 40u8), (lift * 0.5, 55)] {
                    let s: Vec<(f32, f32)> = shape.iter().map(|&(px, py)| (px, py + g)).collect();
                    self.poly(&s, alpha(Color32::BLACK, a), Stroke::NONE);
                }
                let pts: Vec<Pos2> = shape.iter().map(|&(px, py)| self.pt(px, py)).collect();
                self.gradient_poly(&pts, top_fill, bottom_fill);
                self.poly(&shape, Color32::TRANSPARENT, stroke);
                self.line(
                    (x + c + 4.0, y + 2.5),
                    (x + w - 6.0, y + 2.5),
                    1.4,
                    alpha(Color32::WHITE, 30),
                );
            }
            Corner::Round | Corner::Square => {
                let r = if t.corner == Corner::Round { 14.0 } else { 0.0 } * self.u;
                let lift = if hover { 7.0 } else { 4.0 } * self.u;
                for (g, a) in [(lift, 38u8), (lift * 0.5, 52)] {
                    self.p.rect_filled(
                        rect.translate(Vec2::new(0.0, g)),
                        r,
                        alpha(Color32::BLACK, a),
                    );
                }
                if hover {
                    self.p.rect_filled(
                        rect.expand(4.0 * self.u),
                        r + 4.0 * self.u,
                        alpha(col(t.accent), 34),
                    );
                }
                self.gradient_rrect(rect, r, top_fill, bottom_fill);
                self.p.rect_stroke(rect, r, stroke, StrokeKind::Inside);
                let inset = (r * 0.6).max(8.0 * self.u);
                self.p.line_segment(
                    [
                        Pos2::new(rect.min.x + inset, rect.min.y + 2.5 * self.u),
                        Pos2::new(rect.max.x - inset, rect.min.y + 2.5 * self.u),
                    ],
                    Stroke::new(1.4 * self.u, alpha(Color32::WHITE, 30)),
                );
            }
        }
        if let Some(a) = act {
            self.hits.push((self.rect(x, y, w, h), a));
        }
    }

    fn title(&self, x: f32, y: f32, s: &str) {
        self.text(
            x,
            y,
            Align2::LEFT_BOTTOM,
            &s.to_uppercase(),
            14.0,
            col(self.t.accent),
        );
    }

    /// Is the pointer over this rectangle, and is the button held down on it?
    pub(crate) fn touch(&self, rect: Rect) -> (bool, bool) {
        let hover = self.pointer.is_some_and(|p| rect.contains(p));
        (hover, hover && self.down)
    }

    /// A button: filled with a colour, a label in the middle. It lights up under the pointer, sinks when it is pressed and has a shadow,
    /// a shine on its top edge and an edge that follows the corner style of the theme.
    #[allow(clippy::too_many_arguments)]
    fn button(&mut self, x: f32, y: f32, w: f32, h: f32, label: &str, c: Color32, act: Act) {
        let rect = self.rect(x, y, w, h);
        let (hover, down) = self.touch(rect);
        let ink = if self.t.corner == Corner::Square {
            Color32::BLACK
        } else {
            col(self.t.background.0)
        };
        // pressed: darker and lower; under the pointer: brighter with a glow
        let fill = if down {
            shade(c, 0.78)
        } else if hover {
            shade(c, 1.14)
        } else {
            c
        };
        let sink = if down { 3.0 } else { 0.0 };
        let (y, corner) = (y + sink, 12.0);
        let shape: Vec<(f32, f32)> = if self.t.corner == Corner::Cut {
            vec![
                (x + corner, y),
                (x + w, y),
                (x + w, y + h - corner),
                (x + w - corner, y + h),
                (x, y + h),
                (x, y + corner),
            ]
        } else {
            vec![(x, y), (x + w, y), (x + w, y + h), (x, y + h)]
        };
        let r = match self.t.corner {
            Corner::Round => 12.0 * self.u,
            _ => 0.0,
        };
        if !down {
            // the shadow under the button, wider when it is lit
            let k = if hover { 7.0 } else { 4.0 };
            for (i, a) in [(2.0, 26u8), (1.0, 40)] {
                let grow = k * i / 2.0;
                if self.t.corner == Corner::Cut {
                    let s: Vec<(f32, f32)> =
                        shape.iter().map(|&(px, py)| (px, py + grow)).collect();
                    self.poly(&s, alpha(Color32::BLACK, a), Stroke::NONE);
                } else {
                    self.p.rect_filled(
                        self.rect(x, y + grow, w, h).expand(grow * 0.4 * self.u),
                        r,
                        alpha(Color32::BLACK, a),
                    );
                }
            }
        }
        if hover && !down {
            // the glow around the button under the pointer
            for (g, a) in [(8.0, 18u8), (4.0, 34)] {
                if self.t.corner == Corner::Cut {
                    let s = [
                        (x + corner - g, y - g),
                        (x + w + g, y - g),
                        (x + w + g, y + h - corner + g),
                        (x + w - corner + g, y + h + g),
                        (x - g, y + h + g),
                        (x - g, y + corner - g),
                    ];
                    self.poly(&s, alpha(c, a), Stroke::NONE);
                } else {
                    self.p
                        .rect_filled(rect.expand(g * self.u), r + g * self.u, alpha(c, a));
                }
            }
        }
        if self.t.corner == Corner::Cut {
            let pts: Vec<Pos2> = shape.iter().map(|&(px, py)| self.pt(px, py)).collect();
            self.gradient_poly(&pts, shade(fill, 1.2), shade(fill, 0.8));
            self.poly(
                &shape,
                Color32::TRANSPARENT,
                Stroke::new(
                    1.2 * self.u,
                    alpha(shade(c, 1.35), if hover { 255 } else { 140 }),
                ),
            );
        } else {
            let body = self.rect(x, y, w, h);
            self.gradient_rrect(body, r, shade(fill, 1.2), shade(fill, 0.8));
            self.p.rect_stroke(
                body,
                r,
                Stroke::new(
                    1.2 * self.u,
                    alpha(shade(c, 1.35), if hover { 255 } else { 140 }),
                ),
                StrokeKind::Inside,
            );
        }
        // the shine along the top edge, and the shade along the bottom one
        let (sx0, sx1) = (
            x + if self.t.corner == Corner::Cut {
                corner
            } else {
                6.0
            },
            x + w - 6.0,
        );
        self.line(
            (sx0, y + 3.0),
            (sx1, y + 3.0),
            1.6,
            alpha(Color32::WHITE, if down { 12 } else { 70 }),
        );
        self.line(
            (x + 6.0, y + h - 3.0),
            (x + w - corner.min(10.0), y + h - 3.0),
            1.4,
            alpha(Color32::BLACK, if down { 20 } else { 55 }),
        );
        self.text_fit(
            x + w / 2.0,
            y + h / 2.0 + 6.0,
            Align2::CENTER_BOTTOM,
            &label.to_uppercase(),
            if hover { 17.5 } else { 17.0 },
            ink,
            w - 16.0,
        );
        self.hits.push((rect, act));
    }
}

fn draw_boxes(
    cv: &Cv,
    iso: &Iso,
    o: (f32, f32),
    shift: (f64, f64),
    faces: &[(f64, f64, f64, f64, f64, f64, Color32)],
    stroke: Color32,
) {
    for &(x, y, z, w, d, h, top) in faces {
        let p = |a: f64, b: f64, c: f64| {
            let (px, py) = iso.project(a + shift.0, b + shift.1, c);
            (o.0 + px as f32, o.1 + py as f32)
        };
        let sk = Stroke::new(1.0 * cv.u, stroke);
        cv.poly(
            &[
                p(x, y + d, z),
                p(x + w, y + d, z),
                p(x + w, y + d, z + h),
                p(x, y + d, z + h),
            ],
            shade(top, 0.55),
            sk,
        );
        cv.poly(
            &[
                p(x + w, y, z),
                p(x + w, y + d, z),
                p(x + w, y + d, z + h),
                p(x + w, y, z + h),
            ],
            shade(top, 0.38),
            sk,
        );
        cv.poly(
            &[
                p(x, y, z + h),
                p(x + w, y, z + h),
                p(x + w, y + d, z + h),
                p(x, y + d, z + h),
            ],
            top,
            sk,
        );
    }
}

impl App {
    fn paint(&mut self, p: &egui::Painter, rect: Rect) {
        // a wide window gets a wider board (more work area to both sides); a small or squarer one keeps the board of 1600 and scales it down
        crate::panes::set_board_width(crate::panes::board_width_for(rect.width(), rect.height()));
        let u = (rect.width() / design_w()).min(rect.height() / DESIGN_H);
        // the design area is centred; the background covers the whole window
        let o = Pos2::new(
            rect.min.x + (rect.width() - design_w() * u) / 2.0,
            rect.min.y + (rect.height() - DESIGN_H * u) / 2.0,
        );
        let t = self.theme.clone();
        p.rect_filled(rect, 0.0, col(t.background.0));
        let strip = Rect::from_min_max(
            Pos2::new(rect.min.x, o.y),
            Pos2::new(rect.max.x, o.y + DESIGN_H * u),
        );
        p.rect_filled(strip, 0.0, col(t.background.1));
        let now = if self.animate {
            self.anim0.elapsed().as_secs_f32()
        } else {
            1.0e6
        };
        if self.anim_screen.0 != Some(self.screen) {
            self.anim_screen = (Some(self.screen), now);
        }
        if self.anim_modal.0 != self.modal {
            self.anim_modal = (self.modal, now);
        }
        // the screen fades in as it opens and the window rises a little while it fades in
        let (screen_k, modal_k) = if self.animate {
            (
                ease_out((now - self.anim_screen.1) / 0.35),
                ease_out((now - self.anim_modal.1) / 0.20),
            )
        } else {
            (1.0, 1.0)
        };
        let mut screen_painter = p.clone();
        screen_painter.set_opacity(screen_k);
        let time = if self.animate { now } else { 0.0 };
        let mut cv = Cv {
            p: &screen_painter,
            o,
            u,
            t: &t,
            hits: Vec::new(),
            zones: Vec::new(),
            combos: Vec::new(),
            bars: Vec::new(),
            modal_from: 0,
            first_card: None,
            pointer: self.pointer,
            down: self.pointer_down,
            time,
            running: self.running.is_some(),
        };
        match self.screen {
            Screen::Launcher => self.paint_launcher(&mut cv),
            Screen::Viewer => self.paint_viewer(&mut cv),
        }
        if let Some(m) = self.modal {
            let mut modal_painter = p.clone();
            modal_painter.set_opacity(modal_k);
            let mut mcv = Cv {
                p: &modal_painter,
                o: Pos2::new(
                    o.x + self.modal_off.0 * u,
                    o.y + (1.0 - modal_k) * 14.0 * u + self.modal_off.1 * u,
                ),
                u,
                t: &t,
                hits: std::mem::take(&mut cv.hits),
                zones: std::mem::take(&mut cv.zones),
                combos: std::mem::take(&mut cv.combos),
                bars: std::mem::take(&mut cv.bars),
                modal_from: 0,
                first_card: None,
                pointer: self.pointer,
                down: self.pointer_down,
                time,
                running: cv.running,
            };
            mcv.modal_from = mcv.hits.len();
            self.paint_modal(&mut mcv, m);
            // every window can be moved by its top and closed with the X at the corner
            if let Some((x, y, w, _)) = mcv.first_card {
                self.modal_card = mcv.first_card;
                self.modal_drag = Some(mcv.rect(x, y, (w - 64.0).max(0.0), 56.0));
                let close = if m == Modal::Resume {
                    Act::ResumeCancel
                } else {
                    Act::CloseModal
                };
                mcv.close_button(x + w - 52.0, y + 9.0, 40.0, 38.0, close);
            } else {
                self.modal_card = None;
                self.modal_drag = None;
            }
            cv.modal_from = mcv.modal_from;
            cv.hits = mcv.hits;
            cv.zones = mcv.zones;
            cv.combos = mcv.combos;
            cv.bars = mcv.bars;
        }
        // an open drop-down list is over everything, and the rest of the window waits while it is open
        self.paint_combo(&mut cv);
        if let Some(m) = &self.message {
            // in the viewer the notice goes in the zone the card under the views keeps for it; on the other screens, in a strip at the foot
            let notes = [(m.clone(), col(t.danger))];
            if self.screen == Screen::Viewer {
                let zone = hudinfo::hud_layout()
                    .warn
                    .translate(Vec2::new(0.0, DESIGN_H - HUD_H - 20.0));
                cv.notices(zone, &notes, false);
            } else {
                let zone = Rect::from_min_size(
                    Pos2::new(40.0, DESIGN_H - 60.0),
                    Vec2::new(design_w() - 80.0, 36.0),
                );
                cv.notices(zone, &notes, true);
            }
        }
        self.hits = std::mem::take(&mut cv.hits);
        self.zones = std::mem::take(&mut cv.zones);
        self.bars = std::mem::take(&mut cv.bars);
        self.modal_from = if self.modal.is_some() || self.combo.is_some() {
            cv.modal_from
        } else {
            0
        };
        self.ui_scale = u;
        // the boxes that are typed in over a window go where the window is
        self.ui_origin = if self.modal.is_some() {
            Pos2::new(o.x + self.modal_off.0 * u, o.y + self.modal_off.1 * u)
        } else {
            self.modal_card = None;
            self.modal_drag = None;
            o
        };
    }

    fn top_bar(&self, cv: &mut Cv, back: bool) {
        let t = cv.t.clone();
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::new(0.0, cv.o.y), Vec2::new(10_000.0, 64.0 * cv.u)),
            0.0,
            alpha(col(t.panel), 235),
        );
        cv.line(
            (0.0, 64.0),
            (design_w(), 64.0),
            1.0,
            alpha(col(t.line), 150),
        );
        let x = if back { 70.0 } else { 28.0 };
        if back {
            cv.icon_chip(
                14.0,
                11.0,
                46.0,
                42.0,
                Icon::Back,
                false,
                self.tr(Text::LauncherAsk),
                Act::Back,
            );
        }
        cv.text(x, 42.0, Align2::LEFT_BOTTOM, "EH3D", 26.0, col(t.text));
        cv.text(
            x + 78.0,
            42.0,
            Align2::LEFT_BOTTOM,
            "BUNGARD",
            26.0,
            col(t.accent),
        );
        // theme and language: a click takes the next one
        let name = t.id.text();
        let theme_label = format!("{}: {}", self.tr(Text::ThemeLabel), self.tr(name));
        let r = cv.text(
            design_w() - 28.0,
            28.0,
            Align2::RIGHT_BOTTOM,
            &theme_label,
            14.0,
            col(t.text),
        );
        cv.hits.push((r.expand(6.0), Act::NextTheme));
        let lang_label = format!(
            "{}: {}",
            self.tr(Text::Language),
            self.lang.code().to_uppercase()
        );
        let r = cv.text(
            design_w() - 28.0,
            50.0,
            Align2::RIGHT_BOTTOM,
            &lang_label,
            14.0,
            col(t.dim),
        );
        cv.hits.push((r.expand(6.0), Act::NextLang));
        // the windows, in groups with a line between them: the file; the job (board, tools, copies); the machine (settings, connection, nodes); the program
        let groups: [&[(Icon, Text, Modal)]; 4] = [
            &[(Icon::Folder, Text::OpenFile, Modal::Open)],
            &[
                (Icon::BoardWin, Text::BoardTitle, Modal::Board),
                (Icon::DrillWin, Text::ToolsTitle, Modal::Tools),
                (Icon::GridWin, Text::PanelTitle, Modal::Panel),
            ],
            &[
                (Icon::Sliders, Text::SettingsTitle, Modal::Settings),
                (Icon::Plug, Text::Connection, Modal::Connection),
                (Icon::Nodes, Text::FleetTitle, Modal::Fleet),
            ],
            &[(Icon::Info, Text::About, Modal::About)],
        ];
        let (bw, bh, gap, sep) = (TOP_BUTTON_W, 40.0, 4.0, 14.0);
        let mut x = TOP_BAR_FROM;
        for (gi, group) in groups.iter().enumerate() {
            // the launcher has the program's button only (at the place it has in the full bar)
            let shown = back || gi == 3;
            if gi > 0 && back {
                cv.line(
                    (x + sep / 2.0 - gap / 2.0, 16.0),
                    (x + sep / 2.0 - gap / 2.0, 48.0),
                    1.0,
                    alpha(col(t.line), 170),
                );
            }
            for &(icon, text, modal) in group.iter() {
                let w = match modal {
                    Modal::Fleet => TOP_BUTTON_WIDE,
                    _ => bw,
                };
                if shown {
                    cv.top_button(
                        x,
                        12.0,
                        w,
                        bh,
                        icon,
                        self.tr(text),
                        self.modal == Some(modal),
                        Act::Show(modal),
                    );
                }
                x += w + gap;
            }
            x += sep - gap;
        }
    }

    fn paint_launcher(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        self.top_bar(cv, false);
        cv.text(
            design_w() / 2.0,
            160.0,
            Align2::CENTER_BOTTOM,
            "EH3D-BUNGARD",
            44.0,
            col(t.text),
        );
        cv.text(
            design_w() / 2.0,
            200.0,
            Align2::CENTER_BOTTOM,
            &self.tr(Text::LauncherAsk).to_uppercase(),
            16.0,
            col(t.dim),
        );
        // the modern interface: the big card
        let (mx, my, mw, mh) = (150.0, 250.0, 560.0, 430.0);
        cv.card(mx, my, mw, mh, true, Some(Act::Choose(Choice::Modern)));
        cv.text(
            mx + mw / 2.0,
            my + 70.0,
            Align2::CENTER_BOTTOM,
            &self.tr(Text::LauncherModernTag).to_uppercase(),
            16.0,
            col(t.accent),
        );
        cv.text(
            mx + mw / 2.0,
            my + 130.0,
            Align2::CENTER_BOTTOM,
            "MTC + ATC",
            48.0,
            col(t.text),
        );
        // a small preview of the viewer
        cv.p.rect_filled(
            cv.rect(mx + 50.0, my + 170.0, mw - 100.0, 170.0),
            6.0 * cv.u,
            col(t.table),
        );
        for i in 0..24 {
            let (cx, cy) = (
                mx + 80.0 + (i % 12) as f32 * 36.0,
                my + 210.0 + (i / 12) as f32 * 40.0,
            );
            cv.p.circle_filled(
                cv.pt(cx, cy),
                (3.0 + (i % 4) as f32 * 1.4) * cv.u,
                col(t.size_color(i % 4)),
            );
        }
        cv.polyline(
            &[
                (mx + 60.0, my + 320.0),
                (mx + 260.0, my + 250.0),
                (mx + 500.0, my + 322.0),
            ],
            2.0,
            col(t.accent2),
            true,
        );
        cv.text(
            mx + mw / 2.0,
            my + mh - 40.0,
            Align2::CENTER_BOTTOM,
            &self.tr(Text::PressEnter).to_uppercase(),
            14.0,
            col(t.ok),
        );
        // the classic programs: four small cards
        let kinds = [
            Choice::Classic(Program::Drill, Machine::Mtc),
            Choice::Classic(Program::Drill, Machine::Atc),
            Choice::Classic(Program::Route, Machine::Mtc),
            Choice::Classic(Program::Route, Machine::Atc),
        ];
        for (i, c) in kinds.into_iter().enumerate() {
            let (x, y) = (
                760.0 + (i % 2) as f32 * 360.0,
                250.0 + (i / 2) as f32 * 220.0,
            );
            cv.card(x, y, 330.0, 200.0, false, Some(Act::Choose(c)));
            let Choice::Classic(prog, mach) = c else {
                continue;
            };
            let program = self.tr(if prog == Program::Drill {
                Text::Drilling
            } else {
                Text::Routing
            });
            let machine = if mach == Machine::Mtc { "MTC" } else { "ATC" };
            cv.text(
                x + 165.0,
                y + 56.0,
                Align2::CENTER_BOTTOM,
                &self
                    .tr(Text::LauncherClassicMtc)
                    .split(' ')
                    .next()
                    .unwrap_or("")
                    .to_uppercase(),
                14.0,
                col(t.dim),
            );
            cv.text(
                x + 165.0,
                y + 112.0,
                Align2::CENTER_BOTTOM,
                machine,
                40.0,
                col(t.text),
            );
            cv.text(
                x + 165.0,
                y + 160.0,
                Align2::CENTER_BOTTOM,
                program,
                18.0,
                col(t.dim),
            );
        }
        // remember
        let on = self.remember;
        let (bx, by) = (design_w() / 2.0 - 140.0, 730.0);
        let box_rect = cv.rect(bx, by, 22.0, 22.0);
        let (hover, down) = cv.touch(cv.rect(bx - 6.0, by - 6.0, 300.0, 34.0));
        cv.check_box_small(box_rect, on, hover, down);
        let r = cv.text(
            bx + 34.0,
            by + 17.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::Remember),
            16.0,
            col(t.dim),
        );
        cv.hits.push((
            Rect::from_min_max(box_rect.min, r.max).expand(6.0),
            Act::Remember,
        ));
    }

    /// The wheel over a list with a bar: `dir` rows up (-1) or down (1).
    fn bar_wheel(&mut self, bar: Bar, dir: i32) {
        match bar {
            Bar::Combo => self.combo_scroll(2 * dir),
            Bar::Tools => {
                let total = self.tool_lines().len();
                let shown = self
                    .bars
                    .iter()
                    .find(|b| b.bar == Bar::Tools)
                    .map_or(total, |b| b.shown);
                self.tool_scroll = (self.tool_scroll as i64 + dir as i64)
                    .clamp(0, total.saturating_sub(shown) as i64)
                    as usize;
            }
            Bar::Catalogue => self.cat_scroll_by(3 * dir),
            Bar::Browse => self.browse_scroll(2 * dir),
        }
    }

    /// A click on an arrow of a bar: the list moves one row up (-1) or down (1).
    fn bar_step(&mut self, bar: Bar, dir: i32) {
        match bar {
            Bar::Combo => self.combo_scroll(dir),
            Bar::Tools => self.bar_wheel(Bar::Tools, dir),
            Bar::Catalogue => self.cat_scroll_by(dir),
            Bar::Browse => self.browse_scroll(dir),
        }
    }

    /// The pointer is at `p` on the track of a bar: the list goes to the row that the place stands for.
    fn bar_to(&mut self, bar: Bar, p: Pos2) {
        let Some(info) = self.bars.iter().rev().find(|b| b.bar == bar).copied() else {
            return;
        };
        let max = info.total.saturating_sub(info.shown);
        let frac = ((p.y - info.track.min.y) / info.track.height().max(1.0)).clamp(0.0, 1.0);
        let first = (frac * max as f32).round() as usize;
        match bar {
            Bar::Combo => {
                if let Some(c) = self.combo {
                    self.combo_scroll(first as i32 - c.scroll as i32);
                }
            }
            Bar::Tools => self.tool_scroll = first.min(max),
            Bar::Catalogue => self.cat_scroll_by(first as i32 - self.cat_scroll as i32),
            Bar::Browse => {
                let now = self.browser.as_ref().map_or(0, |b| b.scroll);
                self.browse_scroll(first as i32 - now as i32);
            }
        }
    }

    fn paint_viewer(&self, cv: &mut Cv) {
        let t = cv.t.clone();
        self.top_bar(cv, true);
        let sizes = self.sizes();
        let (left_w, mid_w, right_w) = crate::panes::split(self.panes.0, self.panes.1);
        let (top, hud_y) = (84.0, DESIGN_H - HUD_H - 20.0);
        let card_h = hud_y - top - 18.0;

        // ---- left: the job
        cv.card(20.0, top, left_w, card_h, true, None);
        cv.title(38.0, top + 32.0, &self.tr(Text::HudJob).to_uppercase());
        let name = self
            .drill
            .engine
            .file_name()
            .or_else(|| self.route.engine.file_name())
            .unwrap_or_else(|| self.tr(Text::NoFile).to_string());
        cv.text(
            38.0,
            top + 70.0,
            Align2::LEFT_BOTTOM,
            &board::fit_text(&name, left_w - 50.0, 19.0),
            19.0,
            col(t.text),
        );
        // the list of tools takes the room that the layers and the archive leave: with more tools than fit it scrolls, so the layers never leave the card
        let zip_h = self.archive.as_ref().map_or(0.0, |a| {
            6.0 + 10.0
                + a.entries
                    .iter()
                    .filter(|e| e.kind != Kind::Other)
                    .count()
                    .min(6) as f32
                    * 22.0
        });
        let max_y = top + card_h - 134.0 - zip_h;
        let mut y = self.paint_job_tools(cv, top + 94.0, max_y);
        if let Some(a) = &self.archive {
            y += 6.0;
            cv.text(38.0, y, Align2::LEFT_BOTTOM, "ZIP", 12.0, col(t.dim));
            let usable: Vec<_> = a.entries.iter().filter(|e| e.kind != Kind::Other).collect();
            for (i, e) in usable.iter().take(6).enumerate() {
                let ry = y + 8.0 + i as f32 * 22.0;
                let tag = if e.kind == Kind::Drill { "D" } else { "R" };
                let file = e.name.rsplit('/').next().unwrap_or(&e.name);
                let shown: String = file.chars().take(24).collect();
                cv.text(
                    38.0,
                    ry + 15.0,
                    Align2::LEFT_BOTTOM,
                    &format!("{tag}  {shown}"),
                    13.0,
                    col(t.text),
                );
                cv.hits
                    .push((cv.rect(38.0, ry, left_w - 36.0, 20.0), Act::Entry(i)));
            }
            y += 10.0 + usable.len().min(6) as f32 * 22.0;
        }
        y += 14.0;
        cv.text(
            38.0,
            y,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LblLayers).to_uppercase(),
            12.0,
            col(t.dim),
        );
        let layer_names = [
            Text::LayerHoles,
            Text::LayerPaths,
            Text::LayerMachine,
            Text::LayerGrid,
        ];
        for (i, n) in layer_names.into_iter().enumerate() {
            let ly = y + 12.0 + i as f32 * 26.0;
            let r = cv.rect(38.0, ly, 16.0, 16.0);
            let (hover, down) = cv.touch(cv.rect(34.0, ly - 4.0, 200.0, 24.0));
            cv.check_box_small(r, self.layers[i], hover, down);
            cv.text(
                64.0,
                ly + 13.0,
                Align2::LEFT_BOTTOM,
                self.tr(n),
                14.0,
                col(t.text),
            );
            cv.hits
                .push((cv.rect(38.0, ly - 4.0, 200.0, 24.0), Act::Layer(i)));
        }

        // ---- centre: the plan from above
        let (cx, cw) = (
            20.0 + left_w + 16.0,
            if self.plan_big {
                mid_w + right_w + 16.0
            } else {
                mid_w
            },
        );
        cv.card(cx, top, cw, card_h, true, None);
        cv.title(cx + 18.0, top + 32.0, self.tr(Text::ViewPlan));
        let (vx, vy, vw, vh) = (cx + 14.0, top + 48.0, cw - 28.0, card_h - 48.0 - 108.0);
        let head_pos = self.head();
        let mut plan = Scene::new(self.table).view2d(vw as f64, vh as f64, 10.0);
        // the machines with the tool changer show the bar past the edge of the table: the plan is fitted to hold both
        if let Some(fit) = self.bar_fit((vw as f64, vh as f64)) {
            plan = fit;
        }
        {
            let (z, pan) = (self.view2d.zoom as f64, self.view2d.pan);
            let (mx, my) = (vw as f64 / 2.0, vh as f64 / 2.0);
            plan.scale *= z;
            plan.origin = (mx + (plan.origin.0 - mx) * z, my + (plan.origin.1 - my) * z);
            // centred on the tool that works, or on the point chosen
            let focus = if self.follow {
                head_pos
            } else {
                self.view2d.focus
            };
            if let Some((fx, fy)) = focus {
                let (px, py) = plan.to_screen(&self.table, fx, fy);
                plan.origin.0 += mx - px;
                plan.origin.1 += my - py;
            }
            plan.origin.0 += pan.0 as f64;
            plan.origin.1 += pan.1 as f64;
            self.plan_scale.set(plan.scale);
            self.plan_xf.set((
                plan.scale,
                plan.origin.0,
                plan.origin.1,
                vx as f64,
                vy as f64,
            ));
        }
        let screen = |x: f64, y: f64| {
            let (px, py) = plan.to_screen(&self.table, x, y);
            (vx + px as f32, vy + py as f32)
        };
        let head = self.head();
        {
            let clip = cv.p.with_clip_rect(cv.rect(vx, vy, vw, vh));
            let mut inner = cv.with_painter(&clip);
            let cv = &mut inner;
            let (tx0, ty0) = screen(0.0, self.table.depth_mm);
            let (tx1, ty1) = screen(self.table.width_mm, 0.0);
            cv.p.rect_filled(
                cv.rect(tx0, ty0, tx1 - tx0, ty1 - ty0),
                4.0 * cv.u,
                col(t.table),
            );
            if self.layers[3] {
                let mut gx = 0.0;
                while gx <= self.table.width_mm {
                    let (a, _) = screen(gx, 0.0);
                    cv.line(
                        (a, ty0),
                        (a, ty1),
                        if gx % 100.0 == 0.0 { 1.4 } else { 0.6 },
                        col(t.grid),
                    );
                    gx += 25.0;
                }
                let mut gy = 0.0;
                while gy <= self.table.depth_mm {
                    let (_, b) = screen(0.0, gy);
                    cv.line(
                        (tx0, b),
                        (tx1, b),
                        if gy % 100.0 == 0.0 { 1.4 } else { 0.6 },
                        col(t.grid),
                    );
                    gy += 25.0;
                }
            }
            cv.p.rect_stroke(
                cv.rect(tx0, ty0, tx1 - tx0, ty1 - ty0),
                4.0 * cv.u,
                Stroke::new(1.2 * cv.u, alpha(col(t.line), 140)),
                StrokeKind::Inside,
            );
            if let Some((x0, y0, x1, y1)) = self.panel_rect() {
                let (a, b) = screen(x0, y1);
                let (c, d) = screen(x1, y0);
                cv.p.rect_filled(
                    cv.rect(a, b, c - a, d - b),
                    4.0 * cv.u,
                    alpha(col(t.board), 200),
                );
            }
            if self.layers[1] {
                let tool_colors = self.route_tool_colors();
                let (mdx, mdy) = self.sel_move.unwrap_or((0.0, 0.0));
                for (pi, path) in self.route.scene.paths.iter().enumerate() {
                    let (ox, oy) = if self.sel_paths.contains(&pi) {
                        (mdx, mdy)
                    } else {
                        (0.0, 0.0)
                    };
                    let pts: Vec<(f32, f32)> = path
                        .points
                        .iter()
                        .map(|&(x, y)| screen(x + ox, y + oy))
                        .collect();
                    if self.sel_paths.contains(&pi) {
                        // what is chosen has a wide halo of the accent colour under it
                        cv.polyline(&pts, 7.0, alpha(col(t.accent), 110), false);
                    }
                    let line_color = tool_colors
                        .iter()
                        .find(|(tool, _)| *tool == path.tool)
                        .map_or(col(t.accent2), |&(_, i)| col(t.size_color(i)));
                    if cv.running {
                        // the dashes of the path to come move along it, slowly
                        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| cv.pt(x, y)).collect();
                        if v.len() >= 2 {
                            cv.p.extend(Shape::dashed_line_with_offset(
                                &v,
                                Stroke::new(2.0 * cv.u, line_color),
                                &[7.0 * cv.u],
                                &[5.0 * cv.u],
                                -cv.time * 14.0 * cv.u,
                            ));
                        }
                    } else {
                        cv.polyline(&pts, 2.0, line_color, true);
                    }
                    if path.reached > 1 {
                        cv.polyline(&pts[..path.reached.min(pts.len())], 2.4, col(t.ok), false);
                    }
                }
            }
            if self.layers[0] {
                let (mdx, mdy) = self.sel_move.unwrap_or((0.0, 0.0));
                for (hi, h) in self.drill.scene.holes.iter().enumerate() {
                    let idx = sizes
                        .iter()
                        .position(|d| (*d - h.diameter_mm).abs() < 0.005)
                        .unwrap_or(0);
                    let c = col(t.size_color(idx));
                    let r = plan.hole_radius(h.diameter_mm, 2.0) as f32;
                    let (px, py) = if self.sel_holes.contains(&hi) {
                        screen(h.x_mm + mdx, h.y_mm + mdy)
                    } else {
                        screen(h.x_mm, h.y_mm)
                    };
                    if self.sel_holes.contains(&hi) {
                        cv.p.circle_filled(
                            cv.pt(px, py),
                            (r + 5.0) * cv.u,
                            alpha(col(t.accent), 90),
                        );
                        cv.p.circle_stroke(
                            cv.pt(px, py),
                            (r + 5.0) * cv.u,
                            Stroke::new(1.6 * cv.u, col(t.accent)),
                        );
                    }
                    if h.done {
                        cv.p.circle_filled(cv.pt(px, py), r * cv.u, c);
                    } else {
                        cv.p.circle_stroke(
                            cv.pt(px, py),
                            r * cv.u,
                            Stroke::new(1.4 * cv.u, alpha(c, 220)),
                        );
                    }
                }
            }
            // the box being dragged to choose: whole (solid line) from left to right, touching (dashed) from right to left
            if let Some((a, b)) = self.band {
                let (ax, ay) = screen(a.0, a.1);
                let (bx, by) = screen(b.0, b.1);
                let r = cv.rect(ax.min(bx), ay.min(by), (ax - bx).abs(), (ay - by).abs());
                cv.p.rect_filled(r, 0.0, alpha(col(t.accent), 36));
                let line = Stroke::new(1.6 * cv.u, col(t.accent));
                if b.0 < a.0 {
                    cv.p.extend(Shape::dashed_line(
                        &[
                            r.left_top(),
                            r.right_top(),
                            r.right_bottom(),
                            r.left_bottom(),
                            r.left_top(),
                        ],
                        line,
                        6.0 * cv.u,
                        4.0 * cv.u,
                    ));
                } else {
                    cv.p.rect_stroke(r, 0.0, line, StrokeKind::Inside);
                }
            }
            // the bar of the tool changer with its places and the tools in them
            self.paint_bar_plan(cv, &screen, plan.scale);
            if let Some((hx, hy)) = head {
                let (px, py) = screen(hx, hy);
                let breath = if cv.running { wave(cv.time, 1.6) } else { 0.0 };
                cv.p.circle_stroke(
                    cv.pt(px, py),
                    (13.0 + 1.5 * breath) * cv.u,
                    Stroke::new(2.0 * cv.u, col(t.ok)),
                );
                if cv.running {
                    // a ripple that spreads and fades every second and a half
                    let ripple = (cv.time / 1.5).fract();
                    cv.p.circle_stroke(
                        cv.pt(px, py),
                        (13.0 + 26.0 * ease_out(ripple)) * cv.u,
                        Stroke::new(1.4 * cv.u, alpha(col(t.ok), (150.0 * (1.0 - ripple)) as u8)),
                    );
                }
                cv.line((px - 22.0, py), (px + 22.0, py), 1.5, col(t.ok));
                cv.line((px, py - 22.0), (px, py + 22.0), 1.5, col(t.ok));
            }
        }
        cv.text(
            cx + 18.0,
            top + card_h - 8.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{} {:.0} x {:.0} mm",
                self.tr(Text::TableFromConfig),
                self.table.width_mm,
                self.table.depth_mm
            ),
            13.0,
            col(t.dim),
        );

        cv.zones.push((cv.rect(vx, vy, vw, vh), Zone::Plan));
        // the board does not fit the table of the active machine, even turned: the warning lies over the plan
        self.paint_fit_warning(cv, vx + 12.0, vy + 22.0);
        if self.design_locked() {
            // while the machine works the design cannot be dragged, turned or changed: the view only moves and zooms
            let label = self.tr(Text::DesignLocked);
            let (lx, ly) = lock_label_anchor(vx, vy, vw);
            let r = cv.text_fit(
                lx,
                ly,
                Align2::RIGHT_BOTTOM,
                label,
                14.0,
                col(t.warn),
                vw * 0.5,
            );
            cv.p.rect_stroke(
                r.expand(4.0 * cv.u),
                4.0 * cv.u,
                Stroke::new(1.2 * cv.u, col(t.warn)),
                StrokeKind::Outside,
            );
        }
        let ix = cx + cw - 52.0;
        cv.icon_chip(
            ix,
            top + 8.0,
            40.0,
            32.0,
            Icon::Target,
            false,
            self.tr(Text::ResetView),
            Act::ResetView(Zone::Plan),
        );
        cv.icon_chip(
            ix - 44.0,
            top + 8.0,
            40.0,
            32.0,
            Icon::Rotate,
            false,
            self.tr(Text::Rotate),
            Act::Rotate(1),
        );
        // choose, move, copy, cut, paste and delete what is drawn, and undo and redo it
        let tools = [
            (
                -464.0,
                Icon::Jog,
                self.jog_ui.open,
                Text::JogTitle,
                Act::Jog(JogAct::Toggle),
            ),
            (-96.0, Icon::Redo, false, Text::RedoEdit, Act::RedoEdit),
            (-140.0, Icon::Undo, false, Text::UndoEdit, Act::UndoEdit),
            (-192.0, Icon::Trash, false, Text::DeleteSel, Act::DeleteSel),
            (-236.0, Icon::Paste, false, Text::PasteSel, Act::PasteSel),
            (-280.0, Icon::Cut, false, Text::CutSel, Act::CutSel),
            (-324.0, Icon::Copy, false, Text::CopySel, Act::CopySel),
            (
                -368.0,
                Icon::Select,
                self.select_mode,
                Text::SelectTool,
                Act::ToggleSelect,
            ),
            (
                -420.0,
                Icon::TurnBoard,
                false,
                Text::BoardSwap,
                Act::BoardSwap,
            ),
        ];
        for (dx, icon, on, tip, act) in tools {
            let note = if act == Act::Jog(JogAct::Toggle) {
                self.jog_label()
            } else {
                self.tr(tip).to_string()
            };
            cv.icon_chip(ix + dx, top + 8.0, 40.0, 32.0, icon, on, &note, act);
        }
        cv.text(
            cx + cw - 18.0,
            top + card_h - 8.0,
            Align2::RIGHT_BOTTOM,
            self.tr(Text::NavHint2d),
            13.0,
            col(t.dim),
        );
        if self.drill.scene.holes.is_empty() && self.route.scene.paths.is_empty() {
            cv.text(
                cx + cw / 2.0,
                top + card_h / 2.0,
                Align2::CENTER_CENTER,
                self.tr(Text::DropHint),
                22.0,
                col(t.dim),
            );
            cv.chip(
                cx + cw / 2.0 - 100.0,
                top + card_h / 2.0 + 30.0,
                200.0,
                44.0,
                self.tr(Text::OpenFile),
                true,
                Act::Show(Modal::Open),
            );
        }

        self.paint_plan_tools(cv, cx + 14.0, top + card_h - 104.0, cw - 28.0);

        // ---- right: the machine
        if !self.plan_big {
            let rx = design_w() - 20.0 - right_w;
            cv.card(rx, top, right_w, card_h, false, None);
            cv.title(rx + 18.0, top + 32.0, self.tr(Text::ViewMachine));
            if self.layers[2] {
                let clip = cv.p.with_clip_rect(cv.rect(
                    rx + 4.0,
                    top + 44.0,
                    right_w - 8.0,
                    card_h - 44.0,
                ));
                let mut inner = cv.with_painter(&clip);
                self.paint_machine(
                    &mut inner,
                    rx,
                    top + 44.0,
                    right_w,
                    card_h - 70.0,
                    &sizes,
                    self.head3d(),
                );
            }
            cv.zones.push((
                cv.rect(rx + 4.0, top + 44.0, right_w - 8.0, card_h - 70.0),
                Zone::Machine,
            ));
            cv.chip(
                rx + right_w - 204.0,
                top + 8.0,
                190.0,
                30.0,
                self.tr(Text::ResetView),
                false,
                Act::ResetView(Zone::Machine),
            );
            if right_w >= 560.0 {
                cv.chip(
                    rx + right_w - 204.0 - 232.0,
                    top + 8.0,
                    222.0,
                    30.0,
                    &self.jog_label(),
                    self.jog_ui.open,
                    Act::Jog(JogAct::Toggle),
                );
            }
            cv.text(
                rx + right_w - 18.0,
                top + card_h - 16.0,
                Align2::RIGHT_BOTTOM,
                self.tr(Text::NavHint3d),
                13.0,
                col(t.dim),
            );
        }

        // ---- the thin bars between the panes: almost invisible until the pointer is over one, then lit; they are dragged to share the width
        {
            let (bl, br) = crate::panes::bars(self.panes.0, self.panes.1);
            let bars: Vec<(f32, Zone)> = if self.plan_big {
                vec![(bl, Zone::SplitLeft)]
            } else {
                vec![(bl, Zone::SplitLeft), (br, Zone::SplitRight)]
            };
            for (bx, z) in bars {
                let r = cv.rect(bx - 8.0, top, 16.0, card_h);
                let (hover, _) = cv.touch(r);
                let lit = hover || self.drag_zone == Some(z);
                let (a, w) = if lit { (200, 3.0) } else { (22, 1.0) };
                cv.line(
                    (bx, top + 24.0),
                    (bx, top + card_h - 24.0),
                    w,
                    alpha(col(t.accent), a),
                );
                cv.zones.push((r, z));
            }
        }

        // ---- bottom: the numbers and the buttons (every place comes from `hud_layout`, counted from the top of the card)
        cv.card(20.0, hud_y, design_w() - 40.0, HUD_H, false, None);
        let lay = hudinfo::hud_layout();
        let at = |r: Rect| r.translate(Vec2::new(0.0, hud_y));
        let view = self.running_view();
        let vals = [
            (
                "X",
                format!(
                    "{} {}",
                    self.len_text(view.as_ref().map_or(0.0, |v| v.x_mm)),
                    self.len_unit()
                ),
            ),
            (
                "Y",
                format!(
                    "{} {}",
                    self.len_text(view.as_ref().map_or(0.0, |v| v.y_mm)),
                    self.len_unit()
                ),
            ),
            (
                self.tr(Text::HudFeedZ),
                format!("{}", view.as_ref().map_or(0, |v| v.z_feed)),
            ),
            (
                self.tr(Text::HudSpindle),
                format!("{}", view.as_ref().map_or(0, |v| v.rpm)),
            ),
            (
                self.tr(Text::HudTool),
                view.as_ref()
                    .and_then(|v| v.tool)
                    .map_or("--".into(), |n| format!("T{n:02}")),
            ),
            (self.tr(Text::LayerMachine), self.connection.clone()),
        ];
        for (i, ((lab, val), cell)) in vals.iter().zip(&lay.vals).enumerate() {
            let (l, v) = (at(cell.label), at(cell.value));
            cv.text_fit(
                l.min.x,
                l.max.y,
                Align2::LEFT_BOTTOM,
                &lab.to_uppercase(),
                11.0,
                col(t.dim),
                l.width(),
            );
            cv.text_fit(
                v.min.x,
                v.max.y,
                Align2::LEFT_BOTTOM,
                val,
                if i == 5 { 19.0 } else { 26.0 },
                col(t.text),
                v.width(),
            );
        }
        let progress = self.progress();
        let (pl, bar, pct) = (at(lay.progress_label), at(lay.bar), at(lay.percent));
        cv.text(
            pl.min.x,
            pl.max.y,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::Progress).to_uppercase(),
            11.0,
            col(t.dim),
        );
        cv.p.rect_filled(
            cv.rect(bar.min.x, bar.min.y, bar.width(), bar.height()),
            (bar.height() / 2.0) * cv.u,
            alpha(Color32::WHITE, 26),
        );
        cv.p.rect_filled(
            cv.rect(
                bar.min.x,
                bar.min.y,
                bar.width() * progress as f32,
                bar.height(),
            ),
            (bar.height() / 2.0) * cv.u,
            col(t.ok),
        );
        if cv.running && progress > 0.01 {
            // a soft light that crosses the part that is done, once every two seconds
            let fill_w = bar.width() * progress as f32;
            let from = (cv.time / 2.0).fract() * (fill_w + 80.0) - 40.0;
            let (a, b) = (from.max(0.0), (from + 40.0).min(fill_w));
            if b > a {
                cv.p.rect_filled(
                    cv.rect(bar.min.x + a, bar.min.y + 2.0, b - a, bar.height() - 4.0),
                    3.0 * cv.u,
                    alpha(Color32::WHITE, 70),
                );
            }
        }
        cv.text_fit(
            pct.min.x,
            pct.max.y,
            Align2::LEFT_BOTTOM,
            &format!("{} %", (progress * 100.0).round()),
            16.0,
            col(t.text),
            pct.width(),
        );
        // what is being done, in four blocks (small label, big value, line of detail): the tool and its step, the times, the position and the depth, and the counters
        if let Some(blocks) = self.hud_blocks() {
            for (b, cell) in blocks.iter().zip(&lay.blocks) {
                let (l, v, d) = (at(cell.label), at(cell.value), at(cell.detail));
                cv.text_fit(
                    l.min.x,
                    l.max.y,
                    Align2::LEFT_BOTTOM,
                    &b.label.to_uppercase(),
                    10.5,
                    col(t.dim),
                    l.width(),
                );
                cv.text_fit(
                    v.min.x,
                    v.max.y,
                    Align2::LEFT_BOTTOM,
                    &b.value,
                    20.0,
                    col(t.text),
                    v.width(),
                );
                cv.text_fit(
                    d.min.x,
                    d.max.y,
                    Align2::LEFT_BOTTOM,
                    &b.detail,
                    13.0,
                    col(t.dim),
                    d.width(),
                );
            }
        }
        // the buttons of the job: Start goes out when the job runs and comes back with Pause or Stop; Pause and Stop work only while it runs
        let (can_start, can_pause, can_stop) = self.job_buttons();
        // bright colours that mean the same in every theme: green to go, blue to hold (brighter while it holds), red to stop
        let (pause_c, start_c, stop_c) = (
            if self.paused {
                Color32::from_rgb(90, 200, 255)
            } else {
                Color32::from_rgb(40, 150, 255)
            },
            Color32::from_rgb(40, 230, 90),
            Color32::from_rgb(255, 40, 40),
        );
        let [pb, sb, xb] = lay.buttons.map(at);
        if can_pause {
            cv.icon_button(
                pb.min.x,
                pb.min.y,
                pb.width(),
                pb.height(),
                Icon::Pause,
                pause_c,
                Act::Pause,
            );
        } else {
            cv.icon_button_off(
                pb.min.x,
                pb.min.y,
                pb.width(),
                pb.height(),
                Icon::Pause,
                pause_c,
            );
        }
        if can_start {
            cv.icon_button(
                sb.min.x,
                sb.min.y,
                sb.width(),
                sb.height(),
                Icon::Play,
                start_c,
                Act::Start,
            );
        } else {
            cv.icon_button_off(
                sb.min.x,
                sb.min.y,
                sb.width(),
                sb.height(),
                Icon::Play,
                start_c,
            );
        }
        if can_stop {
            cv.icon_button(
                xb.min.x,
                xb.min.y,
                xb.width(),
                xb.height(),
                Icon::Stop,
                stop_c,
                Act::Stop,
            );
        } else {
            cv.icon_button_off(
                xb.min.x,
                xb.min.y,
                xb.width(),
                xb.height(),
                Icon::Stop,
                stop_c,
            );
        }
        // the button with text that opens the panel to move the machine, under Pause, Start and Stop (lit while the panel is open)
        let jog = at(lay.jog);
        cv.chip(
            jog.min.x,
            jog.min.y,
            jog.width(),
            jog.height(),
            &self.jog_label(),
            self.jog_ui.open,
            Act::Jog(JogAct::Toggle),
        );
        // the note of a job that is only a simulation, on the line of the label of the progress and at its right
        let note = at(lay.note);
        cv.text_fit(
            note.max.x,
            note.max.y,
            Align2::RIGHT_BOTTOM,
            self.tr(Text::SimulatedJob),
            11.0,
            col(t.dim),
            note.width(),
        );
        self.paint_estop(cv);
        self.paint_jog(cv);

        // ---- a question of the machine for the operator
        if let Some(p) = self.running {
            let side = if p == Program::Drill {
                &self.drill
            } else {
                &self.route
            };
            if let Some(q) = side.engine.question() {
                let text = match q {
                    Question::SpindleStart => self.tr(Text::AskSpindle).to_string(),
                    Question::ToolChange(n) => format!("{} {n:02}", self.tr(Text::AskTool)),
                };
                cv.p.rect_filled(
                    Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10_000.0, 10_000.0)),
                    0.0,
                    alpha(Color32::BLACK, 150),
                );
                cv.card(500.0, 300.0, 600.0, 260.0, true, None);
                cv.text(
                    800.0,
                    390.0,
                    Align2::CENTER_BOTTOM,
                    &text,
                    28.0,
                    col(t.text),
                );
                cv.button(
                    560.0,
                    450.0,
                    220.0,
                    70.0,
                    self.tr(Text::Ok),
                    col(t.ok),
                    Act::Answer(true),
                );
                cv.button(
                    820.0,
                    450.0,
                    220.0,
                    70.0,
                    self.tr(Text::Abort),
                    col(t.danger),
                    Act::Answer(false),
                );
            }
        }
        // ---- the drilling is done and the routing comes next: the person follows the process when ready
        if self.stage_wait {
            cv.p.rect_filled(
                Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10_000.0, 10_000.0)),
                0.0,
                alpha(Color32::BLACK, 150),
            );
            cv.card(400.0, 300.0, 800.0, 260.0, true, None);
            cv.text_fit(
                800.0,
                400.0,
                Align2::CENTER_BOTTOM,
                self.tr(Text::StageDoneTitle),
                26.0,
                col(t.text),
                740.0,
            );
            cv.button(
                450.0,
                450.0,
                460.0,
                70.0,
                self.tr(Text::StageFollow),
                col(t.ok),
                Act::Start,
            );
            cv.button(
                940.0,
                450.0,
                210.0,
                70.0,
                self.tr(Text::Abort),
                col(t.danger),
                Act::Stop,
            );
        }
        self.paint_summary(cv);
    }

    /// The summary of the job over everything else, with its two buttons.
    fn paint_summary(&self, cv: &mut Cv) {
        let Some((_, _, lines)) = &self.summary_ask else {
            return;
        };
        let t = cv.t.clone();
        let h = 190.0 + 34.0 * lines.len() as f32;
        let y = ((DESIGN_H - h) / 2.0).max(20.0);
        cv.p.rect_filled(
            Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(10_000.0, 10_000.0)),
            0.0,
            alpha(Color32::BLACK, 150),
        );
        cv.card(250.0, y, 1100.0, h, true, None);
        for (i, l) in lines.iter().enumerate() {
            let (size, color) = if i == 0 {
                (30.0, t.text)
            } else {
                (22.0, t.dim)
            };
            cv.text(
                290.0,
                y + 24.0 + 34.0 * i as f32,
                Align2::LEFT_TOP,
                l,
                size,
                col(color),
            );
        }
        cv.button(
            560.0,
            y + h - 100.0,
            220.0,
            70.0,
            self.tr(Text::Ok),
            col(t.ok),
            Act::Summary(true),
        );
        cv.button(
            820.0,
            y + h - 100.0,
            220.0,
            70.0,
            self.tr(Text::Abort),
            col(t.danger),
            Act::Summary(false),
        );
    }

    fn running_view(&self) -> Option<eh3d_drillpro::classic::engine::JobView> {
        let p = self.running?;
        if p == Program::Drill {
            self.drill.engine.job_view()
        } else {
            self.route.engine.job_view()
        }
    }

    /// Where the head of the machine of the 3D view is: the machine has a place of its own (it starts, goes home, travels to the first point and works), so the board
    /// can be moved in the plan view without the machine following it.
    fn head3d(&self) -> Option<(f64, f64)> {
        if let Some(h) = self.jog_head() {
            return Some((h.0, h.1));
        }
        Some(self.m3d.pos())
    }

    /// How high the tip of the tool is over the top of the board in the 3D view, in mm: the steps of the plan say how far down the head is (home is the top).
    fn head_lift_mm(&self) -> f64 {
        let at_board = self
            .drill
            .engine
            .settings()
            .map_or(23.7, |m| m.z_max_depth_mm - m.base_mm - m.board_mm)
            .clamp(8.0, 60.0);
        // moved by hand: the panel says how deep
        if let Some(h) = self.jog_head() {
            return (at_board - h.2).clamp(-4.0, 60.0);
        }
        match (self.running, self.head_z) {
            (Some(_), Some(z)) => (at_board - z).clamp(-4.0, 60.0),
            _ => at_board,
        }
    }

    /// The diameter of the tool in the head, in mm: the drill of the hole that is next, or a cutter while a routing job runs.
    fn head_tool_diameter(&self) -> f64 {
        let tool = self.running_view().and_then(|v| v.tool);
        match self.running {
            Some(Program::Route) => 2.0,
            _ => self
                .drill
                .scene
                .holes
                .iter()
                .find(|h| Some(h.tool) == tool)
                .or_else(|| self.drill.scene.holes.first())
                .map_or(1.0, |h| h.diameter_mm),
        }
    }

    /// Where the head is: the running job says it; otherwise the next hole to drill.
    /// Where the head of the running job is along the steps of its plan (X, Y, Z in mm), the trips to the tool bar included.
    fn running_head_mm(&self) -> Option<(f64, f64, f64)> {
        match self.running? {
            Program::Drill => self.drill.engine.head_mm(),
            Program::Route => self.route.engine.head_mm(),
        }
    }

    fn head(&self) -> Option<(f64, f64)> {
        // moved by hand from the panel
        if let Some(h) = self.jog_head() {
            return Some((h.0, h.1));
        }
        // the simulated head: the plan view and the bar follow it to the bar and back, as the 3D view does
        if let Some(h) = self.running_head_mm() {
            return Some((h.0, h.1));
        }
        if let Some(v) = self.running_view() {
            return Some((v.x_mm, v.y_mm));
        }
        self.drill
            .scene
            .holes
            .iter()
            .find(|h| !h.done)
            .map(|h| (h.x_mm, h.y_mm))
    }

    fn progress(&self) -> f64 {
        let mut all = self.drill.scene.clone();
        all.overlay(&self.route.scene);
        all.progress()
    }

    /// The machine in 3D, built as the Bungard CCD machines are and as big as the one the table of the configuration belongs to: two
    /// aluminium rails with black end caps and adjustable feet, the white perforated plate between them, the gantry that runs along the
    /// rails (its motor at one end) with the head and the spindle, and, on the models with the tool changer, the rack at the short edge
    /// with the tools at the positions of the configuration. The published work area is drawn thin and the usable area of the
    /// configuration (which may be a few centimetres bigger or smaller) dashed; the frame grows when the usable area is bigger.
    #[allow(clippy::too_many_arguments)]
    fn paint_machine(
        &self,
        cv: &mut Cv,
        x0: f32,
        y0: f32,
        w: f32,
        h: f32,
        sizes: &[f64],
        head: Option<(f64, f64)>,
    ) {
        let t = cv.t.clone();
        let (tw, td) = (self.table.width_mm, self.table.depth_mm);
        // the machine chosen in the settings; the one the table is most like when the name is not one of the models
        let profile = match machines::Model::from_name(&self.settings.model) {
            Some(i) => machines::models()[i].profile(),
            None => machines::for_table(&self.table),
        };
        let (wx, wy) = (profile.work_mm.0.max(tw), profile.work_mm.1.max(td));
        let (px_m, py_m) = profile.plate_margins((tw, td));
        let frame = self.machine_frame();
        // the view holds the whole frame: the plate, the rails, the motor and the rack
        let shift = (px_m + 60.0, py_m + 60.0);
        let mut iso = Iso::fit(
            &Table::new(wx + 2.0 * shift.0, wy + 2.0 * shift.1 + 80.0),
            // the gantry and the spindle of its real height (174 mm long, over the board)
            230.0,
            w as f64,
            h as f64,
        );
        {
            let nav = self.view3d;
            iso.yaw_deg = nav.yaw as f64;
            iso.pitch_deg = nav.pitch as f64;
            iso.pivot = (
                (wx + 2.0 * shift.0) / 2.0,
                (wy + 2.0 * shift.1 + 80.0) / 2.0,
            );
            let (mx, my, z) = (w as f64 / 2.0, h as f64 / 2.0, nav.zoom as f64);
            iso.k *= z;
            iso.origin = (
                mx + (iso.origin.0 - mx) * z + nav.pan.0 as f64,
                my + (iso.origin.1 - my) * z + nav.pan.1 as f64,
            );
            // centred on the middle of the work: Y of the files goes away from the operator, as the work is drawn
            if let Some((fx, fy)) = nav.focus {
                let (px, py) = iso.project(fx + shift.0, td - fy + shift.1, 30.0);
                iso.origin.0 += mx - px;
                iso.origin.1 += my - py;
            }
        }
        let o = (x0, y0);
        let p = |x: f64, y: f64, z: f64| {
            let (px, py) = iso.project(x + shift.0, y + shift.1, z);
            (o.0 + px as f32, o.1 + py as f32)
        };
        let edge = alpha(col(t.line), 170);
        let alu = Color32::from_rgb(196, 200, 206);
        let black = Color32::from_rgb(34, 34, 37);
        let plate = Color32::from_rgb(214, 215, 210);
        let steel = Color32::from_rgb(168, 174, 182);
        let motor = Color32::from_rgb(70, 72, 78);
        let boxes = |cv: &Cv, list: &[(f64, f64, f64, f64, f64, f64, Color32)]| {
            draw_boxes(cv, &iso, o, shift, list, edge);
        };
        let (hx, hy) = head.unwrap_or((tw / 2.0, td / 2.0));
        let lift = self.head_lift_mm();
        // the work is drawn as in the plan view: Y of the files goes away from the operator, and the machine's Y comes towards him
        let hy = td - hy;
        let board_z = 30.0;
        // the frame: the plate goes from -px_m to wx + px_m along the rails and from -py_m to wy + py_m across them
        let (fx0, fx1) = (frame.fx0, frame.fx1);
        let (fy0, fy1) = (frame.fy0, frame.fy1);
        let flen = fx1 - fx0;
        // the aluminium profiles of the axes (the plan view draws the same ones)
        let (rail_w, rail_h) = (frame.rail_across, frame.rail_tall);
        let (beam_w, beam_h) = (frame.beam_across, frame.beam_tall);

        // the rail at the back and its feet
        boxes(
            cv,
            &[
                (fx0 + 10.0, fy0 - 34.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx1 - 36.0, fy0 - 34.0, -18.0, 26.0, 26.0, 18.0, black),
                (
                    fx0 - 34.0,
                    fy0 - rail_w,
                    0.0,
                    flen + 68.0,
                    rail_w,
                    rail_h,
                    alu,
                ),
                (fx0 - 38.0, fy0 - rail_w, 0.0, 4.0, rail_w, rail_h, black),
            ],
        );
        // the mouths of the slots along the rail and its section at the end: the profile, not a tube
        let dims = self.settings.profile;
        let back = self::frame::extrude(
            &dims,
            self::frame::Axis::X,
            (fx0 - 34.0, fy0 - rail_w, 0.0),
            flen + 68.0,
        );
        let strips: Vec<_> = back
            .strips
            .iter()
            .map(|s| (s.0, s.1, s.2, s.3, s.4, s.5, black))
            .collect();
        boxes(cv, &strips);
        self::frame::paint_cap(cv, &p, &back.cap, alu, black, edge);
        // the cable chain along the rail
        boxes(
            cv,
            &[(fx0 + 20.0, fy0 - 8.0, 40.0, flen - 40.0, 12.0, 8.0, black)],
        );
        // the plate with its grid of holes, three pins and two discs
        boxes(cv, &[(fx0, fy0, 22.0, flen, wy + 2.0 * py_m, 8.0, plate)]);
        let mut gx = fx0 + 25.0;
        while gx < fx1 - 15.0 {
            let mut gy = fy0 + 25.0;
            while gy < fy1 - 15.0 {
                let (px, py) = p(gx, gy, board_z + 0.2);
                let r = (4.0 * iso.k) as f32;
                let pts: Vec<(f32, f32)> = (0..8)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / 8.0;
                        (px + r * 1.15 * a.cos(), py + r * 0.62 * a.sin())
                    })
                    .collect();
                cv.poly(&pts, alpha(Color32::BLACK, 120), Stroke::NONE);
                gy += 40.0;
            }
            gx += 40.0;
        }
        for (cx, cy) in [(wx * 0.62, wy * 0.22), (wx * 0.30, wy * 0.74)] {
            let (px, py) = p(cx, cy, board_z + 0.4);
            let r = (22.0 * iso.k) as f32;
            let pts: Vec<(f32, f32)> = (0..14)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 14.0;
                    (px + r * 1.15 * a.cos(), py + r * 0.62 * a.sin())
                })
                .collect();
            cv.poly(
                &pts,
                Color32::from_rgb(244, 244, 240),
                Stroke::new(1.0 * cv.u, edge),
            );
        }
        boxes(
            cv,
            &[
                (fx0 + 40.0, fy1 - 40.0, board_z, 12.0, 12.0, 6.0, steel),
                (wx * 0.5, fy0 + 30.0, board_z, 12.0, 12.0, 6.0, steel),
                (fx1 - 50.0, fy1 - 60.0, board_z, 12.0, 12.0, 6.0, steel),
            ],
        );
        // the work area of the maker (thin) and the usable area of the configuration (dashed)
        let ring = |x0r: f64, y0r: f64, x1r: f64, y1r: f64| -> Vec<(f32, f32)> {
            [(x0r, y0r), (x1r, y0r), (x1r, y1r), (x0r, y1r), (x0r, y0r)]
                .iter()
                .map(|&(x, y)| p(x, y, board_z + 0.6))
                .collect()
        };
        cv.polyline(
            &ring(0.0, 0.0, profile.work_mm.0, profile.work_mm.1),
            1.0,
            alpha(col(t.dim), 200),
            false,
        );
        cv.polyline(&ring(0.0, 0.0, tw, td), 1.6, col(t.accent), true);
        // the rail at the front and its feet
        boxes(
            cv,
            &[
                (fx0 + 10.0, fy1 + 8.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx1 - 36.0, fy1 + 8.0, -18.0, 26.0, 26.0, 18.0, black),
                (fx0 - 34.0, fy1, 0.0, flen + 68.0, rail_w, rail_h, alu),
                (fx0 - 38.0, fy1, 0.0, 4.0, rail_w, rail_h, black),
            ],
        );
        let front = self::frame::extrude(
            &dims,
            self::frame::Axis::X,
            (fx0 - 34.0, fy1, 0.0),
            flen + 68.0,
        );
        let strips: Vec<_> = front
            .strips
            .iter()
            .map(|s| (s.0, s.1, s.2, s.3, s.4, s.5, black))
            .collect();
        boxes(cv, &strips);
        self::frame::paint_cap(cv, &p, &front.cap, alu, black, edge);
        // the board and the work on it
        // the same board as the plan view shows: the size chosen, or the one that stayed where it was while the design moved alone
        if let Some((x0b, y0b, x1b, y1b)) = self.panel_rect() {
            boxes(
                cv,
                &[(
                    x0b,
                    td - y1b,
                    board_z,
                    x1b - x0b,
                    y1b - y0b,
                    3.0,
                    col(t.board),
                )],
            );
        }
        let top = board_z + 3.2;
        if self.layers[0] {
            for hole in &self.drill.scene.holes {
                let idx = sizes
                    .iter()
                    .position(|d| (*d - hole.diameter_mm).abs() < 0.005)
                    .unwrap_or(0);
                let c = col(t.size_color(idx));
                let (px, py) = p(hole.x_mm, td - hole.y_mm, top);
                let r = ((hole.diameter_mm * iso.k * 0.9).max(1.4)) as f32;
                let pts: Vec<(f32, f32)> = (0..10)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / 10.0;
                        (px + r * 1.15 * a.cos(), py + r * 0.62 * a.sin())
                    })
                    .collect();
                cv.poly(
                    &pts,
                    if hole.done {
                        c
                    } else {
                        alpha(Color32::BLACK, 140)
                    },
                    Stroke::new(1.0 * cv.u, c),
                );
            }
        }
        if self.layers[1] {
            let tool_colors = self.route_tool_colors();
            for path in &self.route.scene.paths {
                let pts: Vec<(f32, f32)> = path
                    .points
                    .iter()
                    .map(|&(x, y)| p(x, td - y, top))
                    .collect();
                let line_color = tool_colors
                    .iter()
                    .find(|(tool, _)| *tool == path.tool)
                    .map_or(col(t.accent2), |&(_, i)| col(t.size_color(i)));
                cv.polyline(&pts, 1.8, line_color, true);
                if path.reached > 1 {
                    cv.polyline(&pts[..path.reached.min(pts.len())], 2.2, col(t.ok), false);
                }
            }
        }
        // the gantry: two carriages on the rails, the beam across, its motor at the end, the head and the spindle
        boxes(
            cv,
            &[
                (hx - 34.0, fy0 - rail_w, rail_h, 64.0, rail_w, 26.0, alu),
                (hx - 34.0, fy1, rail_h, 64.0, rail_w, 26.0, alu),
                (
                    hx - self::frame::BEAM_BEHIND,
                    fy0 - rail_w,
                    rail_h + 22.0,
                    beam_w,
                    frame.beam_len(),
                    beam_h,
                    alu,
                ),
                (hx - 64.0, fy1 + rail_w, 62.0, 44.0, 44.0, 44.0, motor),
                // the slide that carries the spindle, beside it (the spindle itself is drawn below, with its own sizes)
                (
                    hx - 32.0,
                    hy - 22.0,
                    top + lift + 30.0,
                    14.0,
                    44.0,
                    150.0,
                    alu,
                ),
            ],
        );
        // the beam is the same profile lying the same way: the mouths of its slots along it and its section at the end
        let beam = self::frame::extrude(
            &dims,
            self::frame::Axis::Y,
            (hx - self::frame::BEAM_BEHIND, fy0 - rail_w, rail_h + 22.0),
            frame.beam_len(),
        );
        let strips: Vec<_> = beam
            .strips
            .iter()
            .map(|s| (s.0, s.1, s.2, s.3, s.4, s.5, black))
            .collect();
        boxes(cv, &strips);
        self::frame::paint_cap(cv, &p, &beam.cap, alu, black, edge);
        // the spindle of the original machine at its real size, with the tool in its collet
        let tool_d = self.head_tool_diameter();
        self.paint_spindle(
            cv,
            &iso,
            &|x, y, z| p(x, y, z),
            (hx, hy, top + lift),
            tool_d,
        );
        let tip = p(hx, hy, top + lift);
        cv.p.circle_filled(cv.pt(tip.0, tip.1), 2.4 * cv.u, col(t.ok));
        // the tool changer: the bar at the short edge with the tools in it, and the three roles of the exchange
        self.paint_bar_3d(cv, &iso, o, shift, edge, board_z, td, tip);
        cv.text(
            x0 + 18.0,
            y0 + h + 6.0,
            Align2::LEFT_BOTTOM,
            &format!(
                "{} - {:.0} x {:.0} x {:.0} mm - {:.0} kg",
                profile.name,
                profile.work_mm.0,
                profile.work_mm.1,
                profile.work_mm.2,
                profile.weight_kg
            ),
            12.0,
            col(t.dim),
        );
    }
}

fn install_fonts(ctx: &egui::Context) {
    let candidates = [
        "C:/Windows/Fonts/msyh.ttc",
        "C:/Windows/Fonts/YuGothM.ttc",
        "C:/Windows/Fonts/meiryo.ttc",
        "C:/Windows/Fonts/simsun.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        "/System/Library/Fonts/PingFang.ttc",
    ];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            let mut fonts = FontDefinitions::default();
            fonts
                .font_data
                .insert("cjk".into(), Arc::new(FontData::from_owned(bytes)));
            for family in [FontFamily::Monospace, FontFamily::Proportional] {
                fonts.families.entry(family).or_default().push("cjk".into());
            }
            ctx.set_fonts(fonts);
            return;
        }
    }
}

fn save_png(img: &egui::ColorImage, path: &Path) -> std::io::Result<()> {
    let file = std::io::BufWriter::new(std::fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, img.size[0] as u32, img.size[1] as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut w = enc.write_header().map_err(std::io::Error::other)?;
    let data: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
    w.write_image_data(&data).map_err(std::io::Error::other)
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let frame_began = Instant::now();
        self.frame += 1;
        self.diag_begin(&ctx);
        if self.visuals_theme != Some(self.theme.id) {
            apply_visuals(&ctx, &self.theme);
            self.visuals_theme = Some(self.theme.id);
        }
        (self.pointer, self.pointer_down) =
            ctx.input(|i| (i.pointer.hover_pos(), i.pointer.primary_down()));
        if self.start_at_frame == Some(self.frame) {
            self.start_job();
        }
        if self.check_at_frame == Some(self.frame) {
            self.insp_run();
        }
        // the keys: Enter takes the modern interface at the launcher; Esc stops the job
        if ctx.input(|i| i.key_pressed(egui::Key::Enter)) && self.screen == Screen::Launcher {
            self.act(Act::Choose(Choice::Modern));
        }
        // the panel that moves the machine takes Esc (and its own keys) first, when it is open and busy
        self.jog_hotkey(&ctx);
        self.jog_keys(&ctx);
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.edit.is_some() {
                self.edit = None;
            } else if self.modal.is_some() {
                self.modal = None;
            } else if self.running.is_some() {
                self.stop_job();
            }
        }
        self.key_mods = ctx.input(|i| (i.modifiers.command, i.modifiers.shift));
        // the words of the menu of the text boxes
        ctx.data_mut(|d| d.insert_temp(egui::Id::unique("ui-language"), self.lang));
        self.run_pending_text_keys(&ctx);
        if let Some(text) = self.pending_copy.take() {
            ctx.copy_text(text);
        }
        // F11 fills the screen or gives it back; the window follows the setting (a picture keeps its size)
        if self.shot.is_none() {
            if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
                self.act(Act::ToggleFullscreen);
            }
            if self.settings.fullscreen != self.fullscreen_applied {
                self.fullscreen_applied = self.settings.fullscreen;
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen_applied));
                if !self.fullscreen_applied {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                }
            }
        }
        // editing the job in the plan view: copy, cut, paste, choose all and delete (not while a box is being typed in)
        if self.screen == Screen::Viewer
            && self.modal.is_none()
            && self.edit.is_none()
            && !ctx.egui_wants_keyboard_input()
            && self.running.is_none()
        {
            let (copy, cut, paste, all, del) = ctx.input(|i| {
                let cmd = i.modifiers.command;
                (
                    i.events.iter().any(|e| matches!(e, egui::Event::Copy)),
                    i.events.iter().any(|e| matches!(e, egui::Event::Cut)),
                    i.events.iter().any(|e| matches!(e, egui::Event::Paste(_))),
                    cmd && i.key_pressed(egui::Key::A),
                    i.key_pressed(egui::Key::Delete),
                )
            });
            if copy {
                self.copy_selection();
            }
            if cut {
                self.cut_selection();
            }
            if paste {
                self.paste_clip();
            }
            if all {
                self.select_all();
            }
            if del {
                self.delete_selection();
            }
            // the arrows move what is chosen: 1 mm, 0.1 mm with Shift, 10 mm with Ctrl
            if self.selection_count() > 0 {
                let (step, left, right, up, down) = ctx.input(|i| {
                    let step = if i.modifiers.command {
                        10.0
                    } else if i.modifiers.shift {
                        0.1
                    } else {
                        1.0
                    };
                    (
                        step,
                        i.key_pressed(egui::Key::ArrowLeft),
                        i.key_pressed(egui::Key::ArrowRight),
                        i.key_pressed(egui::Key::ArrowUp),
                        i.key_pressed(egui::Key::ArrowDown),
                    )
                });
                let dx = if right {
                    step
                } else if left {
                    -step
                } else {
                    0.0
                };
                let dy = if up {
                    step
                } else if down {
                    -step
                } else {
                    0.0
                };
                if dx != 0.0 || dy != 0.0 {
                    self.move_selection(dx, dy);
                }
            }
        }
        // not while the window is already open: a held key or a second press would build it again and send it back to the folder it starts in
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O))
            && self.screen == Screen::Viewer
            && self.modal != Some(Modal::Open)
        {
            self.show_modal(Modal::Open);
        }
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        for p in dropped {
            crate::applog::info(
                "file",
                format!("file dropped on the window: {}", p.display()),
            );
            self.screen = Screen::Viewer;
            self.modal = None;
            self.open_path_asking(&p);
        }
        self.tick();
        let shot_file = self
            .shot
            .as_ref()
            .map(|s| (s.file.clone(), s.wait, s.done.clone()));
        if let Some((file, wait, done)) = shot_file {
            if self.frame == 3 + wait {
                let (f2, d2, c2) = (file.clone(), done.clone(), ctx.clone());
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                    egui::ScreenshotCallback::new(move |img| {
                        if let Err(e) = save_png(&img, &f2) {
                            eprintln!("cannot save {}: {e}", f2.display());
                        }
                        d2.store(true, Ordering::SeqCst);
                        c2.request_repaint();
                    }),
                ));
            }
            if done.load(Ordering::SeqCst) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        self.upd_poll();
        self.jog_tick(&ctx);
        if self.close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.head_tick(&ctx);
        if self.modal == Some(Modal::Head) {
            self.head_fields(&ctx);
        }
        if self.modal == Some(Modal::Connection) {
            self.connection_fields(&ctx);
        }
        if self.modal == Some(Modal::Open) {
            self.open_fields(&ctx);
        }
        if self.modal == Some(Modal::Settings) && self.settings_tab == 2 {
            self.sound_fields(&ctx);
        }
        self.edit_overlay(&ctx);
        let mut clicked: Option<Pos2> = None;
        let mut double: Option<Pos2> = None;
        egui::Frame::NONE.show(ui, |ui| {
            let (rect, resp) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
            let painter = ui.painter_at(rect);
            self.paint(&painter, rect);
            if resp.drag_started() {
                let at = resp.interact_pointer_pos();
                // a drag that starts on the track of a scroll bar moves the bar, not the view
                self.bar_drag = at.and_then(|p| {
                    self.bars
                        .iter()
                        .rev()
                        .find(|b| b.track.expand(8.0).contains(p))
                        .map(|b| b.bar)
                });
                if self.bar_drag.is_none() {
                    let z = at.and_then(|p| self.zone_at(p));
                    let choosing = z == Some(Zone::Plan)
                        && self.select_mode
                        && !self.move_board
                        && !self.move_design
                        && !resp.dragged_by(egui::PointerButton::Secondary)
                        && !resp.dragged_by(egui::PointerButton::Middle);
                    if let (true, Some(m)) = (choosing, at.and_then(|p| self.plan_point(p))) {
                        if self.is_over_choice(m) {
                            // the choice is dragged to another place
                            self.sel_move_from = Some(m);
                            self.sel_move = Some((0.0, 0.0));
                        } else {
                            // a box to choose with, instead of moving the view
                            self.band = Some((m, m));
                        }
                        self.drag_zone = None;
                    } else {
                        self.drag_zone = z;
                    }
                }
            }
            if resp.dragged() && self.sel_move_from.is_some() {
                if let (Some(m), Some(from)) = (
                    resp.interact_pointer_pos().and_then(|p| self.plan_point(p)),
                    self.sel_move_from,
                ) {
                    self.sel_move = Some((m.0 - from.0, m.1 - from.1));
                }
            } else if resp.dragged() && self.band.is_some() {
                if let Some(m) = resp.interact_pointer_pos().and_then(|p| self.plan_point(p)) {
                    if let Some(b) = self.band.as_mut() {
                        b.1 = m;
                    }
                }
            } else if resp.dragged() && self.bar_drag.is_some() {
                if let (Some(bar), Some(p)) = (self.bar_drag, resp.interact_pointer_pos()) {
                    self.bar_to(bar, p);
                }
            } else if resp.dragged() {
                if let Some(z) = self.drag_zone {
                    let d = resp.drag_delta() / self.ui_scale.max(0.01);
                    let pan = resp.dragged_by(egui::PointerButton::Secondary)
                        || resp.dragged_by(egui::PointerButton::Middle)
                        || ctx.input(|i| i.modifiers.shift);
                    self.drag(z, (d.x, d.y), pan);
                }
            }
            if resp.drag_stopped() {
                if let (Some((dx, dy)), Some(_)) = (self.sel_move.take(), self.sel_move_from.take())
                {
                    if dx.abs() + dy.abs() > 0.05 {
                        self.move_selection(dx, dy);
                    }
                }
                if let Some((a, b)) = self.band.take() {
                    let add = self.key_mods.0 || self.key_mods.1;
                    self.select_in_box(a, b, b.0 < a.0, add);
                }
                self.bar_drag = None;
                if self.drag_zone == Some(Zone::Estop) {
                    self.remember_settings();
                }
                if matches!(self.drag_zone, Some(Zone::SplitLeft | Zone::SplitRight)) {
                    self.keep_panes();
                }
                self.drag_zone = None;
            }
            if resp.double_clicked() {
                double = resp.interact_pointer_pos();
            } else if resp.clicked() {
                clicked = resp.interact_pointer_pos();
            }
        });
        if let Some(p) = self.pointer {
            let from = if self.modal.is_some() {
                self.modal_from
            } else {
                0
            };
            if self
                .hits
                .get(from..)
                .unwrap_or(&[])
                .iter()
                .any(|(r, _)| r.contains(p))
            {
                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }
        if let Some(pos) = double {
            if let Some(z) = self.zone_at(pos) {
                self.reset_view(z);
            }
        }
        // over a thin bar (or while one is dragged) the pointer shows that the width can be changed
        if let Some(p) = ctx.input(|i| i.pointer.hover_pos()) {
            if matches!(self.zone_at(p), Some(Zone::SplitLeft | Zone::SplitRight))
                || matches!(self.drag_zone, Some(Zone::SplitLeft | Zone::SplitRight))
            {
                ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
        }
        let scroll = ctx.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            if self.combo.is_some() {
                self.combo_scroll(if scroll > 0.0 { -2 } else { 2 });
            } else if self.modal == Some(Modal::Catalogue) {
                let over_detail = ctx
                    .input(|i| i.pointer.hover_pos())
                    .is_some_and(|p| self.cat_detail_area.get().contains(p));
                if over_detail {
                    self.cat_detail_scroll_by(if scroll > 0.0 { -60.0 } else { 60.0 });
                } else {
                    self.cat_scroll_by(if scroll > 0.0 { -3 } else { 3 });
                }
            } else if self.modal == Some(Modal::Open) {
                self.browse_scroll(if scroll > 0.0 { -2 } else { 2 });
            } else if self.modal == Some(Modal::Board) {
                self.mat_scroll_by(if scroll > 0.0 { -2 } else { 2 });
            } else if let Some(bar) = ctx
                .input(|i| i.pointer.hover_pos())
                .and_then(|p| self.bars.iter().find(|b| b.area.contains(p)).map(|b| b.bar))
            {
                self.bar_wheel(bar, if scroll > 0.0 { -1 } else { 1 });
            } else if let Some(z) = ctx
                .input(|i| i.pointer.hover_pos())
                .and_then(|p| self.zone_at(p))
            {
                self.zoom(z, scroll);
            }
        }
        // a click on an arrow of a scroll bar moves the list one row, and goes no further
        if let Some(pos) = clicked {
            let arrow = self.bars.iter().rev().find_map(|b| {
                if b.up.is_some_and(|r| r.expand(2.0).contains(pos)) {
                    Some((b.bar, -1))
                } else if b.down.is_some_and(|r| r.expand(2.0).contains(pos)) {
                    Some((b.bar, 1))
                } else {
                    None
                }
            });
            if let Some((bar, dir)) = arrow {
                self.bar_step(bar, dir);
                clicked = None;
            }
        }
        // a click on the track of a scroll bar puts the list there, and goes no further
        if let Some(pos) = clicked {
            if let Some(bar) = self
                .bars
                .iter()
                .rev()
                .find(|b| b.track.expand(8.0).contains(pos))
                .map(|b| b.bar)
            {
                self.bar_to(bar, pos);
                clicked = None;
            }
        }
        if let Some(pos) = clicked {
            // the last card or button drawn on top is the one that is hit; with a window on top, only its own
            let hit = self
                .hits
                .get(self.modal_from..)
                .unwrap_or(&[])
                .iter()
                .rev()
                .find(|(r, _)| r.contains(pos))
                .map(|(_, a)| *a);
            if let Some(a) = hit {
                self.act(a);
            } else if self.select_mode
                && self.modal.is_none()
                && self.zone_at(pos) == Some(Zone::Plan)
            {
                // a click on the plan view with the choosing tool on
                if let Some(m) = self.plan_point(pos) {
                    let add = self.key_mods.0 || self.key_mods.1;
                    self.select_at(m, add);
                }
            }
        }
        self.diag_end(frame_began);
        // fast while a job runs or a window or screen is coming in, a calm 15 frames a second otherwise (the slow breathing of the lights)
        let settling = self.animate
            && self.anim0.elapsed().as_secs_f32() - self.anim_screen.1.max(self.anim_modal.1)
                < 0.45;
        ctx.request_repaint_after(Duration::from_millis(
            if self.running.is_some() || settling {
                16
            } else if self.animate {
                66
            } else {
                100
            },
        ));
    }
}

/// Opens the window.
pub fn run(o: &Options) -> eframe::Result {
    let app = App::new(o);
    let fullscreen = o.shot.is_none() && app.settings.fullscreen;
    let size = if o.shot.is_some() {
        o.size
            .map_or(Vec2::new(1600.0, 900.0), |(w, h)| Vec2::new(w, h))
    } else {
        Vec2::new(1400.0, 820.0)
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(size)
            .with_min_inner_size([900.0, 560.0])
            .with_resizable(true)
            .with_maximize_button(true)
            .with_maximized(o.shot.is_none() && !fullscreen)
            .with_fullscreen(fullscreen)
            .with_title("EH3D-BUNGARD-GUI"),
        ..Default::default()
    };
    eframe::run_native(
        "EH3D-BUNGARD-GUI",
        options,
        Box::new(move |cc| {
            install_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    )
}
