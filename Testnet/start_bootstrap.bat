@echo off
echo ========================================
echo MRBN Bootstrap Node (Your Local Network)
echo ========================================
echo.
echo Starting bootstrap node...
echo.

cargo run --release --bin mrbn-node -- --data-dir ./data_bootstrap

pause
