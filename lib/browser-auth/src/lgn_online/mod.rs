//! Implementation of the types copy pasted from `lgn-online`

use core::fmt::{Display, Formatter};

use alloc::{
    borrow::Cow,
    format,
    string::{String, ToString},
    vec::Vec,
};
use anyhow::anyhow;
use js_sys::Date;
use types::{AwsCognitoClientAuthenticator, ClientTokenSet, TokenCache, UserInfo};
use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Request, RequestInit, RequestMode, Response, UrlSearchParams};

use crate::utils::{get_document, get_url_query_string_value, get_url_query_value, parse_cookies};

pub(crate) mod types;

#[derive(Debug)]
pub(crate) struct UserAlreadyLoggedInError;

// We can't use thiserror since it doesn't support `no_std` yet
// https://github.com/dtolnay/thiserror/pull/64
impl Display for UserAlreadyLoggedInError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "User is already logged in")
    }
}

/// When requesting a new client token set both code and request token can be used.
#[derive(Debug)]
pub enum ClientTokenSetRequest {
    Code(String),
    RefreshToken(String),
}

impl AwsCognitoClientAuthenticator {
    pub fn from_authorization_url(authorization_url: Url) -> anyhow::Result<Self> {
        if authorization_url.path() != "/oauth2/authorize" {
            return Err(anyhow!("URL must be an AWS Cognito authorization URL"));
        }

        let host_parts = authorization_url
            .host()
            .ok_or_else(|| anyhow!("no host in URL".to_string()))?
            .to_string();

        let (domain_name, region) = match host_parts.split('.').collect::<Vec<_>>().as_slice() {
            [domain_name, "auth", region, "amazoncognito", "com"] => {
                ((*domain_name).to_string(), (*region).to_string())
            }
            _ => {
                return Err(anyhow!(
                    "host must respect the `<domain_name>.auth.<region>.amazoncognito.com` format"
                ))
            }
        };

        let client_id = get_url_query_string_value(&authorization_url, "client_id")?.into_owned();

        let scopes = get_url_query_value(&authorization_url, "scope", |scopes| {
            scopes.split('+').map(ToString::to_string).collect()
        })?;

        let identity_provider = get_url_query_string_value(&authorization_url, "identity_provider")
            .ok()
            .map(Cow::into_owned);

        let redirect_uri = get_url_query_string_value(&authorization_url, "redirect_uri")?
            .parse::<Url>()
            .map_err(anyhow::Error::msg)?;

        // If there is no explicit port, assume the default port for the scheme.
        let port = redirect_uri.port().unwrap_or(80);

        Ok(Self {
            authorization_url,
            domain_name,
            region,
            client_id,
            scopes,
            identity_provider,
            port,
        })
    }

    fn get_base_url(&self, path: &str) -> Url {
        Url::parse(&format!(
            "https://{}.auth.{}.amazoncognito.com/{}",
            self.domain_name, self.region, path
        ))
        .unwrap()
    }

    fn get_redirect_uri(&self) -> String {
        format!("http://localhost:{}/", self.port)
    }

    pub fn get_access_token_url(&self) -> String {
        self.get_base_url("oauth2/token").to_string()
    }

    pub fn get_user_info_url(&self) -> String {
        self.get_base_url("oauth2/userInfo").to_string()
    }

    fn get_authorization_url(&self) -> String {
        let mut url = self.get_base_url("oauth2/authorize");

        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.scopes.join("+"))
            .append_pair("redirect_uri", &self.get_redirect_uri());

        if let Some(ref identity_provider) = self.identity_provider {
            url.query_pairs_mut()
                .append_pair("identity_provider", identity_provider);
        }

