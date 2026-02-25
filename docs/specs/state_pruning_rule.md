# Phase 6: Code Schemas

## 5. The State Pruning Protocol

Our architecture mandates **Full Validation** by every active identity. We explicitly banned Light Clients to ensure security. However, if a browser must store every JSON payload (every Proof, every Bond) from Epoch 0 to Epoch 10,000, the storage requirements will explode, inevitably forcing users onto centralized RPCs.

To prevent this, the protocol must enforce **Strict State Pruning**.

### 1. The Archival Window (e.g., 12 Epochs / ~1 Year)
Active nodes are only required to store the *complete, raw JSON payloads* (the `ProofOfImpact` and `VouchBond` objects with all signatures and pointers) for the trailing 12 epochs. 
Why 12? Because it matches the half-life of the Decay Constant. After 1 year, the mathematical influence of those specific objects has degraded by 50% and their results have been permanently baked into the newer `canoncial_state_roots`.

### 2. The Pruning Mechanism (Merkle Retention)
When Epoch 42 finalizes, the raw payload data from Epoch 30 is pruned from active browser local storage.

**However, the node does NOT delete history. It compresses it.**
The node deletes the raw JSON, but retains the **Merkle Inclusion Proofs** (the hash path from the leaf up to the `EpochStateRoot`) for those objects.

*   By retaining the 32-byte hashes, the browser can still mathematically prove that a specific event occurred in Epoch 30 without needing to store the 2KB JSON payload of that event.
*   This reduces storage growth from linear/exponential to logarithmic.

### 3. The Archival Duty (The Root Layer)
If a user wishes to query the raw historical text of a 5-year-old `ProofOfImpact` (e.g., to read the justification), where do they get it?

This is the secondary function of the **Root Layer**. Dedicated archival nodes at the Root Layer store the full historical payloads of domains. 

Crucially, **the browser does not trust the Archival Node**. 
If a browser asks the Archival Node for a 5-year-old Proof, the Archival Node must return the JSON *and* the Merkle Proof. The browser hashes the JSON and checks the Merkle Proof against its own locally stored, fully validated historical `EpochStateRoot`.

### Conclusion
By enforcing state pruning while retaining Merkle proofs, we achieve the holy grail of decentralized engineering:
1.  **Browser Sustainability:** A user can run a full node on a laptop indefinitely without running out of disk space.
2.  **Trustless Archival:** We can rely on massive data centers to store the heavy raw history without ever having to trust their data. The browser mathematically verifies everything they serve.
