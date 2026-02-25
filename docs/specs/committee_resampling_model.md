# Phase 2: The Sealing Committee Model (Cryptographic Sortition)

The user poses the final structural question of Phase 2: Will the 67% sealing committee be static for the entire epoch, or re-sampled pseudo-randomly for each seal event?

**The Decision: The committee is re-sampled pseudo-randomly at the exact moment of the epoch boundary. There is no static committee.**

---

## 1. The Danger of a Static Committee
If the top 67% Accountability-weighted actors are a known, static list for the entire 30-day epoch, the civilisation has accidentally created a standing aristocracy. 

The attack surface becomes enormous:
1.  **Distributed Denial of Service (DDoS):** An attacker knows exactly which IP addresses hold the threshold weight. They can DDoS the top validators right before the epoch boundary to stall the seal.
2.  **Collusion and Bribery:** A static committee has 30 days to secretly organize off-chain, negotiate bribes, and plan a malicious State Root.
3.  **Elite Ossification:** Even if honest, a static committee calcifies the social perception of power. The elite become minor celebrities, destroying the egalitarian nature of the mesh.

---

## 2. The Solution: VDF-Anchored Cryptographic Sortition

We use **Accountability-Weighted Sortition** (a cryptographic lottery) to draw the committee dynamically. Power is constantly reshuffled by the protocol.

### The Source of Randomness: The VDF Output
To perform a secure lottery, you need a source of randomness that no one can predict or manipulate. 
In Phase 2, we established the Verifiable Delay Function (VDF) as our Sovereign Clock. The output of a VDF is pseudo-random and strictly unpredictable until the exact millisecond the computation finishes. 

**The Sortition Mechanic:**
1.  **The Reveal:** A node finishes the VDF computation and gossips the Proof.
2.  **The Lottery:** Every node takes the cryptographic hash of the VDF Proof and uses it as the random seed. 
3.  **The Draw:** Using this completely deterministic but previously unpredictable seed, the nodes run a sorting algorithm across the entire domain's population. 
4.  **The Weighting:** The probability of a user being selected for the committee is strictly weighted by their *Accountability Score * Entropy Coefficient*. 

### Why This is Antifragile
At 11:59 PM, nobody in the civilisation knows who will be on the sealing committee. 
At 12:00 AM, the VDF completes, the seed is revealed, and the committee is instantly drawn from the population. 

*   **No Targeted Attacks:** You cannot DDoS or bribe the committee, because the committee does not exist until the very second they are required to sign the State Root.
*   **Democratic Rotation:** While high-score actors are mathematically more likely to be drawn, the pseudo-randomness ensures that low-score but high-entropy actors frequently cycle into the sealing threshold. 
*   **Constant Churn:** Even if a cartel controls a massive amount of weight, the randomness fragments their ability to predictably control the exact 67% threshold needed for a specific epoch.

We do not just rely on decay to slowly erode elites over decades. We use cryptographic sortition to shatter their coordination every 30 days. Power is never held; it is only briefly borrowed from the physics engine.
