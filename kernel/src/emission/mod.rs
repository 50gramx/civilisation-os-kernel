//! Emission module: the EmissionPolicy trait and ZeroEmission default.
//! The emission model is intentionally decoupled from the physics engine.
//! Physics compiles and passes determinism tests WITHOUT any emission logic.
//! The SublinearBondEmission implementation is injected only after adversarial simulation.
pub mod policy;
pub mod zero;
