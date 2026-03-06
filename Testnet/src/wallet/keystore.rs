// Keystore - secure encrypted storage for private keys

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Encrypted keystore file format
#[derive(Debug, Serialize, Deserialize)]
pub struct KeystoreFile {
    /// Version of keystore format
    pub version: u32,
    
    /// Encrypted private key
    pub encrypted_key: Vec<u8>,
    
    /// Salt for key derivation
    pub salt: Vec<u8>,
    
    /// Public address (not encrypted)
    pub address: Vec<u8>,
}

/// Keystore for managing encrypted keys
pub struct Keystore {
    /// Path to keystore directory
    keystore_path: std::path::PathBuf,
}

impl Keystore {
    /// Create a new keystore
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let keystore_path = path.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !keystore_path.exists() {
            fs::create_dir_all(&keystore_path)?;
        }
        
        Ok(Keystore { keystore_path })
    }

    /// Save a key to the keystore with password encryption
    pub fn save_key(
        &self,
        signing_key: &SigningKey,
        password: &str,
        name: &str,
    ) -> Result<()> {
        // Derive encryption key from password
        let (encryption_key, salt) = Self::derive_key(password, None)?;
        
        // Encrypt the private key
        let key_bytes = signing_key.to_bytes();
        let encrypted_key = Self::xor_encrypt(&key_bytes, &encryption_key);
        
        // Get public address
        let address = signing_key.verifying_key().as_bytes().to_vec();
        
        // Create keystore file
        let keystore_file = KeystoreFile {
            version: 1,
            encrypted_key,
            salt,
            address,
        };
        
        // Serialize and save
        let json = serde_json::to_string_pretty(&keystore_file)?;
        let file_path = self.keystore_path.join(format!("{}.json", name));
        fs::write(file_path, json)?;
        
        Ok(())
    }

    /// Load a key from the keystore
    pub fn load_key(&self, name: &str, password: &str) -> Result<SigningKey> {
        // Read keystore file
        let file_path = self.keystore_path.join(format!("{}.json", name));
        let json = fs::read_to_string(file_path)?;
        let keystore_file: KeystoreFile = serde_json::from_str(&json)?;
        
        // Derive decryption key
        let (decryption_key, _) = Self::derive_key(password, Some(&keystore_file.salt))?;
        
        // Decrypt the private key
        let key_bytes = Self::xor_encrypt(&keystore_file.encrypted_key, &decryption_key);
        
        // Parse signing key
        let key_array: [u8; 32] = key_bytes
            .try_into()
            .map_err(|_| anyhow!("Invalid key length"))?;
        let signing_key = SigningKey::from_bytes(&key_array);
        
        // Verify address matches
        let address = signing_key.verifying_key().as_bytes().to_vec();
        if address != keystore_file.address {
            return Err(anyhow!("Invalid password or corrupted keystore"));
        }
        
        Ok(signing_key)
    }

    /// List all keystore files
    pub fn list_keys(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        
        for entry in fs::read_dir(&self.keystore_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    keys.push(name.to_string());
                }
            }
        }
        
        Ok(keys)
    }

    /// Delete a key from the keystore
    pub fn delete_key(&self, name: &str) -> Result<()> {
        let file_path = self.keystore_path.join(format!("{}.json", name));
        fs::remove_file(file_path)?;
        Ok(())
    }

    /// Derive encryption key from password using SHA-256
    fn derive_key(password: &str, salt: Option<&[u8]>) -> Result<(Vec<u8>, Vec<u8>)> {
        // Generate or use provided salt
        let salt = match salt {
            Some(s) => s.to_vec(),
            None => {
                // Generate random salt
                use rand::Rng;
                let mut rng = rand::thread_rng();
                (0..32).map(|_| rng.gen()).collect()
            }
        };
        
        // Simple key derivation (in production, use PBKDF2 or Argon2)
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(&salt);
        let key = hasher.finalize().to_vec();
        
        Ok((key, salt))
    }

    /// Simple XOR encryption (in production, use AES-GCM)
    fn xor_encrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
        data.iter()
            .enumerate()
            .map(|(i, &byte)| byte ^ key[i % key.len()])
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_keypair;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load_key() {
        let temp_dir = TempDir::new().unwrap();
        let keystore = Keystore::new(temp_dir.path()).unwrap();

        let signing_key = generate_keypair();
        let password = "test_password_123";

        // Save key
        keystore.save_key(&signing_key, password, "test_key").unwrap();

        // Load key
        let loaded_key = keystore.load_key("test_key", password).unwrap();

        // Verify keys match
        assert_eq!(
            signing_key.verifying_key().as_bytes(),
            loaded_key.verifying_key().as_bytes()
        );
    }

    #[test]
    fn test_wrong_password() {
        let temp_dir = TempDir::new().unwrap();
        let keystore = Keystore::new(temp_dir.path()).unwrap();

        let signing_key = generate_keypair();
        keystore.save_key(&signing_key, "correct_password", "test_key").unwrap();

        // Try to load with wrong password
        let result = keystore.load_key("test_key", "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_keys() {
        let temp_dir = TempDir::new().unwrap();
        let keystore = Keystore::new(temp_dir.path()).unwrap();

        let key1 = generate_keypair();
        let key2 = generate_keypair();

        keystore.save_key(&key1, "pass1", "key1").unwrap();
        keystore.save_key(&key2, "pass2", "key2").unwrap();

        let keys = keystore.list_keys().unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
}
