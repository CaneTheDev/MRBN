// Consensus engine - orchestrates the entire consensus process

use super::block::{Block, Blockchain, CommitteeSignature};
use super::committee::{Committee, CommitteeSelector};
use super::validation::ValidationCoordinator;
use crate::transaction::{Transaction, pool::TransactionPool};
use anyhow::Result;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

/// Parallel batch processor - MRBN's key innovation
pub struct ParallelBatchProcessor {
    /// Active batches being processed
    active_batches: HashMap<u64, BatchProcessingState>,
    
    /// Maximum parallel batches
    max_parallel_batches: usize,
}

pub struct BatchProcessingState {
    pub batch_id: u64,
    pub committee: Committee,
    pub transactions: Vec<Transaction>,
    pub started_at: std::time::Instant,
}

impl ParallelBatchProcessor {
    /// Create a new parallel batch processor
    pub fn new(max_parallel_batches: usize) -> Self {
        info!("⚡ Initializing parallel batch processor: max {} batches", max_parallel_batches);
        
        ParallelBatchProcessor {
            active_batches: HashMap::new(),
            max_parallel_batches,
        }
    }

    /// Check if we can process more batches
    pub fn can_process_more(&self) -> bool {
        self.active_batches.len() < self.max_parallel_batches
    }

    /// Start processing a batch
    pub fn start_batch(
        &mut self,
        batch_id: u64,
        committee: Committee,
        transactions: Vec<Transaction>,
    ) -> Result<()> {
        if !self.can_process_more() {
            return Err(anyhow::anyhow!("Maximum parallel batches reached"));
        }

        info!(
            "🚀 Starting parallel batch {} with {} transactions",
            batch_id,
            transactions.len()
        );

        let state = BatchProcessingState {
            batch_id,
            committee,
            transactions,
            started_at: std::time::Instant::now(),
        };

        self.active_batches.insert(batch_id, state);
        Ok(())
    }

    /// Complete a batch
    fn complete_batch(&mut self, batch_id: u64) -> Option<BatchProcessingState> {
        if let Some(state) = self.active_batches.remove(&batch_id) {
            let elapsed = state.started_at.elapsed();
            info!(
                "✅ Completed batch {} in {:.2}s",
                batch_id,
                elapsed.as_secs_f64()
            );
            Some(state)
        } else {
            None
        }
    }

    /// Get active batch count
    pub fn active_count(&self) -> usize {
        self.active_batches.len()
    }

    /// Get batch processing time
    pub fn get_batch_duration(&self, batch_id: u64) -> Option<Duration> {
        self.active_batches
            .get(&batch_id)
            .map(|state| state.started_at.elapsed())
    }
}

/// Consensus orchestrator - ties everything together
pub struct ConsensusOrchestrator {
    /// Blockchain state
    blockchain: Blockchain,
    
    /// Transaction pool
    tx_pool: TransactionPool,
    
    /// Committee selector
    committee_selector: CommitteeSelector,
    
    /// Validation coordinator
    validation_coordinator: ValidationCoordinator,
    
    /// Parallel batch processor
    batch_processor: ParallelBatchProcessor,
    
    /// Next batch ID
    next_batch_id: u64,
    
    /// Batch size (transactions per batch)
    batch_size: usize,
    
    /// Block creation interval (seconds)
    block_interval: u64,
}

impl ConsensusOrchestrator {
    /// Create a new consensus orchestrator
    pub fn new(
        network_size: usize,
        target_committee_size: usize,
        batch_size: usize,
        max_parallel_batches: usize,
    ) -> Self {
        info!("🎯 Initializing consensus orchestrator");
        info!("   Network size: {}", network_size);
        info!("   Target committee size: {}", target_committee_size);
        info!("   Batch size: {} transactions", batch_size);
        info!("   Max parallel batches: {}", max_parallel_batches);
        
        let blockchain = Blockchain::new();
        let tx_pool = TransactionPool::new(10000);
        let committee_selector = CommitteeSelector::new(network_size, target_committee_size);
        let validation_coordinator = ValidationCoordinator::new(30);
        let batch_processor = ParallelBatchProcessor::new(max_parallel_batches);
        
        ConsensusOrchestrator {
            blockchain,
            tx_pool,
            committee_selector,
            validation_coordinator,
            batch_processor,
            next_batch_id: 1,
            batch_size,
            block_interval: 10, // 10 seconds per block
        }
    }

