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

use alloc::{
    boxed::Box,
    string::{String, ToString},
};
use js_sys::Date;
use lgn_online::{
    types::{AwsCognitoClientAuthenticator, ClientTokenSet, TokenCache},
    ClientTokenSetRequest, UserAlreadyLoggedInError,
};
use utils::{document_set_cookie, get_document, parse_cookies, set_panic_hook};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

mod lgn_online;
mod utils;

// TODO: Move to a config file
const AUTHORIZATION_URL: &str = "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize?client_id=5m58nrjfv6kr144prif9jk62di&response_type=code&scope=aws.cognito.signin.user.admin+email+https://legionlabs.com/editor/allocate+openid+profile&redirect_uri=http://localhost:3000/&identity_provider=Azure";

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;

/// Takes a [`ClientTokenSet`] and set all the cookies accordingly.
fn populate_client_token_set_cookies(client_token_set: &ClientTokenSet) -> anyhow::Result<()> {
    let document = get_document();

    document_set_cookie(
        &document,
        "access_token",
        &client_token_set.access_token,
        client_token_set.expires_in,
    )?;

    if let Some(ref refresh_token) = client_token_set.refresh_token {
        document_set_cookie(
            &document,
            "refresh_token",
            refresh_token,
            client_token_set.expires_in,
        )?;
    }

    let expires_at = (Date::now() as u64) + client_token_set.expires_in * 1_000;

    document_set_cookie(
        &document,
        "expires_at",
        &expires_at,
        client_token_set.expires_in,
    )?;

    Ok(())
}

/// If the access token (stored in cookies) is not found or expired
/// the user is redirected to Cognito in order to issue a new token.
///
/// Ultimately, after the user is authenticated, they will be redirected
/// to the aplication with a code, provided by Cognito, in the URL.
#[wasm_bindgen(js_name = "getAuthorizationCodeInteractive")]
pub async fn get_authorization_code_interactive() -> Result<(), JsValue> {
    set_panic_hook();

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(AUTHORIZATION_URL.parse().unwrap())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let token_cache = TokenCache::new(authenticator);

    match token_cache.get_authorization_code_interactive().await {
        Ok(()) => Ok(()),
        Err(error) if error.is::<UserAlreadyLoggedInError>() => Ok(()),
        Err(error) => Err(JsValue::from_str(&error.to_string())),
    }
}

#[wasm_bindgen(js_name = "refreshClientTokenSet")]
pub async fn refresh_client_token_set(refresh_token: String) -> Result<(), JsValue> {
    set_panic_hook();

    let authenticator =
        AwsCognitoClientAuthenticator::from_authorization_url(AUTHORIZATION_URL.parse().unwrap())
            .map_err(|error| JsValue::from_str(&error.to_string()))?;

    let client_token_set = authenticator
        .get_token_set_from(&ClientTokenSetRequest::RefreshToken(refresh_token))
        .await
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    populate_client_token_set_cookies(&client_token_set)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    Ok(())
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
        .get_token_set_from(&ClientTokenSetRequest::Code(code))
        .await;

    let client_token_set = match client_token_set {
        Ok(client_token_set) => client_token_set,
        Err(error) if error.is::<UserAlreadyLoggedInError>() => return Ok(()),
        Err(error) => return Err(JsValue::from_str(&error.to_string())),
    };

    populate_client_token_set_cookies(&client_token_set)
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

/// Gets the refresh token currently stored in cookies
#[wasm_bindgen(js_name = "getRefreshToken")]
pub fn get_refresh_token() -> Option<String> {
    set_panic_hook();

    let cookies = get_document().cookie().unwrap();

    let cookies = parse_cookies(&cookies);

    cookies.get("refresh_token").map(ToString::to_string)
}

/// Gets the "expires at" value currently stored in cookies
#[wasm_bindgen(js_name = "getExpiresAt")]
pub fn get_expires_at() -> Option<u64> {
    set_panic_hook();

    let cookies = get_document().cookie().unwrap();

    let cookies = parse_cookies(&cookies);

    cookies
        .get("expires_at")
        .and_then(|expires_at| expires_at.parse().ok())
}

#[wasm_bindgen]
pub struct TimeoutHandle {
    timeout_id: i32,
    _callback: Closure<dyn FnMut()>,
}

impl Drop for TimeoutHandle {
    fn drop(&mut self) {
        window().unwrap().clear_timeout_with_handle(self.timeout_id);
    }
}

/// Schedule a refresh client token set.
///
/// When the token set is about to expire it'll be automatically refreshed.
#[wasm_bindgen(js_name = "scheduleRefreshClientTokenSet")]
pub fn schedule_refresh_client_token_set() -> Result<TimeoutHandle, JsValue> {
    set_panic_hook();

    let expires_at = match get_expires_at() {
        Some(expires_at) => expires_at,
        None => return Err("Expires at cookie not found".into()),
    };

    #[allow(clippy::cast_precision_loss)]
    let expires_in = expires_at as f64 - Date::now() - 10_000f64;

    let window = window().unwrap();

    let callback = Closure::wrap(Box::new(move || {
        spawn_local(async {
            let refresh_token = match get_refresh_token() {
                Some(refresh_token) => refresh_token,
                None => return,
            };

            refresh_client_token_set(refresh_token).await.unwrap();
        });
    }) as Box<dyn FnMut()>);

    let timeout_id = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        expires_in as i32,
    )?;

    Ok(TimeoutHandle {
        timeout_id,
        _callback: callback,
    })
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
