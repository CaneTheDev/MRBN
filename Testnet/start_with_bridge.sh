#!/bin/bash
# Start both the libp2p node and relay bridge on Railway

echo "Starting MRBN with Relay Bridge..."
echo "Relay is ONLY for initial bootstrap - all other connections are direct P2P"

# Start libp2p node in background
/usr/local/bin/mrbn-node --data-dir /home/mrbn/data &
NODE_PID=$!

# Wait for node to start
sleep 3

# Start relay bridge (foreground)
/usr/local/bin/relay-bridge

# Cleanup on exit
kill $NODE_PID