    /// Add transaction to pool
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        self.tx_pool.add(tx)
    }

    /// Process pending transactions into batches
    pub fn process_batches(&mut self) -> Result<Vec<u64>> {
        let mut created_batches = Vec::new();

        // Process batches while we have capacity and transactions
        while self.batch_processor.can_process_more() && !self.tx_pool.is_empty() {
            // Get a batch of transactions
            let transactions = self.tx_pool.get_batch(self.batch_size);
            if transactions.is_empty() {
                break;
            }

            let batch_id = self.next_batch_id;
            self.next_batch_id += 1;

            // Create committee for this batch
            let previous_block = self.blockchain.get_latest_block();
            let committee = self.committee_selector.create_committee(
                batch_id,
                previous_block.hash().as_bytes(),
            );

            info!(
                "📦 Created batch {} with {} transactions, committee size {}",
                batch_id,
                transactions.len(),
                committee.size()
            );

            // Start validation
            self.validation_coordinator.start_validation(
                batch_id,
                transactions.clone(),
                &committee,
            )?;

            // Start parallel processing
            self.batch_processor.start_batch(batch_id, committee, transactions)?;

            created_batches.push(batch_id);
        }

        Ok(created_batches)
    }

    /// Check for completed batches and create blocks
    pub fn finalize_batches(&mut self) -> Result<Vec<Block>> {
        let completed = self.validation_coordinator.get_completed_batches();
        let mut new_blocks = Vec::new();

        for batch_id in completed {
            if let Some(validation) = self.validation_coordinator.remove_batch(batch_id) {
                // Check if approved
                if validation.status == super::validation::BatchStatus::Approved {
                    info!("✅ Batch {} approved, creating block", batch_id);

                    // Remove transactions from pool
                    let tx_hashes: Vec<String> = validation.task.transactions
                        .iter()
                        .filter_map(|tx| tx.hash.clone())
                        .collect();
                    self.tx_pool.remove_batch(&tx_hashes);

                    // Create block
                    let block_number = self.blockchain.height() + 1;
                    let previous_hash = self.blockchain.get_latest_block().hash().to_string();
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    let mut block = Block::new(
                        block_number,
                        previous_hash,
                        validation.task.transactions,
                        timestamp,
                    );

                    // Add committee signatures (simplified - in real implementation, collect actual signatures)
                    for (public_key, result) in validation.results {
                        if result.is_valid {
                            let sig = CommitteeSignature {
                                public_key,
                                signature: result.signature,
                                vrf_proof: vec![0u8; 64], // Would be from committee member
                                vrf_preout: vec![0u8; 32],
                            };
                            block.add_signature(sig);
                        }
                    }

                    // Add block to chain
                    self.blockchain.add_block(block.clone())?;
                    new_blocks.push(block);

                    info!("🎉 Block {} created and added to chain", block_number);
                } else {
                    warn!("❌ Batch {} rejected or timed out", batch_id);
                }

                // Complete batch processing
                self.batch_processor.complete_batch(batch_id);
            }
        }

        // Cleanup old committees
        self.committee_selector.cleanup_old_committees(self.next_batch_id, 100);

        Ok(new_blocks)
    }

    /// Run consensus loop
    pub async fn run(&mut self) -> Result<()> {
        info!("🚀 Starting consensus loop");

        loop {
            // Process new batches
            if let Ok(batches) = self.process_batches() {
                if !batches.is_empty() {
                    info!("📦 Processing {} batches in parallel", batches.len());
                }
            }

            // Finalize completed batches
            if let Ok(blocks) = self.finalize_batches() {
                if !blocks.is_empty() {
                    info!("🎉 Created {} new blocks", blocks.len());
                }
            }

            // Sleep briefly
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get blockchain height
    pub fn height(&self) -> u64 {
        self.blockchain.height()
    }

    /// Get pending transaction count
    pub fn pending_transactions(&self) -> usize {
        self.tx_pool.size()
    }

    /// Get active batch count
    pub fn active_batches(&self) -> usize {
        self.batch_processor.active_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    fn create_test_transaction() -> Transaction {
        let sender = generate_keypair();
        let recipient = generate_keypair();

        let mut tx = Transaction::new(
            sender.verifying_key().as_bytes().to_vec(),
            recipient.verifying_key().as_bytes().to_vec(),
            1000,
            10,
            1,
        );

        tx.sign(&sender).unwrap();
        tx
    }

    #[test]
    fn test_parallel_batch_processor() {
        let processor = ParallelBatchProcessor::new(3);
        
        assert!(processor.can_process_more());
        assert_eq!(processor.active_count(), 0);
    }

    #[test]
    fn test_consensus_orchestrator_creation() {
        let orchestrator = ConsensusOrchestrator::new(100, 10, 100, 3);
        
        assert_eq!(orchestrator.height(), 0);
        assert_eq!(orchestrator.pending_transactions(), 0);
        assert_eq!(orchestrator.active_batches(), 0);
    }

    #[test]
    fn test_add_transaction() {
        let mut orchestrator = ConsensusOrchestrator::new(100, 10, 100, 3);
        let tx = create_test_transaction();
        
        assert!(orchestrator.add_transaction(tx).is_ok());
        assert_eq!(orchestrator.pending_transactions(), 1);
    }
}
