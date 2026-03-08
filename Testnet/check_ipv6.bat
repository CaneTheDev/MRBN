@echo off
echo ========================================
echo IPv6 Connectivity Check
echo ========================================
echo.

echo Your IPv6 Addresses:
echo --------------------
ipconfig | findstr /i "IPv6"
echo.

echo Testing IPv6 Connectivity:
echo -------------------------
echo Pinging Google's IPv6 DNS (2001:4860:4860::8888)...
ping -6 -n 2 2001:4860:4860::8888
echo.

echo Testing IPv6 Web Access:
echo -----------------------
curl -6 -s -o nul -w "HTTP Status: %%{http_code}\n" https://ipv6.google.com 2>nul
if %errorlevel% equ 0 (
    echo ✅ IPv6 internet access works!
) else (
    echo ❌ No IPv6 internet access
)
echo.

echo Your Public IPv6 Address (if available):
echo ----------------------------------------
curl -6 -s https://api64.ipify.org 2>nul
if %errorlevel% neq 0 (
    echo ❌ Cannot reach IPv6 services
    echo.
    echo DIAGNOSIS:
    echo ----------
    echo You only have link-local IPv6 (fe80::...)
    echo Your ISP does not provide global IPv6 connectivity
    echo.
    echo OPTIONS:
    echo --------
    echo 1. Contact your ISP to enable IPv6
    echo 2. Use a VPN that provides IPv6
    echo 3. Use Fly.io for bootstrap node (supports IPv4/IPv6)
    echo 4. Keep local-only P2P network
)
echo.
pause
