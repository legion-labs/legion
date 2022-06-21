use std::{collections::HashMap, str::FromStr};

use gloo_storage::{LocalStorage, Storage};
use log::debug;
use reqwest::Method;
use serde::Deserialize;
use url::Url;

use crate::errors::{Error, Result};

use super::{dom::get_window, pkce};

static AUTHORIZE_URL: &str =
    "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/authorize";

static TOKEN_URL: &str =
    "https://legionlabs-playground.auth.ca-central-1.amazoncognito.com/oauth2/token";

static REDIRECT_URI: &str = "http://localhost:3000";

static AUTHORIZE_VERIFIER_KEY: &str = "authorize-verifier";

static TARGET_URL_KEY: &str = "target-url";

pub fn get_redirect_uri() -> Result<Url> {
    REDIRECT_URI
        .parse()
        .map_err(|_| Error::Js("couldn't parse redirect url".into()))
}

fn authorize_url(
    response_type: &str,
    scopes: Vec<String>,
    extra_params: Option<HashMap<String, String>>,
    redirect_uri: Option<Url>,
    pkce_challenge: Option<String>,
) -> Result<Url> {
    let mut authorization_url = Url::from_str(AUTHORIZE_URL)
        .map_err(|_| Error::Js("couldn't parse authorization url".into()))?;

    {
        let mut query = authorization_url.query_pairs_mut();

        if let Some(pkce_challenge) = pkce_challenge {
            query.append_pair("code_challenge_method", "S256");
            query.append_pair("code_challenge", &pkce_challenge);
        }

        query.append_pair("client_id", "5m58nrjfv6kr144prif9jk62di");
        query.append_pair("response_type", response_type);
        query.append_pair("scope", &scopes.join(" "));

        if let Some(redirect_uri) = redirect_uri {
            query.append_pair("redirect_uri", redirect_uri.as_str());
        }

        if let Some(extra_params) = extra_params {
            for (key, value) in extra_params {
                query.append_pair(&key, &value);
            }
        }
    }

    Ok(authorization_url)
}

pub fn get_authorization_url() -> Result<Url> {
    let window = get_window()?;

    let verifier = pkce::code_verifier(128);
    let challenge = pkce::code_challenge(&verifier);

    let scopes = vec![
        "aws.cognito.signin.user.admin".to_string(),
        "email".to_string(),
        "https://legionlabs.com/editor/allocate".to_string(),
        "openid".to_string(),
        "profile".to_string(),
    ];

    let extra_params =
        HashMap::from_iter(vec![("identity_provider".to_string(), "Azure".to_string())]);

    let redirect_uri = get_redirect_uri()?;

    let redirect_uri_origin = redirect_uri.origin().unicode_serialization();

    let authorize_url = authorize_url(
        "code",
        scopes,
        Some(extra_params),
        Some(redirect_uri),
        Some(challenge),
    )?;

    LocalStorage::set(AUTHORIZE_VERIFIER_KEY, verifier)
        .map_err(|_| Error::Js("couldn't set authorize verifier value in local storage".into()))?;

    let location_origin = window
        .location()
        .origin()
        .map_err(|_| Error::Js("origin not found in location".into()))?;

    if location_origin == redirect_uri_origin {
        LocalStorage::set(
            TARGET_URL_KEY,
            window
                .location()
                .href()
                .map_err(|_| Error::Js("href not found in location".into()))?,
        )
        .map_err(|_| Error::Js("couldn't set target-url verifier value in local storage".into()))?;
    }

    Ok(authorize_url)
}

pub fn get_code_in_url(url: &Url) -> Option<String> {
    let mut code = None;

    for (key, value) in url.query_pairs() {
        if key.as_ref() == "code" {
            code = Some(value.into());
            break;
        }
    }

    code
}

#[derive(Debug, Deserialize)]
pub struct TokenSet {
    // id_token: String,
    pub access_token: String,
    // refresh_token: String,
    pub expires_in: u64,
    // token_type: String,
}

pub async fn get_token_set(code: impl AsRef<str>) -> Result<TokenSet> {
    let redirect_uri = get_redirect_uri()?;

    let verifier: Vec<u8> = match LocalStorage::get(AUTHORIZE_VERIFIER_KEY) {
        Ok(verifier) => verifier,
        Err(_) => return Err(Error::Auth("authorize verifier not found".into())),
    };

    LocalStorage::delete(AUTHORIZE_VERIFIER_KEY);

    let verifier =
        String::from_utf8(verifier).map_err(|_| Error::Js("verifier is not utf-8".into()))?;

    debug!("Verifier {}", verifier);

    let token_url =
        Url::from_str(TOKEN_URL).map_err(|_| Error::Js("couldn't parse token url".into()))?;

    let mut body = HashMap::new();

    body.insert("grant_type", "authorization_code");
    body.insert("client_id", "5m58nrjfv6kr144prif9jk62di");
    body.insert("redirect_uri", redirect_uri.as_str());
    body.insert("code", code.as_ref());
    body.insert("code_verifier", &verifier);

    let client = reqwest::Client::new();

    let req = client
        .request(Method::POST, token_url.as_str())
        .form(&body)
        .build()
        .map_err(|_| Error::Js("failed to build request".into()))?;

    let res = match client.execute(req).await {
        Ok(res) => res,
        Err(_) => return Err(Error::Auth("token set request failed".into())),
    };

    let token_set = res
        .json()
        .await
        .map_err(|_| Error::Js("failed to parse token set payload".into()))?;

    Ok(token_set)
}
