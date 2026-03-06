// Consensus module - MRBN's novel consensus mechanism

#![allow(dead_code)] // Will be used as we build out the system

use anyhow::Result;
use tracing::info;

// Re-export submodules
pub mod vrf;
pub mod committee;
pub mod block;
pub mod validation;
pub mod engine;

use block::{Block, Blockchain};
use committee::CommitteeSelector;
use crate::transaction::pool::TransactionPool;
use validation::ValidationCoordinator;

/// Consensus engine for MRBN
pub struct ConsensusEngine {
    /// Blockchain state
    blockchain: Blockchain,
    
    /// Transaction pool
    tx_pool: TransactionPool,
    
    /// Committee selector
    committee_selector: CommitteeSelector,
    
    /// Validation coordinator
    validation_coordinator: ValidationCoordinator,
    
    /// Current batch ID counter
    next_batch_id: u64,
    
    /// Batch size (transactions per batch)
    batch_size: usize,
}

impl ConsensusEngine {
    /// Create a new consensus engine
    pub fn new() -> Result<Self> {
        info!("🔧 Initializing Consensus Engine...");
        
        let blockchain = Blockchain::new();
        let tx_pool = TransactionPool::new(10000); // Max 10k pending transactions
        let committee_selector = CommitteeSelector::new(100, 10); // Assume 100 nodes, target 10 per committee
        let validation_coordinator = ValidationCoordinator::new(30); // 30 second timeout
        
        Ok(ConsensusEngine {
            blockchain,
            tx_pool,
            committee_selector,
            validation_coordinator,
            next_batch_id: 1,
            batch_size: 100, // 100 transactions per batch
        })
    }

    /// Start the consensus engine
    pub async fn run(&mut self) -> Result<()> {
        info!("✅ Consensus engine ready!");
        
        // TODO: Implement consensus loop
        // - VRF committee selection
        // - Transaction batch assignment
        // - Validation coordination
        // - Block creation
        
        Ok(())
    }
    
    /// Get current blockchain height
    pub fn height(&self) -> u64 {
        self.blockchain.height()
    }
    
    /// Get latest block
    pub fn latest_block(&self) -> &Block {
        self.blockchain.get_latest_block()
    }
    
    /// Get transaction pool size
    pub fn pending_transactions(&self) -> usize {
        self.tx_pool.size()
    }
}

