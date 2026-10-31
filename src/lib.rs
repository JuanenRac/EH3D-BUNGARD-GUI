//! The modern interface for the Bungard machines. The logic that does not need a window lives here, with tests: what the launcher offers
//! and how it starts the classic programs, what is remembered between runs, how a drilling or a routing file becomes the scene the views
//! draw, and the command line. The window itself is in `gui` (feature `gui`). The machine is driven by the same engines and the same
//! machine file as the classic programs, so every connection they have (the computer's RS-232, a USB-RS232 adapter, the EH3D node
//! directly or through the EH3D server) works here too.

pub mod boards;
pub mod catalogue;
pub mod cli;
pub mod defaults;
pub mod inspection;
pub mod jobs;
pub mod launcher;
pub mod link;
pub mod machines;
pub mod recovery;

#[cfg(feature = "gui")]
pub mod gui;
