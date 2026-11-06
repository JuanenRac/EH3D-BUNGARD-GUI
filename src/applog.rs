//! The log of the program: `bungard-gui.log` in the settings folder of the user, with the other EH3D programs.
//!
//! It exists to leave clues when the program disappears (a crash of the system that gives no panic, a lack of memory, a failure of the graphics
//! driver). Every line is written straight to the file, without a buffer in between, so what was logged before an abort is on the disk. Only changes
//! are logged (a window that opens, a file, a connection, a job that starts), plus a heartbeat every few seconds with the memory and the frame times,
//! never one line per frame. A mistake while writing (a full disk, a locked file) is swallowed: the log never stops the program.
//!
//! What is logged, how big the file may grow and how often the heartbeat beats are values of the settings file (`log`, `log_level`, `log_max_kb`,
//! `log_files`, `log_heartbeat_s`). Passwords, keys and tokens never reach the log: the words of a line are scrubbed before they are written.

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// The name of the log file; the older ones are `bungard-gui.log.1`, `bungard-gui.log.2`...
pub const FILE_NAME: &str = "bungard-gui.log";

/// The mark of the first line of a run, and the one of its last line when it ends by itself. A run with the first and without the second did not end cleanly.
pub const SESSION_START: &str = "=== session start";
pub const CLEAN_EXIT: &str = "=== clean exit";

/// How many identical lines in a row are written before the rest are only counted, how many lines a second are accepted before the rest are only counted,
/// how much of the end of the log is read to find out how the last run ended, and how many of its lines are copied.
const REPEAT_FREE: u32 = 3;
const MAX_LINES_PER_SECOND: u32 = 100;
const TAIL_BYTES: u64 = 64 * 1024;
const TAIL_LINES: usize = 40;
/// A pause between two frames this long is logged (the window is behind another one, or the program stopped for a while).
const FRAME_GAP_MS: u64 = 1500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN ",
            Level::Info => "INFO ",
            Level::Debug => "DEBUG",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Level::Error => "error",
            Level::Warn => "warn",
            Level::Info => "info",
            Level::Debug => "debug",
        }
    }

    pub fn parse(text: &str) -> Option<Level> {
        match text.trim().to_ascii_lowercase().as_str() {
            "error" => Some(Level::Error),
            "warn" | "warning" => Some(Level::Warn),
            "info" => Some(Level::Info),
            "debug" => Some(Level::Debug),
            _ => None,
        }
    }
}

/// What the settings file says about the log.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// The log is written at all.
    pub enabled: bool,
    /// The most detailed level that is written.
    pub level: Level,
    /// The size, in KiB, at which the file is closed and a new one is started.
    pub max_kb: u32,
    /// How many files are kept (the current one and the older ones).
    pub files: u32,
    /// Seconds between two heartbeats.
    pub heartbeat_s: u32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            enabled: true,
            level: Level::Info,
            max_kb: 1024,
            files: 3,
            heartbeat_s: 30,
        }
    }
}

impl Config {
    /// Takes one `key = value` of the settings file (the key already in lower case); a value out of range keeps what was there. Returns whether the key is one of the log.
    pub fn set(&mut self, key: &str, value: &str) -> bool {
        let v = value.trim();
        match key {
            "log" => {
                self.enabled = !matches!(
                    v.to_ascii_lowercase().as_str(),
                    "no" | "off" | "false" | "0"
                )
            }
            "log_level" => self.level = Level::parse(v).unwrap_or(self.level),
            "log_max_kb" => {
                self.max_kb = v
                    .parse()
                    .ok()
                    .filter(|x: &u32| (64..=65_536).contains(x))
                    .unwrap_or(self.max_kb)
            }
            "log_files" => {
                self.files = v
                    .parse()
                    .ok()
                    .filter(|x: &u32| (1..=20).contains(x))
                    .unwrap_or(self.files)
            }
            "log_heartbeat_s" => {
                self.heartbeat_s = v
                    .parse()
                    .ok()
                    .filter(|x: &u32| (5..=3600).contains(x))
                    .unwrap_or(self.heartbeat_s)
            }
            _ => return false,
        }
        true
    }

    pub fn to_text(&self) -> String {
        format!(
            "log = {}\nlog_level = {}\nlog_max_kb = {}\nlog_files = {}\nlog_heartbeat_s = {}\n",
            if self.enabled { "yes" } else { "no" },
            self.level.key(),
            self.max_kb,
            self.files,
            self.heartbeat_s
        )
    }
}

/// The folder of the log: the settings folder of the user (`%APPDATA%\EH3D`, `~/.config/eh3d`), with the other EH3D programs.
pub fn default_dir() -> PathBuf {
    if let Some(a) = std::env::var_os("APPDATA") {
        return PathBuf::from(a).join("EH3D");
    }
    if let Some(h) = std::env::var_os("HOME") {
        return PathBuf::from(h).join(".config").join("eh3d");
    }
    PathBuf::from(".")
}

