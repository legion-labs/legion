pub mod null;

pub mod shared;
// pub use shared::tmp_extract_root_signature_def;

mod deferred_drop;
// #[allow(unused_imports)]
// use deferred_drop::{Drc, DeferredDropper};

#[cfg(feature = "vulkan")]
pub mod vulkan;
