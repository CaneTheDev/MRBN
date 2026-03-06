// CLI commands for wallet and node operations

#![allow(dead_code)]

use anyhow::Result;
use crate::storage::StorageManager;
use crate::wallet::{Wallet, keystore::Keystore};
use crate::crypto::generate_keypair;
use std::path::Path;

/// Execute wallet create command
pub fn wallet_create(name: String, password: String, wallet_dir: &Path) -> Result<()> {
    println!("🔐 Creating wallet: {}", name);
    
    // Create wallet directory if it doesn't exist
    std::fs::create_dir_all(wallet_dir)?;
    
    // Initialize keystore
    let keystore = Keystore::new(wallet_dir)?;
    
    // Generate new keypair
    let signing_key = generate_keypair();
    
    // Save to keystore (correct parameter order: key, name, password)
    keystore.save_key(&signing_key, &name, &password)?;
    
    // Get address
    let wallet = Wallet::from_key(signing_key);
    let address = hex::encode(wallet.address());
    
    println!("✅ Wallet created successfully!");
    println!("📍 Address: {}", address);
    println!("💾 Saved to: {}", wallet_dir.display());
    println!();
    println!("⚠️  IMPORTANT: Keep your password safe! It cannot be recovered.");
    
    Ok(())
}

/// Execute wallet list command
pub fn wallet_list(wallet_dir: &Path) -> Result<()> {
    println!("📋 Listing wallets in: {}", wallet_dir.display());
    println!();
    
    if !wallet_dir.exists() {
        println!("No wallets found. Create one with: mrbn-node wallet create <name>");
        return Ok(());
    }
    
    let keystore = Keystore::new(wallet_dir)?;
    let keys = keystore.list_keys()?;
    
    if keys.is_empty() {
        println!("No wallets found. Create one with: mrbn-node wallet create <name>");
    } else {
        println!("Found {} wallet(s):", keys.len());
        for (i, key_name) in keys.iter().enumerate() {
            println!("  {}. {}", i + 1, key_name);
        }
    }
    
    Ok(())
}

/// Execute wallet balance command
pub fn wallet_balance(address: String, storage: &StorageManager) -> Result<()> {
    println!("💰 Checking balance for: {}", address);
    
    let address_bytes = hex::decode(&address)?;
    let state_store = storage.state_store();
    let balance = state_store.get_balance(&address_bytes)?;
    
    println!("Balance: {} Kain", balance);
    
    Ok(())
}

/// Execute wallet send command
pub fn wallet_send(
    from: String,
    to: String,
    amount: u64,
    gas: u64,
    _storage: &StorageManager,
) -> Result<()> {
    println!("💸 Sending {} Kain from {} to {}", amount, from, to);
    println!("⛽ Gas fee: {} Kain", gas);
    
    // TODO: Implement transaction creation and broadcasting
    // This requires:
    // 1. Load wallet from keystore
    // 2. Create transaction
    // 3. Sign transaction
    // 4. Broadcast to network
    
    println!("⚠️  Transaction sending not yet implemented");
    println!("Coming soon in Phase 5!");
    
    Ok(())
}

/// Execute node status command
pub fn node_status(storage: &StorageManager) -> Result<()> {
    println!("📊 MRBN Node Status");
    println!("═══════════════════════════════════════");
    
    let height = storage.get_chain_height()?;
    let block_count = storage.block_store().block_count()?;
    
    println!("Chain Height: {}", height);
    println!("Total Blocks: {}", block_count);
    println!("Status: Running");
    println!("Network: Testnet");
    
    // TODO: Add more stats
    // - Connected peers
    // - Pending transactions
    // - Validator status
    // - Resource usage
    
    Ok(())
}
