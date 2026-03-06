@echo off
echo ========================================
echo Testing Local P2P Network
echo ========================================
echo.
echo This will start Node 1 as bootstrap.
echo After it starts, open another terminal and run: test_local_p2p_node2.bat
echo.
echo Press Ctrl+C to stop
echo.

cargo run --release --bin mrbn-node -- --data-dir ./data_node1 --port 8333
