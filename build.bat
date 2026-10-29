@echo off
rem Builds the window (release). The window needs the sibling folders EH3D-DRILLPRO, EH3D-ROUTEPRO and EH3D-COMMON.
cd /d "%~dp0"
cargo build --release --features gui
if not defined NOPAUSE pause
