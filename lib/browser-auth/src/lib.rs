//! An experimental Auth crate that compiles to WASM and is meant to be used in the browser.
//!
//! Notice: because of [this limitation](https://github.com/rustwasm/wasm-bindgen/issues/2195)
//! most of the functions exposed in this module take a `String` instead of a `&str` or equivalent.

#![no_std]
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
#![allow(clippy::missing_errors_doc)]

extern crate alloc;

use alloc::string::{String, ToString};
use js_sys::Date;
use lgn_online::{
    types::{AwsCognitoClientAuthenticator, TokenCache},
    UserAlreadyLoggedInError,
};
use utils::{document_set_cookie, get_document, parse_cookies, set_panic_hook};
use wasm_bindgen::prelude::*;

mod lgn_online;
#[macro_use]
mod utils;

// TODO: Move to a config file
const AUTHORIZATION_URL: &str = "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=5m58nrjfv6kr144prif9jk62di&response_type=code&scope=aws.cognito.signin.user.admin+email+https://legionlabs.com/editor/allocate+openid+profile&redirect_uri=http://localhost:3000/&identity_provider=Azure";

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

/// If the access token (stored in cookies) is not found or expired
/// the user is redirected to Cognito in order to issue a new token.
///
/// Ultimately, after the user is authenticated, they will be redirected
/// to the aplication with a code, provided by Cognito, in the URL.
#[wasm_bindgen(js_name = "getAuthorizationCodeInteractive")]
pub fn get_authorization_code_interactive() -> Result<(), JsValue> {
    set_panic_hook();

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(AUTHORIZATION_URL.parse().unwrap())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let token_cache = TokenCache::new(authenticator);

    match token_cache.get_authorization_code_interactive() {
        Ok(()) => Ok(()),
        Err(error) if error.is::<UserAlreadyLoggedInError>() => Ok(()),
        Err(error) => Err(JsValue::from_str(&error.to_string())),
    }
}

/// If the access token (stored in cookies) is not found or expired
/// then a `ClientTokenSet` will be fetched and cookies will be set accordingly
/// in the browser.
///
/// At that point, the user is authenticated.
#[wasm_bindgen(js_name = "finalizeAwsCognitoAuth")]
pub async fn finalize_aws_cognito_auth(code: String) -> Result<(), JsValue> {
    set_panic_hook();

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(AUTHORIZATION_URL.parse().unwrap())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let token_cache = TokenCache::new(authenticator);

    let client_token_set = token_cache
        .get_token_set_from_authorization_code(code.as_ref())
        .await;

    let client_token_set = match client_token_set {
        Ok(client_token_set) => client_token_set,
        Err(error) if error.is::<UserAlreadyLoggedInError>() => return Ok(()),
        Err(error) => return Err(JsValue::from_str(&error.to_string())),
    };

    let document = get_document();

    document_set_cookie(
        &document,
        "access_token",
        &client_token_set.access_token,
        client_token_set.expires_in,
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    if let Some(refresh_token) = client_token_set.refresh_token {
        document_set_cookie(
            &document,
            "refresh_token",
            &refresh_token,
            client_token_set.expires_in,
        )
        .map_err(|error| JsValue::from_str(&error.to_string()))?;
    }

    let expires_at = (Date::now() as u64) + client_token_set.expires_in * 1_000;

    document_set_cookie(
        &document,
        "expires_at",
        &expires_at,
        client_token_set.expires_in,
    )
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    Ok(())
}

/// Gets the access token currently stored in cookies
#[wasm_bindgen(js_name = "getAccessToken")]
pub fn get_access_token() -> Option<String> {
    set_panic_hook();

    let cookies = get_document().cookie().unwrap();

    let cookies = parse_cookies(&cookies);

    cookies.get("access_token").map(ToString::to_string)
}

/// Use the provided access token to fetch the authed user info.
#[wasm_bindgen(js_name = "getUserInfo")]
pub async fn get_user_info(access_token: String) -> Result<JsValue, JsValue> {
    set_panic_hook();

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(AUTHORIZATION_URL.parse().unwrap())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let user_info = authenticator
        .get_user_info(&access_token)
        .await
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let user_info =
        JsValue::from_serde(&user_info).map_err(|error| JsValue::from_str(&error.to_string()))?;

    Ok(user_info)
}
