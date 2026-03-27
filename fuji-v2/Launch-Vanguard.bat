@echo off
echo ===================================================
echo     VANGUARD - Codebase Intelligence Engine
echo ===================================================
echo.
echo Bypassing Windows OS File Locks by using safe temp directory...

set CARGO_TARGET_DIR=C:\temp\vanguard_build
cargo run --release

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Vanguard engine closed unexpectedly.
    pause
)
