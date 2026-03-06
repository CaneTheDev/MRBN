// Integrated MRBN Node - connects all components

#![allow(dead_code)]

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

use crate::network::NetworkNode;
use crate::consensus::engine::ConsensusOrchestrator;
use crate::storage::StorageManager;
use crate::validator::{Validator, ValidatorConfig};
use crate::transaction::Transaction;
use crate::cli::CliConfig;

/// Integrated MRBN node that coordinates all components
pub struct MrbnNode {
    /// Network layer
    network: NetworkNode,
    /// Consensus orchestrator
    consensus: Arc<RwLock<ConsensusOrchestrator>>,
    /// Storage manager
    storage: Arc<StorageManager>,
    /// Validator (optional)
    validator: Option<Validator>,
    /// Configuration
    #[allow(dead_code)]
    config: CliConfig,
}

impl MrbnNode {
    /// Create a new MRBN node
    pub fn new(config: CliConfig) -> Result<Self> {
        info!("Initializing MRBN node...");

        // Initialize storage
        let storage = Arc::new(StorageManager::new(&config.data_dir)?);
        info!("✅ Storage initialized at {:?}", config.data_dir);

        // Initialize network
        let mut network = NetworkNode::new()?;
        info!("✅ Network initialized (Peer ID: {})", network.peer_id());
        
        // Add bootstrap peers if provided
        for bootstrap_addr in &config.bootstrap {
            info!("🔗 Connecting to bootstrap node: {}", bootstrap_addr);
            if let Err(e) = network.add_peer_from_multiaddr(bootstrap_addr) {
                warn!("Failed to add bootstrap peer: {}", e);
            }
        }

        // Initialize consensus orchestrator
        let consensus = Arc::new(RwLock::new(ConsensusOrchestrator::new(
            100,  // network_size
            10,   // batch_size
            100,  // max_pool_size
            3,    // max_parallel_batches
        )));
        info!("✅ Consensus orchestrator initialized");

        // Initialize validator if enabled
        let validator = if config.validator {
            let validator_config = ValidatorConfig::default();
            let validator = Validator::new(validator_config)?;
            info!("✅ Validator initialized");
            Some(validator)
        } else {
            info!("⚠️  Validator disabled");
            None
        };

        Ok(Self {
            network,
            consensus,
            storage,
            validator,
            config,
        })
    }

    /// Start the node
    pub async fn start(&mut self) -> Result<()> {
        info!("🚀 Starting MRBN node...");

        // Bootstrap network
        self.network.bootstrap()?;
        info!("🔄 DHT bootstrap initiated");

        // Start validator if enabled
        if let Some(ref mut validator) = self.validator {
            validator.start()?;
            info!("✅ Validator started");
        }

        // Load chain state from storage
        let chain_height = self.storage.get_chain_height()?;
        info!("📊 Chain height: {}", chain_height);

        info!("✅ Node started successfully!");
        info!("");

        Ok(())
    }

    /// Run the node event loop
    pub async fn run(&mut self) -> Result<()> {
        info!("🔄 Starting event loop...");

        // Create genesis block if needed
        let chain_height = self.storage.get_chain_height()?;
        if chain_height == 0 {
            info!("🌱 Creating genesis block...");
            let genesis = crate::consensus::block::Block::genesis();
            self.storage.block_store().store_block(&genesis)?;
            info!("✅ Genesis block created and stored");
        }

        // Clone Arc references for the async tasks
        let consensus = Arc::clone(&self.consensus);
        let storage = Arc::clone(&self.storage);

        // Spawn consensus task
        let consensus_handle = tokio::spawn(async move {
            loop {
                // Process batches and create blocks
                {
                    let mut consensus_lock = consensus.write().await;
                    
                    // Process new batches
                    if let Ok(batches) = consensus_lock.process_batches() {
                        if !batches.is_empty() {
                            info!("📦 Processing {} batches", batches.len());
                        }
                    }

                    // Finalize completed batches and create blocks
                    if let Ok(blocks) = consensus_lock.finalize_batches() {
                        for block in blocks {
                            // Persist block to storage
                            if let Err(e) = storage.block_store().store_block(&block) {
                                warn!("Failed to store block {}: {}", block.number(), e);
                            } else {
                                info!("💾 Block {} persisted to storage", block.number());
                            }
                        }
                    }
                }

                // Sleep briefly
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });

        // Run network event loop (this blocks)
        tokio::select! {
            result = self.network.run() => {
                warn!("Network event loop ended: {:?}", result);
                result
            }
            result = consensus_handle => {
                warn!("Consensus task ended: {:?}", result);
                Err(anyhow::anyhow!("Consensus task ended unexpectedly"))
            }
        }
    }

    /// Add a transaction to the pool
    pub async fn add_transaction(&self, tx: Transaction) -> Result<()> {
        let mut consensus = self.consensus.write().await;
        consensus.add_transaction(tx)?;
        Ok(())
    }

    /// Get current chain height
    pub fn chain_height(&self) -> Result<u64> {
        self.storage.get_chain_height()
    }

    /// Get storage manager
    pub fn storage(&self) -> &Arc<StorageManager> {
        &self.storage
    }

    /// Get consensus orchestrator
    pub fn consensus(&self) -> &Arc<RwLock<ConsensusOrchestrator>> {
        &self.consensus
    }
}
