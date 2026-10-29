# EH3D-BUNGARD-GUI

The modern interface for the Bungard CCD machines. One window with a launcher (the modern interface, or the classic drilling and routing programs for a machine with manual or automatic tool change) and a viewer that shows the job on the table: every hole by drill size, the routing paths over them, the machine in 3D with the table of the size chosen in the configuration, and the progress of the job as it runs.

**Honesty check - what actually runs today:** a Rust program with a window and a test suite that runs on a computer. The launcher, the plan view, the isometric view of the machine and the live progress of a job work; the job runs on the **simulator** of the engines of EH3D-DRILLPRO and EH3D-ROUTEPRO, and the same machine file they read decides how a real machine is reached (the computer's RS-232, a USB-RS232 adapter, or the EH3D node directly or through the EH3D server) - **none of those has been tried from this window**. Five themes (Forge, Neon Arcade, Glass Pro, the classic Windows grey and one that follows the colours of Windows) and eight languages are in. The classic programs are started from the launcher when they are next to it. There is no editing of the machine and tool files, no connection screen, no 3D view with the real tool paths and no pause yet.

## Project status

| Area | State | What it means here |
|---|---|---|
| Launcher and viewer | 🟡 | A window that runs on a computer; looked at with pictures of the simulated job. |
| Machine connection | 🔴 | Reached through the engines of the classic programs; never tried from this window. |
| Configuration screens | 🔴 | Not started: the machine file is edited with the classic programs. |

## Run

```
cargo run --release --features gui -- --drill FILE.NCD --route FILE.PLT
```

`--connection FILE` points to the machine file the classic programs read; without one the job runs on the simulator. `--theme forge|neon|glass|windows-classic|system` and `--lang en|es|it|de|fr|pt|zh|ja` choose the look and the language; `--screen launcher|viewer` the first screen. The window needs the sibling folders `EH3D-DRILLPRO`, `EH3D-ROUTEPRO` and `EH3D-COMMON` to build (until their shared code is split out).

## Test

`cargo test` runs the tests of the launcher, the settings, the reading of the files into the scene and the command line. `build.bat` / `build.sh` build the window and `test.bat` / `test.sh` run the tests.

## Licence

GPL-3.0-or-later. See `LICENSE`.
