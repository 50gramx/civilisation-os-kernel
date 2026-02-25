# Fixed-Point Engine: Width & Scaling (Phase 9)

Before defining economic formulas or compiling the Rust kernel, we must absolutely freeze the boundaries of the mathematical simulation. The fixed-point engine is the core primitive that governs all state transitions (decay, slash, bond).

## 1. Integer Width Definition
The simulation requires a standardized unsigned integer width.

**Decision: `u128` (Native Rust)**

### Rationale vs `U256`:
While `U256` (often provided by crates like `primitive-types` or `ruint`) is common in EVM environments, it is entirely software-emulated in WASM, adding significant overhead to every operation.
`u128` maps much more cleanly to hardware and WASM execution bounds. 

*   $2^{128} \approx 3.4 \times 10^{38}$
*   Even with a vast internal scaling factor, $3.4 \times 10^{38}$ provides more than enough precision for a planetary-scale domain without risking overflow during intermediate multiplications.

## 2. Precision Scaling Factor
Because we prohibit JSON Number types and floating-point math, all values are parsed from JSON strings into `u128` and treated as whole integers scaled by a constant factor.

**Decision: `SCALE = 1,000,000,000,000` (1e12)**

### Representation
*   `1.0` (User-facing "One Unit of Accountability") is represented internally as `1,000,000,000,000`.
*   `0.000_000_000_001` is represented as `1` (The smallest atomic unit, or "dust").

### Overflow Safety Bounds (The "Planetary Ceiling")
When multiplying scaled values, the intermediate operation is the highest risk point for overflow. Because `DECAY_FACTOR_PER_EPOCH` is itself scaled by 1e12, the intermediate multiplication of `balance * decay_factor` creates the largest internal number.

*   `Max_u128` $\approx 3.4 \times 10^{38}$
*   The safe balance ceiling is therefore `u128::MAX / SCALE`:
*   $3.4 \times 10^{38} / 10^{12} \approx 3.4 \times 10^{26}$ units.
*   This means an individual identity can safely hold over 340 septillion units of Accountability without overflowing during a decay multiplication.

**Chained Multiplication Constraint:**
To permanently guarantee overflow safety, the kernel MUST NEVER multiply more than two scaled values together without dividing by `SCALE` between the operations.
*   **Valid:** `(((A * B) / SCALE) * C) / SCALE`
*   **Invalid:** `(A * B * C) / SCALE^2` (This will immediately panic for large balances).
