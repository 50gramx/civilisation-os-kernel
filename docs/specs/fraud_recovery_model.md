# Fraud Resolution & Finality Physics

A core tension in any deterministically verifiable system is how to handle the discovery of a mathematically invalid state transition *after* it has been proposed and signed by a committee. 

We must define the precise physics of Fraud Recovery. 

## 1. The Philosophical Fork: Rewind vs Forward-Slash
When a `FraudProof` successfully evaluates to `true` inside the WASM kernel, the protocol must choose one of two paths:
*   **Path A (Absolute Rewind):** The chain reorgs. The invalid epoch is entirely erased. The state "rewinds" to the last known valid `previous_root` (the state before the fraud occurred). The malicious committee members are slashed in this re-built epoch, and all honest transactions from the aborted epoch must be re-submitted or re-played against the corrected state.
*   **Path B (Forward-Slash):** The invalid epoch is *finalized anyway* to preserve chain liveness and UX continuity. However, in the *next* epoch, the protocol mathematically subtracts (slashes) the balances of the actors who signed the mathematical error. 

## 2. The Decision: Absolute Rewind (Path A)

We explicitly choose **Absolute Rewind**. 

### Rationale: Semantic Integrity over Liveness
If we chose Forward-Slash (Path B), the protocol would be formally accepting an invalid mathematical transition into the immutable ledger just to avoid a chain reorg. This fundamentally violates the premise of a Thermodynamic truth-machine. You cannot build a "Constitutional Physics Engine" that knowingly persists bad math just because it's convenient for light clients.

If a committee fraudulently mints 1,000,000 Accountable Score to an adversarial key, and we "Forward-Slash" them later, that 1,000,000 Score existed in the state for an epoch. It could have been used to vote on federation bridges, or deceive external indexing systems. 

**Absolute Rewind** ensures that an invalid state *never canonically exists* past the **Provisional Finality Window**. 

## 3. Rewind Horizons & Snapshot Validity
To prevent an archival node from triggering a catastrophic replay storm by identifying a fraud that is years old, the Rewind Horizon is strictly bounded.

**The Fraud Window (Frozen):** Let `K = 1`. A `FraudProof` is ONLY valid if it references the immediately preceding epoch. If an epoch `X` completes, and no `FraudProof` is submitted during epoch `X+1`, epoch `X` is permanently sealed in the historical timeline. No rewind can ever go back further than 1 epoch layer. (This translates to the 7-day wall-clock duration of a single standard domain epoch).

**Snapshot Interaction Constraint:** Since check-pointed snapshots are the foundation of browser-native viability, a Deep Checkpoint `N` can **only** be minted for an epoch whose provisional finality window has expired. For example, if checkpoint cadence is `N=12`, the snapshot payload for epoch 12 is generated and signed during epoch 13.
- If a fraud is proven regarding epoch 12 *during* epoch 13, the in-progress snapshot generation is invalidated and dropped by honest nodes.
- Snapshots must explicitly include the **Kernel Hash** in their metadata. Rewinding across a kernel upgrade is forbidden if the kernel hash diverges.

## 4. The Execution Flow and VDF Reset
When a `FraudProof` is validated:
1. **WASM Execution:** The Kernel verifies the `FraudProof` against the `previous_root`. 
   * *Ordering Constraint:* If multiple fraud proofs exist, they are processed in ascending lexicographical order of their JCS-canonical hash to prevent host-ordering manipulation.
2. **Chain Re-org:** The P2P mesh orphans the fraudulent `EpochStateRoot`. 
3. **State Revert:** The local SQLite ledger rolls back its pointer to `X-1`.
4. **The VDF Reset:** The `vdf_challenge_seed` for the *new* epoch `X'` must be derived anew exclusively from the valid `X-1` state. The fraudulent VDF chain is severed and discarded to prevent seed manipulation.
5. **The Slashing Application:** The new epoch `X'` is initiated. The `slashed_this_epoch` set is populated with the malicious committee. Their balances are slashed in this *new* valid epoch branch. 
6. **Honest Continuity:** Honest payloads that were in the orphaned block are deterministically re-applied by the clean committee.
