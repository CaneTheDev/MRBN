@echo off
echo ========================================
echo Connecting to Railway Bootstrap Node
echo ========================================
echo.

REM Railway's peer ID from logs
set RAILWAY_PEER=12D3KooWPfGAcM19iwTVREV742obUm6zRrFtNFP1rXtFjGVkUL1j

REM Railway's actual proxy address (from Railway dashboard)
set RAILWAY_ADDR=/dns4/switchback.proxy.rlwy.net/tcp/35284/wss/p2p/%RAILWAY_PEER%

echo Connecting via secure WebSocket to Railway...
echo Peer ID: %RAILWAY_PEER%
echo Address: %RAILWAY_ADDR%
echo.
echo After initial WebSocket connection, P2P will work directly!
echo.

cargo run --release --bin mrbn-node -- --bootstrap %RAILWAY_ADDR% --data-dir ./data_test
