# Phase 2: The Substrate (Infrastructure Reality)

We must choose where sovereignty sits. The user presents the standard Web3 menu: A Sovereign Layer 1, a Modular L2 (Cosmos SDK/Rollup), or a Smart Contract on Ethereum.

**The Decision: We reject the standard menu entirely.** 
The Civilisation OS must be a **Browser-Native Peer Mesh (A Federated Deterministic Runtime)**.

---

## 1. Why the Standard Menu Fails the Constitution

Our Minimal Immutable Core established that Accountability Score is non-transferable and based purely on epistemic weight, decay, and entropy. 

If we choose the standard blockchain options, the physics break immediately:

### A. The "Smart Contract on Ethereum/Solana" Trap
*   **The Flaw:** To execute the Epoch snapshot and decay function, users (or a keeper bot) must pay Gas in ETH/SOL. 
*   **The Collapse:** The cost of participating in the Civilisation is now denominated in a transferable fiat-equivalent token. Wealthy actors can afford to validate; poor actors cannot. Sovereignty sits with the Ethereum validators, not our citizens. We become tenants on a financialized landlord's property.

### B. The Modular L2 Trap (Rollups)
*   **The Flaw:** L2s require a sequencer and rely on the L1 for forced inclusion and settlement. This inherits the exact same financial friction and censorship risks as the L1, just at a delayed interval.

### C. The Standard Sovereign L1 Trap (Cosmos/Substrate Token PoS)
*   **The Flaw:** Building our own L1 gives us sovereignty, but standard L1s require **Proof of Stake (PoS)** for consensus security. PoS requires a *transferable financial token* to be staked and slashed. But we decreed that Accountability Score is *non-transferable*. You cannot run standard Tendermint PoS without a liquid token economy. If we introduce a liquid token just to secure the network, we have accidentally recreated the exact financial aristocracy we sought to destroy.

---

## 2. The Solution: The Browser-Native Peer Mesh

The Civilisation OS (`eapp-live`) is already building a "Hybrid Meta-OS Browser." The infrastructure substrate must live directly inside the browser instances of the users. 

Sovereignty does not sit on a blockchain in an Amazon data center. Sovereignty sits on the local hard drives of the citizens.

### The Architecture: Federated Deterministic Runtime (like Holochain)
Instead of a global, globally-ordered blockchain where every node must validate every single transaction of every domain (which is mathematically devastating for scale and requires gas to prevent spam), we use a purely relational DHT (Distributed Hash Table) Peer Mesh.

1.  **Domain-Scoped Consensus:** `climate.science` does not need to know about the state of `niche-hobby.local`. When a user operates in a domain, their browser joins the P2P mesh *for that specific domain*. 
2.  **Agent-Centric Ledgers:** Every user maintains their own local immutable ledger of their Vouch-Bonds and Yields (signed by their local secure enclave key).
3.  **Gossip Validation:** When a user creates a Vouch-Bond, they gossip that signature to their domain mesh. The nodes in that mesh independently run the exact mathematical Validation Function (`Accountability(t+1)` rules) to verify it structurally before holding it in their DHT. 
4.  **Proof-of-Accountability:** Sybil attacks on the network layer (spamming bad data) are prevented because you can only write to the domain's DHT if you possess non-zero Accountability Score. Your write-access is throttled by your thermodynamic weight.

### The Civilisational Advantage
*   **Zero Gas Fees:** Because there is no global bottleneck (mining/staking), there are no transaction fees. The only cost to participate is your local device's compute and network bandwidth.
*   **Infinite Scalability:** Domains scale infinitely because each domain is a sealed thermodynamic partition. Adding 1,000 domains does not slow down the root network. 
*   **Fork Friction:** If a cartel wants to fork the network, they cannot just fork a smart contract. They must convince millions of users to physically download a new browser binary. The fork friction is at the sociological/distribution layer, matching our resilience thesis.

We do not build a blockchain. We build a verifiable browser.
