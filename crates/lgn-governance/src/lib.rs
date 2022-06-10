//! Legion Labs Governance library.
//!
//! This crate contains both the API and the implementation of the governance
//! service, which controls the main aspects of the Legion Engine ecosystem.

mod api;
mod client;
mod errors;
mod server;
pub mod types;

pub use api::register_routes;
pub use client::Client;
pub use errors::{Error, Result};
pub use server::{
    PermissionsCache, Server, ServerAwsCognitoOptions, ServerMySqlOptions, ServerOptions,
};
