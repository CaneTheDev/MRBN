// HTTP/WebSocket to libp2p relay bridge for Railway compatibility
// This allows libp2p nodes to connect through HTTP proxies for INITIAL bootstrap only
// After bootstrap, all connections are direct P2P

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{info, warn, error};
use std::sync::Arc;

/// Relay bridge that accepts HTTP/WebSocket connections and forwards to libp2p
pub struct RelayBridge {
    listen_addr: String,
    target_addr: String,
}

impl RelayBridge {
    pub fn new(listen_addr: String, target_addr: String) -> Self {
        Self {
            listen_addr,
            target_addr,
        }
    }

    /// Start the relay bridge server
    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(&self.listen_addr).await?;
        info!("🌉 Relay bridge listening on {}", self.listen_addr);
        info!("🎯 Forwarding to libp2p at {}", self.target_addr);
        info!("ℹ️  This relay is ONLY for initial bootstrap - all other connections are direct P2P");

        let target_addr = Arc::new(self.target_addr);

        loop {
            match listener.accept().await {
                Ok((client_stream, client_addr)) => {
                    info!("📥 New connection from {}", client_addr);
                    let target = Arc::clone(&target_addr);
                    
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(client_stream, &target).await {
                            warn!("❌ Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("❌ Accept error: {}", e);
                }
            }
        }
    }
}

/// Handle a single client connection
async fn handle_connection(mut client: TcpStream, target_addr: &str) -> Result<()> {
    // Read the first bytes to detect protocol
    let mut buffer = vec![0u8; 4096];
    let n = client.peek(&mut buffer).await?;
    
    if n == 0 {
        return Ok(());
    }

    // Check if it's an HTTP request
    let is_http = buffer[..n].starts_with(b"GET ") 
        || buffer[..n].starts_with(b"POST ")
        || buffer[..n].starts_with(b"HEAD ");

    if is_http {
        // Handle WebSocket upgrade
        handle_websocket_upgrade(&mut client, target_addr).await?;
    } else {
        // Direct TCP forwarding for libp2p
        handle_tcp_forward(client, target_addr).await?;
    }

    Ok(())
}

/// Handle WebSocket upgrade and forward to libp2p
async fn handle_websocket_upgrade(client: &mut TcpStream, target_addr: &str) -> Result<()> {
    // Read HTTP request
    let mut buffer = vec![0u8; 4096];
    let n = client.read(&mut buffer).await?;
    let request = String::from_utf8_lossy(&buffer[..n]);

    // Check for WebSocket upgrade
    if !request.contains("Upgrade: websocket") {
        // Not a WebSocket request, send error
        let response = "HTTP/1.1 400 Bad Request\r\n\r\n";
        client.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    // Extract WebSocket key
    let key = request
        .lines()
        .find(|line| line.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim())
        .ok_or_else(|| anyhow::anyhow!("Missing WebSocket key"))?;

    // Generate accept key
    let accept_key = generate_accept_key(key);

    // Send WebSocket upgrade response
    let response = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         \r\n",
        accept_key
    );
    client.write_all(response.as_bytes()).await?;

    info!("✅ WebSocket upgrade successful - forwarding to libp2p");

    // Connect to libp2p target
    let mut target = TcpStream::connect(target_addr).await?;
    info!("🔗 Connected to libp2p at {}", target_addr);

    // Bidirectional forwarding with WebSocket framing
    forward_websocket(client, &mut target).await?;

    Ok(())
}

/// Handle direct TCP forwarding
async fn handle_tcp_forward(mut client: TcpStream, target_addr: &str) -> Result<()> {
    // Connect to libp2p target
    let mut target = TcpStream::connect(target_addr).await?;
    info!("🔗 Direct TCP forward to {}", target_addr);

    // Bidirectional forwarding
    let (mut client_read, mut client_write) = client.split();
    let (mut target_read, mut target_write) = target.split();

    let client_to_target = tokio::io::copy(&mut client_read, &mut target_write);
    let target_to_client = tokio::io::copy(&mut target_read, &mut client_write);

    tokio::select! {
        result = client_to_target => {
            if let Err(e) = result {
                warn!("Client to target error: {}", e);
            }
        }
        result = target_to_client => {
            if let Err(e) = result {
                warn!("Target to client error: {}", e);
            }
        }
    }

    Ok(())
}

/// Forward data between WebSocket client and libp2p target
async fn forward_websocket(client: &mut TcpStream, target: &mut TcpStream) -> Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut target_read, mut target_write) = target.split();

    let mut client_buf = vec![0u8; 65536];
    let mut target_buf = vec![0u8; 65536];

    loop {
        tokio::select! {
            // Client to target
            result = client_read.read(&mut client_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        // Decode WebSocket frame
                        if let Some(payload) = decode_websocket_frame(&client_buf[..n]) {
                            target_write.write_all(&payload).await?;
                        }
                    }
                    Err(e) => {
                        warn!("Client read error: {}", e);
                        break;
                    }
                }
            }
            // Target to client
            result = target_read.read(&mut target_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        // Encode as WebSocket frame
                        let frame = encode_websocket_frame(&target_buf[..n]);
                        client_write.write_all(&frame).await?;
                    }
                    Err(e) => {
                        warn!("Target read error: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Generate WebSocket accept key from client key
fn generate_accept_key(key: &str) -> String {
    use sha2::{Sha256, Digest};
    
    const MAGIC: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    hasher.update(MAGIC.as_bytes());
    let hash = hasher.finalize();
    
    base64_encode(&hash)
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let b1 = chunk[0];
        let b2 = chunk.get(1).copied().unwrap_or(0);
        let b3 = chunk.get(2).copied().unwrap_or(0);
        
        result.push(CHARS[(b1 >> 2) as usize] as char);
        result.push(CHARS[(((b1 & 0x03) << 4) | (b2 >> 4)) as usize] as char);
        
        if chunk.len() > 1 {
            result.push(CHARS[(((b2 & 0x0F) << 2) | (b3 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(CHARS[(b3 & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

/// Decode WebSocket frame (simplified - handles binary frames only)
fn decode_websocket_frame(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 2 {
        return None;
    }

    let fin = (data[0] & 0x80) != 0;
    let opcode = data[0] & 0x0F;
    let masked = (data[1] & 0x80) != 0;
    let mut payload_len = (data[1] & 0x7F) as usize;
    let mut pos = 2;

    // Handle extended payload length
    if payload_len == 126 {
        if data.len() < 4 {
            return None;
        }
        payload_len = u16::from_be_bytes([data[2], data[3]]) as usize;
        pos = 4;
    } else if payload_len == 127 {
        if data.len() < 10 {
            return None;
        }
        payload_len = u64::from_be_bytes([
            data[2], data[3], data[4], data[5],
            data[6], data[7], data[8], data[9],
        ]) as usize;
        pos = 10;
    }

    // Get masking key if present
    let mask = if masked {
        if data.len() < pos + 4 {
            return None;
        }
        let mask = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
        pos += 4;
        Some(mask)
    } else {
        None
    };

    // Extract payload
    if data.len() < pos + payload_len {
        return None;
    }

    let mut payload = data[pos..pos + payload_len].to_vec();

    // Unmask if needed
    if let Some(mask) = mask {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }

    // Only handle binary frames (opcode 2) and continuation (0)
    if fin && (opcode == 2 || opcode == 0) {
        Some(payload)
    } else {
        None
    }
}

/// Encode data as WebSocket binary frame
fn encode_websocket_frame(data: &[u8]) -> Vec<u8> {
    let len = data.len();
    let mut frame = Vec::new();

    // FIN + binary opcode
    frame.push(0x82);

    // Payload length
    if len < 126 {
        frame.push(len as u8);
    } else if len < 65536 {
        frame.push(126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }

    // Payload
    frame.extend_from_slice(data);

    frame
}
