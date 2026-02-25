# Phase 6: Code Schemas

## 2. The `VouchBond` Object

The `VouchBond` is the forward-looking prediction vector. It is the mechanism by which Epistemic Utility (Accountability Score) is deployed to identify value.

### The JSON Schema

```json
{
  "protocol_version": "1.0",
  "domain_id": "sha256(domain_name)",
  "epoch_index": 42,

  // Deterministic Identifier computed over the serialized payload
  "bond_id": "sha256(canonical_json(protocol_version, domain_id, epoch_index, domain_nonce, bond_payload, timestamp_utc))",

  "forecaster_identity": {
    "public_key_ed25519": "d5e6f7a8b9c0...1234",
    "domain_nonce": 11234, // Monotonically increasing per domain per identity
    "signature": "hex_encoded_signature_of_bond_id"
  },

  "bond_payload": {
    "target_impact_id": "sha256(ProofOfImpact_impact_id)", 
    "staked_weight": 1500000000, // Represented in fixed-point integer limits. Undergoes standard Decay while locked.
    
    // Entropy and Conviction Metrics
    "lock_duration_epochs": 1, // Resolves at the END of the FOLLOWING epoch (Epoch 43). Cannot unlock mid-epoch to farm.
    "justification_hash": "sha256(optional_markdown_rationale)" 
  },

  // Timing and Sealing
  "timestamp_utc": 1735690000,
  "previous_state_root": "sha256(epoch_41_root)"
}
```

### Engineering Constraints
1.  **Anti-Bandwagoning Rule:** If a user submits a bond in Epoch 42 with `duration = 1`, the bond is locked for the *entirety* of Epoch 43, and resolves at the state root generation of Epoch 44. This prevents a user from bonding in the final 5 minutes of Epoch 42 to capture a yield vector they did not commit early risk to.
2.  **Decay Supremacy:** Locked `staked_weight` is NOT shielded from Thermodynamic Decay. The protocol applies the `e^-0.0577` multiplier to the user's locked balance exactly as it does to their liquid balance. You cannot avoid decay by infinitely oscillating locks.
3.  **The Anti-Reflexivity Rule:** The mempool will reject this transaction if `target_impact_id` points to another `bond_id`. Recursive meta-prediction is mechanically impossible.
