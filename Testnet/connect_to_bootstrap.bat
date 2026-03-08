@echo off
echo ========================================
echo Connect to Local Bootstrap Node
echo ========================================
echo.
echo INSTRUCTIONS:
echo 1. Run start_bootstrap.bat first
echo 2. Copy the Peer ID from the logs (12D3Koo...)
echo 3. Get your local IP (run: ipconfig, look for IPv4)
echo 4. Update BOOTSTRAP_PEER_ID and LOCAL_IP below
echo 5. Run this script
echo.
echo ========================================

REM UPDATE THESE:
set BOOTSTRAP_PEER_ID=PASTE_PEER_ID_HERE
set LOCAL_IP=YOUR_LOCAL_IP_HERE

set BOOTSTRAP_ADDR=/ip4/%LOCAL_IP%/tcp/8333/p2p/%BOOTSTRAP_PEER_ID%

echo Bootstrap address: %BOOTSTRAP_ADDR%
echo.

cargo run --release --bin mrbn-node -- --bootstrap %BOOTSTRAP_ADDR% --data-dir ./data_node2

pause
