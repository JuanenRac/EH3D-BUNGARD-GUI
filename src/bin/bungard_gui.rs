//! The modern interface for the Bungard machines.
//!
//! Usage: `bungard-gui [--drill FILE.NCD] [--route FILE.PLT] [--lang CODE] [--theme NAME] [--data DIR] [--connection FILE]`
//! The connection is the machine file the classic programs read (the computer's RS-232, a USB-RS232 adapter, the EH3D node directly or
//! through the EH3D server); without one the job runs on the simulator.

use eh3d_bungard_gui::cli::Options;

fn main() -> eframe::Result {
    match Options::parse(std::env::args().skip(1)) {
        Ok(o) => eh3d_bungard_gui::gui::run(&o),
        Err(e) => {
            eprintln!("bungard-gui: {e}");
            std::process::exit(2);
        }
    }
}
