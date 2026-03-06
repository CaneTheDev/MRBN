@echo off
REM Automated network synchronization test

echo ╔════════════════════════════════════════╗
echo ║   MRBN Network Synchronization Test   ║
echo ╚════════════════════════════════════════╝
echo.

REM Check if bootstrap address is provided
if "%1"=="" (
    echo ❌ Error: Bootstrap address required!
    echo.
    echo Usage: test_railway_connection.bat BOOTSTRAP_ADDR
    echo.
    echo Example:
    echo   test_railway_connection.bat "/ip4/1.2.3.4/tcp/8333/p2p/12D3KooW..."
    echo.
    pause
    exit /b 1
)

set BOOTSTRAP_ADDR=%1

echo 🔨 Building network test...
cargo build --release --bin network-test
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Build failed!
    pause
    exit /b 1
)
echo ✅ Build successful!
echo.

echo 🚀 Running automated network test...
echo    This will test:
echo    - Node initialization
echo    - Network connection to Railway
echo    - Peer discovery
echo    - Genesis block sync
echo    - Transaction creation and submission
echo    - Consensus operation
echo    - Storage integrity
echo    - 30-second stability test
echo.

target\release\network-test.exe %BOOTSTRAP_ADDR%

echo.
pause

