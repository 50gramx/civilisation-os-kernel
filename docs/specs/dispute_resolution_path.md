# Phase 7: The Ultimate Resolution Fork

The final architectural decision defines what happens the exact millisecond a valid `FraudProof` is verified by the network. We know the fraudulent committee members are slashed to zero. But what happens to the State Root?

We have two options:
*   **Option A: Mid-epoch Slash.** Slash the guilty parties immediately, keep the epoch running, and mathematically patch the State Root on the fly.
*   **Option B: Full Epoch Rollback.** Slash the guilty parties, completely orphan the fraudulent State Root, rewind time to the `previous_valid_state_root`, and force the network to replay the entire epoch under a newly sorted committee.

**The Answer: Option B (Full Epoch Rollback & Replay).**

---

## 1. Why Option A (Mid-epoch Slash) Fails

Option A seems desirable because it is fast. It minimizes disruption. However, mathematically patching a State Root mid-epoch destroys **Determinism**.

If an epoch contains 10,000 VouchBonds and 5,000 Proofs of Impact, and the committee lied about the `balance` of a single user, or lied about the `entropy_metric`, we cannot simply "correct" that one number.
*   That balance affects the user's bonding power.
*   Their bonding power affects the yield distribution of dozens of other users.
*   The entropy metric dictates whether the domain survives or is purged.

A single lie in the State Root cascades through the entire mathematical state. "Patching" it is computationally equivalent to re-executing the entire epoch anyway, but introduces a massive attack vector where different nodes might patch the state at slightly different network latencies, resulting in cascading, unresolvable forks.

---

## 2. The Power of Option B (Full Epoch Rollback)

Option B is brutal, slow, and mathematically perfect.

### The Resolution Sequence:
1.  **The FraudProof Propagates:** A single honest node gossips the valid `FraudProof`.
2.  **The Slash:** Every node mathematically verifies the proof instantly. The `signers_bitmap` identities are slashed to zero.
3.  **The Orphan:** The `challenged_state_root` is permanently orphaned. It is erased from canonical history.
4.  **The Rollback:** Every node rewinds their local state to the `previous_valid_state_root`.
5.  **The Re-Sortition:** Because the VDF `challenge_seed` (from the previous valid root) remains identical, the protocol automatically computes the *next* valid deterministic pseudo-random seed to draw a completely new 67% Sealing Committee from the remaining un-slashed validator pool.
6.  **The Replay:** The mempool transactions (Bonds and Proofs) are re-processed deterministically under the new committee.

### The Civilisational Outcome
Option B prioritizes **Integrity over Speed**.

If a committee acts maliciously, the civilisation does not try to stumble forward with a patched engine. It completely halts, executes the traitors, resets the clock to the last known truth, and tries again. 

This creates immense social and economic pressure. Because a rollback delays the entire domain's yield minting and state finality by 30 days, validators are fiercely incentivized to verify perfectly the first time. The network will socially ostracize any node runner who signs a fraudulent root, because their negligence cost the entire domain a month of time.

This is the ultimate expression of the High-Trust, Low-Velocity philosophy. We do not patch lies. We erase them.
