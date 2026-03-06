#!/bin/bash
# Start both the libp2p node and relay bridge on Railway

echo "Starting MRBN with Relay Bridge..."
echo "Relay is ONLY for initial bootstrap - all other connections are direct P2P"

# Start libp2p node in background
/usr/local/bin/mrbn-node --data-dir /home/mrbn/data &
NODE_PID=$!

echo "Node started with PID: $NODE_PID"

# Wait for node to start
sleep 3

echo "Starting relay bridge..."

# Start relay bridge (foreground - keeps container running)
/usr/local/bin/relay-bridge

# If relay bridge exits, cleanup
echo "Relay bridge stopped, cleaning up..."
kill $NODE_PID 2>/dev/null
