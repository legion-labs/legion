use std::time::Duration;

use openidconnect::{core::CoreTokenResponse, OAuth2TokenResponse, TokenResponse};
use serde::{Deserialize, Serialize};

use super::Error;

/// A set of tokens as given to clients.
#[derive(Serialize, Deserialize, Debug)]
pub struct ClientTokenSet {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<Duration>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

impl TryFrom<CoreTokenResponse> for ClientTokenSet {
    type Error = Error;

    fn try_from(core_token_response: CoreTokenResponse) -> Result<Self, Self::Error> {
        Ok(Self {
            access_token: core_token_response.access_token().secret().clone(),
            token_type: serde_json::to_value(core_token_response.token_type())
                .map_err(Error::internal)?
                .as_str()
                .ok_or_else(|| Error::Internal("Token type is not a value of type string".into()))?
                .to_string(),
            expires_in: core_token_response.expires_in(),
            refresh_token: core_token_response
                .refresh_token()
                .map(|refresh_token| refresh_token.secret().clone()),
            id_token: core_token_response.id_token().map(ToString::to_string),
        })
    }
}
