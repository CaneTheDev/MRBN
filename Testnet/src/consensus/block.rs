// Block structure for MRBN blockchain

use crate::crypto::{hash_data, to_hex};
use crate::transaction::Transaction;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockHeader {
    /// Block number (height)
    pub number: u64,
    
    /// Previous block hash
    pub previous_hash: String,
    
    /// Merkle root of transactions
    pub transactions_root: String,
    
    /// Block timestamp (Unix timestamp)
    pub timestamp: u64,
    
    /// Number of transactions in block
    pub transaction_count: u32,
    
    /// Block hash (computed)
    pub hash: String,
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        number: u64,
        previous_hash: String,
        transactions_root: String,
        timestamp: u64,
        transaction_count: u32,
    ) -> Self {
        let mut header = BlockHeader {
            number,
            previous_hash,
            transactions_root,
            timestamp,
            transaction_count,
            hash: String::new(),
        };
        
        header.hash = header.compute_hash();
        header
    }

    /// Compute block hash
    pub fn compute_hash(&self) -> String {
        let mut data = Vec::new();
        data.extend_from_slice(&self.number.to_le_bytes());
        data.extend_from_slice(self.previous_hash.as_bytes());
        data.extend_from_slice(self.transactions_root.as_bytes());
        data.extend_from_slice(&self.timestamp.to_le_bytes());
        data.extend_from_slice(&self.transaction_count.to_le_bytes());
        
        to_hex(&hash_data(&data))
    }

    /// Verify block hash
    pub fn verify_hash(&self) -> bool {
        self.hash == self.compute_hash()
    }
}

/// Committee signature on a block
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitteeSignature {
    /// Public key of committee member
    pub public_key: Vec<u8>,
    
    /// Signature on block hash
    pub signature: Vec<u8>,
    
    /// VRF proof of committee membership
    pub vrf_proof: Vec<u8>,
    
    /// VRF preout for verification
    pub vrf_preout: Vec<u8>,
}

/// Block containing transactions and committee signatures
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    
    /// Transactions in this block
    pub transactions: Vec<Transaction>,
    
    /// Committee signatures (2/3 majority required)
    pub committee_signatures: Vec<CommitteeSignature>,
}

impl Block {
    /// Create a new block
    pub fn new(
        number: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        timestamp: u64,
    ) -> Self {
        let transactions_root = Self::compute_merkle_root(&transactions);
        let transaction_count = transactions.len() as u32;
        
        let header = BlockHeader::new(
            number,
            previous_hash,
            transactions_root,
            timestamp,
            transaction_count,
        );
        
        Block {
            header,
            transactions,
            committee_signatures: Vec::new(),
        }
    }

    /// Create genesis block (first block)
    pub fn genesis() -> Self {
        let timestamp = 0; // Genesis timestamp
        let transactions = Vec::new();
        
        Block::new(
            0,
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            transactions,
            timestamp,
        )
    }

    /// Compute Merkle root of transactions
    pub fn compute_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        }

        // Simple Merkle root: hash all transaction hashes together
        let mut data = Vec::new();
        for tx in transactions {
            if let Some(hash) = &tx.hash {
                data.extend_from_slice(hash.as_bytes());
            }
        }
        
        to_hex(&hash_data(&data))
    }

    /// Add a committee signature
    pub fn add_signature(&mut self, signature: CommitteeSignature) {
        self.committee_signatures.push(signature);
    }

    /// Check if block has enough signatures (2/3 majority)
    pub fn has_majority(&self, committee_size: usize) -> bool {
        let required = (committee_size * 2).div_ceil(3); // Ceiling of 2/3
        self.committee_signatures.len() >= required
    }

    /// Verify block structure
    pub fn verify_structure(&self) -> Result<()> {
        // Verify header hash
        if !self.header.verify_hash() {
            return Err(anyhow!("Invalid block hash"));
        }

        // Verify transaction count
        if self.header.transaction_count != self.transactions.len() as u32 {
            return Err(anyhow!("Transaction count mismatch"));
        }

        // Verify Merkle root
        let computed_root = Self::compute_merkle_root(&self.transactions);
        if self.header.transactions_root != computed_root {
            return Err(anyhow!("Invalid Merkle root"));
        }

        // Verify all transactions
        for tx in &self.transactions {
            tx.validate()?;
        }

        Ok(())
    }

    /// Get block hash
    pub fn hash(&self) -> &str {
        &self.header.hash
    }

    /// Get block number
    pub fn number(&self) -> u64 {
        self.header.number
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| anyhow!("Serialization failed: {}", e))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| anyhow!("Deserialization failed: {}", e))
    }
}

