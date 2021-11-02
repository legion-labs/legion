use std::collections::HashMap;

use anyhow::{bail, Context};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::UserInfo;

pub struct Validator {
    keys: HashMap<String, KeyInfo>,
}

impl Validator {
    pub async fn new(region: &str, aws_cognito_user_pool_id: &str) -> anyhow::Result<Self> {
        info!(
            "Loading JWKS for region `{}` and AWS Cognito User pool `{}`...",
            region, aws_cognito_user_pool_id
        );

        let keys = Jwks::fetch(region, aws_cognito_user_pool_id)
            .await?
            .keys
            .iter()
            .filter_map(|jkw| match jkw.try_into() {
                Ok(key_info) => {
                    let key_info: KeyInfo = key_info;

                    info!("Loaded JWK for key id `{}`", key_info.kid);

                    Some((key_info.kid.clone(), key_info))
                }
                Err(err) => {
                    warn!("Ignoring key id `{}`: {}", jkw.kid, err);

                    None
                }
            })
            .collect();

        Ok(Self { keys })
    }

    pub async fn validate(&self, kid: &str, token: &str) -> anyhow::Result<UserInfo> {
        let key = self.keys.get(kid).context("JWK not found")?;

        let validation = jsonwebtoken::Validation {
            validate_exp: true,
            leeway: 60, // Seconds.
            algorithms: vec![key.algorithm],
            ..jsonwebtoken::Validation::default()
        };

        jsonwebtoken::decode(token, &key.key, &validation)
            .context("Failed to validate token")
            .map(|data| data.claims)
    }
}

struct KeyInfo {
    kid: String,
    key: jsonwebtoken::DecodingKey<'static>,
    algorithm: jsonwebtoken::Algorithm,
}

impl TryFrom<&Jwk> for KeyInfo {
    type Error = anyhow::Error;

    fn try_from(value: &Jwk) -> Result<Self, Self::Error> {
        match value.kty.as_ref() {
            "RSA" => Ok(Self {
                kid: value.kid.clone(),
                key: jsonwebtoken::DecodingKey::from_rsa_components(&value.n, &value.e)
                    .into_static(),
                algorithm: jsonwebtoken::Algorithm::RS256,
            }),
            _ => bail!("Unsupported key type `{}`", value.kty),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Deserialize, Serialize, Debug)]
struct Jwk {
    kid: String,
    alg: String,
    kty: String,
    e: String,
    n: String,
    #[serde(rename = "use")]
    use_: String,
}

impl Jwks {
    async fn fetch(region: &str, aws_cognito_user_pool_id: &str) -> anyhow::Result<Self> {
        let url = format!(
            "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
            region, aws_cognito_user_pool_id,
        );

        debug!("Loading JWKS from {}", url);

        let resp = reqwest::get(url).await.context("Failed to fetch JWKS")?;
        let data = resp.text().await.context("Failed to read response body")?;

        serde_json::from_str(&data)
            .map_err::<anyhow::Error, _>(Into::into)
            .context("Failed to deserialize JWKS payload")
    }
}
