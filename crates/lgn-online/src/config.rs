use http::Uri;
use serde::Deserialize;

use crate::{
    authentication::{
        jwt::{
            signature_validation::{
                AwsCognitoSignatureValidation, BoxedSignatureValidation, NoSignatureValidation,
                RsaSignatureValidation,
            },
            Validation,
        },
        BoxedAuthenticator, OAuthClient, TokenCache,
    },
    grpc::{AuthenticatedClient, GrpcClient, GrpcWebClient},
    Result,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The base URL for api requests.
    #[serde(with = "http_serde::uri")]
    pub api_base_url: Uri,

    /// The base URL for web-api requests.
    #[serde(with = "http_serde::uri")]
    pub web_api_base_url: Uri,

    /// The authentication settings.
    pub authentication: Option<AuthenticationConfig>,

    /// The signature validation settings.
    #[serde(default)]
    pub signature_validation: SignatureValidationConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_base_url: "http://localhost:8000".parse().unwrap(),
            web_api_base_url: "http://localhost:8000".parse().unwrap(),
            authentication: None,
            signature_validation: SignatureValidationConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(lgn_config::get("online")?.unwrap_or_default())
    }

    /// Instantiate a API client.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication settings are invalid.
    pub async fn instantiate_api_client(
        &self,
        scopes: &[String],
    ) -> Result<AuthenticatedClient<GrpcClient, BoxedAuthenticator>> {
        self.instantiate_api_client_with_url(None, scopes).await
    }

    /// Instantiate a API client.
    ///
    /// If an URL is specified, is it used instead of the default.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication settings are invalid.
    pub async fn instantiate_api_client_with_url(
        &self,
        url: Option<&Uri>,
        scopes: &[String],
    ) -> Result<AuthenticatedClient<GrpcClient, BoxedAuthenticator>> {
        let client = GrpcClient::new(url.unwrap_or(&self.api_base_url).clone());

        let authenticator = match &self.authentication {
            Some(config) => Some(config.instantiate_authenticator().await?),
            None => None,
        };

        let client = AuthenticatedClient::new(client, authenticator, scopes);

        Ok(client)
    }

    /// Instantiate a Web API client.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication settings are invalid.
    pub async fn instantiate_web_api_client(
        &self,
        scopes: &[String],
    ) -> Result<AuthenticatedClient<GrpcWebClient, BoxedAuthenticator>> {
        self.instantiate_web_api_client_with_url(None, scopes).await
    }

    /// Instantiate a Web API client.
    ///
    /// If an URL is specified, is it used instead of the default.
    ///
    /// # Errors
    ///
    /// Returns an error if the authentication settings are invalid.
    pub async fn instantiate_web_api_client_with_url(
        &self,
        url: Option<&Uri>,
        scopes: &[String],
    ) -> Result<AuthenticatedClient<GrpcWebClient, BoxedAuthenticator>> {
        let client = GrpcWebClient::new(url.unwrap_or(&self.api_base_url).clone());

        let authenticator = match &self.authentication {
            Some(config) => Some(config.instantiate_authenticator().await?),
            None => None,
        };

        let client = AuthenticatedClient::new(client, authenticator, scopes);

        Ok(client)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthenticationConfig {
    #[serde(rename = "oauth")]
    OAuth(OAuthClientConfig),
}

impl AuthenticationConfig {
    /// Instantiate the authenticator from the configuration.
    ///
    /// # Errors
    ///
    /// If the configuration is invalid, an error is returned.
    pub async fn instantiate_authenticator(&self) -> Result<BoxedAuthenticator> {
        match self {
            Self::OAuth(config) => config.instantiate_authenticator().await,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthClientConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    #[serde(default, with = "http_serde::uri")]
    pub redirect_uri: Uri,

    #[serde(default = "OAuthClientConfig::default_token_cache_application_name")]
    pub token_cache_application_name: String,
}

impl OAuthClientConfig {
    fn default_token_cache_application_name() -> String {
        "lgn-online".to_string()
    }

    /// Instantiate the authenticator from the configuration.
    ///
    /// # Errors
    ///
    /// If the configuration is invalid, an error is returned.
    pub async fn instantiate_authenticator(&self) -> Result<BoxedAuthenticator> {
        Ok(BoxedAuthenticator(Box::new(
            TokenCache::new_with_application_name(
                OAuthClient::new_from_config(self).await?,
                &self.token_cache_application_name,
            ),
        )))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SignatureValidationConfig {
    Disabled {},
    Rsa(RsaSignatureValidationConfig),
    AwsCognito(AwsCognitoSignatureValidationConfig),
}

impl Default for SignatureValidationConfig {
    fn default() -> Self {
        Self::Disabled {}
    }
}

impl SignatureValidationConfig {
    /// Instanciate a `SignatureValidation` from the configuration.
    pub async fn instantiate_signature_validation(&self) -> Result<BoxedSignatureValidation> {
        Ok(BoxedSignatureValidation(match self {
            Self::Disabled {} => Box::new(NoSignatureValidation {}),
            Self::Rsa(config) => Box::new(config.instantiate_signature_validation()?),
            Self::AwsCognito(config) => Box::new(config.instantiate_signature_validation().await?),
        }))
    }

    /// Instanciate a `Validation` from the configuration.
    pub async fn instantiate_validation(&self) -> Result<Validation<BoxedSignatureValidation>> {
        Ok(Validation::new(
            self.instantiate_signature_validation().await?,
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RsaSignatureValidationConfig {
    pub n: String,
    pub e: String,
}

impl RsaSignatureValidationConfig {
    /// Instanciate a `SignatureValidation` from the configuration.
    pub fn instantiate_signature_validation(&self) -> Result<RsaSignatureValidation> {
        Ok(RsaSignatureValidation::new_from_components(
            &self.n, &self.e,
        )?)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct AwsCognitoSignatureValidationConfig {
    pub region: String,
    pub user_pool_id: String,
}

impl AwsCognitoSignatureValidationConfig {
    /// Instanciate a `SignatureValidation` from the configuration.
    pub async fn instantiate_signature_validation(&self) -> Result<AwsCognitoSignatureValidation> {
        Ok(AwsCognitoSignatureValidation::new(&self.region, &self.user_pool_id).await?)
    }
}
