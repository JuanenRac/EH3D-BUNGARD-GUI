#!/usr/bin/env python3
"""Takes the pictures of the README with the window itself (`--shot`).

    python tools/make_shots.py --exe PATH/TO/bungard-gui --out FOLDER            # every picture
    python tools/make_shots.py --exe PATH/TO/bungard-gui --out FOLDER --only jog  # one of them
    python tools/make_shots.py --list                                              # the names and the options of each picture

The program is the one of a release (download it into an empty folder and give its path), the pictures go to a folder of their own (never the drive of a
removable disk) and every picture has to be looked at before it replaces one of `images/screenshots/`. A picture run opens the window for a moment,
saves the picture and leaves; it writes no log and remembers nothing. Needs the test board of `testdata/` next to this repository.
"""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "testdata"
BOARD = ["--drill", str(DATA / "05_small_board.ncd"), "--route", str(DATA / "05_board_outline.plt")]
COMMON = ["--data", str(DATA), "--lang", "en", "--theme", "forge"]

# name -> options after the common ones (the window of the picture, the model, the turn of the 3D view...)
SHOTS: dict[str, list[str]] = {
    "main": ["--screen", "viewer", *BOARD],
    "viewer-3d": ["--screen", "viewer", *BOARD, "--yaw", "75"],
    "atc-slots": ["--screen", "viewer", *BOARD, "--machine", "CCD/2 ATC", "--modal", "slots"],
    "settings": ["--screen", "viewer", *BOARD, "--modal", "settings", "--tab", "0"],
    "board": ["--screen", "viewer", *BOARD, "--modal", "board"],
    "jog": ["--screen", "viewer", *BOARD, "--modal", "jog"],
    "report": ["--screen", "viewer", *BOARD, "--modal", "report"],
}


def refuse_removable(path: Path) -> None:
    """The pictures never go to the D: drive."""
    if path.resolve().drive.upper() == "D:":
        sys.exit(f"make_shots: {path} is on the D: drive, give a folder on another drive")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--exe", type=Path, help="the bungard-gui program of the release")
    ap.add_argument("--out", type=Path, help="the folder the pictures go to (made if it is not there)")
    ap.add_argument("--only", action="append", choices=sorted(SHOTS), help="take only this picture (can be repeated)")
    ap.add_argument("--wait", type=int, default=12, help="frames to wait before the picture (12)")
    ap.add_argument("--list", action="store_true", help="print the pictures and their options, and leave")
    args = ap.parse_args()
    if args.list:
        for name, opts in SHOTS.items():
            print(name, " ".join(COMMON + opts))
        return 0
    if not args.exe or not args.out:
        ap.error("--exe and --out are needed")
    if not args.exe.is_file():
        sys.exit(f"make_shots: {args.exe} is not a file")
    refuse_removable(args.out)
    args.out.mkdir(parents=True, exist_ok=True)
    failed = 0
    for name in args.only or list(SHOTS):
        png = args.out / f"{name}.png"
        png.unlink(missing_ok=True)
        cmd = [str(args.exe), *COMMON, *SHOTS[name], "--wait", str(args.wait), "--shot", str(png)]
        done = subprocess.run(cmd, cwd=ROOT, timeout=120, check=False)
        ok = done.returncode == 0 and png.is_file() and png.stat().st_size > 10_000
        print(f"{'OK  ' if ok else 'FAIL'} {name} {png}")
        failed += 0 if ok else 1
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
