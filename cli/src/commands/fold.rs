//! `rorah fold` command - fold a sequence of R1CS instances.

use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use log::info;
use rorah_core::{CommitmentParams, fold_instances, NovaAccumulator};
use rorah_core::r1cs::{R1CSInstance, Witness};
use rorah_core::field::bn254::BN254FieldElement;
use rorah_utils::{json_from_string, json_to_string};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// Arguments for the fold command.
#[derive(Args, Debug)]
pub struct FoldCommand {
    /// Path to JSON file containing R1CS instances and witnesses
    #[clap(short, long, value_name = "FILE")]
    pub proofs: PathBuf,

    /// Output path for the resulting accumulator
    #[clap(short, long, value_name = "FILE", default_value = "accumulator.json")]
    pub output: PathBuf,

    /// Maximum number of generators to precompute
    #[clap(long, default_value = "1024")]
    pub max_generators: usize,

    /// Print timing information
    #[clap(long)]
    pub timing: bool,
}

/// Input format for fold command.
#[derive(Serialize, Deserialize, Debug)]
struct FoldInput {
    instances: Vec<FoldInputEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FoldInputEntry {
    num_variables: usize,
    num_public_inputs: usize,
    public_inputs: Vec<String>,
    witness: Vec<String>,
}

/// Output format for fold command.
#[derive(Serialize, Deserialize, Debug)]
struct FoldOutput {
    success: bool,
    num_instances_folded: usize,
    num_constraints: usize,
    num_variables: usize,
    time_ms: u128,
    accumulator_file: String,
}

impl FoldCommand {
    pub async fn run(self) -> Result<()> {
        println!("{}", "RORAH Nova Folding Engine".bold().cyan());
        println!("{}", "─".repeat(50).dimmed());

        // Read input file
        let input_str = std::fs::read_to_string(&self.proofs)
            .with_context(|| format!("Failed to read input file: {:?}", self.proofs))?;

        let input: FoldInput = json_from_string(&input_str)
            .with_context(|| "Failed to parse input JSON")?;

        if input.instances.is_empty() {
            anyhow::bail!("No instances provided in input file");
        }

        println!(
            "  {} instances to fold",
            input.instances.len().to_string().bold()
        );
        println!(
            "  Max generators: {}",
            self.max_generators.to_string().bold()
        );
        println!();

        // Initialize commitment parameters
        println!("Initializing commitment parameters...");
        let param_start = Instant::now();
        let params = CommitmentParams::new(self.max_generators);
        println!(
            "  Done in {}ms",
            param_start.elapsed().as_millis().to_string().dimmed()
        );

        // Initialize empty accumulator
        let num_variables = input.instances[0].num_variables;
        let mut accumulator = NovaAccumulator::empty(num_variables)
            .with_context(|| "Failed to initialize empty accumulator")?;

        // Progress bar
        let progress = ProgressBar::new(input.instances.len() as u64);
        progress.set_style(
            ProgressStyle::with_template(
                "  Folding [{bar:40.cyan/blue}] {pos}/{len} {msg}"
            )
            .unwrap()
            .progress_chars("=>-"),
        );

        let fold_start = Instant::now();
        let mut proofs_collected = Vec::new();

        // Fold each instance
        for (i, entry) in input.instances.iter().enumerate() {
            progress.set_message(format!("instance {}", i));

            // Parse instance and witness from entry
            let (instance, witness) = parse_entry(entry)
                .with_context(|| format!("Failed to parse instance {}", i))?;

            // Fold
            let (new_acc, proof) = fold_instances(accumulator, instance, witness, &params)
                .with_context(|| format!("Folding failed at instance {}", i))?;

            accumulator = new_acc;
            proofs_collected.push(proof);
            progress.inc(1);
        }

        let fold_time_ms = fold_start.elapsed().as_millis();
        progress.finish_with_message("done");
        println!();

        // Verify final accumulator
        println!("Verifying final accumulator...");
        accumulator
            .is_valid()
            .with_context(|| "Final accumulator verification failed")?;

        println!("  {}", "Valid".bold().green());

        // Save accumulator
        let acc_json = json_to_string(&accumulator)
            .with_context(|| "Failed to serialize accumulator")?;

        std::fs::write(&self.output, &acc_json)
            .with_context(|| format!("Failed to write accumulator to {:?}", self.output))?;

        // Print summary
        println!();
        println!("{}", "Summary".bold());
        println!("{}", "─".repeat(50).dimmed());
        println!(
            "  Instances folded:  {}",
            input.instances.len().to_string().bold().green()
        );
        println!(
            "  Constraints:       {}",
            accumulator.num_constraints().to_string().bold()
        );
        println!(
            "  Variables:         {}",
            accumulator.num_variables().to_string().bold()
        );
        println!(
            "  Proof size:        {} bytes",
            rorah_core::NovaProof::size_bytes()
                .to_string()
                .bold()
        );
        println!(
            "  Time:              {}ms",
            fold_time_ms.to_string().bold()
        );
        println!(
            "  Output:            {}",
            self.output.display().to_string().bold()
        );

        info!("Folding completed successfully");

        Ok(())
    }
}

/// Parse a fold input entry into R1CSInstance and Witness.
fn parse_entry(entry: &FoldInputEntry) -> Result<(R1CSInstance, Witness)> {
    // Parse public inputs
    let public_inputs: Result<Vec<BN254FieldElement>> = entry
        .public_inputs
        .iter()
        .map(|hex_str| {
            let bytes = rorah_utils::decode_hex(hex_str)
                .with_context(|| format!("Invalid hex in public_inputs: {}", hex_str))?;
            let padded = rorah_utils::pad_left(&bytes, 32);
            BN254FieldElement::from_bytes(
                <&[u8; 32]>::try_from(padded.as_slice())
                    .map_err(|_| anyhow::anyhow!("Failed to convert to [u8; 32]"))?
            )
            .ok_or_else(|| anyhow::anyhow!("Failed to parse public input as field element"))
        })
        .collect();

    let public_inputs = public_inputs?;

    // Parse witness
    let witness_vars: Result<Vec<BN254FieldElement>> = entry
        .witness
        .iter()
        .map(|hex_str| {
            let bytes = rorah_utils::decode_hex(hex_str)
                .with_context(|| format!("Invalid hex in witness: {}", hex_str))?;
            let padded = rorah_utils::pad_left(&bytes, 32);
            BN254FieldElement::from_bytes(
                <&[u8; 32]>::try_from(padded.as_slice())
                    .map_err(|_| anyhow::anyhow!("Failed to convert to [u8; 32]"))?
            )
            .ok_or_else(|| anyhow::anyhow!("Failed to parse witness element as field element"))
        })
        .collect();

    let witness_vars = witness_vars?;
    let public_len = entry.num_public_inputs + 1; // +1 for constant 1

    let witness = Witness::new(witness_vars, public_len)
        .with_context(|| "Failed to create witness")?;

    // For testing: build a simple constraint structure
    // In production this would come from the verifier circuit
    use rorah_core::r1cs::constraint::{Constraint, LinearCombination};
    use rorah_core::field::traits::FieldElement as _;

    let mut constraints = Vec::new();

    // Simple placeholder constraints for demonstration
    for i in 1..entry.num_variables.saturating_sub(1) {
        let mut a = LinearCombination::zero();
        a.add_term(i, BN254FieldElement::one());

        let mut b = LinearCombination::zero();
        b.add_term(i, BN254FieldElement::one());

        let mut c = LinearCombination::zero();
        c.add_term(i + 1, BN254FieldElement::one());

        constraints.push(Constraint::new(a, b, c));
    }

    let instance = R1CSInstance::from_constraints(
        constraints,
        entry.num_variables,
        public_inputs,
    )
    .with_context(|| "Failed to create R1CS instance")?;

    Ok((instance, witness))
}