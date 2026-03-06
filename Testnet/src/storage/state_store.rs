// State storage - account balances, nonces, and state root

#![allow(dead_code)]

use crate::crypto::{hash_data, to_hex};
use crate::storage::COL_STATE;
use anyhow::{anyhow, Result};
use parity_db::Db;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Account state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountState {
    /// Account balance (in smallest unit)
    pub balance: u64,
    
    /// Transaction nonce (prevents replay attacks)
    pub nonce: u64,
}

impl AccountState {
    /// Create a new account state
    pub fn new(balance: u64, nonce: u64) -> Self {
        AccountState { balance, nonce }
    }
    
    /// Create an empty account
    pub fn empty() -> Self {
        AccountState {
            balance: 0,
            nonce: 0,
        }
    }
}

/// State store for account data
pub struct StateStore<'a> {
    db: &'a Db,
}

impl<'a> StateStore<'a> {
    /// Create a new state store
    pub fn new(db: &'a Db) -> Self {
        StateStore { db }
    }

    /// Get account state
    pub fn get_account(&self, address: &[u8]) -> Result<AccountState> {
        let key = Self::account_key(address);
        
        match self.db.get(COL_STATE, &key)? {
            Some(bytes) => {
                let state: AccountState = bincode::deserialize(&bytes)
                    .map_err(|e| anyhow!("Failed to deserialize account state: {}", e))?;
                Ok(state)
            }
            None => Ok(AccountState::empty()),
        }
    }

    /// Set account state
    pub fn set_account(&self, address: &[u8], state: &AccountState) -> Result<()> {
        let key = Self::account_key(address);
        let bytes = bincode::serialize(state)
            .map_err(|e| anyhow!("Failed to serialize account state: {}", e))?;
        
        self.db.commit(vec![
            (COL_STATE, key, Some(bytes)),
        ])?;
        
        Ok(())
    }

    /// Get account balance
    pub fn get_balance(&self, address: &[u8]) -> Result<u64> {
        let state = self.get_account(address)?;
        Ok(state.balance)
    }

    /// Set account balance
    pub fn set_balance(&self, address: &[u8], balance: u64) -> Result<()> {
        let mut state = self.get_account(address)?;
        state.balance = balance;
        self.set_account(address, &state)
    }

    /// Get account nonce
    pub fn get_nonce(&self, address: &[u8]) -> Result<u64> {
        let state = self.get_account(address)?;
        Ok(state.nonce)
    }

    /// Increment account nonce
    pub fn increment_nonce(&self, address: &[u8]) -> Result<()> {
        let mut state = self.get_account(address)?;
        state.nonce += 1;
        self.set_account(address, &state)
    }

    /// Transfer balance between accounts
    pub fn transfer(&self, from: &[u8], to: &[u8], amount: u64) -> Result<()> {
        // Get sender state
        let mut from_state = self.get_account(from)?;
        
        // Check sufficient balance
        if from_state.balance < amount {
            return Err(anyhow!("Insufficient balance"));
        }
        
        // Get recipient state
        let mut to_state = self.get_account(to)?;
        
        // Update balances
        from_state.balance -= amount;
        to_state.balance += amount;
        
        // Store updated states in a transaction
        let from_key = Self::account_key(from);
        let to_key = Self::account_key(to);
        
        let from_bytes = bincode::serialize(&from_state)?;
        let to_bytes = bincode::serialize(&to_state)?;
        
        self.db.commit(vec![
            (COL_STATE, from_key, Some(from_bytes)),
            (COL_STATE, to_key, Some(to_bytes)),
        ])?;
        
        info!("💸 Transferred {} from {:?} to {:?}", amount, &from[..8], &to[..8]);
        
        Ok(())
    }

    /// Apply transaction to state (transfer + gas + nonce)
    pub fn apply_transaction(
        &self,
        from: &[u8],
        to: &[u8],
        amount: u64,
        gas: u64,
    ) -> Result<()> {
        // Get sender state
        let mut from_state = self.get_account(from)?;
        
        // Check sufficient balance (amount + gas)
        let total_cost = amount + gas;
        if from_state.balance < total_cost {
            return Err(anyhow!("Insufficient balance for transaction + gas"));
        }
        
        // Get recipient state
        let mut to_state = self.get_account(to)?;
        
        // Update balances
        from_state.balance -= total_cost;
        from_state.nonce += 1;
        to_state.balance += amount;
        // Note: gas is distributed to validators separately
        
        // Store updated states
        let from_key = Self::account_key(from);
        let to_key = Self::account_key(to);
        
        let from_bytes = bincode::serialize(&from_state)?;
        let to_bytes = bincode::serialize(&to_state)?;
        
        self.db.commit(vec![
            (COL_STATE, from_key, Some(from_bytes)),
            (COL_STATE, to_key, Some(to_bytes)),
        ])?;
        
        Ok(())
    }

