# Phase 1: The Minimal Immutable Core

We pour the concrete.

## 1. The Pure Mathematical Decay Function

This is the spine of the Civilisation OS. It executes exactly once per Epoch (e.g., every 30 days) across the domain mesh. Every other mechanic exists to feed or survive this single equation.

**The State Transition Function:**
`Accountability(t+1) = max(0, [Accountability(t) - Slashing(t)] * e^(-λ) + Yield(t))`

### The Variables:
*   **`Accountability(t)`**: The user's total active locked and unlocked score at the start of Epoch `t`.
*   **`Slashing(t)`**: Objective score burned during Epoch `t` due to verified protocol violations of Vouch-Bonded entities. This resolves *before* decay, instantly punishing bad conviction.
*   **`λ` (Lambda - The Decay Constant)**: The immutable thermodynamic constant (e.g., `0.05` for ~5% decay per epoch). This is hardcoded into the genesis block and cannot be altered by the Federation.
*   **`e^(-λ)`**: The discrete exponential decay multiplier for the epoch.
*   **`Yield(t)`**: New score minted or reallocated to the user during Epoch `t`. Resolves *after* decay to ensure the new energy enters the next epoch intact.
    *   `Yield(t) = ProofOfImpactYield(t) + (PredictiveYield(t) * EntropyInversionMultiplier(t))`

This deterministic function guarantees that without continuous high-entropy validation (`Yield > 0`), all actors mathematically approach zero. Time destroys static power.

---

## 2. The Identity Concrete: Pseudonymous vs. Proof-of-Personhood

**The Decision: Identity MUST be strictly Pseudonymous (Local Cryptographic Keypair).**

If the OS requires Proof-of-Personhood (e.g., a biometric oracle, national ID databases, or a Worldcoin orb), the system is instantly compromised at the civilisational level.

1.  **Violation of Sovereignty:** You create a hard dependency on an external, centralizing physical verifier. If biological verification is required, the Root is no longer the cryptography; the Root is the corporate or state biometric database.
2.  **Censorship Vector:** External Oracles can blanket-ban or de-platform populations. You lose censorship resistance.

### How We Survive Without Biological Proof: The Mathematical Purge

The primary argument against pseudonymous keys is the **Sybil Swarm** (attackers spinning up 10,000 keys to vote for themselves). 

However, because the Civilisation OS runs on the immutable physics of **Graph Distance Validation** and **Decay**, Sybils are neutralized at the protocol level without ever needing to scan a retina:

1.  **Zero Graph Distance:** 10,000 Sybil drones created by one actor have a graph distance of `0` from each other. They interact entirely within a cloned cluster.
2.  **Zero Entropy Inversion:** Because they exist in a closed cluster, their `EntropyMultiplier` is strictly `0.0x`.
3.  **Zero Yield Generation:** Because their multiplier is zero, they generate `0` Predictive Yield when they inevitably Vouch-Bond each other's fake actions.
4.  **The Purge:** Because `Yield(t) = 0`, the immutable Decay Function `e^(-λ)` engages. Over a few epochs, the mathematical engine grinds the entire automated network to dust. 

We do not need an external oracle to prove a node has a biological pulse. 
We only need to mathematically prove the node has sustained, high-entropy validation from distant strangers over time.

**Pseudonymity + Thermodynamics yields un-fakeable civilizational weight.** 
The foundation remains purely native, sovereign, and mathematical.
