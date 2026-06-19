pub trait Circuit: Send + Sync {
    fn num_constraints(&self) -> usize;
    fn num_inputs(&self) -> usize;
    fn num_witnesses(&self) -> usize;
}

pub struct CircuitMetadata {
    pub num_constraints: usize,
    pub num_inputs: usize,
    pub num_witnesses: usize,
}

impl CircuitMetadata {
    pub fn from_circuit(circuit: &dyn Circuit) -> Self {
        CircuitMetadata {
            num_constraints: circuit.num_constraints(),
            num_inputs: circuit.num_inputs(),
            num_witnesses: circuit.num_witnesses(),
        }
    }

    pub fn total_variables(&self) -> usize {
        self.num_inputs + self.num_witnesses
    }

    pub fn density(&self) -> f64 {
        if self.total_variables() == 0 {
            return 0.0;
        }
        self.num_constraints as f64 / self.total_variables() as f64
    }
}