    /// Distribute gas fees to validators
    pub fn distribute_gas(&self, validators: &[Vec<u8>], total_gas: u64) -> Result<()> {
        if validators.is_empty() {
            return Ok(());
        }
        
        let gas_per_validator = total_gas / validators.len() as u64;
        
        let mut changes = Vec::new();
        
        for validator in validators {
            let mut state = self.get_account(validator)?;
            state.balance += gas_per_validator;
            
            let key = Self::account_key(validator);
            let bytes = bincode::serialize(&state)?;
            changes.push((COL_STATE, key, Some(bytes)));
        }
        
        self.db.commit(changes)?;
        
        info!("💰 Distributed {} gas to {} validators", total_gas, validators.len());
        
        Ok(())
    }

    /// Calculate state root (Merkle root of all accounts)
    pub fn calculate_state_root(&self) -> Result<String> {
        // Simple implementation: hash all account states together
        // In production, this would be a proper Merkle Patricia Trie
        
        let all_data = Vec::new();
        
        // Iterate through all accounts in state column
        // Note: ParityDB doesn't have a direct iterator, so we'd need to track accounts separately
        // For now, return a placeholder
        
        let hash = hash_data(&all_data);
        Ok(to_hex(&hash))
    }

    /// Generate account key
    fn account_key(address: &[u8]) -> Vec<u8> {
        let mut key = b"account_".to_vec();
        key.extend_from_slice(address);
        key
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::generate_keypair;
    use crate::storage::StorageManager;
    use tempfile::TempDir;

    #[test]
    fn test_account_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let state_store = storage.state_store();

        let keypair = generate_keypair();
        let verifying_key = keypair.verifying_key();
        let address = verifying_key.as_bytes();

        // Initial state should be empty
        let state = state_store.get_account(address).unwrap();
        assert_eq!(state.balance, 0);
        assert_eq!(state.nonce, 0);

        // Set balance
        state_store.set_balance(address, 1000).unwrap();
        let balance = state_store.get_balance(address).unwrap();
        assert_eq!(balance, 1000);
    }

    #[test]
    fn test_transfer() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let state_store = storage.state_store();

        let sender = generate_keypair();
        let recipient = generate_keypair();
        
        let sender_vk = sender.verifying_key();
        let recipient_vk = recipient.verifying_key();
        let sender_addr = sender_vk.as_bytes();
        let recipient_addr = recipient_vk.as_bytes();

        // Set initial balance
        state_store.set_balance(sender_addr, 1000).unwrap();

        // Transfer
        state_store.transfer(sender_addr, recipient_addr, 300).unwrap();

        // Check balances
        assert_eq!(state_store.get_balance(sender_addr).unwrap(), 700);
        assert_eq!(state_store.get_balance(recipient_addr).unwrap(), 300);
    }

    #[test]
    fn test_insufficient_balance() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let state_store = storage.state_store();

        let sender = generate_keypair();
        let recipient = generate_keypair();
        
        let sender_vk = sender.verifying_key();
        let recipient_vk = recipient.verifying_key();
        let sender_addr = sender_vk.as_bytes();
        let recipient_addr = recipient_vk.as_bytes();

        // Try to transfer without balance
        let result = state_store.transfer(sender_addr, recipient_addr, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_nonce_increment() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let state_store = storage.state_store();

        let keypair = generate_keypair();
        let verifying_key = keypair.verifying_key();
        let address = verifying_key.as_bytes();

        // Initial nonce is 0
        assert_eq!(state_store.get_nonce(address).unwrap(), 0);

        // Increment nonce
        state_store.increment_nonce(address).unwrap();
        assert_eq!(state_store.get_nonce(address).unwrap(), 1);

        state_store.increment_nonce(address).unwrap();
        assert_eq!(state_store.get_nonce(address).unwrap(), 2);
    }
}
