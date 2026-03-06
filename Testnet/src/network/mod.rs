// Network module - handles all P2P communication using libp2p

use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub, kad, mdns, noise, yamux, ping, relay, dcutr, identify,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, PeerId, Swarm, SwarmBuilder,
};
use std::time::Duration;
use tracing::{info, warn};

// Re-export submodules
pub mod messages;

// Define our network behaviour
#[derive(NetworkBehaviour)]
pub struct MrbnBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub ping: ping::Behaviour,
    pub relay_client: relay::client::Behaviour,
    pub dcutr: dcutr::Behaviour,
    pub identify: identify::Behaviour,
}

pub struct NetworkNode {
    swarm: Swarm<MrbnBehaviour>,
    local_peer_id: PeerId,
}

impl NetworkNode {
    /// Create a new MRBN network node
    pub fn new(external_address: Option<String>) -> Result<Self> {
        info!("🚀 Initializing MRBN Network Node...");

        // Generate node identity
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("📍 Local peer id: {}", local_peer_id);

        // Set up GossipSub for message broadcasting
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid GossipSub config");

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .expect("Valid GossipSub behaviour");

        // Set up mDNS for local peer discovery
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Set up Kademlia DHT for global peer discovery
        let mut kademlia_config = kad::Config::default();
        kademlia_config.set_query_timeout(Duration::from_secs(60));
        let store = kad::store::MemoryStore::new(local_peer_id);
        let mut kademlia = kad::Behaviour::with_config(local_peer_id, store, kademlia_config);

        // Bootstrap with known MRBN nodes (for now, we'll use local bootstrap)
        // In production, these would be public bootstrap nodes
        kademlia.set_mode(Some(kad::Mode::Server));

        info!("🌍 Kademlia DHT initialized for global discovery");

        // Subscribe to MRBN topics
        let tx_topic = gossipsub::IdentTopic::new("mrbn/transactions");
        let block_topic = gossipsub::IdentTopic::new("mrbn/blocks");
        let committee_topic = gossipsub::IdentTopic::new("mrbn/committee");

        // Create ping for connection health checks with custom config
        let ping_config = ping::Config::new()
            .with_interval(std::time::Duration::from_secs(15)); // Ping every 15 seconds
        let ping = ping::Behaviour::new(ping_config);
        
        // Create DCUtR for direct connection upgrade through relay
        let dcutr = dcutr::Behaviour::new(local_peer_id);
        
        // Create identify for peer information exchange (required for relay)
        let identify = identify::Behaviour::new(identify::Config::new(
            "/mrbn/1.0.0".to_string(),
            local_key.public(),
        ));
        
        info!("🔄 NAT traversal protocols initialized (DCUtR + Identify)");
        
        // Create the behaviour first (relay_client will be added by SwarmBuilder)
        let behaviour_components = (gossipsub, mdns, kademlia, ping, dcutr, identify);

        let (gossipsub, mdns, kademlia, ping, dcutr, identify) = behaviour_components;
        
        // Subscribe to topics
        let mut gossipsub_mut = gossipsub;
        gossipsub_mut.subscribe(&tx_topic)?;
        gossipsub_mut.subscribe(&block_topic)?;
        gossipsub_mut.subscribe(&committee_topic)?;

        info!("📡 Subscribed to topics: transactions, blocks, committee");

        // Build the swarm using the new builder API with relay transport
        let mut swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_dns()?
            .with_relay_client(noise::Config::new, yamux::Config::default)?
            .with_behaviour(|_, relay_client| {
                // Create behaviour with relay client from builder
                Ok(MrbnBehaviour {
                    gossipsub: gossipsub_mut,
                    mdns,
                    kademlia,
                    ping,
                    relay_client,
                    dcutr,
                    identify,
                })
            })?
            .with_swarm_config(|cfg| {
                cfg.with_idle_connection_timeout(std::time::Duration::from_secs(60))
            })
            .build();

        // Listen on TCP (for local P2P and direct connections)
        swarm.listen_on("/ip4/0.0.0.0/tcp/8333".parse()?)?;
        
        // Listen on WebSocket (for Railway/cloud connections through HTTP proxy)
        // Railway will route HTTP traffic on port 8334 to this WebSocket listener
        if let Ok(ws_addr) = "/ip4/0.0.0.0/tcp/8334/ws".parse() {
            if let Err(e) = swarm.listen_on(ws_addr) {
                warn!("⚠️ Could not listen on WebSocket (this is OK if not needed): {}", e);
            } else {
                info!("🌐 WebSocket enabled on port 8334 for cloud connectivity");
            }
        }
        
        info!("🎧 Listening on TCP port 8333 (P2P) and WebSocket port 8334 (cloud)...");

        // Create the node
        let mut node = NetworkNode {
            swarm,
            local_peer_id,
        };

        // Add external address if provided (for Railway, Render, etc.)
        if let Some(ext_addr) = external_address {
            info!("🌍 Adding external address: {}", ext_addr);
            if let Ok(addr) = ext_addr.parse() {
                node.swarm.add_external_address(addr);
            } else {
                warn!("Failed to parse external address: {}", ext_addr);
            }
        }

        // Connect to public relay servers for NAT traversal
        node.connect_to_public_relays()?;

        Ok(node)
    }

