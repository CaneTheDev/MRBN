@echo off
echo ╔════════════════════════════════════════╗
echo ║   Connect to Codespaces Bootstrap     ║
echo ╚════════════════════════════════════════╝
echo.
echo Connecting your laptop to Codespaces node...
echo.

.\target\release\mrbn-node.exe ^
    --data-dir ./data_test ^
    --port 8334 ^
    --bootstrap "/ip4/172.166.156.98/tcp/8333/p2p/12D3KooWSyosHi5AyEApcNSTkdGdHUpXJfBXHM1rWKZeK2YZFTNK"

pause
