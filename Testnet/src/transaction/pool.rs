// Transaction pool (mempool) - holds pending transactions

#![allow(dead_code)] // Will be used as we build out the system

use super::Transaction;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::{info, warn};

/// Transaction pool for pending transactions
pub struct TransactionPool {
    /// Pending transactions by hash
    transactions: HashMap<String, Transaction>,
    
    /// Maximum pool size
    max_size: usize,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        info!("💧 Creating transaction pool with max size: {}", max_size);
        TransactionPool {
            transactions: HashMap::new(),
            max_size,
        }
    }

    /// Add a transaction to the pool
    pub fn add(&mut self, tx: Transaction) -> Result<()> {
        // Validate transaction
        tx.validate()?;

        // Check pool size
        if self.transactions.len() >= self.max_size {
            return Err(anyhow!("Transaction pool is full"));
        }

        // Get transaction hash
        let hash = tx.hash.clone().ok_or_else(|| anyhow!("Transaction has no hash"))?;

        // Check if already exists
        if self.transactions.contains_key(&hash) {
            return Err(anyhow!("Transaction already in pool"));
        }

        // Add to pool
        self.transactions.insert(hash.clone(), tx);
        info!("✅ Added transaction to pool: {}", hash);

        Ok(())
    }

    /// Remove a transaction from the pool
    pub fn remove(&mut self, hash: &str) -> Option<Transaction> {
        self.transactions.remove(hash)
    }

    /// Get a transaction by hash
    pub fn get(&self, hash: &str) -> Option<&Transaction> {
        self.transactions.get(hash)
    }

    /// Get all pending transactions
    pub fn get_all(&self) -> Vec<Transaction> {
        self.transactions.values().cloned().collect()
    }

    /// Get a batch of transactions for validation
    pub fn get_batch(&self, size: usize) -> Vec<Transaction> {
        self.transactions
            .values()
            .take(size)
            .cloned()
            .collect()
    }

    /// Remove multiple transactions (after they're included in a block)
    pub fn remove_batch(&mut self, hashes: &[String]) {
        for hash in hashes {
            if self.transactions.remove(hash).is_some() {
                info!("🗑️ Removed transaction from pool: {}", hash);
            }
        }
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Clear all transactions
    pub fn clear(&mut self) {
        warn!("🧹 Clearing transaction pool");
        self.transactions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    fn create_test_transaction(nonce: u64) -> Transaction {
        let sender = generate_keypair();
        let recipient = generate_keypair();

        let mut tx = Transaction::new(
            sender.verifying_key().as_bytes().to_vec(),
            recipient.verifying_key().as_bytes().to_vec(),
            1000,
            10,
            nonce,
        );

        tx.sign(&sender).unwrap();
        tx
    }

    #[test]
    fn test_add_transaction() {
        let mut pool = TransactionPool::new(100);
        let tx = create_test_transaction(1);

        assert!(pool.add(tx).is_ok());
        assert_eq!(pool.size(), 1);
    }

    #[test]
    fn test_remove_transaction() {
        let mut pool = TransactionPool::new(100);
        let tx = create_test_transaction(1);
        let hash = tx.hash.clone().unwrap();

        pool.add(tx).unwrap();
        assert_eq!(pool.size(), 1);

        let removed = pool.remove(&hash);
        assert!(removed.is_some());
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_get_batch() {
        let mut pool = TransactionPool::new(100);

        for i in 0..10 {
            let tx = create_test_transaction(i);
            pool.add(tx).unwrap();
        }

        let batch = pool.get_batch(5);
        assert_eq!(batch.len(), 5);
    }

    #[test]
    fn test_pool_full() {
        let mut pool = TransactionPool::new(2);

        let tx1 = create_test_transaction(1);
        let tx2 = create_test_transaction(2);
        let tx3 = create_test_transaction(3);

        assert!(pool.add(tx1).is_ok());
        assert!(pool.add(tx2).is_ok());
        assert!(pool.add(tx3).is_err()); // Pool is full
    }
}
