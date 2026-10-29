@echo off
cd /d "%~dp0"
cargo build --release --features gui
pause
