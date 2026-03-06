@echo off
echo ========================================
echo Connecting to Railway Bootstrap Node
echo ========================================
echo.

REM Railway's peer ID from logs
set RAILWAY_PEER=12D3KooWRSS1YimY9vMAfy7Q573HbASXK25qiVyWTT86ybaXryr6

REM Railway's public address (using libp2p relay)
set RELAY_ADDR=/dnsaddr/relay.libp2p.io/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN/p2p-circuit/p2p/%RAILWAY_PEER%

echo Connecting to Railway via relay...
echo Peer ID: %RAILWAY_PEER%
echo.

cargo run --release --bin mrbn-node -- --bootstrap %RELAY_ADDR% --data-dir ./data_test
