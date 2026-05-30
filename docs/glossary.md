
# RORAH Technical Glossary

## Core Concepts

### R1CS (Rank-1 Constraint System)
A mathematical representation of computation used in zero-knowledge proof systems.

**Structure:**
- Three sparse matrices: A, B, C
- Constraint form: (Az) ∘ (Bz) = Cz
- z is the witness vector (variable assignments)
- ∘ denotes element-wise (Hadamard) product

**Example:**
```
Compute y = x²
Variables: [1, x, y]
Constraint: x · x = y

In R1CS form:
A = [0, 1, 0]  (selects x)
B = [0, 1, 0]  (selects x)
C = [0, 0, 1]  (selects y)
```

**Security Property:**
Satisfying an R1CS instance proves knowledge of a valid computation without revealing private inputs.

---

### Relaxed R1CS
Extension of standard R1CS that allows "slack" for efficient proof composition.

**Satisfying Condition:**
```
Az ∘ Bz = u·Cz + E
```

Where:
- `u` is a scalar relaxation factor
- `E` is an error vector
- When u=1 and E=0, reduces to standard R1CS

**Purpose:**
Enables Nova's incremental folding without expensive proof recursion.

**Initialization:**
```rust
// Standard R1CS → Relaxed R1CS
u = 1
E = [0, 0, ..., 0]
```

---

### IVC (Incrementally Verifiable Computation)
A proof system where proofs can be built step-by-step.

**Traditional Approach:**
```
Compute f(f(f(x))) → Generate one large proof
```

**IVC Approach:**
```
Step 1: Prove f(x) = y₁
Step 2: Prove f(y₁) = y₂ (using proof from step 1)
Step 3: Prove f(y₂) = y₃ (using proof from step 2)
```

**Advantage:**
- Constant proof size regardless of computation length
- Prover time grows linearly, not exponentially
- Critical for aggregating many proofs

---

### Nova Folding Scheme
A specific IVC construction that uses relaxed R1CS.

**Key Innovation:**
Instead of recursively verifying proofs, Nova "folds" instances together.

**Folding Operation:**
```
Given:
  - Accumulator: (R₁, W₁) with relaxation (u₁, E₁)
  - New instance: (R₂, W₂)

Output:
  - Folded accumulator: (R', W') with relaxation (u', E')
  
Where:
  u' = u₁ + r · u₂
  E' = E₁ + r · T + r² · E₂
  W' = W₁ + r · W₂
```

**Critical Property:**
Folding requires only O(1) group operations, not recursive proof verification.

---

## Proof Systems

### SNARK (Succinct Non-interactive ARgument of Knowledge)
A proof system with:
- **Succinct:** Proof size is small (typically 100-500 bytes)
- **Non-interactive:** Prover sends one message
- **Argument:** Computationally sound (not information-theoretic)
- **Knowledge:** Proves prover knows a witness

**Common SNARK Systems:**
- **Groth16:** Fastest verification, trusted setup, 2 pairings
- **PLONK:** Universal trusted setup, 3 pairings
- **Halo2:** No trusted setup, uses IPA (inner product argument)

---

### STARK (Scalable Transparent ARgument of Knowledge)
A proof system with:
- **Scalable:** Prover time quasi-linear in computation size
- **Transparent:** No trusted setup required
- **Larger proofs:** Typically 50-200 KB

**Common STARK Systems:**
- **Boojum (zkSync):** Custom STARK with FRI
- **Cairo (StarkNet):** AIR-based STARK for Cairo VM
- **RISC Zero:** STARK for RISC-V instruction set

**Advantage over SNARKs:**
- No trusted setup (quantum-resistant)
- Better prover scalability for large computations

**Disadvantage:**
- Larger proof size
- Slower verification

---

### FRI (Fast Reed-Solomon Interactive Oracle Proof)
The core cryptographic primitive in most STARKs.

**Purpose:**
Prove that a committed polynomial has low degree.

**Process:**
1. Commit to polynomial evaluations over large domain
2. Verifier challenges with random points
3. Prover reveals evaluations + Merkle paths
4. Repeat with folded polynomial (half the degree)

