//! Generic data codegen test (offline)

// crate-specific lint exceptions:

#[path = "../codegen/offline/mod.rs"]
#[cfg(feature = "offline")]
pub mod offline;

#[path = "../codegen/runtime/mod.rs"]
#[cfg(feature = "runtime")]
pub mod runtime;

pub mod plugin;