/// The local time as a person reads it, and the seconds since 1970.
fn stamp() -> (String, u64) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    (
        chrono::Local::now()
            .format("%Y-%m-%d %H:%M:%S%.3f")
            .to_string(),
        secs,
    )
}

/// One line of the log: the local time, the seconds since 1970, the version, the process, the level, the part of the program and the words. The lines
/// after the first of a message are indented, so a line of the log always begins with a time.
pub fn format_line(
    local: &str,
    secs: u64,
    version: &str,
    pid: u32,
    level: Level,
    target: &str,
    msg: &str,
) -> String {
    let body = msg.replace('\r', "").replace('\n', "\n    ");
    format!(
        "{local} {secs} v{version} p{pid} {} [{target}] {body}\n",
        level.label()
    )
}

/// Takes out of a text what must not be kept: the user and password of an address, the part after `?` of an address and the value of a word
/// that is written `password=...`, `token=...`, `secret=...`, `key=...` or `auth=...`.
pub fn scrub(text: &str) -> String {
    const WORDS: [&str; 7] = [
        "password", "passwd", "pass", "token", "secret", "key", "auth",
    ];
    let mut out = String::with_capacity(text.len());
    for (n, word) in text.split(' ').enumerate() {
        if n > 0 {
            out.push(' ');
        }
        let lower = word.to_ascii_lowercase();
        if let Some(at) = word.find("://") {
            // the address: without the user and the password, and without the part after the question mark
            let (scheme, rest) = word.split_at(at + 3);
            let rest = rest.split('?').next().unwrap_or(rest);
            let rest = match rest.rfind('@') {
                Some(i) => &rest[i + 1..],
                None => rest,
            };
            out.push_str(scheme);
            out.push_str(rest);
            continue;
        }
        let hidden = WORDS.iter().find_map(|w| {
            let eq = lower.find(&format!("{w}="))?;
            Some(eq + w.len() + 1)
        });
        match hidden {
            Some(cut) if cut <= word.len() && word.is_char_boundary(cut) => {
                out.push_str(&word[..cut]);
                out.push_str("<hidden>");
            }
            _ => out.push_str(word),
        }
    }
    out
}

/// A line of the settings text for the log: the value of a key that holds a secret is hidden and a very long value is cut.
pub fn settings_line(line: &str) -> Option<String> {
    let (k, v) = line.split_once('=')?;
    let key = k.trim();
    let lower = key.to_ascii_lowercase();
    if ["pass", "secret", "token", "key", "auth", "cred", "pwd"]
        .iter()
        .any(|w| lower.contains(w))
    {
        return Some(format!("{key} = <hidden>"));
    }
    let v = v.trim();
    let cut: String = v.chars().take(160).collect();
    let more = if cut.len() < v.len() { "..." } else { "" };
    Some(format!("{key} = {}{more}", scrub(&cut)))
}

/// The lines at the end of the log of the last run, if that run did not end cleanly: the text from the last start of a run on has no mark of a clean exit.
pub fn unclean_tail(text: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.iter().rposition(|l| l.contains(SESSION_START))?;
    if lines[start..].iter().any(|l| l.contains(CLEAN_EXIT)) {
        return None;
    }
    let from = lines.len().saturating_sub(TAIL_LINES);
    Some(
        lines[from..]
            .iter()
            // the marks themselves are not copied, or the copy would be taken for a run
            .map(|l| l.replace("===", "---"))
            .collect(),
    )
}

