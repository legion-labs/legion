//! ## Legion Auth Node

// crate-specific lint exceptions:
//#![allow()]

use std::{collections::HashMap, sync::Arc};

use http::Uri;
use lgn_auth::{AuthenticatorWithClaims, OAuthClient, TokenCache as OnlineTokenCache, UserInfo};
use napi_derive::napi;
use once_cell::sync::OnceCell;

use crate::error::Error;

mod error;

type OAuthClientTokenCache = OnlineTokenCache<OAuthClient>;

static OAUTH_CLIENT: OnceCell<Arc<OAuthClientTokenCache>> = OnceCell::new();

/// Init the global OAuth client.
///
/// ## Errors
///
/// An error occurs if:
///   - The project directory cannot be found (optional)
///   - The OAuth client cannot be built
///   - The global OAuth client cannot be saved
#[napi]
pub async fn init_oauth_client(
    application: String,
    issuer_url: String,
    client_id: String,
    redirect_uri: String,
) -> napi::Result<()> {
    let issuer_url: Uri = issuer_url.parse().map_err(|_error| {
        napi::Error::from_reason("Couldn't parse issuer url as Uri".to_string())
    })?;

    let redirect_uri: Uri = redirect_uri.parse().map_err(|_error| {
        napi::Error::from_reason("Couldn't parse redirect url as Uri".to_string())
    })?;

    let projects_dir = directories::ProjectDirs::from("com", "legionlabs", &application)
        .ok_or_else(|| Error::AccessProjectDirectories(application).to_napi_error())?;

    let oauth_client = OAuthClient::new(issuer_url.to_string(), client_id, Option::<String>::None)
        .await
        .map_err(|_error| Error::OAuthClientInit.to_napi_error())?
        .set_redirect_uri(&redirect_uri)
        .map_err(|_error| Error::OAuthClientSetRedirectUri.to_napi_error())?;

    let oauth_client = Arc::new(OAuthClientTokenCache::new(oauth_client, projects_dir));

    OAUTH_CLIENT
        .set(oauth_client)
        .map_err(|_error| Error::StaticOAuthClient.to_napi_error())?;

    Ok(())
}

/// Authenticate the user.
///
/// ## Errors
///
/// An error occurs if:
///   - The global OAuth client is not found
///   - The user cannot be authenticated
#[napi]
#[allow(clippy::implicit_hasher)]
pub async fn authenticate(
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
) -> napi::Result<UserInfo> {
    let oauth_client = OAUTH_CLIENT
        .get()
        .ok_or_else(|| Error::OAuthClientNotInit.to_napi_error())?;

    let user_info_claims = oauth_client
        .authenticate(&scopes, &extra_params)
        .await
        .map_err(|error| Error::from(error).to_napi_error())?;

    Ok(user_info_claims)
}

/// Returns the current access token.
///
/// ## Errors
///
/// An error occurs if:
///   - The global OAuth client is not found
///   - The access token couldn't be read from disk
#[napi]
pub fn get_access_token() -> napi::Result<String> {
    let oauth_client = OAUTH_CLIENT
        .get()
        .ok_or_else(|| Error::OAuthClientNotInit.to_napi_error())?;

    let access_token = oauth_client
        .read_token_set_from_cache()
        .map_err(|error| Error::from(error).to_napi_error())?
        .access_token;

    Ok(access_token)
}
