//! ## Legion Web Client
//!
//! A Node native plugin that exposes functions needed for Legion's
//! applications that use the Node runtime (typically via Electron) or in the browser.

// crate-specific lint exceptions:
//#![allow()]

use std::{collections::HashMap, sync::Arc};

use http::Uri;
use lgn_online::authentication::{
    Authenticator, OAuthClient, TokenCache as OnlineTokenCache, UserInfo,
};
use napi::bindgen_prelude::{error, Error, Result};
use napi_derive::napi;
use once_cell::sync::OnceCell;

type OAuthClientTokenCache = OnlineTokenCache<OAuthClient>;

static OAUTH_CLIENT: OnceCell<Arc<OAuthClientTokenCache>> = OnceCell::new();

#[derive(Debug, thiserror::Error)]
pub enum WebClientError {
    #[error("Failed to access {0} project directory")]
    AccessProjectDirectories(String),
    #[error("Failed to init OAuth client")]
    OAuthClientInit,
    #[error("Failed to set OAuth client redirect uri")]
    OAuthClientSetRedirectUri,
    #[error("Failed to set static OAuth client")]
    StaticOAuthClient,
    #[error( "OAuth client not initialized, did you call `initOAuthClient` on Node or `init_oauth_client` in Rust?")]
    OAuthClientNotInit,
    #[error("Authentication failed: {0}")]
    Authentication(#[from] lgn_online::authentication::Error),
}

// TODO: Make application name optional for non native applications
/// Init the global OAuth client.
///
/// ## Errors
///
/// An error occurs if:
///   - The project directory cannot be found (optional)
///   - The OAuth client cannot be built
///   - The global OAuth client cannot be saved
pub async fn init_oauth_client(
    application: &str,
    issuer_url: &Uri,
    client_id: &str,
    redirect_uri: &Uri,
) -> std::result::Result<(), WebClientError> {
    let projects_dir = directories::ProjectDirs::from("com", "legionlabs", application)
        .ok_or_else(|| WebClientError::AccessProjectDirectories(application.into()))?;

    let oauth_client = OAuthClient::new(issuer_url.to_string(), client_id, Option::<String>::None)
        .await
        .map_err(|_error| WebClientError::OAuthClientInit)?
        .set_redirect_uri(redirect_uri)
        .map_err(|_error| WebClientError::OAuthClientSetRedirectUri)?;

    let oauth_client = Arc::new(OAuthClientTokenCache::new(oauth_client, projects_dir));

    OAUTH_CLIENT
        .set(oauth_client)
        .map_err(|_error| WebClientError::StaticOAuthClient)?;

    Ok(())
}

/// Init the global OAuth client.
///
/// ## Errors
///
/// An error occurs if:
///   - The project directory cannot be found (optional)
///   - The OAuth client cannot be built
///   - The global OAuth client cannot be saved
#[napi(js_name = "initOAuthClient")]
pub async fn js_init_oauth_client(
    application: String,
    issuer_url: String,
    client_id: String,
    redirect_uri: String,
) -> Result<()> {
    let issuer_url = issuer_url
        .parse()
        .map_err(|_error| Error::from_reason("Couldn't parse issuer url as Uri".to_string()))?;

    let redirect_uri = redirect_uri
        .parse()
        .map_err(|_error| Error::from_reason("Couldn't parse redirect url as Uri".to_string()))?;

    init_oauth_client(&application, &issuer_url, &client_id, &redirect_uri)
        .await
        .map_err(|error| Error::from_reason(error.to_string()))
}

/// Authenticate the user.
///
/// ## Errors
///
/// An error occurs if:
///   - The global OAuth client is not found
///   - The user cannot be authenticated
#[allow(clippy::implicit_hasher)]
pub async fn authenticate(
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
) -> std::result::Result<UserInfo, WebClientError> {
    let oauth_client = OAUTH_CLIENT
        .get()
        .ok_or(WebClientError::OAuthClientNotInit)?;

    let client_token_set = oauth_client
        .login(&scopes, &extra_params)
        .await
        .map_err(WebClientError::from)?;

    let user_info = oauth_client
        .authenticator()
        .await
        .get_user_info(&client_token_set.access_token)
        .await
        .map_err(WebClientError::from)?;

    Ok(user_info)
}

// TODO: Improve returned value type: use `UserInfo`
/// Authenticate the user.
///
/// ## Errors
///
/// An error occurs if:
///   - The global OAuth client is not found
///   - The user cannot be authenticated
#[napi(js_name = "authenticate")]
#[allow(clippy::implicit_hasher)]
pub async fn js_authenticate(
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
) -> Result<serde_json::Value> {
    let user_info = authenticate(scopes, extra_params)
        .await
        .map_err(|error| Error::from_reason(format!("Authentication failed: {}", error)))?;

    let user_info_value = serde_json::to_value(user_info).map_err(|_error| {
        Error::from_reason("Couldn't convert user info object to json value".to_string())
    })?;

    Ok(user_info_value)
}

/// Returns the current access token.
///
/// ## Errors
///
/// An error occurs if:
///   - The global OAuth client is not found
///   - The access token couldn't be read from disk
pub fn get_access_token() -> std::result::Result<String, WebClientError> {
    let oauth_client = OAUTH_CLIENT
        .get()
        .ok_or(WebClientError::OAuthClientNotInit)?;

    Ok(oauth_client
        .read_token_set_from_cache()
        .map_err(WebClientError::from)?
        .access_token)
}

/// Returns the current access token.
///
/// ## Errors
///
/// An error occurs if:
///   - The global OAuth client is not found
///   - The access token couldn't be read from disk
#[napi(js_name = "accessToken")]
pub fn js_get_access_token() -> Result<String> {
    get_access_token()
        .map_err(|error| Error::from_reason(format!("Access token retrieval error: {}", error)))
}