/// The end of a file as text (at most [`TAIL_BYTES`]); an empty text when there is no file.
fn read_tail(path: &Path) -> String {
    let Ok(mut f) = File::open(path) else {
        return String::new();
    };
    let len = f.metadata().map_or(0, |m| m.len());
    if len > TAIL_BYTES && f.seek(SeekFrom::Start(len - TAIL_BYTES)).is_err() {
        return String::new();
    }
    let mut bytes = Vec::new();
    if f.read_to_end(&mut bytes).is_err() {
        return String::new();
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// The writer of the log. The program has one (see [`init`]); the tests make their own in their own folder.
pub struct Logger {
    dir: PathBuf,
    name: String,
    cfg: Config,
    version: String,
    pid: u32,
    file: Option<File>,
    size: u64,
    /// The last line (level, part and words) and how many times in a row it came.
    last: Option<(Level, String, String)>,
    repeats: u32,
    window: Option<Instant>,
    in_window: u32,
    dropped: u32,
    retry_after: Option<Instant>,
}

impl Logger {
    pub fn new(dir: &Path, name: &str, cfg: Config, version: &str) -> Logger {
        Logger {
            dir: dir.to_path_buf(),
            name: name.to_string(),
            cfg,
            version: version.to_string(),
            pid: std::process::id(),
            file: None,
            size: 0,
            last: None,
            repeats: 0,
            window: None,
            in_window: 0,
            dropped: 0,
            retry_after: None,
        }
    }

    /// The file of the log (`0`) or an older one (`1` is the last one that was closed).
    pub fn path(&self, n: u32) -> PathBuf {
        if n == 0 {
            self.dir.join(&self.name)
        } else {
            self.dir.join(format!("{}.{n}", self.name))
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// Writes a line if the log is on and the level is wanted. The same line again and again is written a few times and then counted; more lines a
    /// second than the limit are counted too (an error always gets in).
    pub fn log(&mut self, level: Level, target: &str, msg: &str) {
        if !self.cfg.enabled || level > self.cfg.level {
            return;
        }
        self.log_forced(level, target, msg);
    }

    /// Like [`Logger::log`] without looking at the level (the marks of the start and the end of a run must be there whatever the level is).
    pub fn log_forced(&mut self, level: Level, target: &str, msg: &str) {
        if !self.cfg.enabled {
            return;
        }
        let msg = scrub(msg);
        let same = self
            .last
            .as_ref()
            .is_some_and(|(l, t, m)| *l == level && t == target && *m == msg);
        if same {
            self.repeats += 1;
            if self.repeats > REPEAT_FREE {
                return;
            }
        } else {
            self.flush_repeats();
            self.last = Some((level, target.to_string(), msg.clone()));
            self.repeats = 1;
        }
        if !self.room_in_second(level) {
            return;
        }
        self.write_text(level, target, &msg);
    }

    /// Writes the count of the lines that were left out, if there are any.
    fn flush_repeats(&mut self) {
        if self.repeats > REPEAT_FREE {
            let n = self.repeats - REPEAT_FREE;
            if let Some((level, target, _)) = self.last.clone() {
                self.write_text(
                    level,
                    &target,
                    &format!("the last line came {n} more times"),
                );
            }
        }
        self.repeats = 0;
    }

    /// Whether one more line fits in the second that is running; at the start of the next second the lines that were left out are counted.
    fn room_in_second(&mut self, level: Level) -> bool {
        let now = Instant::now();
        match self.window {
            Some(t) if now.duration_since(t) < Duration::from_secs(1) => {}
            _ => {
                self.window = Some(now);
                self.in_window = 0;
                if self.dropped > 0 {
                    let n = self.dropped;
                    self.dropped = 0;
                    self.write_text(
                        Level::Warn,
                        "log",
                        &format!("{n} lines left out: more than {MAX_LINES_PER_SECOND} a second"),
                    );
                }
            }
        }
        self.in_window += 1;
        if self.in_window > MAX_LINES_PER_SECOND && level != Level::Error {
            self.dropped += 1;
            return false;
        }
        true
    }

    fn write_text(&mut self, level: Level, target: &str, msg: &str) {
        let (local, secs) = stamp();
        let line = format_line(&local, secs, &self.version, self.pid, level, target, msg);
        self.write_raw(line.as_bytes());
    }

    /// Puts bytes at the end of the file, closing the full one first. Any mistake leaves the log closed; the next line tries again, but not before a few seconds.
    fn write_raw(&mut self, bytes: &[u8]) {
        let max = u64::from(self.cfg.max_kb) * 1024;
        if self.file.is_some() && self.size > 0 && self.size + bytes.len() as u64 > max {
            self.rotate();
        }
        if self.file.is_none() && !self.open() {
            return;
        }
        let Some(f) = self.file.as_mut() else {
            return;
        };
        // File is not buffered: the line is handed to the system now
        if f.write_all(bytes).is_err() {
            self.file = None;
            self.retry_after = Some(Instant::now() + Duration::from_secs(5));
            return;
        }
        let _ = f.flush();
        self.size += bytes.len() as u64;
    }

    fn open(&mut self) -> bool {
        if self.retry_after.is_some_and(|t| Instant::now() < t) {
            return false;
        }
        let path = self.path(0);
        let _ = std::fs::create_dir_all(&self.dir);
        match OpenOptions::new().create(true).append(true).open(&path) {
            Ok(f) => {
                self.size = f.metadata().map_or(0, |m| m.len());
                self.file = Some(f);
                self.retry_after = None;
                true
            }
            Err(_) => {
                self.retry_after = Some(Instant::now() + Duration::from_secs(5));
                false
            }
        }
    }

    /// Closes the file and shifts the older ones: `.log` becomes `.log.1`, `.log.1` becomes `.log.2`, and the oldest of the kept ones is lost.
    fn rotate(&mut self) {
        self.file = None;
        let files = self.cfg.files.max(1);
        for i in (1..files).rev() {
            let _ = std::fs::rename(self.path(i - 1), self.path(i));
        }
        // with a single file, or if the shift failed, the full one is emptied so that the log cannot grow without limit
        if self.path(0).exists() {
            let _ = std::fs::remove_file(self.path(0));
        }
        self.size = 0;
    }

    /// Writes what is pending.
    pub fn finish(&mut self) {
        self.flush_repeats();
        if let Some(f) = self.file.as_mut() {
            let _ = f.flush();
        }
    }

    /// The end of the last run, if it did not end cleanly (read before the first line of this run is written).
    pub fn previous_unclean(&self) -> Option<Vec<String>> {
        unclean_tail(&read_tail(&self.path(0)))
    }
}

struct Global {
    logger: Mutex<Logger>,
    /// The first file of the log, for the lines written without taking the lock (a panic in the middle of a line, a crash of the system).
    path: PathBuf,
    version: String,
}

static GLOBAL: OnceLock<Global> = OnceLock::new();
static THREADS: AtomicI32 = AtomicI32::new(0);

/// Starts the log of the program: opens it, notes the end of the last run if it was not a clean one, and writes the mark of the start of this one.
/// Without this call every function of the module does nothing.
pub fn init(dir: &Path, cfg: Config, version: &str) {
    if !cfg.enabled || GLOBAL.get().is_some() {
        return;
    }
    let mut logger = Logger::new(dir, FILE_NAME, cfg, version);
    let unclean = logger.previous_unclean();
    let path = logger.path(0);
    let args: Vec<String> = std::env::args().skip(1).collect();
    logger.log_forced(
        Level::Info,
        "run",
        &format!(
            "{SESSION_START} version {version} os {} {} pid {} args {:?}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            std::process::id(),
            args
        ),
    );
    if let Some(tail) = unclean {
        logger.log_forced(
            Level::Warn,
            "run",
            "previous run ended without a clean exit; the end of its log follows",
        );
        for l in tail {
            logger.log_forced(Level::Warn, "previous", &l);
        }
    }
    let _ = GLOBAL.set(Global {
        logger: Mutex::new(logger),
        path,
        version: version.to_string(),
    });
}

/// Whether the log of the program is on.
pub fn active() -> bool {
    GLOBAL.get().is_some()
}

fn with_logger(f: impl FnOnce(&mut Logger)) {
    if let Some(g) = GLOBAL.get() {
        // a panic while a line was being written must not close the log for the rest of the program
        let mut l = g.logger.lock().unwrap_or_else(|e| e.into_inner());
        f(&mut l);
    }
}

pub fn log(level: Level, target: &str, msg: impl std::fmt::Display) {
    if GLOBAL.get().is_some() {
        let text = msg.to_string();
        with_logger(|l| l.log(level, target, &text));
    }
}

pub fn error(target: &str, msg: impl std::fmt::Display) {
    log(Level::Error, target, msg);
}

pub fn warn(target: &str, msg: impl std::fmt::Display) {
    log(Level::Warn, target, msg);
}

pub fn info(target: &str, msg: impl std::fmt::Display) {
    log(Level::Info, target, msg);
}

pub fn debug(target: &str, msg: impl std::fmt::Display) {
    log(Level::Debug, target, msg);
}

/// The last line of a run that ends by itself.
pub fn clean_exit() {
    if GLOBAL.get().is_some() {
        with_logger(|l| {
            l.log_forced(Level::Info, "run", CLEAN_EXIT);
            l.finish();
        });
    }
}

/// A line written without waiting for the lock and without the limits: for a panic or a crash, when the lock may be held by the very thread that is dying.
pub fn emergency(level: Level, target: &str, msg: &str) {
    let Some(g) = GLOBAL.get() else {
        return;
    };
    let text = scrub(msg);
    if let Ok(mut l) = g.logger.try_lock() {
        l.log_forced(level, target, &text);
        l.finish();
        return;
    }
    let (local, secs) = stamp();
    let line = format_line(
        &local,
        secs,
        &g.version,
        std::process::id(),
        level,
        target,
        &text,
    );
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&g.path) {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
}

/// Logs the settings read at the start, a line each, with the secrets hidden.
pub fn log_settings(text: &str) {
    for line in text.lines().filter_map(settings_line) {
        debug("settings", line);
    }
}

/// A background thread that is alive: the guard counts it while it lives and logs how it ended (a thread that dies by a panic is an error).
pub struct ThreadGuard {
    name: &'static str,
}

pub fn thread_guard(name: &'static str) -> ThreadGuard {
    THREADS.fetch_add(1, Ordering::SeqCst);
    debug("thread", format!("{name} started"));
    ThreadGuard { name }
}

impl Drop for ThreadGuard {
    fn drop(&mut self) {
        THREADS.fetch_sub(1, Ordering::SeqCst);
        if std::thread::panicking() {
            error("thread", format!("{} ended by a panic", self.name));
        } else {
            debug("thread", format!("{} ended", self.name));
        }
    }
}

/// How many background threads of the program are alive.
pub fn live_threads() -> i32 {
    THREADS.load(Ordering::SeqCst)
}

/// From now on a panic is also written to the log, with the thread, the place, the words and the backtrace, before the hook that was there goes on.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let place = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or_default();
        let words = if let Some(s) = info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown".to_string()
        };
        let backtrace = std::backtrace::Backtrace::force_capture();
        emergency(
            Level::Error,
            "panic",
            &format!(
                "thread '{}' panicked at {place}: {words}\n{backtrace}",
                thread.name().unwrap_or("unnamed")
            ),
        );
        previous(info);
    }));
}

