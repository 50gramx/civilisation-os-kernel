# The Ultimate Axiom: Resilience > Liveness

The Civilisation OS is now fully theorized. In closing the final attack vectors, we have made a definitive, systemic choice: **We prioritize maximum resilience against corruption over maximum liveness under adversarial pressure.**

This is not a compromise. It is the defining feature of the architecture.

## 1. The Death of the Genesis Oracle
To prevent the Genesis Founders from becoming a sovereign political class:
*   The `GenesisManifest` is a cryptographically committed JSON object, sealed at the instant the domain spawns. The list of the 50 Genesis public keys is absolutely **immutable**. They cannot be voted out, and new ones cannot be voted in without a complete network hard-fork. They are structural fixtures, not politicians.
*   If the Domain triggers Radioactive Exclusion (a 3-epoch, **inflexible, hardcoded** suspension of the active validator pool), the Founders must deterministically replay the epoch. If they publish a fraudulent root, their locked Root Stake is destroyed, and the domain instantly dies. They have no upside in manipulation, only catastrophic downside.

## 2. The Deterministic Anchor
To prevent rollback contamination, mempool payloads (`ProofOfImpact`, `VouchBond`) are only valid if their internal `previous_state_root` perfectly matches the epoch they are attempting to enter. If a payload was gossiped late and anchored to Epoch 41, it cannot be accidentally swept into a replay of Epoch 42.

## 3. The UX Reality (The Slow Cathedral)
Because we chose Resilience over Liveness, the user experience (UX) and adoption curve must deliberately reflect this.

A system that can halt for 30 days to execute a rollback is not a system for day-trading, instant messaging, or viral trends. It is a system for scientific consensus, constitutional law, and multi-decade capital allocation.

**UX Expectations:**
1.  **Delayed Gratification:** When a user submits a `ProofOfImpact`, nothing happens immediately. They must wait for bonds to attach. When a `VouchBond` is locked, and the `EpochStateRoot` is signed, yields still do not unlock for 7 days (the FraudChallenge Window). The UI must communicate "Waiting for Epistemic Finality" rather than "Transaction Confirmed."
2.  **Heavy Duty:** Running this application means running a full browser-native node. The battery drain and bandwidth usage will be higher than a social media app. The UI must explicitly treat the user as a "Validator Citizen" whose machine is performing constitutional labor.
3.  **High Stakes:** The UI cannot hide the slashing mechanics behind abstraction. When a user clicks "Sign Vouch-Bond," the interface must treat it like signing a mortgage, highlighting the exact decay and slashing risks.

We are not building a consumer app. We are building the physics engine for a digital republic. The friction is the point.
