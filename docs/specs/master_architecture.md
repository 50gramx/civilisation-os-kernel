# Civilisation OS: The Master Architecture

This document consolidates the complete theoretical and structural blueprint of the Civilisation OS, generated across four phases of adversarial stress-testing.

---

## Part 1: The Minimal Immutable Core (Physics & Economics)
*Reference Artifacts: `phase_1_immutable_core.md`, `atomic_action_definition.md`, `vouch_bond_thermodynamics.md`, `yield_economics.md`, `transferability_mechanics.md`, `meaning_of_accountability.md`*

**The Goal:** Optimize for Epistemic Utility (Predictive Accuracy of Reality).
**The Metric:** Accountability Score (a measure of a user's historical capacity to correctly predict what the civilisation will eventually find valuable).

1.  **The Atomic Action (Vouch-Bond):** Users lock a percentage of their Accountability Score to endorse a verified, external Proof of Impact.
2.  **The Dual Engine:** Yield is only minted when a forward-looking prediction (Vouch-Bond) correctly anchors to a backward-looking demonstrated contribution (Proof of Impact). You cannot bond to a bond.
3.  **Strict Non-Transferability:** Accountability Score cannot be transferred, sold, delegated, lent, or pooled. Power is permanently embodied.
4.  **The Thermodynamic Decay:** Score strictly decays over time `e^(-λ)`. Inactive power dissolves.
5.  **Asymptotic Equilibrium Yield:** The total supply of Accountability Score naturally stabilizes based on the civilisation's real-world output, preventing hyperinflation or deflation spirals.

---

## Part 2: The Substrate (Infrastructure & Consensus)
*Reference Artifacts: `phase_2_substrate.md`, `epoch_boundary_mechanism.md`, `consensus_finality_model.md`, `validation_client_model.md`, `committee_resampling_model.md`*

**The Decision:** We reject L1 blockchains and L2 rollups built on financial stake/gas. We use a **Browser-Native Peer Mesh (Federated Deterministic Runtime)** to ensure sovereignty remains physically on the citizen's local device.

1.  **Domain Sharding:** Each domain is its own isolated Distributed Hash Table (DHT) partition.
2.  **The Sovereign Clock (VDFs):** Without a global blockchain, time is measured via Verifiable Delay Functions. A 30-day sequential math problem acts as the civilisational metronome to define the Canonical Epoch Boundary.
3.  **Mandatory Full Validation (No Light Clients):** Every browser instance fully validates its subscribed domains. "Light clients" relying on centralized RPCs are forbidden.
4.  **Cryptographic Sortition:** The 67% Sealing Committee is drawn pseudo-randomly at the end of the epoch using the VDF output as the seed. There are no static elites.
5.  **Deterministic Fraud Proofs:** If the Sealing Committee signs a fraudulent state root, any single node can mathematically prove the fraud. Submitting a verified fraud proof instantly slashes the entire cartel's Accountability Score to absolute zero (Nuclear Finality).

---

## Part 3: The Boundaries of Freedom (Sociology)
*Reference Artifacts: `freedom_vs_resilience.md`, `identity_singularity.md`, `delegation_and_embodiment.md`*

**The Challenge:** A perfectly resilient system enforced by nuclear slashing creates paralyzing risk aversion.
**The Solution:** Infinite freedom, provided it is unmeasured.

1.  **The Citadel vs. The Agora:** The system separates Measured Power (The Citadel: Vouch-Bonds, sealed State Roots, slashing) from Unmeasured Freedom (The Agora: ephemeral P2P WebRTC gossip, likes, chatting). The physics engine ignores the Agora entirely.
2.  **Modular Selfhood (Contextual Identity):** Identity is not singular across the civilisation. A user generates a unique cryptographic key (starting at 0 score) for every new domain they join. Power cannot be transferred across disciplines. There are no global "God-Kings."
3.  **The Ambient Base Yield (UBI):** A tiny fraction of epoch yield is distributed as an ambient thermodynamic floor to any active validating node. This prevents demographic starvation while remaining mathematically invisible to high-score elites due to decay.

---

## Part 4: Implementation Reality (Genesis & Bootstrapping)
*Reference Artifacts: `domain_genesis_protocol.md`, `root_layer_mechanical_ignition.md`*

**The Challenge:** If identity is modular (everyone starts at 0), how does a new domain generate its first yield?
**The Solution:** Stake-Weighted Mechanical Ignition.

1.  **Mechanized Root Layer:** The Root Layer is not a permanent political Senate. It is an automated namespace registry and stake-locking protocol.
2.  **The Genesis Cohort:** A minimum of 50 Root-Layer identities must sign a Genesis Manifest and lock a portion of their Root Score.
3.  **The Ignition Threshold:** When the aggregate locked stake breaches a dynamic algorithmic threshold (which adjusts based on global domain density), the domain automatically spawns. No voting is permitted.
4.  **The Thermodynamic Purge:** If a domain fails to generate ongoing impact and its aggregate Accountability Score decays near zero, the DHT is mathematically purged to reclaim attention bandwidth.

---

### Conclusion of Architectural Design
The framework is philosophically coherent, game-theoretically stable against capture/cartels, and politically survivable due to its absolute rejection of centralized choke points and unearned authority. It optimizes for Epistemic Density rather than liquidity or volume.

We are now prepared to move to formal Code Schemas (JSON/YAML structures for Vouch-Bonds and Impacts) in Phase 4.
