// MRBN-specific message types for network communication

#![allow(dead_code)] // Will be used as we build out the system

use serde::{Deserialize, Serialize};
use crate::transaction::Transaction;
use crate::consensus::block::Block;
use crate::consensus::validation::{ValidationTask, ValidationResult};

/// Message types for MRBN network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MrbnMessage {
    /// Broadcast a new transaction
    TransactionBroadcast(Transaction),
    
    /// Announce a new block
    BlockAnnounce(Block),
    
    /// Request a specific block
    BlockRequest { block_number: u64 },
    
    /// Response with requested block
    BlockResponse(Block),
    
    /// Validation task for committee members
    ValidationTaskBroadcast(ValidationTask),
    
    /// Validation result from committee member
    ValidationResultBroadcast(ValidationResult),
    
    /// Request peer count (for network size estimation)
    PeerCountRequest,
    
    /// Response with peer count
    PeerCountResponse { count: usize },
}

impl MrbnMessage {
    /// Serialize message to bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserialize message from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }
    
    /// Get message type as string
    pub fn message_type(&self) -> &str {
        match self {
            MrbnMessage::TransactionBroadcast(_) => "TransactionBroadcast",
            MrbnMessage::BlockAnnounce(_) => "BlockAnnounce",
            MrbnMessage::BlockRequest { .. } => "BlockRequest",
            MrbnMessage::BlockResponse(_) => "BlockResponse",
            MrbnMessage::ValidationTaskBroadcast(_) => "ValidationTaskBroadcast",
            MrbnMessage::ValidationResultBroadcast(_) => "ValidationResultBroadcast",
            MrbnMessage::PeerCountRequest => "PeerCountRequest",
            MrbnMessage::PeerCountResponse { .. } => "PeerCountResponse",
        }
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
    fn test_message_serialization() {
        let tx = create_test_transaction();
        let msg = MrbnMessage::TransactionBroadcast(tx);

        let bytes = msg.to_bytes().unwrap();
        let msg2 = MrbnMessage::from_bytes(&bytes).unwrap();

        assert_eq!(msg.message_type(), msg2.message_type());
    }

    #[test]
    fn test_message_types() {
        let tx = create_test_transaction();
        let msg = MrbnMessage::TransactionBroadcast(tx);
        assert_eq!(msg.message_type(), "TransactionBroadcast");

        let msg = MrbnMessage::PeerCountRequest;
        assert_eq!(msg.message_type(), "PeerCountRequest");
    }
}

