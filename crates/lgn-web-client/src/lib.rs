//! ## Legion Browser
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
use napi::bindgen_prelude::*;
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

#[napi(js_name = "initOAuthClient")]
pub async fn js_init_oauth_client(
    application: String,
    issuer_url: String,
    client_id: String,
    redirect_uri: String,
) -> Result<()> {
    let issuer_url = issuer_url
        .parse()
        .map_err(|_error| Error::from_reason("Couldn't parse issuer url as Uri".into()))?;

    let redirect_uri = redirect_uri
        .parse()
        .map_err(|_error| Error::from_reason("Couldn't parse redirect url as Uri".into()))?;

    init_oauth_client(&application, &issuer_url, &client_id, &redirect_uri)
        .await
        .map_err(|error| Error::from_reason(error.to_string()))
}

pub async fn authenticate(
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
) -> std::result::Result<UserInfo, WebClientError> {
    let oauth_client = OAUTH_CLIENT
        .get()
        .ok_or_else(|| WebClientError::OAuthClientNotInit)?;

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
#[napi(js_name = "authenticate")]
pub async fn js_authenticate(
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
) -> Result<serde_json::Value> {
    let user_info = authenticate(scopes, extra_params)
        .await
        .map_err(|error| Error::from_reason(format!("Authentication failed: {}", error)))?;

    let user_info_value = serde_json::to_value(user_info).map_err(|_error| {
        Error::from_reason("Couldn't convert user info object to json value".into())
    })?;

    Ok(user_info_value)
}

pub fn get_access_token() -> std::result::Result<String, WebClientError> {
    let oauth_client = OAUTH_CLIENT
        .get()
        .ok_or_else(|| WebClientError::OAuthClientNotInit)?;

    Ok(oauth_client
        .read_token_set_from_cache()
        .map_err(WebClientError::from)?
        .access_token)
}

#[napi(js_name = "accessToken")]
pub fn js_get_access_token() -> Result<String> {
    get_access_token()
        .map_err(|error| Error::from_reason(format!("Access token retrieval error: {}", error)))
}

// TODO: Follows legacy code, should be dropped
#[deprecated(note = "Tauri is not used anymore, consider using the new functions instead")]
pub mod tauri_app {
    //! ## Legion Browser
    //!
    //! A Tauri plugin that exposes functions needed for Legion's
    //! applications that run on Tauri and in the browser.
    //!
    //! ## Example
    //!
    //! ```no_run
    //! # use http::Uri;
    //! #
    //! # #[tokio::main]
    //! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //! # let issuer_url = "https://something.auth-provider.com".parse().unwrap();
    //! # let client_id = "foobar";
    //! # let redirect_uri = "http://whatever".parse().unwrap();
    //! #
    //! // First you need the plugin itself:
    //! let browser_plugin = lgn_web_client::tauri_app::BrowserPlugin::new(
    //!     "my-app",
    //!     &issuer_url,
    //!     &client_id,
    //!     &redirect_uri,
    //! )
    //!     .await
    //!     .expect("Couldn't build the BrowserPlugin");
    //!
    //! // Now we can build a Tauri application with the browser plugin:
    //! let builder = tauri::Builder::default()
    //!    .plugin(browser_plugin)
    //!    .invoke_handler(tauri::generate_handler![]);
    //! #
    //! # Ok(())
    //! # }
    //! ```
    //!

    use std::{collections::HashMap, sync::Arc};

    use anyhow::anyhow;
    use http::Uri;
    use lgn_online::authentication::{Authenticator, OAuthClient, UserInfo};
    use tauri::{plugin::Plugin, Invoke, Manager, Runtime};

    use super::OAuthClientTokenCache;

    #[tauri::command]
    async fn authenticate(
        oauth_client: tauri::State<'_, Arc<OAuthClientTokenCache>>,
        scopes: Vec<String>,
        extra_params: Option<HashMap<String, String>>,
    ) -> std::result::Result<UserInfo, String> {
        let client_token_set = oauth_client
            .login(&scopes, &extra_params)
            .await
            .map_err(|error| error.to_string())?;

        let user_info = oauth_client
            .authenticator()
            .await
            .get_user_info(&client_token_set.access_token)
            .await
            .map_err(|error| error.to_string())?;

        Ok(user_info)
    }

    #[tauri::command]
    #[allow(clippy::needless_pass_by_value)]
    fn get_access_token(
        oauth_client: tauri::State<'_, Arc<OAuthClientTokenCache>>,
    ) -> std::result::Result<String, String> {
        Ok(oauth_client
            .read_token_set_from_cache()
            .map_err(|error| error.to_string())?
            .access_token)
    }

    pub struct BrowserPlugin<R: Runtime> {
        invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
        pub oauth_client: Arc<OAuthClientTokenCache>,
    }

    impl<R: Runtime> BrowserPlugin<R> {
        /// Creates a [`BrowserPlugin`] from an [`http::Uri`] and an application
        /// name. The application name will be used to lookup the
        /// `directories::ProjectDirs`.
        ///
        /// # Errors
        ///
        /// Returns an error if the Url format is invalid (i.e. compliant with Aws
        /// Cognito) or if the project directories can't be found.
        pub async fn new(
            application: &str,
            issuer_url: &Uri,
            client_id: &str,
            redirect_uri: &Uri,
        ) -> anyhow::Result<Self> {
            let projects_dir = directories::ProjectDirs::from("com", "legionlabs", application)
                .ok_or_else(|| anyhow!("Failed to get project directory"))?;

            let oauth_client =
                OAuthClient::new(issuer_url.to_string(), client_id, Option::<String>::None)
                    .await?
                    .set_redirect_uri(redirect_uri)?;

            let oauth_client = Arc::new(OAuthClientTokenCache::new(oauth_client, projects_dir));

            Ok(Self {
                invoke_handler: Box::new(tauri::generate_handler![authenticate, get_access_token]),
                oauth_client,
            })
        }
    }

    impl<R: Runtime> Plugin<R> for BrowserPlugin<R> {
        fn name(&self) -> &'static str {
            "browser"
        }

        fn initialize(
            &mut self,
            app: &tauri::AppHandle<R>,
            _config: serde_json::Value,
        ) -> tauri::plugin::Result<()> {
            let oauth_client = Arc::clone(&self.oauth_client);

            app.manage(oauth_client);

            Ok(())
        }

        fn extend_api(&mut self, invoke: Invoke<R>) {
            (self.invoke_handler)(invoke);
        }
    }
}
