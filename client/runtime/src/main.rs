//! Runtime client executable

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

    let user_info = token_cache
        .authenticator()
        .await
        .get_user_info(&access_token)
        .await
        .map_err::<anyhow::Error, _>(Into::into)?;

    Ok(user_info)
}

#[lgn_tauri_command]
#[allow(clippy::needless_pass_by_value)]
fn get_access_token(token_cache: tauri::State<'_, Arc<TokenCache>>) -> anyhow::Result<String> {
    Ok(token_cache.read_token_set_from_cache()?.access_token)
}

fn main() -> anyhow::Result<()> {
    let config = Config::new_from_environment()?;

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(&config.authorization_url)?;

    let projects_dir = directories::ProjectDirs::from("com", "legionlabs", "legion-editor")
        .expect("Failed to get project directory");

    let token_cache = Arc::new(TokenCache::new(authenticator, projects_dir));

    let builder = tauri::Builder::default()
        .manage(Arc::clone(&token_cache))
        .invoke_handler(tauri::generate_handler![authenticate, get_access_token]);

    App::new()
        .insert_non_send_resource(TauriPluginSettings::new(builder))
        .add_plugin(TauriPlugin::new(tauri::generate_context!()))
        .add_plugin(AsyncPlugin)
        .run();

    Ok(())
}
