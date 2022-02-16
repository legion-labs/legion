//! ## Legion Browser
//!
//! A Tauri plugin that exposes functions needed for Legion's
//! applications that run on Tauri and in the browser.
//!
//! ## Example
//!
//! ```no_run
//! # use url::Url;
//! #
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let issuer_url = Url::parse("https://something.auth-provider.com").unwrap();
//! # let client_id = "foobar";
//! # let redirect_uri = Url::parse("http://whatever").unwrap();
//! #
//! // First you need the plugin itself:
//! let browser_plugin = lgn_web_client::BrowserPlugin::new(
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

// crate-specific lint exceptions:
//#![allow()]

use std::{collections::HashMap, sync::Arc};

use anyhow::anyhow;
use lgn_online::authentication::{
    Authenticator, OAuthClient, TokenCache as OnlineTokenCache, UserInfo,
};
use tauri::{plugin::Plugin, Invoke, Manager, Runtime};
use url::Url;

type OAuthClientTokenCache = OnlineTokenCache<OAuthClient>;

#[tauri::command]
async fn authenticate(
    oauth_client: tauri::State<'_, Arc<OAuthClientTokenCache>>,
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
) -> Result<UserInfo, String> {
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
) -> Result<String, String> {
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
    /// Creates a [`BrowserPlugin`] from an [`url::Url`] and an application
    /// name. The application name will be used to lookup the
    /// `directories::ProjectDirs`.
    ///
    /// # Errors
    ///
    /// Returns an error if the Url format is invalid (i.e. compliant with Aws
    /// Cognito) or if the project directories can't be found.
    pub async fn new(
        application: &str,
        issuer_url: &Url,
        client_id: &str,
        redirect_uri: &Url,
    ) -> anyhow::Result<Self> {
        let projects_dir = directories::ProjectDirs::from("com", "legionlabs", application)
            .ok_or_else(|| anyhow!("Failed to get project directory"))?;

        let mut oauth_client = OAuthClient::new(issuer_url.to_string(), client_id).await?;

        oauth_client = oauth_client.set_redirect_uri(redirect_uri)?;

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
