#!/usr/bin/env bash
# Runs BUNGARD-GUI. Builds it first if needed. Without arguments it opens the launcher with the test files; extra arguments are passed on:
#   ./run.sh [--drill FILE.NCD] [--route FILE.PLT] [--theme forge|neon|glass|windows-classic|system]
#            [--lang en|es|it|de|fr|pt|zh|ja] [--data DIR] [--connection FILE] [--screen launcher|viewer]
set -u
cd "$(dirname "$0")"
pause() { [ -n "${NOPAUSE:-}" ] || { read -r -p "Press Enter to close..." _ || true; }; }
trap pause EXIT
if [ ! -x "target/release/bungard-gui" ]; then
    NOPAUSE=1 ./build.sh || exit 1
fi
if [ $# -eq 0 ]; then
    set -- --data "$PWD/testdata" --drill "$PWD/testdata/05_small_board.ncd" --route "$PWD/testdata/05_board_outline.plt"
fi
"target/release/bungard-gui" "$@"
