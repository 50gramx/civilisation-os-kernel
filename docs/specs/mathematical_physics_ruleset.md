# Phase 5: The Mathematical Physics Ruleset

Before we can define JSON data structures, we must formalize the math. If we define schemas before locking the physics, we risk fundamental redesigns when variables don't fit the structure.

We are building a High-Trust, Low-Velocity civilisational substrate. The math must reflect friction, careful rotation, and absolute resistance to instant scaling.

---

## 1. The Global Ignition Threshold (T)
*How much Root Stake is required to spawn a new domain?*

The Threshold (`T`) is not static. It must scale with the size and density of the civilisation to prevent namespace fragmentation.

*   `T = B + (√D * M)`
*   `B` = Base minimum stake (e.g., 5,000 Root Score)
*   `D` = Total Active Domains currently in the system
*   `M` = Multiplier coefficient (e.g., 500 Score)

**Logic:** As the system grows, it becomes harder to spawn new partitions. We use **sublinear scaling (`√D`)** instead of linear. This prevents runaway barrier growth that would mathematically lock out all future domains. It preserves friction but avoids total intellectual stagnation.

---

## 2. The Domain Death Function (D)
*When is a domain mathematically purged for low entropy?*

Purging based purely on an aggregate aggregate score threshold is dangerous—it kills high-value, low-frequency niche domains (e.g., theoretical physics). We must purge based on *Validator Liveness*, not absolute score mass.

*   `Active Validators over trailing W Epochs < Minimum Quorum` **OR**
*   `Bonding Entropy over trailing W Epochs < ε`

Where:
*   `W` = Tracking Window (e.g., 3 Epochs / ~90 days)
*   `Minimum Quorum` = e.g., 10 unique nodes successfully signing State Roots.
*   `ε` = Minimum Entropy threshold (e.g., Gini coefficient of bonding distribution > 0.3, or >30% churn in active validators).

**Logic:** If a niche domain only mints two major Proofs of Impact per year, it survives—*provided* there are human watchdogs actively verifying it, AND they are not a stagnant cartel of 10 people circularly signing each other's roots. We require both Liveness and Entropy to prevent ideological islands.

---

## 3. The Decay Constant (λ)
*How fast does power dissolve?*

The formula is `Accountability(t+1) = Accountability(t) * e^(-λ)`. We measure decay in **Half-Life**, because standard percentages mask the compounding reality.

*   **The Half-Life Constraint:** Accountability Score should halve every **12 Epochs (approx. 1 Year)**.
*   **The Math:** `0.5 = e^(-12λ)` solving to `λ ≈ 0.0577` per epoch.

**Logic:** If you do absolutely nothing for one year, you lose 50% of your power. If you are inactive for two years, you retain 25%. This allows individuals to survive illness, parental leave, or sabbaticals without being zeroed out, but prevents retired "founders" from holding power across decades. It forces generational rotation.

---

## 4. The Genesis Stake Lockup (L)
*How long is Root cross-domain stake locked?*

If founders can ignite a domain and instantly withdraw their Root stake, they can spam ideologies.

*   **Lockup Duration (`L`):** Stake is locked for exactly **6 Epochs (~180 days)**.

**Logic:** 
1. The domain must survive 6 cycles of the VDF/Threshold Seal protocol. 
2. If the domain triggers the Domain Death Function (falls below liveness OR entropy thresholds) within those 180 days, the Genesis Stake is **burned unconditionally** at the Root Layer. 
3. If the domain survives past Epoch 6, the stake unlocks. This forces founders to not only spawn a domain but to actively nurture it through its most vulnerable bootstrapping phase. You cannot ignite a fire and walk away without losing your fingers.

---

By freezing these constants, we have moved from conceptual sociology into hard algorithms. The physics engine is mathematically bounded.
