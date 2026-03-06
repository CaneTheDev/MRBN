// MRBN Testnet Node
// Entry point for the MRBN network node

mod cli;
mod consensus;
mod crypto;
mod network;
mod node;
mod storage;
mod transaction;
mod validator;
mod wallet;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    info!("╔════════════════════════════════════════╗");
    info!("║   MRBN Testnet Node v0.1.0            ║");
    info!("║   Micro Resource Based Network        ║");
    info!("╚════════════════════════════════════════╝");
    info!("");

    // Parse CLI arguments
    let (config, command) = cli::parse_args()?;
    
    // Handle CLI commands
    match command {
        Some(cli::Commands::Wallet { action }) => {
            // Initialize storage for balance queries
            let storage = storage::StorageManager::new(&config.data_dir)?;
            
            match action {
                cli::WalletAction::Create { name, password } => {
                    cli::commands::wallet_create(name, password, &config.wallet_dir)?;
                }
                cli::WalletAction::List => {
                    cli::commands::wallet_list(&config.wallet_dir)?;
                }
                cli::WalletAction::Balance { address } => {
                    cli::commands::wallet_balance(address, &storage)?;
                }
                cli::WalletAction::Send { from, to, amount, gas } => {
                    cli::commands::wallet_send(from, to, amount, gas, &storage)?;
                }
            }
            return Ok(());
        }
        Some(cli::Commands::Status) => {
            let storage = storage::StorageManager::new(&config.data_dir)?;
            cli::commands::node_status(&storage)?;
            return Ok(());
        }
        Some(cli::Commands::Start) | None => {
            // Continue to start node
        }
    }

    info!("📋 Configuration:");
    info!("   Data Dir: {:?}", config.data_dir);
    info!("   Port: {}", config.port);
    info!("   Validator: {}", config.validator);
    info!("");

    // Create and start the integrated node
    let mut node = node::MrbnNode::new(config)?;
    node.start().await?;

    info!("💡 Tip: Start another node in a different terminal to see peer discovery!");
    info!("");

    // Run the node event loop
    node.run().await?;

    Ok(())
}
