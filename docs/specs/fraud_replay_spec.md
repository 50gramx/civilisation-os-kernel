# State Snapshot & Fraud Replay Architecture (Step 5)

The Civilisation OS must balance absolute mathematical determinism with the reality of browser-native execution constraints. A mobile browser cannot practically replay 10 years of thermodynamic decay from genesis just to join a domain.

## 1. The Sync Model: Checkpoint-Based Cryptographic Snapshots

We adopt **Option B: Checkpoint-Based Cryptographic Snapshot Sync** for standard Browser Nodes, while enforcing **Absolute Genesis Replay** for Root/Institutional Nodes.

### The Mechanism
1.  **Archival Epochs:** Every `N` epochs (e.g., `N=12`, aligning with the archival pruning window), the network produces a **Deep Checkpoint**. 
2.  **Snapshot Payload:** This checkpoint contains the fully materialized State Map (all active identity balances, locks, and Merkle tree leaves) compressed and cryptographically signed (`BLS` threshold) by the active committee.
    *   *Constraint:* The checkpoint payload MUST include the explicit WASM `kernel_hash`. Nodes syncing from snapshots will abort if the snapshot's kernel hash diverges from their local execution engine, preventing unverified semantic overrides.
    *   *Constraint:* Checkpoints can ONLY be minted for an epoch AFTER its `MAX_FRAUD_WINDOW_EPOCHS` has expired. If `N=12` and the window is `1`, the checkpoint for Epoch 12 is calculated and signed upon the completion of Epoch 13.
3.  **Browser Syncing:** When a new `eapp-live` browser joins the domain, it downloads the latest Deep Checkpoint payload via the P2P mesh, verifies the BLS committee signature spanning the `state_root`, and loads the materialized state into its local SQLite database. It then only replays the minimal delta of epochs (max `N-1`) to reach the current tip.

### Rationale
This dramatic reduction in sync time is the only path to mass adoption and mobile viability. It does not compromise security because the protocol is designed around **Provisional Finality** (7-day window). By the time a Deep Checkpoint is minted at `N=12` epochs (approx 1 year protocol time), the state has long passed the fraud challenge window and is epistemically irreversible.

*   *Note:* Any individual or institution *can* choose to run an Archival Node and replay from Genesis using the identical WASM kernel. The physics remain unbroken. Snapshots are merely a UX acceleration layer.

## 2. The Fraud Replay Module

Fraud Proof verification relies on the fact that the actual execution math is heavily bounded. A `FraudProof` asserts that the `apply_epoch` transition from `Epoch(x-1)` to `Epoch(x)` violated the rules.

### Single-Step Verification Pipeline & Replay Limits:
1.  **Receive Proof:** The P2P gossip layer receives a `FraudProof` struct. The core engine applies a hard boundary: `MAX_FRAUD_WINDOW_EPOCHS = 1`. A fraud proof is *only* valid if it references the immediately preceding epoch (`X-1`). If epoch `X` completes without a valid challenge against `X-1`, epoch `X-1` is permanently sealed.
2.  **Ordering Execution:** If multiple fraud proofs are received for the same epoch, they are processed strictly in ascending lexicographical order of their JCS-canonical hash.
3.  **Verification Invocation:** The Flutter Host extracts the specific `merkle_state_witnesses` (the branches relating only to the slashed identities and the specific malformed payload) and calls the WASM kernel:
    `verify_fraud_proof(fraud_proof_payload, previous_state_root, witnesses)`
4.  **Kernel Execution:** 
    *   WASM engine reconstructs the minimal involved state from the Merkle branches.
    *   Replays the specific *chronological step* (e.g., computes the decay for the targeted identity).
    *   Compares the re-computed mathematically pure result against the `expected_balance` or `expected_root` declared in the `FraudProof`.
5.  **Boolean Output:** Returns `true` if the committee committed fraud. Returns `false` if the challenger submitted an invalid proof (the challenger is then slashed).

This design ensures that verifying fraud is computationally cheaper than executing an entire epoch, turning the WASM sandbox into an ultra-fast, single-purpose constitutional court.
