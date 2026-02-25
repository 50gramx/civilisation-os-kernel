# Vouch-Bond Thermodynamics: The Physics of Withdrawal and Yield

To make the Vouch-Bond a civilisation-grade primitive, we must solve its thermodynamics: how does "Accountability Score" energy flow, lock, regenerate, and penalize over time?

The most critical question: **Is a Vouch-Bond permanent, or can it be withdrawn?**

---

## 1. The Decision: Withdrawability

**A Vouch-Bond MUST be withdrawable, but subject to an "Unbonding Curve."**

If bonds are permanent, the system freezes. Users will be terrified to lock their finite score forever, leading to extreme risk aversion and systemic stagnation. Vouching must represent *current conviction*, not a permanent tattoo. 

However, if withdrawal is instantaneous and costless, the Vouch-Bond loses its weight. It becomes a "Like." Attackers could front-run slashing events by withdrawing a second before a spam purge.

### The Unbonding Curve Mechanic
When a user clicks "Withdraw Backing":
1.  **The Cooldown (Time Lock):** The locked score does not return immediately. It enters a cryptographically enforced cooldown period (e.g., 7 days). 
2.  **The Penalty Risk:** During these 7 days, the user's score remains entirely exposed to slashing if the entity is purged. You cannot escape a sinking ship instantaneously; you are held accountable for your past endorsement while the exit processes.
3.  **The Yield Reset:** Withdrawing a bond resets the accumulated "predictive yield" (discussed below) tied to that specific bond to zero. 

---

## 2. Refined Mechanics: Surviving Adversarial Scale

To address the 5 advanced failure modes identified:

### A. Bond Concentration Limits (Preventing Venture Capitalism)
*   **The Rule:** A user cannot lock more than **5% of their total current Accountability Score** into a single entity, and cannot lock more than **50% of their total score aggregate** across all entities at any given time.
*   **The Effect:** This forces high-weight users to diversify their backing. They cannot become a kingmaker for a single creator. They must act as a distributed sensing network for the domain.

### B. Predictive Yield (Accuracy-Weighted Regeneration)
*   **The Rule:** If you Vouch-Bond an entity *early* (when it has few bonds), and it later receives a wave of Vouch-Bonds from highly distant nodes in the graph, your locked score generates a **Yield**.
*   **The Effect:** Your bond capacity regenerates *faster* than the baseline decay. You are mathematically rewarded (with more influence capacity) for correctly predicting quality before the consensus. This turns the system from a spam filter into a **Decentralized Discovery Engine.**

### C. Performance Penalty (The False Negative Solution)
*   **The Rule:** If an entity is not purged (no slash), but suffers a massive *withdrawal* of Vouch-Bonds over a short period (a consensus collapse), the remaining early backers suffer a **Soft Penalty** (e.g., a 2% deduction to their total score, or a temporary halt to their regeneration rate). 
*   **The Effect:** You are punished for sustained poor judgment, even if the content wasn't outright malicious.

### D. Anti-Correlation Checks (Breaking Ring Patience)
*   **The Rule:** The protocol runs local entropy checks on bonding patterns. If User A and User B have a 95% overlap in what they Vouch-Bond over 6 months, the system assumes they are a coordinated Sybil ring or a captured clique. 
*   **The Effect:** Their mutual Vouch-Bonds (when applied to the same entity) suffer an **Entropy Discount** (e.g., their combined weight counts for only 1.2x instead of 2.0x). Collusion becomes mathematically inefficient.

---

## 3. Conviction Tiers: Adding Expressive Depth

Yes, Vouch-Bonds should be tiered to allow for expressive conviction without over-complexifying the UX.

*   **Tier 1: Nod (Light Backing)** - Locks 1% of score. Minimum risk, minimum predictive yield. Used for general curation.
*   **Tier 2: Back (Strong Conviction)** - Locks 5% of score. Maximum risk, maximum predictive yield. Used to signal deep trust or flag crucial governance proposals.

---

## 4. The Psychological UX (Framing the Physics)

If we show users complex math and tell them they are "losing 5 points," they will churn. The UX must frame this as **Staking an Identity**, not spending a currency.

**The Interface Design:**
*   Instead of "Costs 5 Points," the UI says: **"Extend Reliability."**
*   The user's profile shows a "Reliability Aura" (a visual representation of their score). 
*   When they back a creator, a visual thread connects their Aura to the creator's post. The tooltip says: *"Your Reliability is extending to support this. If the community agrees, your Aura grows faster."*

By framing the action as extending a piece of yourself that regenerates when proven right, you tap into human psychology: the desire to be recognized as a tastemaker and a trustworthy citizen, without the cognitive friction of calculating "points."
