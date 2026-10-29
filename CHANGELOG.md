# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **Open files:** an Open window with a folder browser (and the drives), files dropped on the window, Ctrl+O, and files given on the command line; a drilling file, a routing file or a ZIP archive is opened as what its extension says.
- **Move around the views:** the plan view is dragged and zoomed with the wheel, the 3D view is turned with a drag, zoomed with the wheel and moved with Shift or the right button; a button (or a double click) puts the view back.
- **Panelization:** copies of the board in columns and rows, placed by the gap between boards, by the distance from one to the next, or as many as fit in the table; the copies are drawn and the job makes them.
- **Settings and About:** the usable width and depth of the table can be changed and saved to the machine files; the About window says the version, the licence and the connection.
- **Command line:** files without an option are opened, and `--modal`, `--panel` and `--yaw` set the window, the panel and the turn of the 3D view for the pictures.

## [0.0.1] - The launcher and the viewer

- **Jobs in ZIP archives:** `--zip JOB.ZIP` opens a job with its drilling files (`.ncd`, `.drl`...) and routing files (`.plt`, `.hpgl`...) inside; the first of each is shown and the others are listed to be loaded with a click. Names that would leave the work folder are not listed, and no file is read past a size limit.
- **The launcher:** the modern interface or the classic drilling and routing programs for a machine with manual or automatic tool change; the choice can be remembered.
- **The viewer:** the plan view of the table with every hole by drill size and the routing paths over it, the isometric view of the machine fitted to the table size of the configuration, the live progress of the job and the questions of the machine to the operator.
- **Themes and languages:** Forge, Neon Arcade, Glass Pro, classic Windows and the colours of Windows; English, Spanish, Italian, German, French, Portuguese, Chinese and Japanese.
- **The machine:** driven by the engines of the classic programs with their machine file (the computer's RS-232, a USB-RS232 adapter, the EH3D node directly or through the EH3D server); tried on the simulator only.
