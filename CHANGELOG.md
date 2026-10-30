# Changelog

All notable changes to this project are documented here.

## [Unreleased]

- **A job that does not fit on the table is refused:** Start checks every hole and every routing point against the table and says so, in the language of the window, instead of finding it out on the machine.

## [0.0.2] - The Windows-style Open window, the board library, the tools with default and typed values, the connection and the settings windows, and buttons that answer to the pointer

- **The settings in tabs, with the machine models:** the General and Job tabs, every Bungard MTC and ATC model with its table, its tool changer and its limits, and the units of the steps and the coordinates (mm or mil), all with a real effect on the job.
- **An emergency button:** it can be shown or hidden, dragged to where it is wanted, and it stops the job at once.
- **Start and pause follow the job:** Start goes out when the job runs, only Pause and Stop answer, and Start comes back with either of them; the simulation goes on while another program is in front.
- **The 2D view follows the work:** a button centres on each drill or router, the drilling and the routing files turn together, the board can be moved on the table, and the view can be enlarged over the 3D one.
- **Living check boxes and text boxes:** they change with a soft animation under the pointer and when they are clicked.
- **Sounds:** a sound when the job ends and another at a tool change, each one chosen, tried and switched on or off in the settings.
- **A bigger library of boards:** more than a hundred base materials of many makers, with a list that scrolls.
- **The holes of each tool:** the tools window shows, next to the diameter of every drill, how many holes the board has for it.
- **The buttons answer to the pointer:** they light up with a glow and a stronger edge under it, sink and darken when they are pressed, have a shadow and a shine on their top edge, and the pointer turns into a hand over them.
- **The simulation goes at the speed of the machine:** a job takes the time it would take with the feed and the speed of each tool; the Settings window chooses how many times faster than the machine it runs (x0.5 to x50).
- **Every value can be typed:** the numbers with buttons to step them (the table, the panel, the board, the feeds and the speeds of the tools) can be clicked and written with the keyboard (a comma is a decimal point; Enter or a click elsewhere accepts, Esc leaves it as it was).
- **Default values and the operator's:** the tools have default values chosen from the diameter of the drill and the material of the board (per tool or all at once), and any value can be typed over them.
- **A lighter debug build:** the `dev` profile keeps the file and line of a panic (`debug = "line-tables-only"`) but not the full debug symbols, so the `target` folder of a test run is about a tenth of its size.
- **Open files:** an Open window with a folder browser (and the drives), files dropped on the window, Ctrl+O, and files given on the command line; a drilling file, a routing file or a ZIP archive is opened as what its extension says.
- **Move around the views:** the plan view is dragged and zoomed with the wheel, the 3D view is turned with a drag, zoomed with the wheel and moved with Shift or the right button; a button (or a double click) puts the view back.
- **Panelization:** copies of the board in columns and rows, placed by the gap between boards, by the distance from one to the next, or as many as fit in the table; the copies are drawn and the job makes them.
- **Settings and About:** the usable width and depth of the table can be changed and saved to the machine files; the About window says the version, the licence and the connection.
- **Connection and pause:** a Connection window chooses the link (the computer's RS-232, a USB-RS232 adapter, the EH3D node or the EH3D server), fills in the port, address, token or machine, tests it without blocking the window and saves it to the machine file without touching its other lines; Pause holds a running job between two moves; the 3D view also tilts up and down.
- **The Open window is made like the one of any Windows program:** the places on the left (the examples, the desktop, the documents, the downloads and the drives), the path above (it can be typed), Back and Up, the list with name, date, type and size that is sorted by a click on its heading, a click that chooses and a second one that opens, the file name box, the type of file (all job files, drilling, routing, ZIP, all) and Open and Cancel.
- **The board:** a library of base materials of many makers (Bungard, Fortex, CIF, generic FR-4, CEM-1, CEM-3 and FR-2, Isola, Panasonic, Shengyi, ITEQ, Ventec, Nelco, Arlon, Rogers, Taconic, Bergquist and DuPont) with their thickness, copper, standard formats, dielectric constant, loss and glass transition; the Board window chooses the material, the thickness, the copper, one or two copper sides and the size, and it is remembered. The nominal values are those of the makers' and sellers' lists: the exact ones are the datasheet's.
- **The tools:** the left card lists every tool of the job, marked drilling or routing, with its size, its feed and its speed, and lights the one that is working; the Tools window changes the extra stroke or the depth, the passes, the feeds and the speed of each tool and saves them to the tool files.
- **The settings window:** the theme and the language can be chosen there, whether the job asks at the tool changes and at the start of the spindle, and whether this window opens directly the next time, besides the usable size of the table.
- **A demo board that fits itself:** the drilling file and the routing file of the example are the same 100 x 80 mm board, with four drill sizes and two routing tools.
- **Command line:** files without an option are opened, and `--modal`, `--panel` and `--yaw` set the window, the panel and the turn of the 3D view for the pictures.
- **Repository:** the README in eight languages with the architecture and structure pictures, the CI and release workflows (the program is built for Windows 64 and 32 bits, Linux and macOS, with the test files as examples), the issue and pull-request templates and the contribution, security and support notes.

## [0.0.1] - The launcher and the viewer

- **Jobs in ZIP archives:** `--zip JOB.ZIP` opens a job with its drilling files (`.ncd`, `.drl`...) and routing files (`.plt`, `.hpgl`...) inside; the first of each is shown and the others are listed to be loaded with a click. Names that would leave the work folder are not listed, and no file is read past a size limit.
- **The launcher:** the modern interface or the classic drilling and routing programs for a machine with manual or automatic tool change; the choice can be remembered.
- **The viewer:** the plan view of the table with every hole by drill size and the routing paths over it, the isometric view of the machine fitted to the table size of the configuration, the live progress of the job and the questions of the machine to the operator.
- **Themes and languages:** Forge, Neon Arcade, Glass Pro, classic Windows and the colours of Windows; English, Spanish, Italian, German, French, Portuguese, Chinese and Japanese.
- **The machine:** driven by the engines of the classic programs with their machine file (the computer's RS-232, a USB-RS232 adapter, the EH3D node directly or through the EH3D server); tried on the simulator only.
