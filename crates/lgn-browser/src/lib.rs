//! ## Legion Browser
//!
//! A Tauri plugin that exposes functions needed for Legion's
//! applications that run on Tauri and in the browser.
//!
//! ## Example
//!
//! ```
//! let authorization_url = "https://my-app.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=XXX&response_type=code&scope=XXX&redirect_uri=http://localhost:3000/";
//!
//! // First you need the plugin itself:
//! let browser_plugin = lgn_browser::BrowserPlugin::from_url_str(authorization_url, "my-app")
//!     .expect("Couldn't build the BrowserPlugin");
//!
//! // Now we can build a Tauri application with the browser plugin:
//! let builder = tauri::Builder::default()
//!    .plugin(browser_plugin)
//!    .invoke_handler(tauri::generate_handler![]);
//! ```

// crate-specific lint exceptions:
//#![allow()]

use std::sync::Arc;

use anyhow::anyhow;
use lgn_online::authentication::{
    Authenticator, AwsCognitoClientAuthenticator, TokenCache as OnlineTokenCache, UserInfo,
};
use tauri::{plugin::Plugin, Invoke, Manager, Runtime};
use url::Url;

type TokenCache = OnlineTokenCache<AwsCognitoClientAuthenticator>;

#[tauri::command]
async fn authenticate(token_cache: tauri::State<'_, Arc<TokenCache>>) -> Result<UserInfo, String> {
    let access_token = token_cache
        .login()
        .await
        .map_err(|error| error.to_string())?
        .access_token;

    let user_info = token_cache
        .authenticator()
        .await
        .get_user_info(&access_token)
        .await
        .map_err(|error| error.to_string())?;

    Ok(user_info)
}

#[tauri::command]
#[allow(clippy::needless_pass_by_value)]
fn get_access_token(token_cache: tauri::State<'_, Arc<TokenCache>>) -> Result<String, String> {
    Ok(token_cache
        .read_token_set_from_cache()
        .map_err(|error| error.to_string())?
        .access_token)
}

pub struct BrowserPlugin<R: Runtime> {
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
    pub token_cache: Arc<TokenCache>,
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
    pub fn new(authorization_url: &Url, application: &str) -> anyhow::Result<Self> {
        let authenticator =
            AwsCognitoClientAuthenticator::from_authorization_url(authorization_url)?;

        let projects_dir = directories::ProjectDirs::from("com", "legionlabs", application)
            .ok_or_else(|| anyhow!("Failed to get project directory"))?;

        let token_cache = Arc::new(TokenCache::new(authenticator, projects_dir));

        Ok(Self {
            invoke_handler: Box::new(tauri::generate_handler![authenticate, get_access_token]),
            token_cache,
        })
    }

    /// Same as [`BrowserPlugin::new`] but accepts an `str` instead of an
    /// [`url::Url`].
    ///
    /// # Errors
    ///
    /// Returns an error if the `str` cannot be parsed.
    pub fn from_url_str(authorization_url: &str, application: &str) -> anyhow::Result<Self> {
        let authorization_url = authorization_url.parse()?;

        Self::new(&authorization_url, application)
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
        let token_cache = Arc::clone(&self.token_cache);

        app.manage(token_cache);

        Ok(())
    }

    fn extend_api(&mut self, invoke: Invoke<R>) {
        (self.invoke_handler)(invoke);
    }
}
