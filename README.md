# Civilisation OS — Deterministic Execution Kernel

> **This is not a blockchain. This is the execution specification that a blockchain runs.**

## What This Is

A deterministic state transition function, compiled to WebAssembly, with no external dependencies and pinned constitutional test vectors.

**What it does:**
- Takes a previous `EpochState` and produces a new `EpochState`
- Computes a cryptographic commitment (`state_root`) over the new state
- Enforces hard limits on input size and nesting depth
- Chains each epoch's root to the previous one

**What it does not do:**
- Run a network
- Produce blocks
- Validate signatures (stubbed — v0.0.2)
- Verify VDF proofs (stubbed — v0.0.2)
- Hold any persistent state
- Allocate beyond what a single function call requires

The networking layer, validator coordination, mempool, and block production are out of scope for this repository. They are downstream of this layer.

## Constitutional Guarantees

The following properties are enforced by pinned test vectors in the CI:

| Property | Mechanism |
|---|---|
| SHA-256 is FIPS 180-4 compliant | Self-contained implementation, 3 NIST CAVP vectors |
| Canonical JSON is RFC 8785 (JCS) | 36 tests including duplicate-key rejection and schema validation |
| Debug build = Release build | Both run the full test suite in CI against identical pinned hashes |
| WASM compiles without error | CI job 3 |
| `state_root` commits to all fields | Genesis and Epoch 1 and Epoch 100 state roots are pinned |
| 100-epoch chain is replay-stable | Identical final hash across 100 transitions, run twice |

### Pinned constitutional vectors

```
SHA256({"a":"1","b":"2"})
= 21f76dfbfe6dfe21f762080ef484112cf2952974cef30741fd1931e1c6d92112

SHA256(genesis EpochState canonical JSON)
= bb44f7d83e9e4e426809a81b66f72a4944329554fbc05bf8f0789a623b1d5ade1

SHA256(epoch 1 EpochState canonical JSON)
= 10dc6e694843a9a3813fecb49199f5f81ab61da20fe536a09db3b1fbf1908ea1

SHA256(epoch 100 EpochState canonical JSON)
= 238615db678acd7be8460b8dd25015f9560670a1ac17d0836fae6a4272b35799
```

If any of these change, it is a protocol fork.

## Running

```sh
# All tests (debug — overflow-checks on)
cargo test

# All tests (release — optimized)
cargo test --release

# WASM build
cargo build --target wasm32-unknown-unknown --release
```

All three must pass before any PR is merged.

## Structure

```
kernel/src/
├── lib.rs                  — TransitionError enum
├── compat.rs               — std compatibility shim
├── math/
│   ├── fixed.rs            — Fixed(u128), SCALE = 10^12, checked arithmetic
│   └── sqrt.rs             — Integer square root (Babylonian, FIPS-stable)
├── physics/
│   ├── hashing.rs          — SHA-256: self-contained FIPS 180-4 implementation
│   ├── merkle.rs           — Perfect binary Merkle tree, domain-separated hashing
│   └── canonical_json.rs   — RFC 8785 canonical JSON, schema validation
├── state/
│   ├── epoch.rs            — EpochState struct + canonical commitment
│   ├── decay.rs            — Thermodynamic decay constant (e^–0.0577 per epoch)
│   └── entropy.rs          — Global entropy scalar computation
└── transition.rs           — apply_epoch_dry_run: deterministic epoch transition
```

## EpochState

The committed state at the end of each epoch. All fields are fixed-width (no generics, no heap):

```rust
pub struct EpochState {
    pub bond_pool_root:        [u8; 32],  // Merkle root of active VouchBond locks
    pub entropy_metric_scaled: u128,      // Global entropy (raw Fixed inner, SCALE=10^12)
    pub epoch_number:          u64,       // Monotonically increasing counter
    pub impact_pool_root:      [u8; 32],  // Merkle root of validated ProofOfImpact records
    pub kernel_hash:           [u8; 32],  // SHA-256 of the WASM binary that produced this state
    pub previous_root:         [u8; 32],  // state_root of the preceding epoch
    pub state_root:            [u8; 32],  // SHA-256(canonical JSON of all other fields)
    pub validator_set_root:    [u8; 32],  // Merkle root of the active validator set
    pub vdf_challenge_seed:    [u8; 32],  // VDF input seed for next epoch's sortition
}
```

`state_root = SHA256(canonicalize(all other fields))`. The `state_root` field is excluded from its own serialization.

## Stub Inventory (v0.0.1-alpha)

These functions have correct signatures and return explicit errors, but do not yet perform real verification:

| Stub | Status | Target |
|---|---|---|
| VDF SNARK proof verification | Returns Ok() in dry_run | v0.0.2 |
| Ed25519 signature verification | Not called | v0.0.2 |
| Per-identity Merkle decay | Roots pass through | v0.0.2 |
| Entropy recalculation | Passes through | v0.0.2 |

Each stub replacement requires a new pinned constitutional vector before merge.

## Upgrading the Kernel

The `kernel_hash` field in `EpochState` is the SHA-256 of the WASM binary that produced the state. When the kernel binary changes:

- The `kernel_hash` in the next epoch will differ from the previous epoch
- All nodes must upgrade together at the same epoch boundary
- Nodes running different kernel versions will compute different `state_root` values and immediately detect the fork

This is intentional. Kernel upgrades are explicit, not silent.

## Dependencies

**Zero external dependencies in production code.**

The Rust standard library is used for tests (`#[cfg(test)]`). The `no_std` constraint will be enforced via feature flag when compiling production WASM (v0.0.2+).

## Toolchain

Pinned to Rust `1.93.1` via `rust-toolchain.toml`. Do not disable the toolchain pin. The constitutional vectors are only meaningful on a pinned toolchain.

## License

TBD.
