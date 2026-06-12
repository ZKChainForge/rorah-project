//! RORAH command-line interface.
//!
//! Provides commands for folding and verifying Nova proofs.

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;

mod commands;

use commands::{fold::FoldCommand, verify::VerifyCommand};

/// RORAH - Rollup-of-Rollups Aggregation Hub
///
/// Nova folding engine for circuit-agnostic proof aggregation.
#[derive(Parser, Debug)]
#[clap(
    name = "rorah",
    version = env!("CARGO_PKG_VERSION"),
    author = "RORAH Team",
    about = "Nova folding engine for rollup proof aggregation",
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// Verbosity level (0 = quiet, 1 = info, 2 = debug)
    #[clap(short, long, default_value = "1", global = true)]
    verbosity: u8,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fold a set of R1CS instances into a Nova accumulator
    Fold(FoldCommand),

    /// Verify a Nova accumulator
    Verify(VerifyCommand),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging based on verbosity
    let log_level = match cli.verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    std::env::set_var("RUST_LOG", log_level);
    env_logger::init();

    info!("RORAH CLI v{}", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Commands::Fold(cmd) => cmd.run().await,
        Commands::Verify(cmd) => cmd.run().await,
    }
}