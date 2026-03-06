@echo off
echo ╔════════════════════════════════════════╗
echo ║   Connect via WebSocket to Railway    ║
echo ╚════════════════════════════════════════╝
echo.
echo Connecting to Railway via WebSocket...
echo This works through HTTP proxies and firewalls!
echo.

.\target\release\mrbn-node.exe ^
    --data-dir ./data_ws_test ^
    --port 8335 ^
    --bootstrap "/dns4/mrbn-production.up.railway.app/tcp/8334/ws/p2p/RAILWAY_PEER_ID"

echo.
echo Replace RAILWAY_PEER_ID with the actual peer ID from Railway logs
pause
