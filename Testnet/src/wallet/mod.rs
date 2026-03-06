// Wallet module - Kain wallet for MRBN

#![allow(dead_code)]

pub mod account;
pub mod keystore;

use crate::crypto::generate_keypair;
use crate::storage::state_store::StateStore;
use crate::transaction::Transaction;
use anyhow::Result;
use ed25519_dalek::SigningKey;
use tracing::info;

/// Wallet for managing Kain accounts
pub struct Wallet {
    /// Signing key for this wallet
    signing_key: SigningKey,
    
    /// Public address (derived from signing key)
    address: Vec<u8>,
}

impl Wallet {
    /// Create a new wallet with a random key
    pub fn new() -> Self {
        let signing_key = generate_keypair();
        let address = signing_key.verifying_key().as_bytes().to_vec();
        
        info!("💼 Created new wallet: {:?}...", &address[..8]);
        
        Wallet {
            signing_key,
            address,
        }
    }

    /// Create wallet from existing signing key
    pub fn from_key(signing_key: SigningKey) -> Self {
        let address = signing_key.verifying_key().as_bytes().to_vec();
        
        Wallet {
            signing_key,
            address,
        }
    }

    /// Get wallet address
    pub fn address(&self) -> &[u8] {
        &self.address
    }

    /// Get signing key reference
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Get balance from state store
    pub fn get_balance(&self, state_store: &StateStore) -> Result<u64> {
        state_store.get_balance(&self.address)
    }

    /// Get nonce from state store
    pub fn get_nonce(&self, state_store: &StateStore) -> Result<u64> {
        state_store.get_nonce(&self.address)
    }

    /// Create and sign a transaction
    pub fn create_transaction(
        &self,
        to: &[u8],
        amount: u64,
        gas: u64,
        state_store: &StateStore,
    ) -> Result<Transaction> {
        // Get current nonce
        let nonce = self.get_nonce(state_store)?;
        
        // Create transaction
        let mut tx = Transaction::new(
            self.address.clone(),
            to.to_vec(),
            amount,
            gas,
            nonce,
        );
        
        // Sign transaction
        tx.sign(&self.signing_key)?;
        
        info!(
            "📝 Created transaction: {} Kain to {:?}... (gas: {}, nonce: {})",
            amount,
            &to[..8],
            gas,
            nonce
        );
        
        Ok(tx)
    }

    /// Calculate gas fee for a transaction
    pub fn calculate_gas_fee(&self, _tx_size: usize) -> u64 {
        // Simple gas calculation
        // In production, this would be more sophisticated
        10 // Base gas fee
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageManager;
    use tempfile::TempDir;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert_eq!(wallet.address().len(), 32);
    }

    #[test]
    fn test_create_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let state_store = storage.state_store();

        let wallet = Wallet::new();
        let recipient = Wallet::new();

        // Set initial balance
        state_store.set_balance(wallet.address(), 1000).unwrap();

        // Create transaction
        let tx = wallet.create_transaction(
            recipient.address(),
            100,
            10,
            &state_store,
        ).unwrap();

        assert_eq!(tx.amount, 100);
        assert_eq!(tx.gas, 10);
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_get_balance() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let state_store = storage.state_store();

        let wallet = Wallet::new();

        // Initial balance should be 0
        assert_eq!(wallet.get_balance(&state_store).unwrap(), 0);

        // Set balance
        state_store.set_balance(wallet.address(), 500).unwrap();
        assert_eq!(wallet.get_balance(&state_store).unwrap(), 500);
    }
}
