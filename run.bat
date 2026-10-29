@echo off
rem Runs BUNGARD-GUI. Builds it first if needed. Without arguments it opens the launcher with the test files; extra arguments are passed on:
rem   run.bat [--drill FILE.NCD] [--route FILE.PLT] [--theme forge^|neon^|glass^|windows-classic^|system]
rem           [--lang en^|es^|it^|de^|fr^|pt^|zh^|ja] [--data DIR] [--connection FILE] [--screen launcher^|viewer]
cd /d "%~dp0"
if not exist "target\release\bungard-gui.exe" (
    set NOPAUSE=1
    call "%~dp0build.bat"
    set NOPAUSE=
)
if not exist "target\release\bungard-gui.exe" (
    echo bungard-gui.exe was not built.
    goto :end
)
if "%~1"=="" (
    "target\release\bungard-gui.exe" --data "%~dp0testdata" --drill "%~dp0testdata\05_small_board.ncd" --route "%~dp0testdata\05_board_outline.plt"
) else (
    "target\release\bungard-gui.exe" %*
)
:end
if not defined NOPAUSE pause