        url.to_string()
    }

    pub async fn get_authorization_code_interactive(&self) {
        let authorization_url = self.get_authorization_url();

        let window = window().unwrap();

        window.location().set_href(&authorization_url).unwrap();
    }

    pub async fn get_token_set_from(
        &self,
        request: &ClientTokenSetRequest,
    ) -> anyhow::Result<ClientTokenSet> {
        let body = UrlSearchParams::new().unwrap();

        body.append("client_id", &self.client_id);
        body.append("redirect_uri", &self.get_redirect_uri());

        match request {
            ClientTokenSetRequest::Code(code) => {
                body.append("grant_type", "authorization_code");
                body.append("code", code);
            }
            ClientTokenSetRequest::RefreshToken(refresh_token) => {
                body.append("grant_type", "refresh_token");
                body.append("refresh_token", refresh_token);
            }
        };

        let mut request_init = RequestInit::new();

        request_init.method("POST");
        request_init.mode(RequestMode::Cors);
        request_init.body(Some(body.as_ref()));

        let access_token_url = self.get_access_token_url();

        let request = Request::new_with_str_and_init(&access_token_url, &request_init).unwrap();

        request
            .headers()
            .set("Content-Type", "application/x-www-form-urlencoded")
            .unwrap();

        let window = window().unwrap();

        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_error| anyhow!("Get token set request failed"))?;

        if !response.is_instance_of::<Response>() {
            return Err(anyhow!("Response was not a valid Response object"));
        }

        let response: Response = response.dyn_into().unwrap();

        let json = JsFuture::from(response.json().unwrap())
            .await
            .map_err(|_error| anyhow!("Response wasn't a valid json string"))?;

        let client_token_set: ClientTokenSet = json
            .into_serde()
            .map_err(|_error| anyhow!("Response was not a valid ClientTokenSet object"))?;

        Ok(client_token_set)
    }

    pub async fn get_user_info(&self, access_token: &str) -> anyhow::Result<UserInfo> {
        let mut request_init = RequestInit::new();

        request_init.method("GET");
        request_init.mode(RequestMode::Cors);

        let user_info_url = self.get_user_info_url();

        let request = Request::new_with_str_and_init(&user_info_url, &request_init).unwrap();

        request
            .headers()
            .set("Authorization", &format!("Bearer {}", access_token))
            .unwrap();

        let window = window().unwrap();

        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_error| anyhow!("Get token set request failed"))?;

        if !response.is_instance_of::<Response>() {
            return Err(anyhow!("Response was not a valid Response object"));
        }

        let response: Response = response.dyn_into().unwrap();

        let json = JsFuture::from(response.json().unwrap())
            .await
            .map_err(|_error| anyhow!("Response wasn't of format json"))?;

        let user_info: UserInfo = json
            .into_serde()
            .map_err(|_error| anyhow!("Response was not a valid UserInfo object"))?;

        Ok(user_info)
    }
}

impl TokenCache {
    pub fn new(authenticator: AwsCognitoClientAuthenticator) -> Self {
        Self { authenticator }
    }

    pub async fn get_authorization_code_interactive(&self) -> anyhow::Result<()> {
        if Self::client_token_set_is_valid() {
            Err(UserAlreadyLoggedInError).map_err(anyhow::Error::msg)
        } else {
            self.authenticator
                .get_authorization_code_interactive()
                .await;

            Ok(())
        }
    }

    pub async fn get_token_set_from(
        &self,
        request: &ClientTokenSetRequest,
    ) -> anyhow::Result<ClientTokenSet> {
        if Self::client_token_set_is_valid() {
            Err(UserAlreadyLoggedInError).map_err(anyhow::Error::msg)
        } else {
            self.authenticator.get_token_set_from(request).await
        }
    }

    fn client_token_set_is_valid() -> bool {
        let cookies = get_document().cookie().unwrap();

        let cookies = parse_cookies(&cookies);

        match (
            cookies.get("access_token"),
            cookies.get("refresh_token"),
            cookies
                .get("expires_at")
                .and_then(|expires_at| expires_at.parse::<u64>().ok()),
        ) {
            (None, _, _) | (_, None, _) | (_, _, None) => false,
            (Some(_), Some(_), Some(expires_at)) => expires_at > Date::now() as u64,
        }
    }
}
