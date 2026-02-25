# Phase 6: Code Schemas

## 3. The `EpochStateRoot` Object

The `EpochStateRoot` is the heartbeat of the Civilisation OS. Every 30 days (determined by the VDF), the pseudo-randomly selected Sealing Committee must agree on this exact JSON object. It represents the absolute canonical truth of the Domain at that instant.

### The JSON Schema

```json
{
  "protocol_version": "1.0",
  "domain_id": "sha256(domain_name)",
  
  "epoch_metadata": {
    "epoch_index": 42,
    "previous_state_root": "sha256(epoch_41_root)",
    "timestamp_utc_start": 1733000000,
    "timestamp_utc_end": 1735592000
  },

  // The Proof of Time
  "vdf_proof": {
    "challenge_seed": "sha256(epoch_41_root)",
    "difficulty_iterations": 1050000000, // Algorithmically adjusted based on epoch 41 duration to target exactly 30 days
    "output_hash": "a1b2c3d4...", 
    "snark_proof": "hex_encoded_zk_snark_of_vdf_execution"
  },

  // State Transitions (The physical execution of the engine)
  "state_transitions": {
    "accepted_proofs_of_impact_root": "sha256(merkle_of_lexicographically_sorted_proofs)",
    "accepted_vouch_bonds_root": "sha256(merkle_of_lexicographically_sorted_bonds)",
    "active_validator_registry_root": "sha256(merkle_of_active_ed25519_keys)",
    "locked_stakes_root": "sha256(merkle_of_locked_genesis_stakes)" // Tracks Phase 4/5 lockups
  },
  
  // The Thermodynamic Metrics (Used to calculate Domain Death Purge)
  "thermodynamic_state": {
    "active_validator_count": 412, // The Liveness metric
    "entropy_metric": {
      "type": "validator_churn_percentage",
      "value": 0.34 // Must strictly match the delta computed from active_validator_registry_root
    }
  },

  // The Final Output
  "new_canonical_state_root": "sha256(merkle_of_all_identity_accountability_balances)",

  // The 67% Sealing Committee Signatures
  "committee_signatures": {
    "sortition_seed": "vdf_output_hash_from_epoch_41",
    "committee_size_total": 412, // Explicit definition prevents verification ambiguity
    "required_threshold": 277, // Math: ceiling(412 * 0.67)
    "aggregated_bls_signature": "hex_encoded_bls_multisig",
    "signers_bitmap": "11011100..." 
  }
}
```

### Engineering Constraints
1.  **Deterministic Lexicographic Sorting:** To prevent state root mismatches based on network gossip arrival times, all valid objects (Proofs and Bonds) must be sorted lexicographically by their canonical `impact_id` or `bond_id` before constructing the Merkle roots.
2.  **Dynamic VDF Engine:** `difficulty_iterations` is not static. The protocol applies a moving average over the last 10 epochs. If hardware accelerates and Epoch 41 took 28 days, Epoch 42's iterations automatically increase to target 30 days.
3.  **Cryptographically Sealed Entropy:** The `entropy_metric` is not computed arbitrarily off-chain. It must be deterministically calculable by any node comparing the `active_validator_registry_root` of Epoch 41 and Epoch 42. If a committee lies about the entropy value to avoid a purge, the State Root is invalid.
