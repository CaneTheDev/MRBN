@echo off
echo ╔════════════════════════════════════════╗
echo ║   MRBN Test with Public Relay         ║
echo ╚════════════════════════════════════════╝
echo.
echo This test uses a public libp2p relay server to bypass NAT
echo.

cargo run --release -- ^
    --data-dir ./data_relay_test ^
    --port 8335 ^
    --bootstrap "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"

pause
