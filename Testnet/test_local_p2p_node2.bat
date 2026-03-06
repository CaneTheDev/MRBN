@echo off
echo ========================================
echo Starting Node 2 - Connecting to Node 1
echo ========================================
echo.

REM Get Node 1's peer ID from its logs and paste it here
set /p NODE1_PEER="Enter Node 1's Peer ID: "

echo.
echo Connecting to Node 1 at localhost...
echo.

cargo run --release --bin mrbn-node -- --bootstrap /ip4/127.0.0.1/tcp/8333/p2p/%NODE1_PEER% --data-dir ./data_node2 --port 8334
