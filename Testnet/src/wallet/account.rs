// Account management - multiple accounts in a wallet

#![allow(dead_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account name (user-friendly label)
    pub name: String,
    
    /// Public address
    pub address: Vec<u8>,
    
    /// Account index (for HD wallets in the future)
    pub index: u32,
}

impl Account {
    /// Create a new account
    pub fn new(name: String, address: Vec<u8>, index: u32) -> Self {
        Account {
            name,
            address,
            index,
        }
    }

    /// Get address as hex string
    pub fn address_hex(&self) -> String {
        hex::encode(&self.address)
    }
}

/// Account manager for handling multiple accounts
pub struct AccountManager {
    /// List of accounts
    accounts: Vec<Account>,
    
    /// Active account index
    active_index: usize,
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Self {
        AccountManager {
            accounts: Vec::new(),
            active_index: 0,
        }
    }

    /// Add a new account
    pub fn add_account(&mut self, name: String, address: Vec<u8>) -> Result<usize> {
        let index = self.accounts.len() as u32;
        let account = Account::new(name, address, index);
        self.accounts.push(account);
        Ok(index as usize)
    }

    /// Get account by index
    pub fn get_account(&self, index: usize) -> Option<&Account> {
        self.accounts.get(index)
    }

    /// Get active account
    pub fn get_active_account(&self) -> Option<&Account> {
        self.accounts.get(self.active_index)
    }

    /// Set active account
    pub fn set_active_account(&mut self, index: usize) -> Result<()> {
        if index >= self.accounts.len() {
            return Err(anyhow::anyhow!("Account index out of bounds"));
        }
        self.active_index = index;
        Ok(())
    }

    /// List all accounts
    pub fn list_accounts(&self) -> &[Account] {
        &self.accounts
    }

    /// Get account count
    pub fn account_count(&self) -> usize {
        self.accounts.len()
    }

    /// Find account by address
    pub fn find_by_address(&self, address: &[u8]) -> Option<usize> {
        self.accounts
            .iter()
            .position(|acc| acc.address == address)
    }

    /// Remove account by index
    pub fn remove_account(&mut self, index: usize) -> Result<Account> {
        if index >= self.accounts.len() {
            return Err(anyhow::anyhow!("Account index out of bounds"));
        }
        
        let account = self.accounts.remove(index);
        
        // Adjust active index if necessary
        if self.active_index >= self.accounts.len() && !self.accounts.is_empty() {
            self.active_index = self.accounts.len() - 1;
        }
        
        Ok(account)
    }
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    #[test]
    fn test_account_manager() {
        let mut manager = AccountManager::new();
        
        let key1 = generate_keypair();
        let addr1 = key1.verifying_key().as_bytes().to_vec();
        
        let key2 = generate_keypair();
        let addr2 = key2.verifying_key().as_bytes().to_vec();
        
        // Add accounts
        manager.add_account("Main".to_string(), addr1.clone()).unwrap();
        manager.add_account("Savings".to_string(), addr2.clone()).unwrap();
        
        assert_eq!(manager.account_count(), 2);
        
        // Get account
        let account = manager.get_account(0).unwrap();
        assert_eq!(account.name, "Main");
        assert_eq!(account.address, addr1);
        
        // Active account
        let active = manager.get_active_account().unwrap();
        assert_eq!(active.name, "Main");
        
        // Switch active
        manager.set_active_account(1).unwrap();
        let active = manager.get_active_account().unwrap();
        assert_eq!(active.name, "Savings");
    }

    #[test]
    fn test_find_by_address() {
        let mut manager = AccountManager::new();
        
        let key = generate_keypair();
        let addr = key.verifying_key().as_bytes().to_vec();
        
        manager.add_account("Test".to_string(), addr.clone()).unwrap();
        
        let index = manager.find_by_address(&addr);
        assert_eq!(index, Some(0));
    }
}
