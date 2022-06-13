use http::Uri;
use lgn_online::client::AuthenticatedClient;
use serde::Deserialize;

use crate::{client::Client, Result};

#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default, with = "option_uri")]
    pub url: Option<Uri>,
}

mod option_uri {
    use http::Uri;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(de: D) -> Result<Option<Uri>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: Option<serde_json::Value> = Deserialize::deserialize(de)?;

        Ok(match v {
            Some(v) => Some(
                http_serde::uri::deserialize(v)
                    .map_err(|e| serde::de::Error::custom(e.to_string()))?,
            ),
            None => None,
        })
    }
}

impl Config {
    /// Returns a new instance from the `legion.toml`.
    ///
    /// # Errors
    ///
    /// If the configuration section is invalid, `Error::Configuration` is
    /// returned.
    pub fn load() -> Result<Self> {
        Ok(lgn_config::get("governance")?.unwrap_or_default())
    }

    /// Instantiate the client for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client cannot be instantiated.
    pub async fn instantiate_client(&self) -> Result<Client<AuthenticatedClient>> {
        let online_config = lgn_online::Config::load()?;
        let client = online_config.instantiate_client(&[]).await?;

        let uri = self.url.as_ref().unwrap_or(&online_config.base_url).clone();
        Ok(Client::new(client, uri))
    }
}
