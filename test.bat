@echo off
rem Runs the tests only (they never change the version).
cd /d "%~dp0"
cargo test
if not defined NOPAUSE pause
