@echo off
REM Run your laptop as the bootstrap node for cross-network testing

echo ╔════════════════════════════════════════╗
echo ║   MRBN Bootstrap Node (Your Laptop)   ║
echo ╚════════════════════════════════════════╝
echo.

echo 📋 Setup Instructions:
echo    1. Forward port 8333 on your router to this computer
echo    2. Find your public IP: https://whatismyipaddress.com
echo    3. Start this node (it will show your Peer ID)
echo    4. In Gitpod, connect using:
echo       /ip4/YOUR_PUBLIC_IP/tcp/8333/p2p/YOUR_PEER_ID
echo.

REM Get local IP
for /f "tokens=2 delims=:" %%a in ('ipconfig ^| findstr /c:"IPv4 Address"') do set LOCAL_IP=%%a
set LOCAL_IP=%LOCAL_IP:~1%

echo 🌐 Your local IP: %LOCAL_IP%
echo 🌍 Get your public IP from: https://whatismyipaddress.com
echo.

echo 🔨 Building MRBN node...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo ❌ Build failed!
    pause
    exit /b 1
)
echo ✅ Build successful!
echo.

echo 🚀 Starting bootstrap node...
echo.
echo ╔════════════════════════════════════════╗
echo ║   IMPORTANT: Copy these values!        ║
echo ╚════════════════════════════════════════╝
echo.
echo Watch for:
echo    📍 Local peer id: 12D3Koo...
echo    🌐 Listening on /ip4/...
echo.
echo Then in Gitpod, run:
echo    cargo run -- --bootstrap "/ip4/YOUR_PUBLIC_IP/tcp/8333/p2p/PEER_ID_FROM_ABOVE"
echo.
echo Press Ctrl+C to stop
echo.

target\release\mrbn-node.exe --data-dir ./data_bootstrap --port 8333