**Security:**
- Based on hash function collision resistance
- No number-theoretic assumptions (post-quantum secure)

---

## Cryptographic Primitives

### Pedersen Commitment
A vector commitment scheme used in Nova.

**Commitment:**
```
C = Σᵢ mᵢ · Gᵢ + r · H
```

Where:
- `mᵢ` are message elements
- `Gᵢ, H` are random group generators
- `r` is blinding factor (for hiding)

**Properties:**
- **Binding:** Cannot find two different messages with same commitment
- **Hiding:** Commitment reveals no information about message (with blinding)
- **Additively Homomorphic:** C(m₁) + C(m₂) = C(m₁ + m₂)

**Security Assumption:**
Discrete logarithm problem in the group.

**Usage in Nova:**
Commit to cross-term polynomial T during folding.

---

### Fiat-Shamir Transform
Technique to make interactive proofs non-interactive.

**Interactive Protocol:**
```
Prover → Commitment → Verifier
Verifier → Random challenge → Prover
Prover → Response → Verifier
```

**Fiat-Shamir:**
```
Prover computes:
  challenge = Hash(commitment, public_inputs, context)

No verifier interaction needed.
```

**Security:**
Requires random oracle model (hash modeled as truly random function).

**Implementation in RORAH:**
```rust
transcript.absorb(b"commitment", &commitment);
transcript.absorb(b"public_inputs", &public_inputs);
let challenge = transcript.squeeze();
```

---

### Poseidon Hash
A hash function optimized for zero-knowledge circuits.

**Why Poseidon:**
- Uses field operations (additions and multiplications)
- Much cheaper in circuits than SHA-256 or Keccak
- Specifically designed for SNARK/STARK-friendly hashing

**Structure:**
- Sponge construction (like SHA-3)
- Operates on field elements directly
- Uses S-box: x → x^α (typically α=5)

**Security:**
- Based on algebraic attacks analysis
- Conservative parameter choices
- Peer-reviewed design

**Usage in RORAH:**
Used in Fiat-Shamir transcript for generating challenges inside circuits.

---

## EigenLayer Concepts

### Restaking
Using already-staked ETH to secure additional services.

**Traditional Staking:**
```
User stakes 32 ETH → Secures Ethereum consensus → Earns ~4% APR
```

**Restaking:**
```
User stakes 32 ETH → Secures Ethereum consensus
                   ↓
                   Also secures RORAH (AVS)
                   ↓
                   Earns 4% APR + RORAH fees + EigenLayer rewards
```

**Risk:**
- Additional slashing conditions (from AVS)
- If RORAH operator misbehaves, restaked ETH can be slashed

**Benefit:**
- Higher capital efficiency
- More rewards without additional capital

---

### AVS (Actively Validated Service)
A protocol that uses EigenLayer's restaked security.

**Components:**
1. **Service Manager Contract:**
   - Registers operators
   - Assigns tasks
   - Handles slashing

2. **Off-chain Service:**
   - Operators run software (RORAH aggregation node)
   - Perform computation (fold proofs)
   - Submit results to L1

3. **Verification:**
   - On-chain verification of results
   - Slashing for invalid submissions

**RORAH as AVS:**
```
Service: Aggregate rollup proofs using Nova
Task: Fold 50 proofs every 12 seconds
Verification: Groth16 proof verification on L1
Slashing: 50% of stake for invalid proof
```

---

### Slashing
Penalty for misbehavior by a restaked operator.

**Slashing Conditions in RORAH:**

1. **Invalid Proof Submission (50% slash):**
   ```
   Operator submits aggregated proof that fails verification
   → Slash 500 ETH from 1000 ETH stake
   ```

2. **Liveness Fault (10% slash):**
   ```
   Operator fails to submit proof by deadline
   → Slash 100 ETH
   ```

3. **Censorship (25% slash):**
   ```
   Operator intentionally excludes valid rollup proof
   → Slash 250 ETH (provable via fraud proof)
   ```

**Slashed Funds Distribution:**
```
50% → Burned (reduces total supply)
25% → Challenge fund (rewards fraud proof submitters)
25% → RORAH treasury (protocol development)
```

---

## RORAH-Specific Terms

