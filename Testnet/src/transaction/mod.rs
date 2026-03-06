// Transaction module - MRBN transaction format and validation

#![allow(dead_code)] // Will be used as we build out the system

pub mod pool;

use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use crate::crypto::{hash_data, sign_data, to_hex, verify_signature};

/// MRBN Transaction with separate gas field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// Sender's public key (32 bytes)
    pub from: Vec<u8>,
    
    /// Recipient's public key (32 bytes)
    pub to: Vec<u8>,
    
    /// Amount to transfer (in smallest unit)
    pub amount: u64,
    
    /// Gas fee for validators (separate from amount)
    pub gas: u64,
    
    /// Transaction nonce (prevents replay attacks)
    pub nonce: u64,
    
    /// Ed25519 signature (64 bytes)
    pub signature: Vec<u8>,
    
    /// Transaction hash (computed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

impl Transaction {
    /// Create a new unsigned transaction
    pub fn new(from: Vec<u8>, to: Vec<u8>, amount: u64, gas: u64, nonce: u64) -> Self {
        Transaction {
            from,
            to,
            amount,
            gas,
            nonce,
            signature: Vec::new(),
            hash: None,
        }
    }

    /// Sign the transaction with a signing key
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<()> {
        // Verify the signing key matches the 'from' address
        let verifying_key = signing_key.verifying_key();
        if verifying_key.as_bytes() != self.from.as_slice() {
            return Err(anyhow!("Signing key does not match sender address"));
        }

        // Create signature data (everything except signature and hash)
        let sig_data = self.signature_data();
        
        // Sign the data
        let signature = sign_data(signing_key, &sig_data);
        self.signature = signature.to_bytes().to_vec();
        
        // Compute and store hash
        self.hash = Some(self.compute_hash());
        
        Ok(())
    }

    /// Get the data to be signed (excludes signature and hash)
    fn signature_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.from);
        data.extend_from_slice(&self.to);
        data.extend_from_slice(&self.amount.to_le_bytes());
        data.extend_from_slice(&self.gas.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data
    }

    /// Compute transaction hash
    pub fn compute_hash(&self) -> String {
        let mut data = self.signature_data();
        data.extend_from_slice(&self.signature);
        to_hex(&hash_data(&data))
    }

    /// Verify transaction signature
    pub fn verify_signature(&self) -> Result<()> {
        if self.signature.len() != 64 {
            return Err(anyhow!("Invalid signature length: {}", self.signature.len()));
        }

        // Parse verifying key
        let verifying_key = VerifyingKey::from_bytes(
            self.from.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid public key length"))?
        ).map_err(|e| anyhow!("Invalid public key: {}", e))?;

        // Parse signature
        let signature = Signature::from_bytes(
            self.signature.as_slice().try_into()
                .map_err(|_| anyhow!("Invalid signature length"))?
        );

        // Verify
        let sig_data = self.signature_data();
        verify_signature(&verifying_key, &sig_data, &signature)
    }

    /// Validate transaction (signature + basic checks)
    pub fn validate(&self) -> Result<()> {
        // Check signature
        self.verify_signature()?;

        // Check amounts are non-zero
        if self.amount == 0 {
            return Err(anyhow!("Transaction amount must be greater than 0"));
        }

        if self.gas == 0 {
            return Err(anyhow!("Gas fee must be greater than 0"));
        }

        // Check addresses are valid length
        if self.from.len() != 32 {
            return Err(anyhow!("Invalid sender address length"));
        }

        if self.to.len() != 32 {
            return Err(anyhow!("Invalid recipient address length"));
        }

        // Check sender != recipient
        if self.from == self.to {
            return Err(anyhow!("Cannot send to self"));
        }

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;

    #[test]
    fn test_create_and_sign_transaction() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();

        let mut tx = Transaction::new(
            sender_key.verifying_key().as_bytes().to_vec(),
            recipient_key.verifying_key().as_bytes().to_vec(),
            1000,
            10,
            1,
        );

        assert!(tx.sign(&sender_key).is_ok());
        assert!(!tx.signature.is_empty());
        assert!(tx.hash.is_some());
    }

    #[test]
    fn test_verify_valid_signature() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();

        let mut tx = Transaction::new(
            sender_key.verifying_key().as_bytes().to_vec(),
            recipient_key.verifying_key().as_bytes().to_vec(),
            1000,
            10,
            1,
        );

        tx.sign(&sender_key).unwrap();
        assert!(tx.verify_signature().is_ok());
    }

    #[test]
    fn test_validate_transaction() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();

        let mut tx = Transaction::new(
            sender_key.verifying_key().as_bytes().to_vec(),
            recipient_key.verifying_key().as_bytes().to_vec(),
            1000,
            10,
            1,
        );

        tx.sign(&sender_key).unwrap();
        assert!(tx.validate().is_ok());
    }

    #[test]
    fn test_invalid_zero_amount() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();

        let mut tx = Transaction::new(
            sender_key.verifying_key().as_bytes().to_vec(),
            recipient_key.verifying_key().as_bytes().to_vec(),
            0, // Invalid
            10,
            1,
        );

        tx.sign(&sender_key).unwrap();
        assert!(tx.validate().is_err());
    }

    #[test]
    fn test_serialization() {
        let sender_key = generate_keypair();
        let recipient_key = generate_keypair();

        let mut tx = Transaction::new(
            sender_key.verifying_key().as_bytes().to_vec(),
            recipient_key.verifying_key().as_bytes().to_vec(),
            1000,
            10,
            1,
        );

        tx.sign(&sender_key).unwrap();

        let bytes = tx.to_bytes().unwrap();
        let tx2 = Transaction::from_bytes(&bytes).unwrap();

        assert_eq!(tx, tx2);
    }
}
