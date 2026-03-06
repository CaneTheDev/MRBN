@echo off
REM Comprehensive MRBN Test Runner
REM Tests all core functionality in one automated run

echo.
echo ╔════════════════════════════════════════╗
echo ║   MRBN Comprehensive Test Suite       ║
echo ║   Testing All Core Functionality      ║
echo ╚════════════════════════════════════════╝
echo.

echo 🔨 Building test suite...
cargo build --tests --release
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Build failed!
    pause
    exit /b 1
)
echo ✅ Build successful!
echo.

echo 🧪 Running comprehensive tests...
echo This will test:
echo    - Storage Layer (ParityDB)
echo    - Transaction System
echo    - VRF Committee Selection
echo    - Block Creation and Validation
echo    - Consensus Orchestrator
echo    - Validator Resource Management
echo    - Wallet and Keystore
echo    - Blockchain State
echo    - Validation Protocol
echo    - End-to-End Integration
echo.

cargo test --test comprehensive_test -- --nocapture --test-threads=1

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ╔════════════════════════════════════════╗
    echo ║   ✅ ALL TESTS PASSED!                ║
    echo ║   MRBN Core is production-ready!      ║
    echo ╚════════════════════════════════════════╝
) else (
    echo.
    echo ╔════════════════════════════════════════╗
    echo ║   ❌ TESTS FAILED                     ║
    echo ║   Check output above for details      ║
    echo ╚════════════════════════════════════════╝
)

echo.
pause
