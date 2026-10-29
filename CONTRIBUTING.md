# Contributing to EH3D-BUNGARD-GUI

Thank you for looking. This program shows jobs for drilling and routing machines and, one day, starts them, so changes are small and carefully tested.

1. **Nothing moves a real machine without proof.** The window drives the machine through the engines of EH3D-DRILLPRO and EH3D-ROUTEPRO. A change that sends something new to a real machine needs a note of what was observed on hardware and a test with the simulator. Until then the machine link stays in simulation.
2. **Every behaviour has a test**, including the refusals: a file that does not parse, a ZIP archive with names that leave the work folder, a table bigger than any machine, an option that does not exist.
3. **Before a pull request:** `cargo fmt --all`, `cargo clippy --all-targets --features gui -- -D warnings`, `cargo test --features gui` and `python -I tools/ci_validate.py` must pass. The program builds next to its siblings: the folders `EH3D-COMMON`, `EH3D-DRILLPRO` and `EH3D-ROUTEPRO` must sit beside this one.
4. **Texts exist in eight languages** (English, Spanish, French, Italian, German, Chinese, Japanese and European Portuguese) wherever the window shows text to people. The README files keep the same structure, and the two pictures are drawn from `docs/architecture.json` and `docs/structure.json`.
5. **No files of other people's software.** The sample files in `testdata/` are written for the tests; do not add programs, manuals or demo files of the original software.
6. Commits are written in English, small and about one thing.
