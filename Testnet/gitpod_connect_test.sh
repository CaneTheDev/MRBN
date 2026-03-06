#!/bin/bash
# Gitpod script to connect to your laptop bootstrap node

echo "╔════════════════════════════════════════╗"
echo "║   MRBN Gitpod → Laptop Connection     ║"
echo "╚════════════════════════════════════════╝"
echo ""

# Check if bootstrap address provided
if [ -z "$1" ]; then
    echo "❌ Error: Bootstrap address required!"
    echo ""
    echo "Usage: ./gitpod_connect_test.sh BOOTSTRAP_ADDR"
    echo ""
    echo "Example:"
    echo "  ./gitpod_connect_test.sh \"/ip4/1.2.3.4/tcp/8333/p2p/12D3KooW...\""
    echo ""
    echo "Get the bootstrap address from your laptop after running:"
    echo "  test_as_bootstrap.bat"
    echo ""
    exit 1
fi

BOOTSTRAP_ADDR="$1"

echo "🔗 Bootstrap node: $BOOTSTRAP_ADDR"
echo ""

echo "🔨 Building MRBN node..."
cargo build --release --bin network-test
if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi
echo "✅ Build successful!"
echo ""

echo "🚀 Running automated network test..."
echo "   This will test cross-network connectivity between:"
echo "   - Your laptop (Windows, bootstrap node)"
echo "   - Gitpod (Linux, connecting node)"
echo ""
echo "   Tests include:"
echo "   - Peer discovery across the internet"
echo "   - Block synchronization"
echo "   - Transaction propagation"
echo "   - 30-second stability test"
echo ""

./target/release/network-test "$BOOTSTRAP_ADDR"

echo ""
echo "Test complete! Check results above."
