use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

/// Contains user information.
///
/// Standard OIDC claims, as specified in the [`OpenID`
/// RFC](https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims).
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub middle_name: Option<String>,
    pub nickname: Option<String>,
    pub preferred_username: Option<String>,
    pub profile: Option<String>,
    pub picture: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_bool")]
    pub email_verified: Option<bool>,
    pub gender: Option<String>,
    pub birthdate: Option<String>,
    pub zoneinfo: Option<String>,
    pub locale: Option<String>,
    pub phone_number: Option<String>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_string_bool")]
    pub phone_number_verified: Option<bool>,
    pub updated_at: Option<String>,
}

fn deserialize_string_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<String>::deserialize(deserializer)? {
        Some(s) => bool::from_str(&s).map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}
