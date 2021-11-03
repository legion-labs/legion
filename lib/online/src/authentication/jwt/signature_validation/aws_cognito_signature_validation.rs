use anyhow::Context;
use log::debug;
use serde::{Deserialize, Serialize};

pub struct AwsCognitoSignatureValidation;

impl AwsCognitoSignatureValidation {
    pub async fn new(region: &str, aws_cognito_user_pool_id: &str) -> anyhow::Result<Self> {
        let url = format!(
            "https://cognito-idp.{}.amazonaws.com/{}/.well-known/jwks.json",
            region, aws_cognito_user_pool_id,
        );

        debug!("Loading JWKS from {}...", url);

        let resp = reqwest::get(url).await.context("failed to fetch JWKS")?;
        let data = resp.text().await.context("failed to read response body")?;

        //serde_json::from_str(&data)
        //    .map_err::<anyhow::Error, _>(Into::into)
        //    .context("Failed to deserialize JWKS payload")
        Ok(Self)
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
