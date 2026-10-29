#!/usr/bin/env bash
# Runs the tests only (they never change the version).
set -u
cd "$(dirname "$0")"
pause() { [ -n "${NOPAUSE:-}" ] || { read -r -p "Press Enter to close..." _ || true; }; }
trap pause EXIT
cargo test
