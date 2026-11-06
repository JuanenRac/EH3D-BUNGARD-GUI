//! The log section of the settings window: whether the log (`bungard-gui.log`, see `applog`) is written, how detailed it is, how big a file may grow, and two
//! buttons, one to open the folder of the log and one to copy the path of its file. What is chosen is kept in the settings file (`log`, `log_level`, `log_max_kb`);
//! the log is opened once when the program starts, so a change applies the next time it starts.

use super::*;
use crate::applog::{self, Level};

/// The sizes of a file of the log that the buttons offer, in KiB (the settings file takes any value between 64 and 65536).
pub(crate) const LOG_SIZES_KB: [u32; 4] = [256, 1024, 4096, 16_384];

/// The levels in the order of the buttons, from the least to the most detailed.
const LOG_LEVELS: [Level; 4] = [Level::Error, Level::Warn, Level::Info, Level::Debug];

/// The size of a file as the button says it.
pub(crate) fn size_label(kb: u32) -> String {
    let mb = kb / 1024;
    if mb >= 1 && mb * 1024 == kb {
        format!("{mb} MB")
    } else {
        format!("{kb} KB")
    }
}

/// Opens the folder in the file manager of the system (it is made first when the log has not been written yet).
fn open_dir(dir: &Path) -> std::io::Result<()> {
    // the tests never open a window of the system
    if cfg!(test) {
        return Ok(());
    }
    std::fs::create_dir_all(dir)?;
    let program = if cfg!(windows) {
        "explorer"
    } else if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    std::process::Command::new(program)
        .arg(dir)
        .spawn()
        .map(|_| ())
}

impl App {
    /// The file the log is written to.
    pub(crate) fn log_file_path(&self) -> PathBuf {
        applog::default_dir().join(applog::FILE_NAME)
    }

    pub(crate) fn log_toggle(&mut self) {
        self.settings.log.enabled = !self.settings.log.enabled;
        applog::info(
            "settings",
            format!("log written: {}", self.settings.log.enabled),
        );
        self.remember_settings();
    }

    pub(crate) fn log_set_level(&mut self, level: Level) {
        self.settings.log.set("log_level", level.key());
        applog::info("settings", format!("log level: {}", level.key()));
        self.remember_settings();
    }

    /// The largest size of a file; a value out of the range the settings file accepts leaves the one there was.
    pub(crate) fn log_set_size(&mut self, kb: u32) {
        self.settings.log.set("log_max_kb", &kb.to_string());
        applog::info(
            "settings",
            format!("log file size: {} KiB", self.settings.log.max_kb),
        );
        self.remember_settings();
    }

    pub(crate) fn log_open_folder(&mut self) {
        let dir = applog::default_dir();
        if let Err(e) = open_dir(&dir) {
            applog::warn("settings", format!("cannot open the log folder: {e}"));
            self.message = Some(format!("{}: {e}", dir.display()));
        }
    }

    /// Puts the path of the log file on the clipboard (the window does it at the start of the next frame) and says so.
    pub(crate) fn log_copy_path(&mut self) {
        let path = self.log_file_path().display().to_string();
        self.message = Some(format!("{}: {path}", self.tr(Text::LogCopied)));
        self.pending_copy = Some(path);
    }

