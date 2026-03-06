// Simple CLI to test MRBN consensus mechanism

use mrbn_node::consensus::engine::ConsensusOrchestrator;
use mrbn_node::crypto::generate_keypair;
use mrbn_node::transaction::Transaction;
use std::io::{self, Write};

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

fn print_menu() {
    println!("\n╔════════════════════════════════════════╗");
    println!("║   MRBN Consensus Test CLI             ║");
    println!("╚════════════════════════════════════════╝");
    println!("\nCommands:");
    println!("  1. Add transaction");
    println!("  2. Add multiple transactions");
    println!("  3. Process batches");
    println!("  4. Show status");
    println!("  5. Finalize batches");
    println!("  6. Run full cycle");
    println!("  q. Quit");
    print!("\nEnter command: ");
    io::stdout().flush().unwrap();
}

fn show_status(orchestrator: &ConsensusOrchestrator) {
    println!("\n📊 Current Status:");
    println!("   Blockchain height: {}", orchestrator.height());
    println!("   Pending transactions: {}", orchestrator.pending_transactions());
    println!("   Active batches: {}", orchestrator.active_batches());
}

fn main() {
    println!("🚀 Initializing MRBN Consensus Test Environment...\n");
    
    let mut orchestrator = ConsensusOrchestrator::new(
        100,  // network_size
        10,   // target_committee_size
        5,    // batch_size (small for testing)
        3,    // max_parallel_batches
    );
    
    let mut nonce = 0u64;
    
    loop {
        print_menu();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        
        match input {
            "1" => {
                let tx = create_test_transaction(nonce);
                nonce += 1;
                
                match orchestrator.add_transaction(tx) {
                    Ok(_) => println!("✅ Transaction added (nonce: {})", nonce - 1),
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "2" => {
                print!("How many transactions? ");
                io::stdout().flush().unwrap();
                
                let mut count_input = String::new();
                io::stdin().read_line(&mut count_input).unwrap();
                
                if let Ok(count) = count_input.trim().parse::<usize>() {
                    for _ in 0..count {
                        let tx = create_test_transaction(nonce);
                        nonce += 1;
                        orchestrator.add_transaction(tx).ok();
                    }
                    println!("✅ Added {} transactions", count);
                } else {
                    println!("❌ Invalid number");
                }
            }
            "3" => {
                match orchestrator.process_batches() {
                    Ok(batches) => {
                        if batches.is_empty() {
                            println!("ℹ️ No transactions to process");
                        } else {
                            println!("✅ Created {} batches: {:?}", batches.len(), batches);
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "4" => {
                show_status(&orchestrator);
            }
            "5" => {
                match orchestrator.finalize_batches() {
                    Ok(blocks) => {
                        if blocks.is_empty() {
                            println!("ℹ️ No batches ready to finalize");
                        } else {
                            println!("✅ Created {} blocks!", blocks.len());
                            for block in blocks {
                                println!("   Block #{}: {} transactions", 
                                    block.number(), 
                                    block.transactions.len()
                                );
                            }
                        }
                    }
                    Err(e) => println!("❌ Error: {}", e),
                }
            }
            "6" => {
                println!("\n🔄 Running full consensus cycle...\n");
                
                // Add transactions
                println!("1️⃣ Adding 15 test transactions...");
                for _ in 0..15 {
                    let tx = create_test_transaction(nonce);
                    nonce += 1;
                    orchestrator.add_transaction(tx).ok();
                }
                println!("   ✅ Transactions added\n");
                
                // Process batches
                println!("2️⃣ Processing batches...");
                match orchestrator.process_batches() {
                    Ok(batches) => {
                        println!("   ✅ Created {} batches\n", batches.len());
                    }
                    Err(e) => {
                        println!("   ❌ Error: {}\n", e);
                        continue;
                    }
                }
                
                // Show status
                show_status(&orchestrator);
                
                println!("\n💡 Note: In a real network, committee members would validate");
                println!("   and vote on batches. For this test, batches remain pending.");
            }
            "q" | "quit" | "exit" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => {
                println!("❌ Invalid command");
            }
        }
    }
}
