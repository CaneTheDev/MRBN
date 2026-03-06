@echo off
echo Building optimized release version...
cargo build --release
echo.
echo Build complete! Executable at: target\release\mrbn-node.exe
