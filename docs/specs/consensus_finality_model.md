# Phase 2: The Consensus Finality Model

The user has correctly identified that by requiring a 67% Accountability-weighted threshold signature to seal the State Root, we have built a **Reputation-Weighted BFT Mesh**. 

The ultimate question of this consensus architecture: What prevents the top 67% of Accountability actors from colluding in a single epoch to sign a historically fraudulent State Root (e.g., giving themselves infinite yield, or deleting a rival's score)?

**The Decision: We do not rely on Optimistic Civilisational Trust. We rely on Deterministic Cryptographic Enforcement via Fraud Proofs.**

---

## 1. Why Optimistic Trust Fails at Scale

If we rely solely on "long-term decay" to deter a cartel, we are assuming the cartel cares about their long-term score. But an attacker doesn't need to hold power for 10 years if they can achieve their goal in 1 day. 

If a 67% elite cartel signs a fraudulent state root that deletes all their political enemies, and the only defense is "well, their score will gradually decay over the next 14 months," the system has fundamentally failed. A 51% attack cannot be solved via gradual thermodynamics; it must be solved via instantaneous execution.

---

## 2. The Defense: Deterministic Fraud Proofs

Because the `Accountability(t+1)` state transition function is a pure, deterministic mathematical formula, the peer mesh can enforce reality without needing to vote on it.

** The Mechanics:**
1.  **The Optimistic Seal:** The 67% Threshold Committee signs the new State Root for Epoch `t` and gossips it to the mesh. The network optimistically accepts it, saving immense local validation compute.
2.  **The Challenge Window:** There is a short window (e.g., 24 hours) where the State Root is pending finalization. 
3.  **The Watchdogs:** Any single node—even a new user with 0 Accountability Score running a browser on a phone—can independently run the deterministic state transition locally. 
4.  **The Fraud Proof:** If the mathematical outcome of the local browser *does not match* the State Root signed by the 67% cartel, that single node constructs a Cryptographic Fraud Proof. This proof contains the exact Vouch-Bond data and the deterministic math step that fails.
5.  **The Execution:** The node gossips the Fraud Proof. Because verifying the proof is mathematically trivial, every other node in the network instantly verifies it.

---

## 3. The Ultimate Slashing (The Nuclear Deterrent)

Once a Fraud Proof is mathematically verified by the mesh, it triggers the absolute highest penalty in the Civilisation OS.

**The signature of a fraudulent state root is not a minor protocol violation. It is High Treason.**

1.  The invalid State Root is immediately rejected and erased.
2.  Every single node that applied their identity key signature to that threshold seal—all 67% of the highest-weight actors in the domain—has their **Accountability Score slashed instantly to 0.**
3.  Their predictive yields are wiped. Their unbonding curves vanish. They are mathematically ground to dust in a single millisecond.

### The Civilisational Insight

We do not need 51% honest actors. **We only need ONE honest actor.** 
As long as a single node in the entire network is willing to run the math and generate the Fraud Proof, the entire accumulated power of the corrupt elite is annihilated.

This is the ultimate bridge between Accountability and Physics. Power can be aggregated via the Threshold Seal, but the moment that power lies about the physics, the physics executes them.
