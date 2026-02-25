# The `Fixed` Wrapper Struct (Phase 11)

To mechanically guarantee the bounds of the 1e12 fixed-point engine, the underlying `u128` scale must never be manipulated directly by the consensus layers. 

## 1. Encapsulation Constraint

The Rust implementation will strictly define a wrapper tuple struct. The internal `.0` value is private to the `math` module. The consensus executor (`apply_epoch`) imports the struct but cannot access the private inner value, making unchecked mathematical chaining a compiler error.

```rust
pub mod math {
    const SCALE: u128 = 1_000_000_000_000;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub struct Fixed(u128);
    
    // ... trait implementations ...
}
```

## 2. Permitted Methods

The `math` module exposes strictly checked methods that inherently manage the scaling conversions and overflow checks. 

```rust
impl Fixed {
    /// Creates a Fixed value from raw units, validating it does not exceed 
    /// the safe chained-multiplication ceiling (u128::MAX / SCALE).
    pub fn from_scaled(val: u128) -> Result<Self, TransitionError>;

    /// Multiplies exactly two scaled values, performing the `/ SCALE` reduction 
    /// safely internally before returning the new Fixed struct.
    pub fn mul_scaled(self, other: Fixed) -> Result<Self, TransitionError>;

    /// Divides two scaled values: (A * SCALE) / B.
    /// Explicitly catches division by zero, returning MathOverflow rather than a Rust panic.
    pub fn div_scaled(self, other: Fixed) -> Result<Self, TransitionError>;
    
    // Checked Addition & Subtraction ...
}
```

## 3. Division By Zero Policy

Rust compilers will predictably panic if dividing by zero (`10 / 0`). In WASM, this triggers an unrecoverable trap. 
All consensus logic MUST manually check denominator values *before* invoking Rust's native `/` operator.

If `.div_scaled(other)` encounters `other.0 == 0`, it MUST immediately return `Err(TransitionError::MathOverflow)`. The transition halts cleanly and logically. 

This guarantees the constitutional laws defined in the specs are mechanically enforced by the compiler, decoupling consensus stability from environmental optimizer behavior.
