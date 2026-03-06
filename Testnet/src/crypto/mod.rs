// Cryptography module - handles keys, signatures, and hashing

#![allow(dead_code)] // Will be used as we build out the system

use anyhow::Result;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey, Signature};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Generate a new Ed25519 keypair
pub fn generate_keypair() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

/// Sign data with a signing key
pub fn sign_data(signing_key: &SigningKey, data: &[u8]) -> Signature {
    signing_key.sign(data)
}

/// Verify a signature
pub fn verify_signature(verifying_key: &VerifyingKey, data: &[u8], signature: &Signature) -> Result<()> {
    verifying_key
        .verify(data, signature)
        .map_err(|e| anyhow::anyhow!("Signature verification failed: {}", e))
}

/// Hash data using SHA-256
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Convert bytes to hex string
pub fn to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hex string to bytes
pub fn from_hex(hex_str: &str) -> Result<Vec<u8>> {
    hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Hex decode failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair_generation() {
        let signing_key = generate_keypair();
        let verifying_key = signing_key.verifying_key();
        assert_eq!(verifying_key.as_bytes().len(), 32);
        assert_eq!(signing_key.to_bytes().len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let signing_key = generate_keypair();
        let verifying_key = signing_key.verifying_key();
        let data = b"Hello, MRBN!";
        
        let signature = sign_data(&signing_key, data);
        let result = verify_signature(&verifying_key, data, &signature);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_invalid_signature() {
        let signing_key1 = generate_keypair();
        let signing_key2 = generate_keypair();
        let verifying_key2 = signing_key2.verifying_key();
        let data = b"Hello, MRBN!";
        
        let signature = sign_data(&signing_key1, data);
        let result = verify_signature(&verifying_key2, data, &signature);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_data() {
        let data = b"Hello, MRBN!";
        let hash = hash_data(data);
        
        assert_eq!(hash.len(), 32);
        
        // Same data should produce same hash
        let hash2 = hash_data(data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hex_conversion() {
        let data = b"Hello, MRBN!";
        let hex_str = to_hex(data);
        let decoded = from_hex(&hex_str).unwrap();
        
        assert_eq!(data.to_vec(), decoded);
    }
}
