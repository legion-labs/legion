//! Legion core crate, contains core services and systems used by other modules
//! The crate is not allowed to depend on other legion modules

// crate-specific lint exceptions:
#![allow(clippy::implicit_hasher, clippy::missing_errors_doc)]

use std::{future::Future, pin::Pin};

pub mod decimal;
pub mod memory;
pub mod trust_cell;

pub mod label;

mod hash;
pub use hash::*;

#[cfg(not(target_arch = "wasm32"))]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[cfg(target_arch = "wasm32")]
pub type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;