### Cross-Term Polynomial (T)
The key computation in Nova folding.

**Definition:**
```
T = (Az₁) ∘ (Bz₂) + (Az₂) ∘ (Bz₁) - u₁·(Cz₂) - u₂·(Cz₁)
```

**Purpose:**
Captures the "interaction" between two R1CS instances during folding.

**Size:**
- Vector with `num_constraints` elements
- Must be committed with Pedersen commitment

**Computation Cost:**
- 4 matrix-vector multiplications
- O(num_constraints) field operations
- Dominates folding time

**Security:**
Commitment to T binds the prover to a specific folding.

---

### Verifier Circuit
An R1CS circuit that verifies a proof from another system.

**Example - Groth16 Verifier Circuit:**
```
Input (public): verification key hash, public inputs
Input (private): Groth16 proof (A, B, C points)

Constraints:
  1. Verify proof points are on curve
  2. Compute pairing checks
  3. Output: is_valid (0 or 1)
```

**Challenge:**
Pairings require ~6M R1CS constraints (very expensive).

**Why Needed:**
To fold heterogeneous proofs, we verify each proof's native verifier inside a circuit.

---

### Heterogeneous Proof Aggregation
Aggregating proofs from different proof systems.

**Example:**
```
Proof 1: Boojum (STARK with FRI)
Proof 2: Plonky2 (SNARK with custom gates)
Proof 3: Halo2 (SNARK with IPA)

Traditional approach: Convert all to Groth16 (expensive!)

RORAH approach:
  - Wrap each verifier in R1CS circuit
  - Fold all verifier executions with Nova
  - No proof system translation needed
```

**Advantage:**
Each rollup keeps its optimized proof system.

---

### Accumulator
The running state in Nova's IVC.

**Contents:**
```rust
struct NovaAccumulator {
    instance: RelaxedR1CSInstance,  // Current R1CS state
    witness: Witness,                // Current witness
    u: FieldElement,                 // Relaxation factor
    E: Vec<FieldElement>,            // Error vector
}
```

**Initialization:**
```
u = 0
E = []
instance = empty R1CS
```

**After Folding N Proofs:**
```
u = 1 + r₁ + r₂ + ... + rₙ₋₁
E = [sum of all cross-terms and errors]
instance = constraints from all N R1CS instances
```

**Final Verification:**
Prove accumulator is valid → proves all N original instances were valid.

---

## Field Theory

### BN254 (Barreto-Naehrig Curve)
An elliptic curve used in many SNARKs.

**Scalar Field Prime:**
```
p = 21888242871839275222246405745257275088548364400416034343698204186575808495617
```

**Properties:**
- 254-bit prime
- Pairing-friendly (supports bilinear maps)
- Used by: Groth16, PLONK, some Halo2 implementations

**Security Level:**
~128-bit security (before recent pairing attacks reduced it slightly).

---

### Goldilocks Field
A 64-bit prime field used in Plonky2.

**Prime:**
```
p = 2^64 - 2^32 + 1
```

**Advantages:**
- Fits in a 64-bit integer
- Fast native CPU arithmetic (no bigint needed)
- SIMD-friendly for vectorization

**Disadvantage:**
- Smaller field (less security margin)
- Requires extension fields for pairing-like operations

---

### Field Element
An element of a finite field F_p.

**Operations:**
```
Addition: (a + b) mod p
Subtraction: (a - b) mod p
Multiplication: (a × b) mod p
Inversion: a^(-1) such that a × a^(-1) = 1 (mod p)
```

**Representation:**
- Montgomery form (for fast multiplication)
- Residue number system (for parallel operations)

**Security Consideration:**
All operations must be constant-time to prevent timing attacks.

---

## Matrix Operations

### Sparse Matrix
A matrix where most elements are zero.

**Representation:**
```rust
struct SparseMatrix {
    entries: Vec<(row, col, value)>  // Only non-zero entries
}
```

**Advantage:**
- R1CS matrices are typically 99%+ sparse
- Memory: O(non-zero entries) instead of O(rows × cols)
- Multiplication: O(non-zero entries) instead of O(rows × cols)

