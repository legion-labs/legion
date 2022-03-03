use super::{
    jwt::{
        signature_validation::{
            AwsCognitoSignatureValidation, BoxedSignatureValidation, NoSignatureValidation,
            RsaSignatureValidation,
        },
        Validation,
    },
    BoxedAuthenticator, Error, OAuthClient, Result, TokenCache,
};
use serde::Deserialize;
use url::Url;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum AuthenticatorConfig {
    #[serde(rename = "oauth")]
    OAuth(OAuthClientConfig),
}

#[derive(Deserialize, Debug)]
pub struct OAuthClientConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub redirect_uri: Option<Url>,
}

impl AuthenticatorConfig {
    pub fn new() -> Result<Self> {
        lgn_config::Config::new()
            .get("authentication.authenticator")
            .ok_or_else(|| {
                Error::Other(anyhow::anyhow!("failed to load authenticator config").into())
            })
    }

    /// Instanciate an `Authenticator` from the configuration.
    pub async fn authenticator(&self) -> anyhow::Result<BoxedAuthenticator> {
        match self {
            AuthenticatorConfig::OAuth(config) => Ok(BoxedAuthenticator(Box::new(
                TokenCache::new_with_application_name(
                    OAuthClient::new_from_config(config).await?,
                    "lgn-online",
                ),
            ))),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SignatureValidationConfig {
    Disabled {},
    Rsa(RsaSignatureValidationConfig),
    AwsCognito(AwsCognitoSignatureValidationConfig),
}

#[derive(Deserialize, Debug)]
pub struct RsaSignatureValidationConfig {
    n: String,
    e: String,
}

#[derive(Deserialize, Debug)]
pub struct AwsCognitoSignatureValidationConfig {
    region: String,
    user_pool_id: String,
}

impl SignatureValidationConfig {
    pub fn new() -> Result<Self> {
        lgn_config::Config::new()
            .get("authentication.signature_validation")
            .ok_or_else(|| {
                Error::Other(anyhow::anyhow!("failed to load signature validation config").into())
            })
    }

    /// Instanciate a `SignatureValidation` from the configuration.
    pub async fn signature_validation(&self) -> anyhow::Result<BoxedSignatureValidation> {
        Ok(BoxedSignatureValidation(match self {
            SignatureValidationConfig::Disabled {} => Box::new(NoSignatureValidation {}),
            SignatureValidationConfig::Rsa(config) => Box::new(
                RsaSignatureValidation::new_from_components(&config.n, &config.e)?,
            ),
            SignatureValidationConfig::AwsCognito(config) => Box::new(
                AwsCognitoSignatureValidation::new(&config.region, &config.user_pool_id).await?,
            ),
        }))
    }

    /// Instanciate a `Validation` from the configuration.
    pub async fn validation(&self) -> anyhow::Result<Validation<BoxedSignatureValidation>> {
        match self.signature_validation().await {
            Ok(signature_validation) => Ok(Validation::new(signature_validation)),
            Err(err) => Err(err),
        }
    }
}
