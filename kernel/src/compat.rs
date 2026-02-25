//! Compatibility shim — simply re-exports from std.
//! In the future, when no_std is activated via a feature flag for WASM production
//! builds, this module will conditionally re-export from `alloc` instead.
pub use std::collections::BTreeMap;
pub use std::collections::BTreeSet;
pub use std::string::String;
pub use std::vec;
pub use std::vec::Vec;
