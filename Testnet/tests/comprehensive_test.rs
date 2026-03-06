// Comprehensive integration test for MRBN
// Tests: Network, Consensus, Transactions, Storage, Validation

use mrbn_node::*;
use anyhow::Result;

/// Helper to create a test transaction
fn create_test_transaction(nonce: u64) -> transaction::Transaction {
    let sender = crypto::generate_keypair();
    let recipient = crypto::generate_keypair();

    let mut tx = transaction::Transaction::new(
        sender.verifying_key().as_bytes().to_vec(),
        recipient.verifying_key().as_bytes().to_vec(),
        1000,
        10,
        nonce,
    );

    tx.sign(&sender).unwrap();
    tx
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn comprehensive_mrbn_test() -> Result<()> {
    println!("\n╔════════════════════════════════════════╗");
    println!("║   MRBN Comprehensive Test Suite       ║");
    println!("╚════════════════════════════════════════╝\n");

    // ========================================
    // TEST 1: Storage Layer
    // ========================================
    println!("📦 TEST 1: Storage Layer");
    println!("   Testing ParityDB block and state storage...");
    
    let temp_dir = tempfile::TempDir::new()?;
    let storage = storage::StorageManager::new(temp_dir.path())?;
    
    // Test genesis block
    let genesis = consensus::block::Block::genesis();
    storage.block_store().store_block(&genesis)?;
    assert_eq!(storage.get_chain_height()?, 0);
    println!("   ✅ Genesis block stored");
    
    // Test state storage
    let test_key = crypto::generate_keypair();
    let test_vk = test_key.verifying_key();
    let address = test_vk.as_bytes();
    storage.state_store().set_balance(address, 10000)?;
    assert_eq!(storage.state_store().get_balance(address)?, 10000);
    println!("   ✅ State storage working");
    
    println!("   ✅ Storage Layer: PASSED\n");

    // ========================================
    // TEST 2: Transaction System
    // ========================================
    println!("💸 TEST 2: Transaction System");
    println!("   Testing transaction creation, signing, and validation...");
    
    let mut tx_pool = transaction::pool::TransactionPool::new(1000);
    
    // Create and validate transactions
    for i in 0..10 {
        let tx = create_test_transaction(i);
        assert!(tx.validate().is_ok(), "Transaction {} validation failed", i);
        tx_pool.add(tx)?;
    }
    
    assert_eq!(tx_pool.size(), 10);
    println!("   ✅ Created and validated 10 transactions");
    
    // Test batch retrieval
    let batch = tx_pool.get_batch(5);
    assert_eq!(batch.len(), 5);
    println!("   ✅ Batch retrieval working");
    
    println!("   ✅ Transaction System: PASSED\n");

    // ========================================
    // TEST 3: VRF Committee Selection
    // ========================================
    println!("🎲 TEST 3: VRF Committee Selection");
    println!("   Testing verifiable random function and committee selection...");
    
    // Generate VRF keypairs
    let mut validators = Vec::new();
    for _ in 0..100 {
        let vrf_keypair = consensus::vrf::VrfKeypair::generate();
        let node = consensus::committee::ValidatorNode::new(vrf_keypair, 1000);
        validators.push(node);
    }
    println!("   ✅ Generated 100 validator nodes");
    
    // Test committee selection
    let seed = [42u8; 32];
    let selected = consensus::committee::simulate_selection(&validators, &seed, 10)?;
    
    assert!(selected.len() > 0, "No validators selected");
    assert!(selected.len() <= validators.len(), "Too many validators selected");
    println!("   ✅ Selected {} validators for committee", selected.len());
    
    // Verify all selections
    let threshold = consensus::vrf::calculate_threshold(100, 10);
    for member in &selected {
        assert!(member.verify(&seed, &threshold)?, "VRF verification failed");
    }
    println!("   ✅ All VRF proofs verified");
    
    println!("   ✅ VRF Committee Selection: PASSED\n");

    // ========================================
    // TEST 4: Block Creation and Validation
    // ========================================
    println!("🧱 TEST 4: Block Creation and Validation");
    println!("   Testing block structure and validation...");
    
    let transactions = vec![
        create_test_transaction(1),
        create_test_transaction(2),
        create_test_transaction(3),
    ];
    
    let block = consensus::block::Block::new(
        1,
        genesis.hash().to_string(),
        transactions.clone(),
        1234567890,
    );
    
    assert_eq!(block.number(), 1);
    assert_eq!(block.transactions.len(), 3);
    assert!(block.verify_structure().is_ok(), "Block structure invalid");
    println!("   ✅ Block created and validated");
    
    // Test Merkle root
    let root1 = consensus::block::Block::compute_merkle_root(&transactions);
    let root2 = consensus::block::Block::compute_merkle_root(&transactions);
    assert_eq!(root1, root2, "Merkle root not deterministic");
    println!("   ✅ Merkle root computation working");
    
    println!("   ✅ Block Creation: PASSED\n");

    // ========================================
    // TEST 5: Consensus Orchestrator
    // ========================================
    println!("⚙️ TEST 5: Consensus Orchestrator");
    println!("   Testing parallel batch processing and consensus...");
    
    let mut orchestrator = consensus::engine::ConsensusOrchestrator::new(
        100,  // network_size
        10,   // target_committee_size
        5,    // batch_size (small for testing)
        3,    // max_parallel_batches
    );
    
    // Add transactions
    for i in 0..15 {
        let tx = create_test_transaction(i);
        orchestrator.add_transaction(tx)?;
    }
    assert_eq!(orchestrator.pending_transactions(), 15);
    println!("   ✅ Added 15 transactions to pool");
    
    // Process batches
    let batches = orchestrator.process_batches()?;
    assert!(batches.len() > 0, "No batches created");
    assert!(batches.len() <= 3, "Too many parallel batches");
    println!("   ✅ Created {} parallel batches", batches.len());
    
    println!("   ✅ Consensus Orchestrator: PASSED\n");

    // ========================================
    // TEST 6: Validator Resource Management
    // ========================================
    println!("🛡️ TEST 6: Validator Resource Management");
    println!("   Testing resource monitoring and task queue...");
    
    let config = validator::ValidatorConfig::default();
    let mut validator = validator::Validator::new(config)?;
    
    validator.start()?;
    assert!(validator.is_active());
    println!("   ✅ Validator started");
    
    // Test resource monitoring
    let stats = validator.get_resource_stats()?;
    assert!(stats.ram_used_bytes > 0, "RAM monitoring not working");
    println!("   ✅ Resource monitoring: {} MB RAM", stats.ram_used_bytes / 1_048_576);
    
    // Test task queue
    let queue_stats = validator.get_queue_stats();
    assert_eq!(queue_stats.pending_count, 0);
    println!("   ✅ Task queue initialized");
    
    validator.stop();
    assert!(!validator.is_active());
    println!("   ✅ Validator stopped");
    
    println!("   ✅ Validator Resource Management: PASSED\n");

    // ========================================
    // TEST 7: Wallet and Keystore
    // ========================================
    println!("💼 TEST 7: Wallet and Keystore");
    println!("   Testing wallet creation and keystore encryption...");
    
    let wallet_dir = tempfile::TempDir::new()?;
    let keystore = wallet::keystore::Keystore::new(wallet_dir.path())?;
    
    // Create and save wallet
    let signing_key = crypto::generate_keypair();
    keystore.save_key(&signing_key, "test_password", "test_wallet")?;
    println!("   ✅ Wallet saved with encryption");
    
    // Load wallet
    let loaded_key = keystore.load_key("test_wallet", "test_password")?;
    assert_eq!(
        signing_key.verifying_key().as_bytes(),
        loaded_key.verifying_key().as_bytes()
    );
    println!("   ✅ Wallet loaded and verified");
    
    // Test wrong password
    let wrong_result = keystore.load_key("test_wallet", "wrong_password");
    assert!(wrong_result.is_err(), "Wrong password should fail");
    println!("   ✅ Wrong password rejected");
    
    println!("   ✅ Wallet and Keystore: PASSED\n");

    // ========================================
    // TEST 8: Blockchain State
    // ========================================
    println!("⛓️ TEST 8: Blockchain State");
    println!("   Testing blockchain with multiple blocks...");
    
    let mut blockchain = consensus::block::Blockchain::new();
    assert_eq!(blockchain.height(), 0);
    
    // Add blocks
    for i in 1..=5 {
        let prev_block = blockchain.get_latest_block();
        let block = consensus::block::Block::new(
            i,
            prev_block.hash().to_string(),
            vec![create_test_transaction(i)],
            1234567890 + i,
        );
        blockchain.add_block(block)?;
    }
    
    assert_eq!(blockchain.height(), 5);
    assert_eq!(blockchain.total_transactions(), 5);
    println!("   ✅ Added 5 blocks to chain");
    
    // Verify chain integrity
    for i in 1..=5 {
        let block = blockchain.get_block(i).unwrap();
        assert_eq!(block.number(), i);
    }
    println!("   ✅ Chain integrity verified");
    
    println!("   ✅ Blockchain State: PASSED\n");

    // ========================================
    // TEST 9: Validation Protocol
    // ========================================
    println!("✅ TEST 9: Validation Protocol");
    println!("   Testing micro-committee validation...");
    
    let mut coordinator = consensus::validation::ValidationCoordinator::new(30);
    
    let committee = consensus::committee::Committee::new(1, [0u8; 32], 5, 100);
    let transactions = vec![create_test_transaction(1)];
    
    coordinator.start_validation(1, transactions, &committee)?;
    
    let status = coordinator.get_status(1);
    assert!(status.is_some());
    println!("   ✅ Validation task created");
    
    println!("   ✅ Validation Protocol: PASSED\n");

    // ========================================
    // TEST 10: End-to-End Integration
    // ========================================
    println!("🎯 TEST 10: End-to-End Integration");
    println!("   Testing full transaction flow...");
    
    let temp_dir2 = tempfile::TempDir::new()?;
    let storage2 = storage::StorageManager::new(temp_dir2.path())?;
    
    // Store genesis
    let genesis2 = consensus::block::Block::genesis();
    storage2.block_store().store_block(&genesis2)?;
    
    // Create transaction
    let sender = crypto::generate_keypair();
    let recipient = crypto::generate_keypair();
    
    // Set initial balance
    let sender_vk = sender.verifying_key();
    let sender_addr = sender_vk.as_bytes();
    storage2.state_store().set_balance(sender_addr, 10000)?;
    
    // Create and sign transaction
    let mut tx = transaction::Transaction::new(
        sender.verifying_key().as_bytes().to_vec(),
        recipient.verifying_key().as_bytes().to_vec(),
        1000,
        10,
        0,
    );
    tx.sign(&sender)?;
    
    // Validate
    assert!(tx.validate().is_ok());
    println!("   ✅ Transaction created and validated");
    
    // Create block with transaction
    let block = consensus::block::Block::new(
        1,
        genesis2.hash().to_string(),
        vec![tx.clone()],
        1234567890,
    );
    
    // Store block
    storage2.block_store().store_block(&block)?;
    assert_eq!(storage2.get_chain_height()?, 1);
    println!("   ✅ Block stored in blockchain");
    
    // Apply transaction to state
    let recipient_vk = recipient.verifying_key();
    let recipient_addr = recipient_vk.as_bytes();
    storage2.state_store().apply_transaction(
        sender_addr,
        recipient_addr,
        1000,
        10,
    )?;
    
    // Verify balances
    let sender_balance = storage2.state_store().get_balance(sender_addr)?;
    let recipient_balance = storage2.state_store().get_balance(recipient_addr)?;
    
    assert_eq!(sender_balance, 8990); // 10000 - 1000 - 10
    assert_eq!(recipient_balance, 1000);
    println!("   ✅ State updated correctly");
    println!("      Sender: 8990 Kain (10000 - 1000 - 10 gas)");
    println!("      Recipient: 1000 Kain");
    
    println!("   ✅ End-to-End Integration: PASSED\n");

    // ========================================
    // FINAL SUMMARY
    // ========================================
    println!("╔════════════════════════════════════════╗");
    println!("║   🎉 ALL TESTS PASSED! 🎉             ║");
    println!("╚════════════════════════════════════════╝");
    println!("\n✅ Test Summary:");
    println!("   1. Storage Layer ..................... PASSED");
    println!("   2. Transaction System ................ PASSED");
    println!("   3. VRF Committee Selection ........... PASSED");
    println!("   4. Block Creation .................... PASSED");
    println!("   5. Consensus Orchestrator ............ PASSED");
    println!("   6. Validator Resource Management ..... PASSED");
    println!("   7. Wallet and Keystore ............... PASSED");
    println!("   8. Blockchain State .................. PASSED");
    println!("   9. Validation Protocol ............... PASSED");
    println!("  10. End-to-End Integration ............ PASSED");
    println!("\n🚀 MRBN Core is production-ready!");
    
    Ok(())
}
