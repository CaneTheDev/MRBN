@echo off
echo ========================================
echo Connecting to Railway Bootstrap Node
echo ========================================
echo.

REM Railway's peer ID from logs
set RAILWAY_PEER=12D3KooWE9KDFDTmAs8UFpqaQtNLx2B7Kzk2173gotgxATtsWWXF

REM Connect via WebSocket (Railway's HTTP proxy supports this)
set RAILWAY_ADDR=/dns4/mrbn-production.up.railway.app/tcp/8334/ws/p2p/%RAILWAY_PEER%

echo Connecting via WebSocket to Railway...
echo Peer ID: %RAILWAY_PEER%
echo Address: %RAILWAY_ADDR%
echo.
echo After initial WebSocket connection, P2P will work directly!
echo.

cargo run --release --bin mrbn-node -- --bootstrap %RAILWAY_ADDR% --data-dir ./data_test
