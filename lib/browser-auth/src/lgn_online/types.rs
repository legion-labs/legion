//! The content of this file is mostly copied from the legion-online crate
//! We could consider abstracting some of the types and traits to reuse them in this crate

use alloc::{string::String, vec::Vec};
use serde::{de, Deserialize, Deserializer, Serialize};
use url::Url;
use wasm_bindgen::prelude::*;

fn deserialize_string_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: serde_json::Value = Deserialize::deserialize(deserializer)?;

    if value.is_null() {
        return Ok(None);
    }

    if let Some(b) = value.as_bool() {
        return Ok(Some(b));
    }

    match value.as_str() {
        Some(s) => Ok(Some(s.parse().map_err(de::Error::custom)?)),
        None => Err(de::Error::custom("expected bool or string")),
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    // Azure-specific fields.
    //
    // This is a merely a convention, but we need one.
    //
    // These fields contains the Azure-specific information about the user, which allow us to query
    // the Azure API for extended user information (like the user's photo).
    #[serde(rename = "custom:azure_oid")]
    pub azure_oid: Option<String>,
    #[serde(rename = "custom:azure_tid")]
    pub azure_tid: Option<String>,
}

// This static string will be appended as is to the generated typescript file
// its purpose is to keep the code as safe as possible and centralize the types in this file.
#[wasm_bindgen(typescript_custom_section)]
const TS_USER_INFO_TYPE: &'static str = r#"
export type UserInfo = Readonly<{
    sub: string,
    name: string | null,
    given_name: string | null,
    family_name: string | null,
    middle_name: string | null,
    nickname: string | null,
    preferred_username: string | null,
    profile: string | null,
    picture: string | null,
    website: string | null,
    email: string | null,
    email_verified: boolean,
    gender: string | null,
    birthdate: string | null,
    zoneinfo: string | null,
    locale: string | null,
    phone_number: string | null,
    phone_number_verified: boolean,
    updated_at: string | null,

    // Azure-specific fields.
    //
    // This is a merely a convention, but we need one.
    //
    // These fields contains the Azure-specific information about the user, which allow us to query
    // the Azure API for extended user information (like the user's photo).
    "custom:azure_oid": string | null,
    "custom:azure_tid": string | null,
}>;
"#;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
}

#[derive(Debug)]
pub struct AwsCognitoClientAuthenticator {
    pub authorization_url: Url,
    pub domain_name: String,
    pub region: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub identity_provider: Option<String>,
    pub port: u16,
}

#[derive(Debug)]
pub struct TokenCache {
    pub authenticator: AwsCognitoClientAuthenticator,
}