    /// Draws the section from `y` down: the switch, the level, the size, the two buttons and the note.
    pub(super) fn paint_log_section(&self, cv: &mut Cv, x: f32, y: f32) {
        let t = cv.t.clone();
        let log = &self.settings.log;
        cv.text(
            x + 30.0,
            y,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LogTitle).to_uppercase(),
            12.0,
            col(t.dim),
        );
        cv.checkbox(
            x + 30.0,
            y + 8.0,
            780.0,
            self.tr(Text::LogWrite),
            log.enabled,
            Act::LogToggle,
        );
        cv.text(
            x + 30.0,
            y + 66.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LogDetail).to_uppercase(),
            12.0,
            col(t.dim),
        );
        for (i, l) in LOG_LEVELS.into_iter().enumerate() {
            cv.chip(
                x + 30.0 + i as f32 * 130.0,
                y + 76.0,
                122.0,
                38.0,
                &l.key().to_uppercase(),
                log.level == l,
                Act::LogLevel(l),
            );
        }
        cv.text(
            x + 30.0,
            y + 142.0,
            Align2::LEFT_BOTTOM,
            &self.tr(Text::LogMaxSize).to_uppercase(),
            12.0,
            col(t.dim),
        );
        for (i, kb) in LOG_SIZES_KB.into_iter().enumerate() {
            cv.chip(
                x + 30.0 + i as f32 * 130.0,
                y + 152.0,
                122.0,
                38.0,
                &size_label(kb),
                log.max_kb == kb,
                Act::LogSize(kb),
            );
        }
        cv.chip(
            x + 30.0,
            y + 208.0,
            380.0,
            38.0,
            self.tr(Text::LogOpenFolder),
            false,
            Act::LogOpenFolder,
        );
        cv.chip(
            x + 430.0,
            y + 208.0,
            380.0,
            38.0,
            self.tr(Text::LogCopyPath),
            false,
            Act::LogCopyPath,
        );
        cv.text_fit(
            x + 30.0,
            y + 276.0,
            Align2::LEFT_BOTTOM,
            self.tr(Text::LogNote),
            14.0,
            col(t.dim),
            780.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Options;

    fn app() -> App {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("eh3d-bg-logpane-{}-{n}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        for f in ["DRILLPRO.DCF", "DRILLPRO.DTD", "ROUTEPRO.RCF", "ROUTEPRO.RTD"] {
            std::fs::copy(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("testdata")
                    .join(f),
                dir.join(f),
            )
            .unwrap();
        }
        App::new(&Options {
            data: Some(dir),
            screen: Some(Screen::Viewer),
            ..Default::default()
        })
    }

    #[test]
    fn the_sizes_of_the_buttons_are_said_in_kilobytes_or_megabytes_and_the_settings_file_accepts_them() {
        let said: Vec<String> = LOG_SIZES_KB.iter().map(|k| size_label(*k)).collect();
        assert_eq!(said, ["256 KB", "1 MB", "4 MB", "16 MB"]);
        assert_eq!(size_label(1536), "1536 KB");
        for kb in LOG_SIZES_KB {
            let mut c = applog::Config {
                max_kb: 1,
                ..Default::default()
            };
            assert!(c.set("log_max_kb", &kb.to_string()));
            assert_eq!(c.max_kb, kb);
        }
        // the levels of the buttons are all of the levels, each once
        let keys: Vec<&str> = LOG_LEVELS.iter().map(|l| l.key()).collect();
        assert_eq!(keys, ["error", "warn", "info", "debug"]);
    }

    #[test]
    fn the_switch_the_level_and_the_size_are_kept_in_the_settings_file() {
        let mut a = app();
        assert!(a.settings.log.enabled);
        a.log_toggle();
        a.log_set_level(Level::Debug);
        a.log_set_size(4096);
        let saved = Settings::load(&a.settings_path);
        assert!(!saved.log.enabled);
        assert_eq!(saved.log.level, Level::Debug);
        assert_eq!(saved.log.max_kb, 4096);
        // the other keys of the log stay as they were
        assert_eq!(saved.log.files, applog::Config::default().files);
        a.log_toggle();
        assert!(Settings::load(&a.settings_path).log.enabled);
    }

    #[test]
    fn a_size_out_of_the_range_leaves_the_one_there_was() {
        let mut a = app();
        a.log_set_size(4096);
        a.log_set_size(1);
        assert_eq!(a.settings.log.max_kb, 4096);
        a.log_set_size(1_000_000);
        assert_eq!(a.settings.log.max_kb, 4096);
    }

    #[test]
    fn the_path_of_the_log_is_handed_to_the_clipboard_and_the_window_says_so() {
        let mut a = app();
        a.log_copy_path();
        let path = a.pending_copy.clone().unwrap();
        assert!(path.ends_with(applog::FILE_NAME), "{path}");
        let said = a.message.clone().unwrap();
        assert!(said.starts_with(a.tr(Text::LogCopied)), "{said}");
        assert!(said.ends_with(&path));
    }

    #[test]
    fn opening_the_folder_in_the_tests_opens_nothing_and_says_nothing() {
        let mut a = app();
        a.message = None;
        a.log_open_folder();
        assert!(a.message.is_none());
    }
}
