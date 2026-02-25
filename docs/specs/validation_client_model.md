# Phase 2: The Client Validation Model

The user has posed the final architectural fork of Phase 2: Will fraud-proof verification be required on every client by default, or optional (light clients vs full validators)?

If we allow "light clients," we break our own security model. 

**The Decision: There are no Light Clients. Every browser instance is a Full Verifier by default for its active domains.**

---

## 1. The Danger of the Light Client Illusion

In traditional blockchains (Ethereum, Solana), running a full node requires terabytes of storage, massive RAM, and high continuous bandwidth. Because it is so expensive, 99.9% of users run "Light Clients" (like MetaMask) which connect to a centralized RPC provider (like Infura or Alchemy) to read the state.

If we allow this in the Civilisation OS:
1.  **The "One Honest Node" Guarantee Dies:** If the vast majority of citizens do not run the state transition locally, they cannot detect a fraudulent State Root.
2.  **The Centralization Vector Returns:** Users will rely on third-party infrastructure companies to tell them the truth about the domain's state. If a cartel captures the RPC providers, they can sign a fraudulent State Root and the Light Clients will blindly accept it because they cannot see the underlying math.
3.  **Data Availability Fails:** If the cartel withholds the Vouch-Bond history from the RPC nodes, fraud proofs become impossible to construct.

A civilisation where only the affluent have the capacity to verify the law is an aristocracy. Verification must be inherently democratic.

---

## 2. Why Full Validation is Possible in the Mesh

How do we force a mobile browser to be a "Full Validator" without melting the phone's battery?

We can do this because **we abandoned the global blockchain.**

1.  **Domain Sharding by Default:** You are not validating every transaction in the entire civilisation. You are only validating the ledger of the specific domains (sub-graphs) you are actively participating in. If you join `biotech.local`, your browser only downloads and verifies the DHT state for `biotech.local`. 
2.  **Stateless Catch-up:** The `Accountability(t+1)` decay function is extremely lightweight math. It is just applying `e^(-λ)` to a matrix of historical scores. A mobile phone can calculate the entire decay state of a 100,000-user domain in milliseconds. 
3.  **Data Pruning:** Because Accountability Score is not a currency, we do not need to keep the entire historical ledger of every Vouch-Bond from Epoch 1. We only need the State Root of the previous epoch, and the net-new Vouch-Bonds minted in the *current* epoch, to calculate the next state. Old data can be safely dropped from local cache or moved to deep cold storage backups without breaking deterministic validation.

---

## 3. The Mechanics of the Distributed Immune System

By making Full Validation mandatory and invisible to the user:
1.  **Guaranteed Watchdogs:** If a domain has 10,000 users, there are physically 10,000 watchdogs independently running the state transition math. 
2.  **Total Data Availability:** The Domain DHT is redundantly stored across all 10,000 clients. A cartel cannot withhold data because the data is gossiped to everyone at the moment of creation. If data is missing from the DHT, it mathematically doesn't exist and cannot be included in the Threshold State Root.
3.  **Instant Execution:** If the 67% cartel signs a fraudulent root, 10,000 independent phones and laptops will simultaneously generate fraud proofs and slash the cartel within seconds.

Sovereignty is not a feature you opt into. It is the operating environment itself. In the Civilisation OS, if you want to read the code, your machine must enforce it.
