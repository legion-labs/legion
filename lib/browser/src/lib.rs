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

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow()]

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