/// How the time between frames runs: the heartbeat that is due, and a pause that is too long.
pub struct Heartbeat {
    every: Duration,
    last_beat: Instant,
    last_frame: Option<Instant>,
    total: u64,
    frames: u64,
    sum_us: u128,
    max_us: u128,
}

/// What a heartbeat tells of the frames since the last one.
#[derive(Debug, Clone, PartialEq)]
pub struct Beat {
    pub total_frames: u64,
    pub frames: u64,
    pub mean_ms: f64,
    pub max_ms: f64,
}

impl Heartbeat {
    pub fn new(every_s: u32, now: Instant) -> Heartbeat {
        Heartbeat {
            every: Duration::from_secs(u64::from(every_s.max(1))),
            last_beat: now,
            last_frame: None,
            total: 0,
            frames: 0,
            sum_us: 0,
            max_us: 0,
        }
    }

    /// A frame was drawn, and drawing it took `work`. Returns the heartbeat if it is due, and the pause since the frame before if it was long.
    pub fn frame(&mut self, now: Instant, work: Duration) -> (Option<Beat>, Option<Duration>) {
        let mut gap = None;
        if let Some(prev) = self.last_frame {
            if now.saturating_duration_since(prev) >= Duration::from_millis(FRAME_GAP_MS) {
                gap = Some(now.saturating_duration_since(prev));
            }
        }
        self.sum_us += work.as_micros();
        self.max_us = self.max_us.max(work.as_micros());
        self.last_frame = Some(now);
        self.total += 1;
        self.frames += 1;
        if now.saturating_duration_since(self.last_beat) < self.every {
            return (None, gap);
        }
        let mean_us = self.sum_us.checked_div(self.frames as u128).unwrap_or(0);
        let beat = Beat {
            total_frames: self.total,
            frames: self.frames,
            mean_ms: mean_us as f64 / 1000.0,
            max_ms: self.max_us as f64 / 1000.0,
        };
        self.last_beat = now;
        self.frames = 0;
        self.sum_us = 0;
        self.max_us = 0;
        (Some(beat), gap)
    }
}

