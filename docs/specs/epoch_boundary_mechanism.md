# Phase 2: The Sovereign Clock (Epoch Boundaries)

The transition to a Browser-Native Peer Mesh eliminates the global blockchain, which means we lose the global "Block Height." If Accountability is calculated per Epoch, but there is no central server and no blockchain to tell the mesh "Epoch `t` has ended," how do we prevent temporal drift and clock spoofing?

**We cannot rely on Wall-Clock Time (NTP).**
If the mesh relies on an external Network Time Protocol (NTP) server, we introduce a centralizing oracle. If we allow nodes to use their local system clocks, malicious actors will simply fast-forward their clocks to accelerate Yield or manipulate Decay triggers.

**The Decision: The Canonical Epoch Boundary is established via Verifiable Delay Functions (VDFs) and Threshold Seals.**

---

## 1. Cryptographic Time: The Verifiable Delay Function (VDF)

Time in the Civilisation OS is not measured in seconds. It is measured in cryptographic friction.

A **Verifiable Delay Function (VDF)** is a mathematical function that requires a specific number of *strictly sequential* steps to evaluate, but produces a unique proof that can be verified almost instantly by anyone.
*   **Sequential:** You cannot compute a VDF faster by throwing 10,000 parallel GPUs at it. It simulates the absolute passage of time because step 2 requires the output of step 1. 
*   **Verifiable:** Once a node computes the VDF, any browser in the mesh can cryptographically verify it in milliseconds.

### The Mechanics of the Epoch Tick
1.  **The Epoch Seed:** When Epoch 1 begins, the hash of the finalized Epoch 1 State serves as the "Seed."
2.  **The Time Computation:** Dedicated nodes within the domain mesh continuously run the VDF algorithm against the Seed. The algorithm is calibrated so that the required sequential iterations take approximately 30 days of average consumer hardware compute (this is the physical constant of civilisational time).
3.  **The Proof Gossip:** The first node to complete the VDF broadcasts the resulting Proof and the new Seed across the domain mesh.
4.  **The Mathematical Tick:** Browsers receive the VDF Proof, verify it instantly, and mathematically recognize: *"The sequential compute required for 30 days has been proven. The Epoch is over."*

---

## 2. The Finality Layer: Accountability-Weighted Threshold Signatures

A VDF proves time has passed, but we must also ensure the mesh agrees on the exact *State Snapshot* (the ledger of who bonded what) at that precise temporal milestone.

1.  **The Snapshot Freeze:** When the VDF Proof hits the mesh, all nodes freeze the intake of new Vouch-Bonds for the exiting epoch.
2.  **The Ledger Gossip:** Nodes aggressively gossip their local DHT ledgers to reconcile the final state.
3.  **The Threshold Seal:** The top nodes in the domain (ranked by Accountability Score and Graph Entropy) act as the finality layer. When they successfully reconcile the DHT state up to the VDF Proof timestamp, they digitally sign the State Root.
4.  **Consensus:** Once a supermajority (e.g., 67% of the total domain Accountability weight) has signed the State Root, the Epoch is cryptographically sealed. 

### Why This Works
The VDF acts as the un-fakeable metronome. The high-entropy nodes act as the jury verifying the ledger matches the metronome.

If a local user goes offline for 3 months, they don't lose sync with reality. When they reconnect, their browser simply downloads the last 3 VDF proofs and the 3 Threshold Signatures. Their local client mathematically proves that 3 epochs of time have passed and executes the Decay function exactly 3 times, perfectly synchronizing their local reality with the aggregate civilisation.

**We do not trust clocks. We trust physics and math.**