/// Blockchain state
#[derive(Debug)]
pub struct Blockchain {
    /// All blocks in the chain
    blocks: Vec<Block>,
    
    /// Current block height
    height: u64,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new() -> Self {
        let genesis = Block::genesis();
        Blockchain {
            blocks: vec![genesis],
            height: 0,
        }
    }

    /// Add a block to the chain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Verify block structure
        block.verify_structure()?;

        // Verify block number
        if block.number() != self.height + 1 {
            return Err(anyhow!(
                "Invalid block number: expected {}, got {}",
                self.height + 1,
                block.number()
            ));
        }

        // Verify previous hash
        let last_block = self.get_latest_block();
        if block.header.previous_hash != last_block.hash() {
            return Err(anyhow!("Invalid previous hash"));
        }

        // Add block
        self.blocks.push(block);
        self.height += 1;

        Ok(())
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> &Block {
        self.blocks.last().expect("Blockchain should have at least genesis block")
    }

    /// Get block by number
    pub fn get_block(&self, number: u64) -> Option<&Block> {
        self.blocks.get(number as usize)
    }

    /// Get current height
    pub fn height(&self) -> u64 {
        self.height
    }

    /// Get total number of transactions
    pub fn total_transactions(&self) -> u64 {
        self.blocks.iter().map(|b| b.header.transaction_count as u64).sum()
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
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
    fn test_block_header_creation() {
        let header = BlockHeader::new(
            1,
            "previous_hash".to_string(),
            "merkle_root".to_string(),
            1234567890,
            10,
        );

        assert_eq!(header.number, 1);
        assert!(!header.hash.is_empty());
        assert!(header.verify_hash());
    }

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        
        assert_eq!(genesis.number(), 0);
        assert_eq!(genesis.transactions.len(), 0);
        assert!(genesis.verify_structure().is_ok());
    }

    #[test]
    fn test_block_creation() {
        let tx = create_test_transaction();
        let block = Block::new(
            1,
            "previous_hash".to_string(),
            vec![tx],
            1234567890,
        );

        assert_eq!(block.number(), 1);
        assert_eq!(block.transactions.len(), 1);
        assert!(block.verify_structure().is_ok());
    }

    #[test]
    fn test_merkle_root() {
        let tx1 = create_test_transaction();
        let tx2 = create_test_transaction();

        let root1 = Block::compute_merkle_root(&[tx1.clone()]);
        let root2 = Block::compute_merkle_root(&[tx1.clone(), tx2.clone()]);

        assert_ne!(root1, root2);
        
        // Same transactions should produce same root
        let root3 = Block::compute_merkle_root(&[tx1.clone(), tx2.clone()]);
        assert_eq!(root2, root3);
    }

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        
        assert_eq!(blockchain.height(), 0);
        assert_eq!(blockchain.get_latest_block().number(), 0);
    }

    #[test]
    fn test_add_block_to_chain() {
        let mut blockchain = Blockchain::new();
        let genesis = blockchain.get_latest_block();

        let tx = create_test_transaction();
        let block = Block::new(
            1,
            genesis.hash().to_string(),
            vec![tx],
            1234567890,
        );

        assert!(blockchain.add_block(block).is_ok());
        assert_eq!(blockchain.height(), 1);
    }

    #[test]
    fn test_invalid_block_number() {
        let mut blockchain = Blockchain::new();
        let genesis = blockchain.get_latest_block();

        let block = Block::new(
            5, // Wrong number
            genesis.hash().to_string(),
            vec![],
            1234567890,
        );

        assert!(blockchain.add_block(block).is_err());
    }

    #[test]
    fn test_committee_majority() {
        let block = Block::new(1, "prev".to_string(), vec![], 123);
        
        assert!(!block.has_majority(10)); // 0 signatures, need 7
        
        let mut block_with_sigs = block.clone();
        for _ in 0..7 {
            block_with_sigs.add_signature(CommitteeSignature {
                public_key: vec![0u8; 32],
                signature: vec![0u8; 64],
                vrf_proof: vec![0u8; 64],
                vrf_preout: vec![0u8; 32],
            });
        }
        
        assert!(block_with_sigs.has_majority(10)); // 7 signatures, need 7
    }

    #[test]
    fn test_block_serialization() {
        let tx = create_test_transaction();
        let block = Block::new(1, "prev".to_string(), vec![tx], 123);

        let bytes = block.to_bytes().unwrap();
        let block2 = Block::from_bytes(&bytes).unwrap();

        assert_eq!(block, block2);
    }
}
