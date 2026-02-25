# Phase 7: Rollback Logistics and Provisional Finality

Option B (Full Epoch Rollback) is geometrically pure, but without mechanical stop-gaps, a malicious minority could force the network into an infinite replay loop. 

To ensure the civilisation progresses, we must constrain the rollback mechanics with three explicit rules.

---

## 1. Canonical Transaction Ordering (Replay Invariance)
When a rollback occurs and the mempool is replayed by the new committee, the order of execution must be perfectly deterministic. Mempool execution order is **NEVER** based on network arrival time or local timestamp.

**The Rules:** 
1.  **Deduplication:** Aggregated payloads must be deduplicated by their SHA-256 IDs. In the event of exact ID collisions across different network paths, only one survives (retention rule is moot because identical IDs mean byte-for-byte identical payloads).
2.  **Lexicographic Sorting:** All valid payloads are sorted Lexicographically by their SHA-256 ID strings (from `0000...` to `ffff...`).
3.  **Two-Phase Execution:** To ensure dependency invariance, the execution engine strictly processes all `ProofOfImpact` objects first (building the Impact Merkle Root), and *only then* processes the `VouchBond` objects (building the Bond Merkle Root). Interleaving is mechanically forbidden.

This ensures that regardless of how many times an epoch is replayed, or which 67% committee is running it, the math will perfectly align every single time.

---

## 2. The FraudChallenge Window (Provisional Finality)
When does an `EpochStateRoot` become canon?

If it becomes canon immediately upon receiving 67% BLS signatures, then a FraudProof discovered 2 days later causes violent, retroactive whiplash to the applications built on top of the domain.

**The Rule: The 7-Day Provisional Window.**
*   When a committee publishes a signed `EpochStateRoot`, it enters **Provisional Finality**.
*   **Protocol Time Pauses:** During this 7-day window, yield minting, decay, and validator tracking halts on the ledger. Wall-clock time continues, but the physics engine freezes. The ledger is locked in "pending finalization."
*   If no valid `FraudProof` is gossiped within 168 hours, the `EpochStateRoot` achieves **Absolute Finality**, the state seals, and the epoch clock unlocks for the next cycle.

This limits rollback chaos to a predictable, bounded temporal window.

---

## 3. The Anti-Recursion Escalation Mechanism
What happens if the second, re-sorted committee *also* signs a fraudulent root? A 34% malicious minority could theoretically trigger endless rollbacks, stalling the 30-day epoch forever.

**The Rule: Radioactive Auto-Exclusion.**
*   **First Strike:** The fraudulent committee signers are slashed to zero. The epoch is replayed.
*   **Second Strike (Same Epoch Index):** If the *new* committee also produces a fraudulent root for the same epoch, their signers are slashed, AND the protocol triggers **Radioactive Exclusion**.
*   **Radioactive Exclusion:** The entire active validator pool for that epoch is suspended for exactly **3 Epochs**. The domain falls back to the **Root Layer Genesis Manifest** identities to serve as an emergency fallback committee to calculate the epoch.

### Bounding the Genesis Fallback (No Sovereignty)
This introduces a severe risk: treating the Founders as an Oracle. To prevent this, the Genesis fallback is **mechanically bounded**:
1.  **No Interpretive Authority:** The Genesis cohort is forced to use the exact same deterministic replay rules as the normal committee.
2.  **Slashable Guardians:** If the Genesis cohort publishes a mathematically incorrect `EpochStateRoot`, any node can submit a `FraudProof` against them. If proven fraudulent, the Genesis cohort is **slashed at the Root Layer**, instantly incinerating the locked collateral that keeps the domain alive, resulting in the immediate algorithmic death of the entire domain.
3.  **Scoped Duration:** The Genesis override is only valid for that single blocked Epoch Index. Normal sortition resumes immediately after.

They are not Kings. They are heavily armed janitors, bonded by their initial stake, called in only to unclog a paralyzed physics engine under threat of systemic suicide.