**Example:**
```
Dense matrix (4×4, 16 elements):
[1 0 0 0]
[0 2 0 0]
[0 0 3 0]
[0 0 0 4]

Sparse representation (4 entries):
[(0,0,1), (1,1,2), (2,2,3), (3,3,4)]
```

---

### Hadamard Product (∘)
Element-wise multiplication of vectors.

**Definition:**
```
[a₁, a₂, a₃] ∘ [b₁, b₂, b₃] = [a₁b₁, a₂b₂, a₃b₃]
```

**Usage in R1CS:**
```
Constraint: Az ∘ Bz = Cz

Means: For each row i:
  (Az)[i] × (Bz)[i] = (Cz)[i]
```

**Not to be confused with:**
- Dot product: a·b = Σ aᵢbᵢ (produces scalar)
- Matrix multiplication: AB (matrix result)

---

## Security Concepts

### Soundness
Property that a proof system cannot prove false statements.

**Formal Definition:**
```
For any efficient adversary A:
  Pr[A generates valid proof for false statement] < negligible(λ)
```

Where λ is the security parameter.

**RORAH Soundness:**
If aggregated proof verifies, all component rollup proofs were valid.

**Depends On:**
1. Nova soundness (proven in original paper)
2. Verifier circuit correctness (must match native verifier)
3. Groth16 soundness (for final compression)

---

### Completeness
Property that valid statements always have valid proofs.

**Formal Definition:**
```
For any true statement S:
  Honest prover can generate proof that verifies with probability 1
```

**RORAH Completeness:**
If all rollup proofs are valid, aggregation always succeeds.

**Failure Modes:**
- Implementation bugs (circuit errors)
- Resource limits (out of memory, timeout)
- L1 gas limits (should never happen with 180k gas)

---

### Knowledge Soundness
Stronger than soundness: prover must "know" a witness.

**Formal Definition:**
```
For any efficient prover P that generates valid proof:
  ∃ efficient extractor E that can extract witness from P
```

**Purpose:**
Prevents proofs that are valid but don't correspond to actual computation.

**RORAH Requirement:**
Each rollup proof must be knowledge-sound (not just sound).

---

### Trusted Setup
Ceremony to generate public parameters for some proof systems.

**Process:**
1. Generate random "toxic waste" τ
2. Compute public parameters: [τ⁰G, τ¹G, τ²G, ...]
3. Delete τ (if anyone knows τ, they can forge proofs)

**Systems Requiring Trusted Setup:**
- Groth16 (circuit-specific)
- PLONK (universal, one-time)

**Systems Without Trusted Setup:**
- STARKs (transparent)
- Halo2 (uses bulletproofs-style IPA)
- Nova (only needs group generators)

**RORAH:**
- Nova folding: No trusted setup
- Final Groth16 compression: Requires trusted setup
  - Can use existing trusted setup (e.g., Zcash, Hermez)
  - Or conduct new ceremony

---

## Performance Metrics

### Constraint Count
Number of R1CS constraints in a circuit.

**Examples:**
- Simple addition: 0 constraints (linear operation)
- Multiplication: 1 constraint
- SHA-256 hash: ~25,000 constraints
- Groth16 pairing check: ~6,000,000 constraints
- Boojum STARK verifier: ~5,000,000 constraints

**Impact:**
- Proving time scales roughly linearly with constraints
- Memory usage also linear
- Verification time independent (for SNARKs)

---

### Proving Time
Time to generate a proof.

**Typical Times (for 1M constraints):**
- Groth16: 5-10 seconds (with GPU)
- PLONK: 10-20 seconds
- Halo2: 8-15 seconds
- STARKs: 2-5 seconds (highly parallel)

**Factors:**
- Number of constraints
- Hardware (CPU cores, GPU, RAM)
- Field arithmetic speed
- Implementation quality

**RORAH Target:**
Fold 50 proofs in < 12 seconds (L1 slot time).

---

### Verification Time
Time to verify a proof.

**On-Chain (L1 Gas):**
- Groth16: ~180,000 gas (~0.5s block time)
- PLONK: ~300,000 gas
- STARKs: 1-5M gas (too expensive for frequent use)

