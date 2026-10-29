#!/usr/bin/env sh
# Runs the tests only.
set -e
cd "$(dirname "$0")"
cargo test
printf "Press Enter to close... "
read -r _
