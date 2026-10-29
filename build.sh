#!/usr/bin/env bash
# Builds the window (release). The window needs the sibling folders EH3D-DRILLPRO, EH3D-ROUTEPRO and EH3D-COMMON.
set -u
cd "$(dirname "$0")"
pause() { [ -n "${NOPAUSE:-}" ] || { read -r -p "Press Enter to close..." _ || true; }; }
trap pause EXIT
cargo build --release --features gui
