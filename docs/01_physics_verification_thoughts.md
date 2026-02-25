# Physics Verification Thoughts

The user's objective is to verify the deterministic physics core of Civilisation OS *before* implementing any serialization (RFC 8785 Canonical JSON). This is a critical constitutional checkpoint.

## Why Test Before Serialization?
Serialization is complex and introduces a lot of cognitive overhead. If we implement it while the math/cryptography layer has hidden bugs (e.g. integer overflow, non-deterministic Merkle padding, floating-point remnants), we risk confusing serialization issues with core arithmetic instability.

## Verification Checklist
We need to ensure that the following core primitives act deterministically and behave identically across native debug, native release, and WASM targets:
1. **isqrt(u128::MAX)**: Must yield exactly `18446744073709551615`. Any deviation breaks determinism.
2. **Merkle 3-leaf padding vector**: Padded tree logic must produce identical roots across builds to guarantee state consistency.
3. **Decay Constant**: `DECAY_FACTOR_SCALED = 943_932_824_245`. We need to test edge cases involving large balances near the `u128` ceiling, and small balances near the dust threshold. Truncation must be mathematically stable and identical across platforms.
4. **Panic on Overflow**: Any unchecked overflow must immediately abort. `checked_*` math must be rigorously enforced so that invalid transitions yield a deterministic `TransitionError::MathOverflow` rather than platform-dependent unwinding/panics.

## Next Steps
Once we pass `cargo test --release` natively, and `cargo build --target wasm32-unknown-unknown --release` without warnings or platform-specific variance, we establish our "physics proof" (v0.0.0-draft). Only then can we safely proceed to canonical JSON.
