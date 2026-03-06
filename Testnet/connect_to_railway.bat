@echo off
echo ╔════════════════════════════════════════╗
echo ║   Connect to Railway Bootstrap Node   ║
echo ╚════════════════════════════════════════╝
echo.
echo Connecting to Railway via TCP Proxy...
echo.

.\target\release\mrbn-node.exe ^
    --data-dir ./data_railway_test ^
    --port 8334 ^
    --bootstrap "/dns4/switchback.proxy.rlwy.net/tcp/35284/p2p/12D3KooWDfHQyD5T9jgNJQ9wFjUE9SXjr51goQeq5ki5GVjpKYWV"

pause
