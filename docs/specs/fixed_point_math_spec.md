# Deterministic Numeric Specification (The Numbers)

Floating-point mathematics (IEEE 754) is inherently non-deterministic across different hardware architectures, compilers, and OS math libraries. To guarantee replay invariance, the Civilisation OS physics kernel permanently bans floating-point numbers.

All math within the WASM execution engine operates on a strictly defined `FixedPoint` scaling model.

## 1. The Fixed-Point Structure

We adopt a base 12 **(10^12)** scaling factor. 
*   `10^9` (Gwei scale) risks precision truncation over decades of compounding decay.
*   `10^18` (Wei scale) requires constant `U256` software emulation for intermediate multiplications because `10^18 * 10^18 = 10^36`, running dangerously close to `u128::MAX` (~`3.4e38`).
*   `10^12` is the perfect WASM sweet spot. It provides high precision while `10^12 * 10^12 = 10^24`, leaving $10^{14}$ base units of headroom for intermediate calculation products before overflowing native `i128` hardware types.

### The Rust Model
Because Accountability Scores (balances) and Yield distributions never drop below zero, the underlying integer is strictly **unsigned** to remove negative overflow edge cases, signed division anomalies, and truncation ambiguity. Slashing reduces balances to a minimum bound of 0 via `checked_sub`. 

```rust
const SCALE: u128 = 1_000_000_000_000; // 10^12 precision

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedPoint {
    pub raw: u128,
}
```

## 2. Arithmetic Rules

All arithmetic operators must explicitly use checked hardware operations. Silent wrapping is a catastrophic protocol failure.

1.  **Checked Everything:**
    *   `raw.checked_add()`
    *   `raw.checked_sub()`
    *   `raw.checked_mul()`
    *   `raw.checked_div()`
2.  **Order of Operations (Multiplication & Division):**
    *   To multiply two FixedPoint numbers: `(a.raw * b.raw) / SCALE` (intermediate product checked).
    *   To divide two FixedPoint numbers: `(a.raw * SCALE) / b.raw` (intermediate product checked).
3.  **Hard Error on Overflow:** If any operation overflows the 128-bit boundary, it throws a deterministic Hard Error. An arithmetic overflow during state transition instantly categorizes the transition as invalid (slashing condition if proposed by a committee).

## 3. The Rounding Rule

When division results in a fractional remainder (e.g., during decay calculation or yield division), the protocol must deterministically resolve the dropped precision.

**The Immutable Rule: Floor Rounding.**
All divisions round DOWN towards negative infinity. 

*Rationale:* Bankers rounding (half-to-even) introduces unnecessary conditional logic and potential for cross-implementation variance. Floor rounding guarantees that the protocol *never* creates value out of thin air via rounding artifacts. The fractional dust is simply permanently burned (thermodynamic heat loss).

## 4. The Decay Function (`e^-0.0577`)

The WASM runtime cannot execute `f64::exp()`. Dynamic exponentials are completely banned from consensus logic.

Instead, the Thermodynamic Decay constant (which represents a 12-epoch half-life) is **precomputed and hardcoded** into the consensus constants as a exact `FixedPoint` scalar multiplier.

**Derivation Method (Frozen):**
The constant is computed entirely offline using arbitrary-precision mathematics. The result is **truncated (not rounded)** precisely at the 12th decimal place before scaling.

```rust
// Calculated offline: e^(-0.0577) ≈ 0.943932824245...
// Truncated at 12 decimals exactly: 943932824245
const DECAY_FACTOR_PER_EPOCH: FixedPoint = FixedPoint {
    raw: 943_932_824_245, // Precomputed at 10^12 scale
};
```

When an epoch rolls over, a user's balance is updated via:
`balance = balance.checked_mul_fixed(DECAY_FACTOR_PER_EPOCH)`

With floor rounding enforced, the decay is absolute, mathematically frozen, and byte-for-byte identical whether run on a Macbook ARM processor or an ancient Windows x86 virtual machine.
