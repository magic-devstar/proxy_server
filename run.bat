@echo off
setlocal

set CONFIG=%1
if "%CONFIG%"=="" set CONFIG=config.json

if not exist "%CONFIG%" (
    echo ERROR: Config file not found: %CONFIG%
    echo.
    echo Usage: run.bat [config-file]
    echo.
    echo Example:
    echo   run.bat config.json
    echo.
    echo Create config.json from config.example.json first:
    echo   copy config.example.json config.json
    echo   REM Edit config.json with your settings
    exit /b 1
)

if not exist "target\release\riptide.exe" (
    echo ERROR: Binary not found. Building...
    call build.bat
)

echo Starting Riptide Proxy...
echo Config: %CONFIG%
echo.

REM Set log level
if "%RUST_LOG%"=="" set RUST_LOG=info

REM Run the proxy
target\release\riptide.exe --config %CONFIG%

