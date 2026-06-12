//! `rorah verify` command - verify a Nova accumulator.

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use log::info;
use rorah_core::NovaAccumulator;
use rorah_utils::json_from_string;
use std::path::PathBuf;
use std::time::Instant;

/// Arguments for the verify command.
#[derive(Args, Debug)]
pub struct VerifyCommand {
    /// Path to accumulator JSON file
    #[clap(short, long, value_name = "FILE")]
    pub accumulator: PathBuf,

    /// Print detailed information about accumulator
    #[clap(long)]
    pub verbose: bool,
}

impl VerifyCommand {
    pub async fn run(self) -> Result<()> {
        println!("{}", "RORAH Nova Accumulator Verifier".bold().cyan());
        println!("{}", "─".repeat(50).dimmed());

        // Read accumulator file
        let acc_str = std::fs::read_to_string(&self.accumulator)
            .with_context(|| {
                format!("Failed to read accumulator file: {:?}", self.accumulator)
            })?;

        // Deserialize
        println!("Loading accumulator...");
        let accumulator: NovaAccumulator = json_from_string(&acc_str)
            .with_context(|| "Failed to deserialize accumulator")?;

        if self.verbose {
            println!();
            println!("{}", "Accumulator Details".bold());
            println!("{}", "─".repeat(50).dimmed());
            println!(
                "  Constraints:  {}",
                accumulator.num_constraints().to_string().bold()
            );
            println!(
                "  Variables:    {}",
                accumulator.num_variables().to_string().bold()
            );
            println!(
                "  Is empty:     {}",
                if accumulator.is_empty() {
                    "yes".yellow().to_string()
                } else {
                    "no".green().to_string()
                }
            );
            println!(
                "  Error terms:  {}",
                accumulator.error_vector().len().to_string().bold()
            );
            println!(
                "  Public inputs:{}",
                accumulator.public_inputs().len().to_string().bold()
            );
        }

        // Verify
        println!();
        println!("Verifying accumulator...");
        let verify_start = Instant::now();

        match accumulator.is_valid() {
            Ok(()) => {
                let elapsed = verify_start.elapsed().as_millis();
                println!();
                println!("  {} Accumulator is VALID", "✓".bold().green());
                println!(
                    "  Verification time: {}ms",
                    elapsed.to_string().dimmed()
                );

                info!("Accumulator verified successfully");

                Ok(())
            }
            Err(e) => {
                println!();
                println!("  {} Accumulator is INVALID", "✗".bold().red());
                println!("  Error: {}", e.to_string().red());

                anyhow::bail!("Accumulator verification failed: {}", e);
            }
        }
    }
}