//! The modern interface for the Bungard machines.
//!
//! Usage: `bungard-gui [--drill FILE.NCD] [--route FILE.PLT] [--lang CODE] [--theme NAME] [--data DIR] [--connection FILE]`
//! The connection is the machine file the classic programs read (the computer's RS-232, a USB-RS232 adapter, the EH3D node directly or
//! through the EH3D server); without one the job runs on the simulator.

#![cfg_attr(windows, windows_subsystem = "windows")]

use eh3d_bungard_gui::cli::Options;

fn main() -> eframe::Result {
    eh3d_drillpro::crash::install("bungard-gui", env!("CARGO_PKG_VERSION"));
    attach_console();
    start_log();
    match Options::parse(std::env::args().skip(1)) {
        Ok(o) => {
            let result = eh3d_bungard_gui::gui::run(&o);
            if let Err(e) = &result {
                eh3d_bungard_gui::applog::error(
                    "run",
                    format!("the window loop ended with an error: {e}"),
                );
            }
            // the window closed by itself (or with a reported error): the next run must not take this one for a crash
            eh3d_bungard_gui::applog::clean_exit();
            result
        }
        Err(e) => {
            eh3d_bungard_gui::applog::error("run", format!("bad command line: {e}"));
            eh3d_bungard_gui::applog::clean_exit();
            eprintln!("bungard-gui: {e}");
            std::process::exit(2);
        }
    }
}

/// Starts the log (`bungard-gui.log` in the settings folder) with the settings it asks for, the hooks that write a panic or a crash of the system to it, and
/// what the run starts with. A picture run (`--shot`) leaves it alone: it would take the place of the run of the person in the record.
fn start_log() {
    use eh3d_bungard_gui::{applog, launcher};
    if std::env::args().any(|a| a == "--shot") {
        return;
    }
    let settings_path = launcher::settings_path();
    let settings = launcher::Settings::load(&settings_path);
    let dir = applog::default_dir();
    applog::init(&dir, settings.log.clone(), env!("CARGO_PKG_VERSION"));
    if !applog::active() {
        return;
    }
    applog::install_panic_hook();
    applog::install_exception_filter();
    applog::info(
        "run",
        format!(
            "settings file {} ({}), log folder {}, working folder {}, program {}, logical processors {}",
            settings_path.display(),
            if settings_path.exists() { "found" } else { "not found: defaults" },
            dir.display(),
            std::env::current_dir().map_or_else(|e| format!("unknown ({e})"), |d| d.display().to_string()),
            std::env::current_exe().map_or_else(|e| format!("unknown ({e})"), |d| d.display().to_string()),
            std::thread::available_parallelism().map_or(0, |n| n.get())
        ),
    );
    applog::info(
        "run",
        format!(
            "log level {}, heartbeat every {} s, files of up to {} KiB, {} kept",
            settings.log.level.key(),
            settings.log.heartbeat_s,
            settings.log.max_kb,
            settings.log.files
        ),
    );
    applog::info("memory", applog::memory_line());
    applog::info(
        "run",
        format!(
            "clock of the system: offset {} from universal time",
            chrono::Local::now().offset()
        ),
    );
    applog::log_settings(&settings.to_text());
}

/// A program with a window has no console of its own on Windows (no black window); when it was started from a terminal, what it prints (the help, a mistake in the options) goes to that terminal.
#[cfg(windows)]
fn attach_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
    // SAFETY: a call of the system with a constant; if there is no parent console it does nothing.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_console() {}