    /// Connect to public libp2p relay servers for NAT traversal
    fn connect_to_public_relays(&mut self) -> Result<()> {
        info!("🔄 Connecting to public relay servers for NAT traversal...");
        
        // List of public libp2p relay servers
        let relays = vec![
            // Use IP addresses instead of dnsaddr (more reliable)
            "/ip4/147.75.83.83/tcp/4001/p2p/12D3KooWDpJ7As7BWAwRMfu1VU2WCqNjvq387JEYKDBj4kx6nXTN",
            "/ip4/147.75.77.187/tcp/4001/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X",
        ];

        for relay_addr in relays {
            if let Ok(addr) = relay_addr.parse::<libp2p::Multiaddr>() {
                info!("🔗 Connecting to relay: {}", relay_addr);
                if let Err(e) = self.swarm.dial(addr) {
                    warn!("Failed to connect to relay {}: {}", relay_addr, e);
                }
            }
        }

        Ok(())
    }

    /// Get the local peer ID
    pub fn peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Bootstrap the Kademlia DHT to start discovering peers
    pub fn bootstrap(&mut self) -> Result<()> {
        info!("🔄 Bootstrapping Kademlia DHT...");
        match self.swarm.behaviour_mut().kademlia.bootstrap() {
            Ok(_) => {
                info!("✅ DHT bootstrap initiated successfully");
            }
            Err(e) => {
                info!("ℹ️ DHT bootstrap skipped: {} (normal for first node)", e);
            }
        }
        Ok(())
    }

