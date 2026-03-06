// Storage module - persistent blockchain and state using ParityDB

#![allow(dead_code)]

pub mod block_store;
pub mod state_store;

use anyhow::Result;
use parity_db::{Db, Options};
use std::path::Path;
use tracing::info;

/// Column IDs for ParityDB
pub const COL_BLOCKS: u8 = 0;
pub const COL_BLOCK_HEADERS: u8 = 1;
pub const COL_STATE: u8 = 2;
pub const COL_METADATA: u8 = 3;

/// Storage manager - coordinates block and state storage
pub struct StorageManager {
    /// ParityDB instance
    db: Db,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        info!("💾 Initializing ParityDB storage at: {:?}", path.as_ref());
        
        // Configure ParityDB with columns
        let mut options = Options::with_columns(path.as_ref(), 4);
        
        // Configure each column
        // COL_BLOCKS: Full block data (larger values)
        options.columns[COL_BLOCKS as usize].btree_index = false;
        
        // COL_BLOCK_HEADERS: Block headers only (smaller, frequent access)
        options.columns[COL_BLOCK_HEADERS as usize].btree_index = true;
        
        // COL_STATE: Account state (frequent reads/writes)
        options.columns[COL_STATE as usize].btree_index = true;
        
        // COL_METADATA: Chain metadata (height, etc.)
        options.columns[COL_METADATA as usize].btree_index = false;
        
        // Open database
        let db = Db::open_or_create(&options)?;
        
        info!("✅ ParityDB opened successfully with 4 columns");
        
        Ok(StorageManager { db })
    }
    
    /// Get database reference
    pub fn db(&self) -> &Db {
        &self.db
    }
    
    /// Get block store
    pub fn block_store(&self) -> block_store::BlockStore<'_> {
        block_store::BlockStore::new(&self.db)
    }
    
    /// Get state store
    pub fn state_store(&self) -> state_store::StateStore<'_> {
        state_store::StateStore::new(&self.db)
    }
    
    /// Get chain height (convenience method)
    pub fn get_chain_height(&self) -> Result<u64> {
        self.block_store().get_chain_height()
    }
    
    /// Close the database (called on shutdown)
    pub fn close(self) -> Result<()> {
        info!("🔒 Closing storage");
        drop(self.db);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path());
        assert!(storage.is_ok());
    }
}