/// The memory of the process and of the system, as one line for the log.
pub fn memory_line() -> String {
    memory::line()
}

/// The value of a line `Key:   1234 kB` of `/proc/self/status` or `/proc/meminfo`, in kB.
pub fn proc_kb(text: &str, key: &str) -> Option<u64> {
    text.lines()
        .find_map(|l| l.strip_prefix(key)?.strip_prefix(':'))
        .and_then(|v| v.split_whitespace().next())
        .and_then(|n| n.parse().ok())
}

#[cfg(windows)]
mod memory {
    use std::ffi::c_void;

    #[repr(C)]
    #[allow(dead_code)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged_pool: usize,
        quota_paged_pool: usize,
        quota_peak_non_paged_pool: usize,
        quota_non_paged_pool: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }

    #[repr(C)]
    #[allow(dead_code)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }

    #[repr(C)]
    #[allow(dead_code)]
    pub struct ExceptionRecord {
        pub code: u32,
        pub flags: u32,
        pub record: *const ExceptionRecord,
        pub address: *const c_void,
        pub number_parameters: u32,
        pub information: [usize; 15],
    }

    #[repr(C)]
    #[allow(dead_code)]
    pub struct ExceptionPointers {
        pub record: *const ExceptionRecord,
        pub context: *const c_void,
    }

    pub type Filter = extern "system" fn(*const ExceptionPointers) -> i32;

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(
            process: isize,
            counters: *mut ProcessMemoryCounters,
            cb: u32,
        ) -> i32;
        fn GlobalMemoryStatusEx(status: *mut MemoryStatusEx) -> i32;
        pub fn SetUnhandledExceptionFilter(filter: Option<Filter>) -> Option<Filter>;
    }

    const MIB: usize = 1024 * 1024;

    pub fn line() -> String {
        let mut text = String::new();
        let mut c = ProcessMemoryCounters {
            cb: std::mem::size_of::<ProcessMemoryCounters>() as u32,
            page_fault_count: 0,
            peak_working_set: 0,
            working_set: 0,
            quota_peak_paged_pool: 0,
            quota_paged_pool: 0,
            quota_peak_non_paged_pool: 0,
            quota_non_paged_pool: 0,
            pagefile: 0,
            peak_pagefile: 0,
        };
        // SAFETY: the structure has the layout of PROCESS_MEMORY_COUNTERS and its size is given; the call only fills it.
        let cb = c.cb;
        let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, cb) };
        if ok != 0 {
            text.push_str(&format!(
                "process: working set {} MiB (peak {}), commit {} MiB (peak {}), page faults {}",
                c.working_set / MIB,
                c.peak_working_set / MIB,
                c.pagefile / MIB,
                c.peak_pagefile / MIB,
                c.page_fault_count
            ));
        } else {
            text.push_str("process: memory unknown");
        }
        let mut s = MemoryStatusEx {
            length: std::mem::size_of::<MemoryStatusEx>() as u32,
            memory_load: 0,
            total_phys: 0,
            avail_phys: 0,
            total_page_file: 0,
            avail_page_file: 0,
            total_virtual: 0,
            avail_virtual: 0,
            avail_extended_virtual: 0,
        };
        // SAFETY: the structure has the layout of MEMORYSTATUSEX and its length is set; the call only fills it.
        if unsafe { GlobalMemoryStatusEx(&mut s) } != 0 {
            let mib = 1024 * 1024u64;
            text.push_str(&format!(
                "; system: {}% in use, {} of {} MiB free, commit {} of {} MiB free, address space {} MiB free",
                s.memory_load,
                s.avail_phys / mib,
                s.total_phys / mib,
                s.avail_page_file / mib,
                s.total_page_file / mib,
                s.avail_virtual / mib
            ));
        }
        text
    }
}

