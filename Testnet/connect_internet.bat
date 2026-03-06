@echo off
echo ========================================
echo MRBN Internet P2P Test
echo ========================================
echo.
echo This will connect your local node to the Railway bootstrap node
echo.

REM Railway TCP proxy details (from Railway dashboard)
set RAILWAY_PROXY=switchback.proxy.rlwy.net
set RAILWAY_PORT=35284

REM You'll get this peer ID from Railway logs after deployment
REM UPDATE THIS after you deploy and check the logs!
set RAILWAY_PEER_ID=PASTE_PEER_ID_FROM_RAILWAY_LOGS_HERE

REM Build the bootstrap address
set BOOTSTRAP_ADDR=/dns4/%RAILWAY_PROXY%/tcp/%RAILWAY_PORT%/p2p/%RAILWAY_PEER_ID%

echo Bootstrap address: %BOOTSTRAP_ADDR%
echo.
echo Starting local node...
echo.

cargo run --release -- --bootstrap %BOOTSTRAP_ADDR% --data-dir ./data_internet_test

pause