    /// Add a known peer to connect to (for bootstrapping)
    pub fn add_peer(&mut self, peer_id: PeerId, addr: libp2p::Multiaddr) {
        info!("➕ Adding bootstrap peer: {} at {}", peer_id, addr);
        self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr.clone());
        // Also try to dial the peer directly
        if let Err(e) = self.swarm.dial(addr) {
            warn!("Failed to dial bootstrap peer: {}", e);
        }
    }
    
    /// Add peer from multiaddr string (format: /ip4/1.2.3.4/tcp/8333/p2p/12D3Koo...)
    pub fn add_peer_from_multiaddr(&mut self, multiaddr_str: &str) -> Result<()> {
        let multiaddr: libp2p::Multiaddr = multiaddr_str.parse()
            .map_err(|e| anyhow::anyhow!("Invalid multiaddr: {}", e))?;
        
        // Extract peer ID from multiaddr
        if let Some(libp2p::multiaddr::Protocol::P2p(peer_id)) = multiaddr.iter().last() {
            self.add_peer(peer_id, multiaddr);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Multiaddr does not contain peer ID"))
        }
    }

    /// Run the network event loop
    pub async fn run(&mut self) -> Result<()> {
        info!("✅ Network node is ready! Waiting for peers...");

        loop {
            match self.swarm.select_next_some().await {
                SwarmEvent::NewListenAddr { address, .. } => {
                    info!("🌐 Listening on {}", address);
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        info!("🔍 mDNS discovered peer: {} at {}", peer_id, multiaddr);
                        self.swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        
                        // Add to Kademlia routing table
                        self.swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                    }
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        warn!("⏰ mDNS peer expired: {}", peer_id);
                        self.swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Kademlia(kad::Event::RoutingUpdated {
                    peer,
                    is_new_peer,
                    addresses,
                    ..
                })) => {
                    if is_new_peer {
                        info!("🌍 Kademlia discovered new peer: {} with {} addresses", peer, addresses.len());
                        
                        // Actively dial the discovered peer
                        for addr in addresses.iter() {
                            info!("📞 Dialing discovered peer {} at {}", peer, addr);
                            if let Err(e) = self.swarm.dial(addr.clone()) {
                                warn!("❌ Failed to dial {}: {:?}", peer, e);
                            }
                        }
                    }
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Kademlia(kad::Event::InboundRequest { .. })) => {
                    // Kademlia received an inbound request (normal DHT operation)
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Ping(ping::Event { peer, result, .. })) => {
                    match result {
                        Ok(rtt) => {
                            // Successful ping - connection is healthy
                            if rtt.as_millis() > 1000 {
                                warn!("⚠️ High latency to {}: {}ms", peer, rtt.as_millis());
                            }
                        }
                        Err(e) => {
                            warn!("❌ Ping failed to {}: {:?}", peer, e);
                        }
                    }
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source,
                    message_id: _,
                    message,
                })) => {
                    // Try to decode MRBN message
                    match messages::MrbnMessage::from_bytes(&message.data) {
                        Ok(mrbn_msg) => {
                            info!(
                                "📨 Received {} from {}",
                                mrbn_msg.message_type(),
                                propagation_source
                            );
                            // TODO: Handle different message types
                            // - TransactionBroadcast: Add to tx pool
                            // - BlockAnnounce: Validate and add to chain
                            // - ValidationTask: Process if in committee
                            // - ValidationResult: Forward to consensus
                        }
                        Err(e) => {
                            warn!("❌ Failed to decode message: {}", e);
                        }
                    }
                }
                SwarmEvent::ConnectionEstablished {
                    peer_id, endpoint, ..
                } => {
                    info!("🤝 Connected to peer: {} at {}", peer_id, endpoint.get_remote_address());
                }
                SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                    warn!("❌ Connection closed with {}: {:?}", peer_id, cause);
                }
                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                    warn!("❌ Outgoing connection error to {:?}: {:?}", peer_id, error);
                }
                SwarmEvent::IncomingConnectionError { error, .. } => {
                    warn!("❌ Incoming connection error: {:?}", error);
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Identify(identify::Event::Received { peer_id, info })) => {
                    info!("🆔 Identified peer {}: Agent={}, Protocols={}", 
                        peer_id, info.agent_version, info.protocols.len());
                    
                    // Add all listen addresses to Kademlia
                    for addr in info.listen_addrs {
                        self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                    }
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Identify(identify::Event::Sent { .. })) => {
                    // Identify info sent to peer (normal operation)
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Identify(identify::Event::Pushed { .. })) => {
                    // Identify info pushed to peer (normal operation)
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Identify(identify::Event::Error { peer_id, error })) => {
                    warn!("❌ Identify error with {}: {:?}", peer_id, error);
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::RelayClient(relay::client::Event::ReservationReqAccepted { relay_peer_id, .. })) => {
                    info!("✅ Relay reservation accepted by {}", relay_peer_id);
                    info!("🔄 You can now be reached through this relay for NAT traversal");
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::RelayClient(event)) => {
                    info!("🔄 Relay event: {:?}", event);
                }
                SwarmEvent::Behaviour(MrbnBehaviourEvent::Dcutr(event)) => {
                    info!("⚡ DCUtR event: {:?}", event);
                }
                _ => {}
            }
        }
    }

    /// Broadcast a message to a topic
    #[allow(dead_code)] // Will be used for transaction/block broadcasting
    pub fn broadcast(&mut self, topic: &str, message: Vec<u8>) -> Result<()> {
        let topic = gossipsub::IdentTopic::new(topic);
        self.swarm
            .behaviour_mut()
            .gossipsub
            .publish(topic, message)?;
        Ok(())
    }
    
    /// Broadcast an MRBN message
    #[allow(dead_code)] // Will be used when consensus is integrated
    pub fn broadcast_message(&mut self, topic: &str, message: messages::MrbnMessage) -> Result<()> {
        let bytes = message.to_bytes()
            .map_err(|e| anyhow::anyhow!("Failed to serialize message: {}", e))?;
        self.broadcast(topic, bytes)
    }
    
    /// Broadcast a transaction
    #[allow(dead_code)] // Will be used when consensus is integrated
    pub fn broadcast_transaction(&mut self, tx: crate::transaction::Transaction) -> Result<()> {
        info!("📤 Broadcasting transaction");
        let msg = messages::MrbnMessage::TransactionBroadcast(tx);
        self.broadcast_message("mrbn/transactions", msg)
    }
    
    /// Broadcast a block
    #[allow(dead_code)] // Will be used when consensus is integrated
    pub fn broadcast_block(&mut self, block: crate::consensus::block::Block) -> Result<()> {
        info!("📤 Broadcasting block {}", block.number());
        let msg = messages::MrbnMessage::BlockAnnounce(block);
        self.broadcast_message("mrbn/blocks", msg)
    }
    
    /// Broadcast a validation task
    #[allow(dead_code)] // Will be used when consensus is integrated
    pub fn broadcast_validation_task(&mut self, task: crate::consensus::validation::ValidationTask) -> Result<()> {
        info!("📤 Broadcasting validation task for batch {}", task.batch_id);
        let msg = messages::MrbnMessage::ValidationTaskBroadcast(task);
        self.broadcast_message("mrbn/committee", msg)
    }
    
    /// Broadcast a validation result
    #[allow(dead_code)] // Will be used when consensus is integrated
    pub fn broadcast_validation_result(&mut self, result: crate::consensus::validation::ValidationResult) -> Result<()> {
        info!("📤 Broadcasting validation result for batch {}", result.batch_id);
        let msg = messages::MrbnMessage::ValidationResultBroadcast(result);
        self.broadcast_message("mrbn/committee", msg)
    }
}
