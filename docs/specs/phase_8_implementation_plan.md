# Phase 8: Codebase Implementation Plan (Foundation)

## Goal Description
Transition the Civilisation OS architecture from theoretical Markdown artifacts into production-grade Dart/Flutter code within the `eapp-live` repository (and potentially `eapp-flow-bob` for core logic). The first step is to build the strictly deterministic foundation: Identity, Data Models, and Local Storage.

## User Review Required
> [!IMPORTANT]
> Please review this technical plan. This is the bridge between theory and code. We need to ensure the directory structures and dependency choices align with your vision for `eapp-live` and the Ethos OS standard.

## Proposed Changes

We will build the deterministic state machine architecture in pure Rust, compiled to WASM. We will NOT implement concrete token emission economics until this skeleton can survive a 1,000-epoch simulated adversarial replay.

### Step 1: Canonical Serialization Specification (The Bytes)
Before we can hash payloads, we must define exactly how they are structured in memory.
- Define strict JSON serialization rules (JCS - RFC 8785).
- Enforce UTF-8 encoding, exact key ordering (lexicographical), and zero whitespace.

### Step 2: Deterministic Data Structures & Math (The Numbers)
- Implement SHA-256 caching and Merkle Tree construction.
- Define the `FixedPoint` numeric struct to prevent floating-point divergence.

### Step 3: The Epoch State Struct
- Define the core `EpochState` Rust struct:
  ```rust
  struct EpochState {
      epoch_number: u64,
      previous_root: Hash,
      validator_set_root: Hash,
      bond_pool_root: Hash,
      impact_pool_root: Hash,
      decay_accumulator: FixedPoint,
  }
  ```

### Step 4: The Transition Executor
- Implement the pure function: `new_state = f(previous_state, ordered_payloads)`.
- Enforce the Two-Phase Execution rule (Impacts first, then Bonds).

### Step 5: The Fraud Replay Engine
- Build the logic to reconstruct state, unroll Merkle proofs, and verify `FraudProof` assertions.

### Step 6: Core Slashing Logic
- Define the slashing trigger mechanics without hardcoding exact economic yields. Emphasize boolean triggers and identity array updates.

### Step 7: The Emission Trait Interface
- Define the economic interfaces (e.g., `on_bond_lock`, `on_epoch_finalize`) as Rust traits. 
- The concrete algorithms (Model C: Bond-Based Minting) will be injected *only* after chaos testing the base skeleton.

## Verification Plan
- **Chaos Testing:** The Rust kernel must successfully execute 1,000 simulated epochs natively. We will inject malformed payloads, force protocol time rollbacks, and verify that the exact same genesis state consistently produces a byte-for-byte identical final State Root hash across Windows, Linux, and macOS environments.
