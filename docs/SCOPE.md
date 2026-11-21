# EH3D-BUNGARD-GUI: scope

Status: **simulator**. EH3D-BUNGARD-GUI is the modern window for the Bungard CCD machines: a launcher, a plan view and a 3D view of the job, drilling and routing together, in 37 languages. It is tried on a computer and on the simulator; it has never driven a real machine. This page says what it is for, what it will not do and where it sits among the other programs of the suite.

## Purpose

The window from which a person opens a drilling file, a plot file or a ZIP archive of a job, sees the board on the table of the chosen machine in two and in three dimensions, edits and panelizes it, and runs it through the engines of EH3D-DRILLPRO and EH3D-ROUTEPRO with the same machine file, on the simulator or, when the file says so, on a machine, the node or the server of the suite. It also starts the classic programs from its launcher.

## What it does

* Opens drilling files, HP-GL plot files, DXF drawings, the outline of a Gerber board and ZIP archives, refuses a name that would leave the work folder and reads no file past a size limit.
* Draws the plan view with every hole by drill size and the routing paths over it, and the 3D view of the machine fitted to the table of the model chosen, with the head following the job.
* Panelizes the board in three modes, edits the job (take a tool out, choose, move, copy, undo and redo) and refuses a job that does not fit the table before it starts; the design is locked while the machine works.
* Runs the job through the engines: drilling and routing one after the other, the tool changer of the ATC models with its places in the bar and its loads, a job that is cut and resumed, a pause between two moves and an emergency button that stops the job and blocks the start.
* Jogs the head by hand inside the travel, measures the table with a probe and corrects the jobs only when the person asks, and looks at the board with the camera of the head after the job.
* Keeps the settings in the machine file, a log without secrets, a report of the job with an estimate of the time, and looks for a new release, which it installs only after the checksum and, when asked, the signature agree.
* Shows the machines and stations of the server, chooses the one the jobs go to, and prints the status of the server from the command line.

## What it will not do

* No emergency stop of its own beyond the button of the window: the control unit of the machine watches the limit switches and the power switch is the real stop (see below).
* No drive of the CCD Premium and the CCD XL: their profiles are drawn, but their own control unit is not supported.
* No pause in the middle of a move: the pause holds the job between two moves.
* No edit of the whole machine and tool files: it writes the keys it offers and leaves every other line as it was.
* No installation of an update without a person clicking it, and none whose checksum does not agree.

## Where it sits

* **It reads:** drilling, plot, DXF, Gerber and ZIP files, the machine file and the tool files of the classic programs, the answers of the machine, the node and the server, and the releases of its own repository on GitHub.
* **It writes:** the keys of the window in the machine file and the recent files in its settings, the state of a job that can be resumed, the log, the bytes of the job to the connection and, when the person asks, a new version of itself.
* **It is read by:** the person at the Bungard machine; no other program of the suite depends on it.
* **It depends on:** `EH3D-COMMON`, `EH3D-DRILLPRO` and `EH3D-ROUTEPRO`, built next to it as sibling folders; the classic programs when its launcher starts them.

## What it never decides

The limits of a machine, its soft limits and its emergency stop belong to the machine and to its control unit, not to this window. A job that was shown and started is not a verified job: nothing here knows where the machine stops, so a person must take the tool out, keep a hand on the power switch and be ready to stop the job.

The requirements of the program, with the state of each and the test that shows it, are in `REQUIREMENTS.md`; the state in a paragraph is in `STATUS.md`.
