# Phase 6: Code Schemas

## 4. The `FraudProof` Object

The `FraudProof` is the nuclear deterrent of the Civilisation OS. Because we mandate **Full Validation** by all active nodes and prohibit Light Clients, any single honest node can mathematically prove that the 67% Sealing Committee signed an invalid `EpochStateRoot`.

To prevent Fraud Proofs from becoming computational Denial-of-Service (DoS) vectors, the proof must be **single-step verifiable** by the network. It cannot require re-executing the entire 30-day epoch.

### The JSON Schema

```json
{
  "protocol_version": "1.0",
  "domain_id": "sha256(domain_name)",
  
  "fraud_proof_id": "sha256(canonical_json(protocol_version, domain_id, challenged_epoch_index, challenged_state_root, proof_type, evidence_payload))",

  "challenger_identity": {
    "public_key_ed25519": "e6f7a8b9c0d1...2345",
    "signature": "hex_encoded_signature_of_fraud_proof_id"
  },

  "target": {
    "challenged_epoch_index": 42,
    "challenged_state_root": "sha256(the_invalid_epoch_42_root)",
    "previous_valid_state_root": "sha256(epoch_41_root)", // MUST anchor to the canonical prior state
    "committee_multisig": "hex_encoded_bls_multisig_that_signed_the_fraud",
    "slashing_scope": "11011100..." // A direct copy of the signers_bitmap. Only these specific keys are slashed.
  },

  "proof_type": "INVALID_DECAY", // Enum: INVALID_DECAY, INVALID_YIELD, INVALID_MERKLE, INVALID_ENTROPY

  // The Evidence Payload is specific to the proof_type.
  // Example for INVALID_DECAY: Proving the committee decayed a specific balance incorrectly.
  "evidence_payload": {
    "target_identity_pubkey": "f7a8b9c0d1e2...3456",
    
    // Step 1: Prove the starting balance from the LAST valid epoch (Epoch 41)
    "epoch_41_balance": 1500,
    "epoch_41_merkle_inclusion_proof": ["hash1", "hash2", "..."], // Proves 1500 was indeed the balance in Epoch 41
    
    // Step 2: Show the invalid balance claimed by the committee in Epoch 42
    "epoch_42_claimed_balance": 1500000000, 
    "epoch_42_merkle_inclusion_proof": ["hashA", "hashB", "..."], // Proves the committee committed this specific false number
    
    // Step 3: The Mathematical Truth (Deterministic fixed-point execution)
    "expected_balance": 1416398241 // Computed locally by verifier using fixed precision (e.g., 6 decimal places floor)
  }
}
```

### Engineering Constraints
1.  **Strict Anchor:** A FraudProof must explicitly reference the `previous_valid_state_root`. A challenger cannot construct synthetic balances from an alternative history.
2.  **Deterministic Math:** The `expected_balance` must follow strict protocol rounding rules (e.g., 6 decimal fixed-point, truncate rounding). String representations of math are fundamentally unverifiable.
3.  **Scoped Slashing:** The FraudProof explicitly extracts the `signers_bitmap` from the challenged root. The protocol slashes *only* the specific identities that cryptographically signed the fraudulent root. Honest validators who abstained or failed to verify are insulated. Atomic nuclear precision.
