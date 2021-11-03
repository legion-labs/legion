use async_trait::async_trait;

mod aws_cognito_client_authenticator;
mod client_token_set;
mod token_cache;
mod user_info;

pub mod jwt;

pub use aws_cognito_client_authenticator::AwsCognitoClientAuthenticator;
pub use client_token_set::ClientTokenSet;
pub use token_cache::TokenCache;
pub use user_info::UserInfo;

#[async_trait]
pub trait Authenticator {
    /// Perform a login.
    async fn login(&self) -> anyhow::Result<ClientTokenSet>;
    ///
    /// Perform a non-interactive login by using a refresh login.
    async fn refresh_login(&self, refresh_token: &str) -> anyhow::Result<ClientTokenSet>;

    /// Perform a logout, possibly using an interactive prompt.
    async fn logout(&self) -> anyhow::Result<()>;
}
