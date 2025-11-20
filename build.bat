@echo off
echo Building Riptide Rust Proxy (Windows)...
echo.

REM Check if cargo is available
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Rust toolchain not found. Please install from https://rustup.rs/
    exit /b 1
)

echo Building release binary (without io_uring - Windows does not support it)...
cargo build --release --no-default-features

if %ERRORLEVEL% EQU 0 (
    echo.
    echo Build successful!
    echo.
    echo Binaries:
    echo   - target\release\riptide.exe       (main proxy server^)
    echo   - target\release\test-client.exe   (test client^)
    echo.
    echo Quick start:
    echo   target\release\riptide.exe --config config.json
    echo.
    echo Test:
    echo   target\release\test-client.exe -u user -P pass -c 10
) else (
    echo.
    echo Build failed!
    exit /b 1
)

