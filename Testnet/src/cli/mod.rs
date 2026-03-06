// CLI module for MRBN node commands

#![allow(dead_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod commands;

/// MRBN Node CLI
#[derive(Parser, Debug, Clone)]
#[command(name = "mrbn-node")]
#[command(about = "MRBN Testnet Node - Micro Resource Based Network", long_about = None)]
pub struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
    
    /// Data directory
    #[arg(long, default_value = "./data")]
    pub data_dir: PathBuf,
    
    /// Network port
    #[arg(long, default_value = "8333")]
    pub port: u16,
    
    /// Enable validator mode
    #[arg(long, default_value = "true")]
    pub validator: bool,
    
    /// Wallet directory
    #[arg(long, default_value = "./wallet")]
    pub wallet_dir: PathBuf,
    
    /// Bootstrap node address (format: /ip4/1.2.3.4/tcp/8333/p2p/12D3Koo...)
    #[arg(long)]
    pub bootstrap: Option<String>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Start the node (default if no command specified)
    Start,
    
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
    
    /// Node status and information
    Status,
}

#[derive(Subcommand, Debug, Clone)]
pub enum WalletAction {
    /// Create a new wallet
    Create {
        /// Wallet name
        name: String,
        /// Password for encryption
        #[arg(short, long)]
        password: String,
    },
    
    /// List all wallets
    List,
    
    /// Check balance
    Balance {
        /// Wallet address (hex)
        address: String,
    },
    
    /// Send Kain
    Send {
        /// Sender address
        from: String,
        /// Recipient address
        to: String,
        /// Amount to send
        amount: u64,
        /// Gas fee
        #[arg(default_value = "10")]
        gas: u64,
    },
}

/// CLI configuration
#[derive(Debug, Clone)]
pub struct CliConfig {
    /// Data directory for node storage
    pub data_dir: PathBuf,
    /// Network port
    pub port: u16,
    /// Enable validator mode
    pub validator: bool,
    /// Wallet directory
    pub wallet_dir: PathBuf,
    /// Bootstrap node multiaddr
    pub bootstrap: Option<String>,
}

impl From<Cli> for CliConfig {
    fn from(cli: Cli) -> Self {
        Self {
            data_dir: cli.data_dir,
            port: cli.port,
            validator: cli.validator,
            wallet_dir: cli.wallet_dir,
            bootstrap: cli.bootstrap,
        }
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            port: 8333,
            validator: true,
            wallet_dir: PathBuf::from("./wallet"),
            bootstrap: None,
        }
    }
}

/// Parse command line arguments
pub fn parse_args() -> Result<(CliConfig, Option<Commands>)> {
    let cli = Cli::parse();
    let config = CliConfig::from(cli.clone());
    Ok((config, cli.command))
}
