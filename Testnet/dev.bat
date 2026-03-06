@echo off
echo Running clippy...
cargo clippy -- -D warnings
if %errorlevel% neq 0 (
    echo Clippy found issues. Fix them before building.
    exit /b %errorlevel%
)

echo.
echo Clippy passed! Building in debug mode...
cargo build
if %errorlevel% neq 0 (
    echo Build failed.
    exit /b %errorlevel%
)

echo.
echo Build successful! Starting MRBN Node...
echo.
cargo run --bin mrbn-node