**Off-Chain:**
- Groth16: <10 ms
- PLONK: ~50 ms
- STARKs: 10-100 ms

**RORAH Design:**
Use Groth16 for final compression to minimize L1 gas cost.

---

### Proof Size
Bytes needed to represent a proof.

**Typical Sizes:**
- Groth16: 128-256 bytes (2-3 group elements)
- PLONK: 384-512 bytes
- Halo2: 384-768 bytes (no pairings, larger commitments)
- STARKs: 50-200 KB (depends on security parameter)

**Nova IVC Proof:**
- Per fold: 32-64 bytes (just one group element commitment)
- After compression: 256 bytes (Groth16 proof)

**RORAH Advantage:**
Aggregate 50 proofs (potentially 10 MB native STARKs) → 256 bytes.

---

## Development Tools

### arkworks
Rust library for zero-knowledge cryptography.

**Components:**
- `ark-ff`: Finite field arithmetic
- `ark-ec`: Elliptic curve operations
- `ark-poly`: Polynomial arithmetic
- `ark-groth16`: Groth16 SNARK implementation
- `ark-bn254`: BN254 curve implementation

**Usage in RORAH:**
```rust
use ark_bn254::Fr;  // BN254 scalar field
use ark_ec::Group;  // Generic group operations
```

---

### Circom
Domain-specific language for writing arithmetic circuits.

**Example:**
```circom
template Multiplier() {
    signal input a;
    signal input b;
    signal output c;
    
    c <== a * b;
}
```

**Compilation:**
```
circom circuit.circom --r1cs --wasm --sym
```

**Output:**
- R1CS constraints
- Witness generation code (WebAssembly)

**RORAH:**
May use Circom to prototype verifier circuits before Rust implementation.

---

### cargo-deny
Tool for supply chain security in Rust.

**Checks:**
- Known vulnerabilities (via RustSec database)
- License compliance
- Duplicate dependencies
- Banned crates

**Configuration:**
```toml
[advisories]
vulnerability = "deny"  # Fail build on known CVEs
```

**RORAH Requirement:**
All dependencies must pass cargo-deny checks.

---

## Common Abbreviations

- **ZK:** Zero-Knowledge
- **zkSNARK:** Zero-Knowledge Succinct Non-interactive Argument of Knowledge
- **zkSTARK:** Zero-Knowledge Scalable Transparent Argument of Knowledge
- **R1CS:** Rank-1 Constraint System
- **QAP:** Quadratic Arithmetic Program (alternative to R1CS)
- **AIR:** Algebraic Intermediate Representation (used in STARKs)
- **FRI:** Fast Reed-Solomon Interactive Oracle Proof
- **IVC:** Incrementally Verifiable Computation
- **PCD:** Proof-Carrying Data (similar to IVC)
- **IOP:** Interactive Oracle Proof
- **IPA:** Inner Product Argument
- **MSM:** Multi-Scalar Multiplication
- **FFT:** Fast Fourier Transform (used in polynomial operations)
- **NTT:** Number-Theoretic Transform (FFT over finite fields)
- **SRS:** Structured Reference String (trusted setup output)
- **CRS:** Common Reference String
- **AVS:** Actively Validated Service (EigenLayer)
- **DA:** Data Availability

---

## References

### Academic Papers
1. **Nova**: Recursive Zero-Knowledge Arguments from Folding Schemes
   - Authors: Kothapalli et al.
   - Link: https://eprint.iacr.org/2021/370

2. **PLONK**: Permutations over Lagrange-bases for Oecumenical Noninteractive arguments of Knowledge
   - Authors: Gabizon et al.
   - Link: https://eprint.iacr.org/2019/953

3. **Groth16**: On the Size of Pairing-based Non-interactive Arguments
   - Author: Jens Groth
   - Link: https://eprint.iacr.org/2016/260

### Implementation Resources
- arkworks documentation: https://docs.rs/ark-ff/
- EigenLayer docs: https://docs.eigenlayer.xyz/
- Nova reference implementation: https://github.com/microsoft/Nova

### Security
- RustSec Advisory Database: https://rustsec.org/
- Zero-Knowledge Security Blog: https://www.zksecurity.xyz/
```