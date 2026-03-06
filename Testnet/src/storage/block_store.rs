// Block storage - persistent block and blockchain data

#![allow(dead_code)]

use crate::consensus::block::{Block, BlockHeader};
use crate::storage::{COL_BLOCKS, COL_BLOCK_HEADERS, COL_METADATA};
use anyhow::{anyhow, Result};
use parity_db::Db;
use tracing::{info, warn};

/// Metadata keys
const KEY_CHAIN_HEIGHT: &[u8] = b"chain_height";
const KEY_GENESIS_HASH: &[u8] = b"genesis_hash";

/// Block store for persistent blockchain storage
pub struct BlockStore<'a> {
    db: &'a Db,
}

impl<'a> BlockStore<'a> {
    /// Create a new block store
    pub fn new(db: &'a Db) -> Self {
        BlockStore { db }
    }

    /// Store a block
    pub fn store_block(&self, block: &Block) -> Result<()> {
        let block_number = block.number();
        
        // Serialize block
        let block_bytes = block.to_bytes()?;
        
        // Store full block
        let block_key = Self::block_key(block_number);
        self.db.commit(vec![
            (COL_BLOCKS, block_key.clone(), Some(block_bytes)),
        ])?;
        
        // Store block header separately for quick access
        let header_bytes = bincode::serialize(&block.header)
            .map_err(|e| anyhow!("Failed to serialize header: {}", e))?;
        let header_key = Self::header_key(block_number);
        self.db.commit(vec![
            (COL_BLOCK_HEADERS, header_key, Some(header_bytes)),
        ])?;
        
        // Update chain height
        let height_bytes = block_number.to_le_bytes().to_vec();
        self.db.commit(vec![
            (COL_METADATA, KEY_CHAIN_HEIGHT.to_vec(), Some(height_bytes)),
        ])?;
        
        // Store genesis hash if this is block 0
        if block_number == 0 {
            self.db.commit(vec![
                (COL_METADATA, KEY_GENESIS_HASH.to_vec(), Some(block.hash().as_bytes().to_vec())),
            ])?;
        }
        
        info!("💾 Stored block {} (hash: {})", block_number, block.hash());
        
        Ok(())
    }

    /// Get a block by number
    pub fn get_block(&self, block_number: u64) -> Result<Option<Block>> {
        let key = Self::block_key(block_number);
        
        match self.db.get(COL_BLOCKS, &key)? {
            Some(bytes) => {
                let block = Block::from_bytes(&bytes)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Get a block header by number
    pub fn get_header(&self, block_number: u64) -> Result<Option<BlockHeader>> {
        let key = Self::header_key(block_number);
        
        match self.db.get(COL_BLOCK_HEADERS, &key)? {
            Some(bytes) => {
                let header: BlockHeader = bincode::deserialize(&bytes)
                    .map_err(|e| anyhow!("Failed to deserialize header: {}", e))?;
                Ok(Some(header))
            }
            None => Ok(None),
        }
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> Result<Option<Block>> {
        let height = self.get_chain_height()?;
        if height == 0 {
            // Return genesis block
            self.get_block(0)
        } else {
            self.get_block(height)
        }
    }

    /// Get chain height (latest block number)
    pub fn get_chain_height(&self) -> Result<u64> {
        match self.db.get(COL_METADATA, KEY_CHAIN_HEIGHT)? {
            Some(bytes) => {
                let height_array: [u8; 8] = bytes[..8].try_into()
                    .map_err(|_| anyhow!("Invalid height bytes"))?;
                let height = u64::from_le_bytes(height_array);
                Ok(height)
            }
            None => Ok(0), // No blocks yet, return 0
        }
    }

    /// Get genesis hash
    pub fn get_genesis_hash(&self) -> Result<Option<String>> {
        match self.db.get(COL_METADATA, KEY_GENESIS_HASH)? {
            Some(bytes) => {
                let hash = String::from_utf8(bytes.to_vec())
                    .map_err(|e| anyhow!("Invalid genesis hash: {}", e))?;
                Ok(Some(hash))
            }
            None => Ok(None),
        }
    }

    /// Check if a block exists
    pub fn has_block(&self, block_number: u64) -> Result<bool> {
        let key = Self::block_key(block_number);
        Ok(self.db.get(COL_BLOCKS, &key)?.is_some())
    }

    /// Get block count
    pub fn block_count(&self) -> Result<u64> {
        let height = self.get_chain_height()?;
        Ok(height + 1) // Height 0 = 1 block (genesis)
    }

    /// Delete a block (for reorganization or pruning)
    pub fn delete_block(&self, block_number: u64) -> Result<()> {
        let block_key = Self::block_key(block_number);
        let header_key = Self::header_key(block_number);
        
        self.db.commit(vec![
            (COL_BLOCKS, block_key, None),
            (COL_BLOCK_HEADERS, header_key, None),
        ])?;
        
        warn!("🗑️ Deleted block {}", block_number);
        
        Ok(())
    }

    /// Generate block key
    fn block_key(block_number: u64) -> Vec<u8> {
        let mut key = b"block_".to_vec();
        key.extend_from_slice(&block_number.to_le_bytes());
        key
    }

    /// Generate header key
    fn header_key(block_number: u64) -> Vec<u8> {
        let mut key = b"header_".to_vec();
        key.extend_from_slice(&block_number.to_le_bytes());
        key
    }
}

#[cfg(test)]
mod tests {
    use crate::consensus::block::Block;
    use crate::crypto::generate_keypair;
    use crate::storage::StorageManager;
    use crate::transaction::Transaction;
    use tempfile::TempDir;

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
    fn test_store_and_retrieve_block() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let block_store = storage.block_store();

        // Create genesis block
        let genesis = Block::genesis();
        
        // Store block
        assert!(block_store.store_block(&genesis).is_ok());
        
        // Retrieve block
        let retrieved = block_store.get_block(0).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().number(), 0);
    }

    #[test]
    fn test_chain_height() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let block_store = storage.block_store();

        // Initial height should be 0
        assert_eq!(block_store.get_chain_height().unwrap(), 0);

        // Store genesis
        let genesis = Block::genesis();
        block_store.store_block(&genesis).unwrap();
        
        assert_eq!(block_store.get_chain_height().unwrap(), 0);
        assert_eq!(block_store.block_count().unwrap(), 1);
    }

    #[test]
    fn test_get_latest_block() {
        let temp_dir = TempDir::new().unwrap();
        let storage = StorageManager::new(temp_dir.path()).unwrap();
        let block_store = storage.block_store();

        // Store genesis
        let genesis = Block::genesis();
        block_store.store_block(&genesis).unwrap();

        // Store block 1
        let tx = create_test_transaction();
        let block1 = Block::new(1, genesis.hash().to_string(), vec![tx], 123);
        block_store.store_block(&block1).unwrap();

        // Get latest should return block 1
        let latest = block_store.get_latest_block().unwrap().unwrap();
        assert_eq!(latest.number(), 1);
    }
}
