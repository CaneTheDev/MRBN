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

REM Railway peer ID from logs
set RAILWAY_PEER_ID=12D3KooWPYAG7VLyAdfXRy79pDd9WYLnhmtEm5BPKE84wm9gnJMm

REM Build the bootstrap address
set BOOTSTRAP_ADDR=/dns4/%RAILWAY_PROXY%/tcp/%RAILWAY_PORT%/p2p/%RAILWAY_PEER_ID%

echo Bootstrap address: %BOOTSTRAP_ADDR%
echo.
echo Starting local node...
echo.

cargo run --release --bin mrbn-node -- --bootstrap %BOOTSTRAP_ADDR% --data-dir ./data_internet_test

pause

