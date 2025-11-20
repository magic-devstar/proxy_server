@echo off
REM ========================================
REM  RIPTIDE PROXY - ONE-CLICK STARTER
REM ========================================

cls
echo.
echo ========================================
echo   RIPTIDE PROXY MVP - AUTO STARTER
echo ========================================
echo.

REM Check if built
if not exist "target\release\riptide.exe" (
    echo [!] Project not built yet!
    echo [*] Building now... please wait...
    echo.
    cargo build --release --no-default-features
    
    if errorlevel 1 (
        echo.
        echo [ERROR] Build failed!
        echo Please install Rust from: https://rustup.rs/
        pause
        exit /b 1
    )
    
    echo.
    echo [OK] Build completed!
    echo.
)

REM Check if Python available
python --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Python not found!
    echo Please install Python from: https://www.python.org/
    pause
    exit /b 1
)

REM Start Mock API in background
echo [1/2] Starting Mock API Server...
start "Riptide Mock API" cmd /k "python mock_api_server.py"
timeout /t 3 /nobreak >nul

REM Start Proxy in background
echo [2/2] Starting Riptide Proxy...
start "Riptide Proxy" cmd /k "target\release\riptide.exe --config config.json"
timeout /t 3 /nobreak >nul

echo.
echo ========================================
echo   SERVICES STARTED!
echo ========================================
echo.
echo Mock API:  http://127.0.0.1:8000
echo Proxy:     http://127.0.0.1:8080
echo.
echo Test Users:
echo   testuser / testpass   (Standard)
echo   premium / premium123  (Premium)
echo   limited / limited123  (Limited)
echo.
echo ========================================
echo   WHAT TO DO NEXT
echo ========================================
echo.
echo [1] Open MVP_DASHBOARD.html
echo     - Beautiful web interface
echo     - Login with testuser / testpass
echo     - Click test buttons
echo.
echo [2] Test from command line:
echo     curl.exe -x http://testuser:testpass@127.0.0.1:8080 http://httpbin.org/ip
echo.
echo [3] Test with load tool:
echo     target\release\test-client.exe --proxy 127.0.0.1:8080 -u testuser -P testpass -c 10
echo.
echo ========================================
echo.
echo Press any key to open the dashboard...
pause >nul

REM Open dashboard
start MVP_DASHBOARD.html

echo.
echo Dashboard opened!
echo.
echo To stop services:
echo   - Close the "Riptide Mock API" window
echo   - Close the "Riptide Proxy" window
echo.
pause

