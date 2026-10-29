#!/usr/bin/env sh
# Builds the window.
set -e
cd "$(dirname "$0")"
cargo build --release --features gui
printf "Press Enter to close... "
read -r _
