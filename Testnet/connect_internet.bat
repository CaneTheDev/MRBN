@echo off
echo ========================================
echo MRBN Internet P2P Test
echo ========================================
echo.
echo Connecting to Railway via WebSocket relay bridge
echo After bootstrap, all connections will be direct P2P!
echo.

REM Railway exposes the relay bridge on their HTTP domain
set RAILWAY_DOMAIN=mrbn-production.up.railway.app
set RAILWAY_PORT=443

REM Railway peer ID from logs
set RAILWAY_PEER_ID=12D3KooWBNncp1R6K1m9wJiJ1L9e13arJ7R2CZYkccEVKVRZDARM

REM Build the bootstrap address using WebSocket (ws not wss - Railway handles TLS)
set BOOTSTRAP_ADDR=/dns4/%RAILWAY_DOMAIN%/tcp/%RAILWAY_PORT%/ws/p2p/%RAILWAY_PEER_ID%

echo Bootstrap address: %BOOTSTRAP_ADDR%
echo.
echo Starting local node...
echo.

cargo run --release --bin mrbn-node -- --bootstrap %BOOTSTRAP_ADDR% --data-dir ./data_internet_test

pause

