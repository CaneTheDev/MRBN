// Relay bridge binary - runs HTTP/WebSocket to libp2p bridge
// Deploy this on Railway to enable connections through their proxy

use mrbn_node::relay_bridge::RelayBridge;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("\n╔════════════════════════════════════════╗");
    println!("║   MRBN Relay Bridge                   ║");
    println!("║   HTTP/WebSocket → libp2p              ║");
    println!("╚════════════════════════════════════════╝\n");

    // Listen on Railway's HTTP port (8334)
    let listen_addr = "0.0.0.0:8334".to_string();
    
    // Forward to local libp2p node (8333)
    let target_addr = "127.0.0.1:8333".to_string();

    let bridge = RelayBridge::new(listen_addr, target_addr);
    bridge.run().await?;

    Ok(())
}
