# Phase 6: Code Schemas

## 1. The `ProofOfImpact` Object

The `ProofOfImpact` is the foundational mass of the Civilisation OS. It is a backward-looking claim of labor. If this is not strictly formatted, the system devolves into subjective "narrative validation."

To prevent this, a Proof of Impact must point to **machine-verifiable external state** or highly specific cryptographic artifacts.

### The JSON Schema

```json
{
  "protocol_version": "1.0",
  "domain_id": "sha256(domain_name)",
  "epoch_index": 42,
  
  // Deterministic Identifier computed over the serialized payload (excluding signature)
  "impact_id": "sha256(canonical_json(protocol_version, domain_id, epoch_index, domain_nonce, impact_payload, timestamp_utc, previous_state_root))",
  
  "creator_identity": {
    "public_key_ed25519": "a1b2c3d4e5f6...890a",
    "domain_nonce": 928374, // Monotonically increasing per domain per identity to prevent replay attacks
    "signature": "hex_encoded_signature_of_impact_id"
  },

  "impact_payload": {
    "impact_type": "EXTERNAL_MERGE", 
    "summary_hash": "sha256(markdown_description_of_work)", // Narrative lives exclusively in the Agora DHT
    
    // The anchor to objective reality. Only the hashes are protocol-enforced.
    "verifiable_pointers": [
      {
        "type": "git_commit",
        "hash": "7a8b9c0d1e2f...", // THIS is what the protocol validates
        "informational_uri": "https://github.com/project/repo/commit/7a8b9c" // This is for human convenience; protocol ignores if broken
      }
    ]
  },

  // Timing and Sealing
  "timestamp_utc": 1735689600,
  "previous_state_root": "sha256(epoch_41_root)" // Anchors this claim to the latest known canonical history
}
```

### Engineering Constraints
1.  **Strict Determinism:** The `impact_id` provides a stable hash for VouchBonds to target. It is computed over a canonically sorted JSON string. 
2.  **Replay Prevention:** The `domain_nonce` prevents attackers from capturing a valid proof and re-submitting it across epochs or mimicking it in other domains to farm yield.
3.  **Hash Supremacy:** The `verifiable_pointers` intentionally demote the URL to `informational_uri`. If a centralized service changes its URL routing, the protocol does not care. Only the cryptographic hash of the external content matters for validity.
