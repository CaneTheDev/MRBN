// Integration tests for MRBN consensus

use mrbn_node::consensus::engine::ConsensusOrchestrator;
use mrbn_node::crypto::generate_keypair;
use mrbn_node::transaction::Transaction;

fn create_test_transaction(nonce: u64) -> Transaction {
    let sender = generate_keypair();
    let recipient = generate_keypair();

    let mut tx = Transaction::new(
        sender.verifying_key().as_bytes().to_vec(),
        recipient.verifying_key().as_bytes().to_vec(),
        1000,
        10,
        nonce,
    );

    tx.sign(&sender).unwrap();
    tx
}

#[test]
fn test_consensus_orchestrator_initialization() {
    let orchestrator = ConsensusOrchestrator::new(100, 10, 100, 3);
    
    assert_eq!(orchestrator.height(), 0);
    assert_eq!(orchestrator.pending_transactions(), 0);
    assert_eq!(orchestrator.active_batches(), 0);
}

#[test]
fn test_add_transactions_to_pool() {
    let mut orchestrator = ConsensusOrchestrator::new(100, 10, 100, 3);
    
    // Add 10 transactions
    for i in 0..10 {
        let tx = create_test_transaction(i);
        assert!(orchestrator.add_transaction(tx).is_ok());
    }
    
    assert_eq!(orchestrator.pending_transactions(), 10);
}

#[test]
fn test_batch_processing() {
    let mut orchestrator = ConsensusOrchestrator::new(100, 10, 5, 3);
    
    // Add 15 transactions (should create 3 batches of 5)
    for i in 0..15 {
        let tx = create_test_transaction(i);
        orchestrator.add_transaction(tx).unwrap();
    }
    
    // Process batches
    let batches = orchestrator.process_batches().unwrap();
    
    // Should create 3 batches (max parallel)
    assert_eq!(batches.len(), 3);
    assert_eq!(orchestrator.active_batches(), 3);
}

#[test]
fn test_genesis_block() {
    let orchestrator = ConsensusOrchestrator::new(100, 10, 100, 3);
    
    // Should start with genesis block
    assert_eq!(orchestrator.height(), 0);
}
