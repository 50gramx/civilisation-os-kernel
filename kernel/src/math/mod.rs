//! The deterministic fixed-point math engine.
//!
//! Module layout:
//!   math::fixed    — The Fixed(u128) wrapper. Private inner value.
//!   math::sqrt     — Constitutional integer square root (Babylonian, floor-rounded).
//!   math::overflow — Checked arithmetic combinators used by the rest of the kernel.

pub mod fixed;
pub mod sqrt;
pub mod overflow;
