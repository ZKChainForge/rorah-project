use crate::cairo::types::CairoProofData;

pub struct ExecutionVerifier;

impl ExecutionVerifier {
    pub fn verify_execution_trace(
        proof: &CairoProofData,
    ) -> anyhow::Result<bool> {
        if proof.num_steps == 0 {
            anyhow::bail!("num_steps must be > 0");
        }

        if proof.trace_commitments.is_empty() {
            anyhow::bail!("trace_commitments cannot be empty");
        }

        let required_trace_columns = 3;
        if proof.trace_commitments.len() < required_trace_columns {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn verify_pc_transitions(
        trace: &[Vec<u8>],
        num_steps: u64,
    ) -> bool {
        if trace.is_empty() {
            return false;
        }

        let trace_length = trace[0].len();
        trace_length as u64 >= num_steps
    }

    pub fn verify_instruction_decoding(
        instructions: &[Vec<u8>],
    ) -> bool {
        instructions.iter().all(|instr| !instr.is_empty() && instr.len() <= 32)
    }

    pub fn verify_operand_resolution(
        memory: &[Vec<u8>],
        operands: &[usize],
    ) -> bool {
        operands.iter().all(|&idx| idx < memory.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_verification() {
        let proof = CairoProofData {
            trace_commitments: vec![vec![0u8; 32], vec![1u8; 32], vec![2u8; 32]],
            composition_polynomial: vec![1u8; 64],
            fri_proof: vec![1u8; 256],
            decommitment_values: vec![vec![1u8; 32]],
            num_steps: 1024,
        };

        let result = ExecutionVerifier::verify_execution_trace(&proof);
        assert!(result.is_ok());
    }
}