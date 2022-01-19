//! Analytics app executable

// crate-specific lint exceptions:
//#![allow()]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use config::Config;
use lgn_app::prelude::*;
use lgn_async::AsyncPlugin;
use lgn_online::authentication::{
    Authenticator, AwsCognitoClientAuthenticator, TokenCache as OnlineTokenCache, UserInfo,
};
use lgn_tauri::{lgn_tauri_command, TauriPlugin, TauriPluginSettings};

mod config;

type TokenCache = OnlineTokenCache<AwsCognitoClientAuthenticator>;

#[lgn_tauri_command]
async fn authenticate(token_cache: tauri::State<'_, Arc<TokenCache>>) -> anyhow::Result<UserInfo> {
    let access_token = token_cache.login().await?.access_token;

    token_cache
        .authenticator()
        .await
        .get_user_info(&access_token)
        .await
        .map_err(Into::into)
}

fn main() -> anyhow::Result<()> {
    let config = Config::new_from_environment()?;

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(&config.authorization_url)?;

    let projects_dir = directories::ProjectDirs::from("com", "legionlabs", "legion-analytics")
        .expect("Failed to get project directory");

    let token_cache = Arc::new(TokenCache::new(authenticator, projects_dir));

    let builder = tauri::Builder::default()
        .manage(Arc::clone(&token_cache))
        .invoke_handler(tauri::generate_handler![authenticate]);

    App::new()
        .insert_non_send_resource(TauriPluginSettings::new(builder))
        .add_plugin(TauriPlugin::new(tauri::generate_context!()))
        .add_plugin(AsyncPlugin)
        .run();

    Ok(())
}
