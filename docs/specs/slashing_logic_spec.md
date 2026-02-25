# Deterministic Slashing Logic (Step 6)

When the Fraud Replay Module proves that an `EpochStateRoot` was maliciously modified, the offending committee members must be slashed. Because we operate in the "Legitimacy-First" paradigm, we must define the physics of slashing *before* we define the monetary emission model (Step 7).

## 1. Slashing is a Mathematical Boolean
The execution of a slash must be purely subtractive and deterministic. It is a boolean trigger: `IF fraud == true -> execute_slash(signers_bitmap)`.

We define slashing independently of yield distributions. Slashing permanently burns a portion of the actor's Accountability Score (their liquid balance and/or locked bonds).

### The Slashing Trigger
Within the `apply_epoch` function (if a valid `FraudProof` is included in the epoch payloads):
1.  Isolate the public keys of the committee members who signed the fraudulent `EpochStateRoot` using the `signers_bitmap`.
2.  Iterate over the specific `merkle_state_witnesses` for those keys.
3.  Perform a `checked_sub` operation, removing the pre-defined `SLASHING_PENALTY_FRACTION`.

## 2. Slashing Idempotency & Bounding
A malicious attacker could theoretically bundle hundreds of valid `FraudProof` payloads targeting the same fraudulent committee in a single epoch, attempting to repeatedly compound the slash and drive validator balances to mathematically unrecoverable dust in a single transition.

**The Immutable Rule: Slash Idempotency (Max One Slash Per Epoch).**
Slashing is capped at *exactly one occurrence per validator, per epoch*. 

Within the `apply_epoch` function:
1.  The execution maintains a local `slashed_this_epoch` HashSet.
2.  When a `FraudProof` triggers a slash against a key, the key is inserted into the set.
3.  Any subsequent `FraudProof` in the identical epoch targeting a key already in the `slashed_this_epoch` set is mathematically valid, but the subtractive penalty is **skipped**. 

This caps the adversarial damage of griefing attacks while ensuring the primary penalty is severe but controlled.

## 3. Parameterizing the Penalty
We define the slashing penalty as a fixed-point constant, applied immediately upon fraud resolution, overriding standard thermodynamic decay for that epoch.

```rust
// Example: A 50% slash of all accumulated weight.
// Scale: 10^12
const SLASHING_PENALTY_FRACTION: FixedPoint = FixedPoint {
    raw: 500_000_000_000, 
};

// Application
let penalty_amount = (current_balance.raw * SLASHING_PENALTY_FRACTION.raw) / SCALE;
// Safe subtraction: Balances cannot go below zero.
let new_balance = FixedPoint { 
    raw: current_balance.raw.saturating_sub(penalty_amount) 
};
```

*Note: While `checked_sub` is used generally to prevent overflow, `saturating_sub` is strictly used for the slashing application to ensure a balance bounded at `0` without causing a transition-halting panic, completely stripping the malicious validator of power.*

## 3. Disconnection from Yield Expansion
Crucially, the slashing logic does *not* redistribute the slashed funds to other validators. This avoids creating an economic incentive for validators to falsely accuse each other to farm slashed yield. 

The slashed energy is **permanently burned from the domain.** It is pure thermodynamic destruction of the malicious actor's weight. This guarantees that the slashing system is structurally decoupled from the soon-to-be-defined Token Economics / Bond-Minting model. The constitutional skeleton can be proven secure in isolation.
