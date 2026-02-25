# Fixed-Point Engine: Overflow & Truncation Semantics

The physics engine must formally declare how it handles mathematical edge cases. Unpredictable rounding or silent state corruption are the most common sources of state-drift in deterministically distributed systems.

## 1. Global Overflow Policy
What happens if the internal math exceeds the `u128` boundary? (Even though we have proven this represents $10^{19}$ scaled units).

**Decision: Explicit Logical Halt  (Checked Math -> Reject)**

We MUST NOT rely on Rust's native panic behavior, as environmental compiler optimizations (e.g., release vs debug flags) can alter how and when a panic traps, destroying cross-engine determinism.

Every single mathematical operation inside the consensus-critical path MUST use explicit checked variants:
*   `checked_add`
*   `checked_sub`
*   `checked_mul`
*   `checked_div`

If any operation returns `None`, the kernel MUST explicitly propagate a `TransitionError::MathOverflow`.
*   The transaction causing the overflow is instantly invalidated.
*   The kernel cleanly halts and rejects the state transition, ensuring deterministic failure regardless of the host environment.

*Note: As defined in the slashing spec, `saturating_sub` is reserved EXCLUSIVELY for penalizing actors down to exactly `0` without halting the epoch transition.*

## 2. Dust Truncation Semantics (Decay Physics)
Because we operate a thermodynamic decay (e.g., `balance * 0.9439`), we are fundamentally multiplying a fixed-point fraction by a base integer.

```rust
let test_balance = 1_000_000_000_005; // 1.000_000_000_005 units
// Decay factor at 1e12 scale: 943_932_824_245
let product = test_balance * DECAY_FACTOR_PER_EPOCH.raw;
let decayed_balance = product / SCALE;
```

**The Rule: Integer Division Truncation (Absolute Floor)**
Rust's integer division (`/`) inherently truncates toward zero. We explicitly rely on and freeze this behavior.
*   The remainder of the division is permanently burned.
*   There is no "rounding to nearest", no floating-point emulation, and no saving the dust to be added to another account.

**Impact & Subtle Centralization Bias:**
The protocol structurally destroys fractional energy outside the $10^{12}$ precision barrier on every decay iteration. This guarantees rounding asymmetries cannot be exploited to artificially generate energy (e.g., splitting an identity to round up twice). 

However, this introduces a **weak anti-fragmentation bias**.
If one identity holds `1,000,000,000,005`, it loses the remainder dust (`0.000_000_000_005`) once during division. If that identical balance is split across three identities, each receives the decay multiplication and the subsequent `/ SCALE` division independently, losing remainder dust *three times*. 

Therefore, large consolidated holders are slightly advantaged thermodynamically over time compared to highly fragmented holders. This explicitly documented consequence reinforces the gravity of Identity Singularity and disincentivizes Sybil dust-spreading.
