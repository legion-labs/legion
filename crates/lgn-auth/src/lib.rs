//! ## Legion Auth

// crate-specific lint exceptions:
#![allow(clippy::missing_errors_doc)]

pub use authenticator::{
    Authenticator, AuthenticatorWithCanonicalClaims, AuthenticatorWithClaims, BoxedAuthenticator,
};
pub use client_token_set::ClientTokenSet;
pub use error::{Error, Result};
pub use oauth_client::OAuthClient;
pub use token_cache::TokenCache;
pub use user_info::{deserialize_string_bool, UserInfo};

mod authenticator;
mod client_token_set;
mod error;
mod oauth_client;
mod token_cache;
mod user_info;

pub mod jwt;
