# Civilisation OS: Core Deterministic Architecture (Final Constitution)

This document serves as the master blueprint for the mathematical physics engine of the Civilisation OS. It synthesizes the deterministic boundaries designed in Phases 8, 9, and 10, establishing the strict laws of execution that the WASM Kernel must obey.

## 1. The Physics Layer (Phase 8)
The foundation of the engine is absolute reproducibility. It eliminates cross-compiler and cross-host fork vectors.

*   **Canonical Serialization:** JCS (RFC 8785) is strictly enforced. Keys MUST be ASCII (`^[a-z0-9_]+$`). Duplicate keys and unknown fields trigger a hard reject (`TransitionError`). Numbers MUST be strings matching `^(0|[1-9][0-9]*)$`. Floats are explicitly banned.
*   **Cryptographic Primitives:** The kernel mandates `SHA-256 (FIPS 180-4)`. 
*   **The Merkle Engine:** State commitments are built using a **Perfect Binary Padded Tree**. Leaves and nodes are rigorously domain-separated (`0x00` and `0x01` prefixes). Leaves are ordered lexicographically by their canonical bytes. If `leaf_count == 0`, the root limits to `SHA256(0x00 || null)`. Trees pad to the next power of two by explicitly duplicating the final odd node per level.
*   **DOS Bounding:** To prevent execution exhaustion, epochs hard-cap at `10,000` payloads, and Merkle witnesses reject depths exceeding `40`.
*   **Chronological Execution Order:** 
    1. VDF Verification
    2. Validator Registration
    3. Thermodynamic Decay (Identity Iteration strictly lexicographical)
    4. Impact Processing
    5. Bond Processing
    6. Yield Processing (Stubbed initially)
    7. Entropy Computation
    8. Self-Committing State Root (Excluding the root field itself)

## 2. The Fixed-Point Math Engine (Phase 9 & 11)
Mathematics operate in a closed, explicitly defined scale.

*   **The `Fixed` Wrapper:** The engine utilizes standard `u128`, but the raw integers MUST be strictly encapsulated inside a `Fixed(u128)` struct. Direct access to the internal `.0` value is forbidden in the consensus layer.
*   **Scaling Factor:** $10^{12}$ (`SCALE`). Thus, `1,000,000,000,005` = 1.000_000_000_005 units.
*   **Chained Multiplication Ceiling:** To prevent bounds breaches during execution, the kernel forbids multiplying more than two scaled values without a `/ SCALE` reduction in between. The math ceiling is roughly 340 septillion units ($3.4 \times 10^{26}$). This constraint must be mechanically enforced by the `mul_scaled` method on the `Fixed` struct.
*   **Overflow Policy:** No environmental panics. All consensus math uses explicit `checked_*` methods inside the `Fixed` struct. If `None` is returned, a logical `TransitionError::MathOverflow` is explicitly propagated, instantly invalidating the transaction or epoch.
*   **Zero-Division Bounds:** Any attempt to divide by zero returns `TransitionError::MathOverflow`, never a WASM trap.
*   **Dust Truncation:** Decay execution explicitly leverages Integer Division Truncation (floor rounding). Fractional remainder dust is perpetually burned from the total scale. This explicitly documents an **anti-fragmentation bias**, slightly prioritizing consolidated capital over identities trying to split to farm rounding errors.

## 3. Fraud and Recovery Logistics
*   **Max Fraud Window:** `1 Epoch` (Approx 7 days).
*   **Recovery Philosophy:** **Absolute Rewind**. Fraud invalidates the forward state. The system rewinds to `X-1`, slashes the malicious committee in a strictly deterministic sequence, invalidates the corrupt VDF seed, and recalculates the new honest timeline from a pristine root.
*   **Slashing Limits:** Slashing subtraction is executing using `saturating_sub` solely to reach zero without panicking. Slashing is bounded to **Max One per Validator per Epoch**, ignoring duplicate hostile fraud proofs. It is purely subtractive; slashed dust is burned, never redistributed.

## 4. Economic Tokenomics (Phase 10)
Token issuance is the final layer, structured as economic policy layered *over* the immutable physics engine. 

*   **Bond-Based Minting:** Tokens are issued strictly when Truth is capitalized (locked in `VouchBonds`). 
*   **Minimum Threshold:** `1 * SCALE`. Dust bonding is instantly rejected to prevent payload bloat.
*   **Minting Curve Shape:** Formally **Sublinear**. The formula strictly limits compounding capital monopolies (Whales) via integer approximations of square root or log curves.
*   **Global Entropy Coupling:** The issuance curve is dynamically multiplied against the system's overall churn. Active validator networks with high identity rotation generate faster yield; stagnant, dormant networks constrict issuance, preventing structural hyperinflation.
*   **Validator Incentives:** Validators receive a fractional cut of successful Epoch bond mints. To prevent validator bankruptcy during dormant cycles, a `Minimum_Network_Entropy_Fee` is minted exclusively to the active validator subset if organic bonded yield drops below baseline.

### Execution Plan
The WASM Kernel will be constructed precisely in this phase order: Physics first, Fixed-Point Math second, and Tokenomics injected via Trait implementations once deterministic stability is mathematically proven.