#[cfg(target_os = "linux")]
mod memory {
    pub fn line() -> String {
        let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
        let info = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        let mib = |kb: Option<u64>| kb.map_or_else(|| "?".to_string(), |k| (k / 1024).to_string());
        format!(
            "process: resident {} MiB (peak {}), virtual {} MiB; system: {} of {} MiB available",
            mib(super::proc_kb(&status, "VmRSS")),
            mib(super::proc_kb(&status, "VmHWM")),
            mib(super::proc_kb(&status, "VmSize")),
            mib(super::proc_kb(&info, "MemAvailable")),
            mib(super::proc_kb(&info, "MemTotal"))
        )
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod memory {
    pub fn line() -> String {
        "memory not available on this system".to_string()
    }
}

/// On Windows: what the system calls an unhandled exception (an access violation, a stack overflow, an illegal instruction) is written to the log, with
/// its code and the address, before the process dies. An abort of the program itself (a failed allocation, `abort()`) is a fail-fast of the system that
/// does not pass through here; the heartbeat and the mark of a clean exit are what show it.
#[cfg(windows)]
pub fn install_exception_filter() {
    use memory::{ExceptionPointers, Filter};

    extern "system" fn filter(info: *const ExceptionPointers) -> i32 {
        // SAFETY: the system gives a pointer to a record that is valid during the call; both pointers are checked for null.
        unsafe {
            if !info.is_null() && !(*info).record.is_null() {
                let r = &*(*info).record;
                emergency(
                    Level::Error,
                    "crash",
                    &format!(
                        "unhandled exception code {:#010x} at address {:p} (flags {:#x}, {} parameters); thread '{}'",
                        r.code,
                        r.address,
                        r.flags,
                        r.number_parameters,
                        std::thread::current().name().unwrap_or("unnamed")
                    ),
                );
            }
        }
        // EXCEPTION_CONTINUE_SEARCH: the system goes on with its own report
        0
    }
    let f: Filter = filter;
    // SAFETY: the function has the signature the system expects and lives for the whole run.
    unsafe {
        memory::SetUnhandledExceptionFilter(Some(f));
    }
}

#[cfg(not(windows))]
pub fn install_exception_filter() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("eh3d-applog-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn cfg() -> Config {
        Config::default()
    }

    #[test]
    fn a_line_has_the_time_the_seconds_the_version_the_process_the_level_and_the_part() {
        let l = format_line(
            "2000-01-02 03:04:05.006",
            946_782_245,
            "1.2.3",
            77,
            Level::Warn,
            "job",
            "one\ntwo",
        );
        assert_eq!(
            l,
            "2000-01-02 03:04:05.006 946782245 v1.2.3 p77 WARN  [job] one\n    two\n"
        );
    }

    #[test]
    fn the_settings_of_the_log_have_limits_and_come_back() {
        let mut c = cfg();
        assert!(c.set("log_level", "debug"));
        assert!(c.set("log_max_kb", "256"));
        assert!(c.set("log_files", "5"));
        assert!(c.set("log_heartbeat_s", "10"));
        assert!(c.set("log", "no"));
        assert!(!c.set("theme", "forge"));
        assert_eq!(c.level, Level::Debug);
        assert!(!c.enabled);
        let mut back = cfg();
        for line in c.to_text().lines() {
            let (k, v) = line.split_once('=').unwrap();
            back.set(k.trim(), v);
        }
        assert_eq!(back, c);
        // out of range, or not a word of the list: what was there stays
        c.set("log_max_kb", "1");
        c.set("log_files", "0");
        c.set("log_heartbeat_s", "999999");
        c.set("log_level", "loud");
        assert_eq!(
            (c.max_kb, c.files, c.heartbeat_s, c.level),
            (256, 5, 10, Level::Debug)
        );
    }

    #[test]
    fn the_secrets_never_get_in() {
        assert_eq!(
            scrub("open https://user:pw@host.example/a/b?token=abc&x=1 now"),
            "open https://host.example/a/b now"
        );
        assert_eq!(
            scrub("call with password=hunter2 done"),
            "call with password=<hidden> done"
        );
        assert_eq!(scrub("an ordinary line"), "an ordinary line");
        assert_eq!(
            settings_line("api_token = abc").unwrap(),
            "api_token = <hidden>"
        );
        assert_eq!(settings_line("theme = forge").unwrap(), "theme = forge");
        assert!(settings_line("no equals sign").is_none());
        let long = format!("recent = {}", "x".repeat(500));
        assert!(settings_line(&long).unwrap().len() < 200);
    }

    #[test]
    fn lines_are_written_at_once_and_the_level_filters() {
        let d = scratch("write");
        let mut l = Logger::new(&d, "t.log", cfg(), "9.9.9");
        l.log(Level::Info, "a", "first");
        l.log(Level::Debug, "a", "too detailed");
        l.log(Level::Error, "a", "second");
        // nothing was finished or dropped: the lines are already in the file
        let text = std::fs::read_to_string(d.join("t.log")).unwrap();
        assert!(text.contains("[a] first") && text.contains("[a] second"));
        assert!(!text.contains("too detailed"));
        assert!(text.contains("v9.9.9"));
        let mut off = Logger::new(
            &d,
            "off.log",
            Config {
                enabled: false,
                ..cfg()
            },
            "1",
        );
        off.log(Level::Error, "a", "never");
        assert!(!d.join("off.log").exists());
    }

    #[test]
    fn the_same_line_again_and_again_is_counted() {
        let d = scratch("repeat");
        let mut l = Logger::new(&d, "t.log", cfg(), "1");
        for _ in 0..10 {
            l.log(Level::Warn, "net", "no answer");
        }
        l.log(Level::Warn, "net", "other");
        let text = std::fs::read_to_string(d.join("t.log")).unwrap();
        assert_eq!(text.matches("no answer").count(), REPEAT_FREE as usize);
        assert!(text.contains("the last line came 7 more times"));
        assert!(text.contains("other"));
    }

    #[test]
    fn too_many_lines_in_a_second_are_counted_but_errors_get_in() {
        let d = scratch("rate");
        let mut l = Logger::new(&d, "t.log", cfg(), "1");
        for i in 0..300 {
            l.log(Level::Info, "x", &format!("line {i}"));
        }
        l.log(Level::Error, "x", "an error");
        let text = std::fs::read_to_string(d.join("t.log")).unwrap();
        assert!(text.matches("] line ").count() <= MAX_LINES_PER_SECOND as usize);
        assert!(text.contains("an error"));
    }

    #[test]
    fn a_full_file_is_closed_and_the_older_ones_are_shifted() {
        let d = scratch("rotate");
        let c = Config {
            max_kb: 64,
            files: 3,
            ..cfg()
        };
        let mut l = Logger::new(&d, "t.log", c, "1");
        let big = "y".repeat(900);
        for i in 0..400 {
            // every line is different, and the limit of lines a second is dodged by the error level
            l.log(Level::Error, "fill", &format!("{i} {big}"));
        }
        l.finish();
        assert!(d.join("t.log").exists());
        assert!(d.join("t.log.1").exists());
        assert!(d.join("t.log.2").exists());
        assert!(!d.join("t.log.3").exists());
        for n in 0..3 {
            assert!(std::fs::metadata(l.path(n)).unwrap().len() <= 64 * 1024 + 1000);
        }
        // a single file never grows past its limit either
        let one = Config {
            max_kb: 64,
            files: 1,
            ..cfg()
        };
        let mut s = Logger::new(&d, "one.log", one, "1");
        for i in 0..400 {
            s.log(Level::Error, "fill", &format!("{i} {big}"));
        }
        assert!(std::fs::metadata(d.join("one.log")).unwrap().len() <= 64 * 1024 + 1000);
        assert!(!d.join("one.log.1").exists());
    }

    #[test]
    fn a_run_without_the_mark_of_a_clean_exit_is_found() {
        let start = format!("t INFO [run] {SESSION_START} version 1");
        let ok = format!("{start}\nt INFO [x] work\nt INFO [run] {CLEAN_EXIT}\n");
        assert!(unclean_tail(&ok).is_none());
        assert!(unclean_tail("").is_none());
        assert!(unclean_tail("a line from nowhere\n").is_none());
        let bad = format!("{start}\nt INFO [x] work\nt INFO [x] the last thing\n");
        let tail = unclean_tail(&bad).unwrap();
        assert!(tail.last().unwrap().contains("the last thing"));
        assert!(tail.iter().all(|l| !l.contains("===")));
        // the first run was clean and the second was not
        let two = format!("{ok}{start}\nt INFO [x] more\n");
        assert!(unclean_tail(&two).is_some());
        // many lines: only the end is copied
        let mut long = format!("{start}\n");
        for i in 0..200 {
            long.push_str(&format!("t INFO [x] line {i}\n"));
        }
        assert_eq!(unclean_tail(&long).unwrap().len(), TAIL_LINES);
    }

    #[test]
    fn the_logger_finds_the_end_of_the_run_before_it_in_its_own_file() {
        let d = scratch("previous");
        {
            let mut first = Logger::new(&d, FILE_NAME, cfg(), "1");
            first.log_forced(Level::Info, "run", &format!("{SESSION_START} version 1"));
            first.log(Level::Info, "x", "doing something");
        }
        let second = Logger::new(&d, FILE_NAME, cfg(), "1");
        let tail = second.previous_unclean().unwrap();
        assert!(tail.iter().any(|l| l.contains("doing something")));
        let mut third = Logger::new(&d, FILE_NAME, cfg(), "1");
        third.log_forced(Level::Info, "run", CLEAN_EXIT);
        assert!(Logger::new(&d, FILE_NAME, cfg(), "1")
            .previous_unclean()
            .is_none());
    }

    #[test]
    fn a_log_that_cannot_be_written_does_not_panic() {
        let d = scratch("blocked");
        // the folder of the log is a file: nothing can be created in it
        let file = d.join("not-a-folder");
        std::fs::write(&file, b"x").unwrap();
        let mut l = Logger::new(&file, "t.log", cfg(), "1");
        for i in 0..5 {
            l.log(Level::Error, "a", &format!("line {i}"));
        }
        l.finish();
        assert!(l.previous_unclean().is_none());
        // the global functions do nothing before the log is started
        info("a", "nothing");
        clean_exit();
    }

    #[test]
    fn the_heartbeat_comes_when_due_and_a_long_pause_is_told() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut h = Heartbeat::new(30, t0);
        let (b, g) = h.frame(t0, ms(4));
        assert!(b.is_none() && g.is_none());
        let (b, g) = h.frame(t0 + ms(100), ms(6));
        assert!(b.is_none() && g.is_none());
        let (b, g) = h.frame(t0 + ms(4000), ms(50));
        assert!(b.is_none());
        assert_eq!(g, Some(ms(3900)));
        let (b, _) = h.frame(t0 + Duration::from_secs(31), ms(20));
        let b = b.unwrap();
        assert_eq!(b.total_frames, 4);
        assert_eq!(b.frames, 4);
        assert_eq!(b.max_ms, 50.0);
        assert_eq!(b.mean_ms, 20.0);
        // the counts start again
        let (b, _) = h.frame(t0 + Duration::from_secs(62), ms(1));
        let b = b.unwrap();
        assert_eq!((b.frames, b.max_ms), (1, 1.0));
    }

    #[test]
    fn the_memory_of_a_proc_file_is_read() {
        let text = "Name:\tx\nVmPeak:\t  900 kB\nVmRSS:\t  2048 kB\n";
        assert_eq!(proc_kb(text, "VmRSS"), Some(2048));
        assert_eq!(proc_kb(text, "VmPeak"), Some(900));
        assert_eq!(proc_kb(text, "VmHWM"), None);
        assert!(!memory_line().is_empty());
    }
}
