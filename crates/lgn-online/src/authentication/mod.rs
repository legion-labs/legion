use std::{collections::HashMap, ops::Deref, sync::Arc};

use async_trait::async_trait;

mod client_token_set;
mod config;
mod errors;
mod oauth_client;
mod token_cache;
mod user_info;

pub mod jwt;

pub use client_token_set::ClientTokenSet;
pub use config::{AuthenticatorConfig, OAuthClientConfig, SignatureValidationConfig};
pub use errors::{Error, Result};
pub use oauth_client::OAuthClient;
pub use token_cache::TokenCache;
pub use user_info::UserInfo;

#[async_trait]
pub trait Authenticator {
    /// Perform a login.
    async fn login(
        &self,
        scopes: &[String],
        extra_params: &Option<HashMap<String, String>>,
    ) -> Result<ClientTokenSet>;

    /// Perform a non-interactive login by using a refresh login.
    async fn refresh_login(&self, client_token_set: ClientTokenSet) -> Result<ClientTokenSet>;

    /// Perform a logout, possibly using an interactive prompt.
    async fn logout(&self) -> Result<()>;
}

#[async_trait]
impl<T> Authenticator for Arc<T>
where
    T: Authenticator + Send + Sync,
{
    async fn login(
        &self,
        scopes: &[String],
        extra_params: &Option<HashMap<String, String>>,
    ) -> Result<ClientTokenSet> {
        self.deref().login(scopes, extra_params).await
    }

    async fn refresh_login(&self, client_token_set: ClientTokenSet) -> Result<ClientTokenSet> {
        self.deref().refresh_login(client_token_set).await
    }

    async fn logout(&self) -> Result<()> {
        self.deref().logout().await
    }
}

#[async_trait]
impl<T> Authenticator for Box<T>
where
    T: Authenticator + Send + Sync,
{
    async fn login(
        &self,
        scopes: &[String],
        extra_params: &Option<HashMap<String, String>>,
    ) -> Result<ClientTokenSet> {
        self.deref().login(scopes, extra_params).await
    }

    async fn refresh_login(&self, client_token_set: ClientTokenSet) -> Result<ClientTokenSet> {
        self.deref().refresh_login(client_token_set).await
    }

    async fn logout(&self) -> Result<()> {
        self.deref().logout().await
    }
}

/// A boxed `Authenticator` that can be used to authenticate requests.
pub struct BoxedAuthenticator(pub Box<dyn Authenticator + Send + Sync>);

#[async_trait]
impl Authenticator for BoxedAuthenticator {
    async fn login(
        &self,
        scopes: &[String],
        extra_params: &Option<HashMap<String, String>>,
    ) -> Result<ClientTokenSet> {
        self.0.login(scopes, extra_params).await
    }

    async fn refresh_login(&self, client_token_set: ClientTokenSet) -> Result<ClientTokenSet> {
        self.0.refresh_login(client_token_set).await
    }

    async fn logout(&self) -> Result<()> {
        self.0.logout().await
    }
}
