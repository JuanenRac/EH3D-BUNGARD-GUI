//! The modern interface for the Bungard machines.
//!
//! Usage: `bungard-gui [--drill FILE.NCD] [--route FILE.PLT] [--lang CODE] [--theme NAME] [--data DIR] [--connection FILE]`
//! The connection is the machine file the classic programs read (the computer's RS-232, a USB-RS232 adapter, the EH3D node directly or
//! through the EH3D server); without one the job runs on the simulator.

#![cfg_attr(windows, windows_subsystem = "windows")]

use eh3d_bungard_gui::cli::Options;

fn main() -> eframe::Result {
    attach_console();
    match Options::parse(std::env::args().skip(1)) {
        Ok(o) => eh3d_bungard_gui::gui::run(&o),
        Err(e) => {
            eprintln!("bungard-gui: {e}");
            std::process::exit(2);
        }
    }
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
