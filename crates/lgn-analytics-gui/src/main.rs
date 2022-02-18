//! Analytics app executable

// crate-specific lint exceptions:
//#![allow()]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use config::Config;
use lgn_app::prelude::*;
use lgn_async::AsyncPlugin;
use lgn_tauri::{TauriPlugin, TauriPluginSettings};
use lgn_web_client::BrowserPlugin;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::new_from_environment()?;

    let browser_plugin = BrowserPlugin::new(
        &config.application_name,
        &config.issuer_url,
        &config.client_id,
        &config.redirect_uri,
    )
    .await?;

    let builder = tauri::Builder::default()
        .plugin(browser_plugin)
        .invoke_handler(tauri::generate_handler![]);

    App::default()
        .insert_non_send_resource(TauriPluginSettings::new(builder))
        .add_plugin(TauriPlugin::new(tauri::generate_context!()))
        .add_plugin(AsyncPlugin)
        .run();

    Ok(())
}
