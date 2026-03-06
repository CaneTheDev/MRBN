#!/bin/bash
# GitHub Codespaces → Local Laptop Connection Test

echo "╔════════════════════════════════════════╗"
echo "║   MRBN Codespaces → Laptop Test       ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Check if bootstrap address provided
if [ -z "$1" ]; then
    echo "❌ Error: Bootstrap address required!"
    echo ""
    echo "Usage: ./codespaces_connect_test.sh BOOTSTRAP_ADDR"
    echo ""
    echo "Example:"
    echo "  ./codespaces_connect_test.sh \"/ip4/203.0.113.45/tcp/8333/p2p/12D3KooW...\""
    echo ""
    echo "Steps to get bootstrap address:"
    echo "  1. On your Windows laptop, run: test_as_bootstrap.bat"
    echo "  2. Copy the Peer ID from the output"
    echo "  3. Get your public IP from: https://whatismyipaddress.com"
    echo "  4. Format: /ip4/YOUR_PUBLIC_IP/tcp/8333/p2p/YOUR_PEER_ID"
    echo ""
    exit 1
fi

BOOTSTRAP_ADDR="$1"

echo "🔗 Bootstrap node: $BOOTSTRAP_ADDR"
echo ""

echo "🔨 Building MRBN node..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi
echo "✅ Build successful!"
echo ""

echo "🚀 Starting test node that will connect to your laptop..."
echo ""
echo "   Your laptop should show:"
echo "   - 🔍 mDNS discovered peer"
echo "   - 🌍 Kademlia discovered new peer"
echo "   - 🤝 Connected to peer"
echo ""

# Run the node with bootstrap
./target/release/mrbn-node \
    --data-dir ./data_codespaces \
    --port 8334 \
    --bootstrap "$BOOTSTRAP_ADDR"
