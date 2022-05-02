#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("Failed to access {0} project directory")]
    AccessProjectDirectories(String),

    #[error("Failed to init OAuth client")]
    OAuthClientInit,

    #[error("Failed to set OAuth client redirect uri")]
    OAuthClientSetRedirectUri,

    #[error("Failed to set static OAuth client")]
    StaticOAuthClient,

    #[error("OAuth client not initialized, did you call `initOAuthClient` on Node or `init_oauth_client` in Rust?")]
    OAuthClientNotInit,

    #[error(transparent)]
    Authentication(#[from] lgn_auth::Error),
}

impl Error {
    pub(crate) fn to_napi_error(&self) -> napi::Error {
        napi::Error::from_reason(self.to_string())
    }
}
