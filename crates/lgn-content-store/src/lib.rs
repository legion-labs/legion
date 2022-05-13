//! A content-store implementation that stores immutable assets in a efficient
//! and cachable manner.

pub mod alias_providers;
mod buf_utils;
mod config;
pub mod content_providers;
mod data_space;
mod errors;
mod grpc_service;
mod identifier;
pub mod indexing;
mod manifest;
mod provider;
mod ref_counter;

pub use alias_providers::*;
pub use config::*;
pub use content_providers::*;
pub use data_space::DataSpace;
pub use errors::{Error, Result};
pub use grpc_service::{GrpcProviderSet, GrpcService};
pub use identifier::{Identifier, InvalidIdentifier};
pub use manifest::{InvalidManifest, Manifest, ManifestFormat};
pub use provider::Provider;
pub(crate) use ref_counter::RefCounter;
