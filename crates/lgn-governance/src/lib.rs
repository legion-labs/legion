//! Legion Labs Governance library.
//!
//! This crate contains both the API and the implementation of the governance
//! service, which controls the main aspects of the Legion Engine ecosystem.

pub mod api;
pub mod client;
mod config;
mod errors;
pub mod server;
pub mod types;

pub use api::register_routes;
pub use config::*;
pub use errors::{Error, Result};
