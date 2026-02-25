//! Additional checked arithmetic combinators.
//! Thin wrappers to make common patterns in the kernel more ergonomic
//! while preserving the explicit checked_* discipline.

use crate::TransitionError;

/// Multiply two raw u128 values with overflow check.
/// Use when you need to multiply before a division without creating Fixed values.
pub fn checked_mul_raw(a: u128, b: u128) -> Result<u128, TransitionError> {
    a.checked_mul(b).ok_or(TransitionError::MathOverflow)
}

/// Divide raw a by raw b. Returns DivisionByZero if b is zero.
pub fn checked_div_raw(a: u128, b: u128) -> Result<u128, TransitionError> {
    if b == 0 {
        return Err(TransitionError::DivisionByZero);
    }
    Ok(a / b)
}

/// Add two raw u128 values with overflow check.
pub fn checked_add_raw(a: u128, b: u128) -> Result<u128, TransitionError> {
    a.checked_add(b).ok_or(TransitionError::MathOverflow)
}

/// Subtract raw b from raw a with underflow check.
pub fn checked_sub_raw(a: u128, b: u128) -> Result<u128, TransitionError> {
    a.checked_sub(b).ok_or(TransitionError::MathOverflow)
}
