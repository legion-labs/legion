use http::Uri;
use lgn_config::RichPathBuf;
use serde::{Deserialize, Deserializer};

use crate::{GrpcRepositoryIndex, LocalRepositoryIndex, RepositoryIndex, Result};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    repository_index: RepositoryIndexConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RepositoryIndexConfig {
    Grpc(GrpcConfig),
    Local(LocalConfig),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct GrpcConfig {
    /// The Web API URL to use for requests.
    ///
    /// If not specified, the global `online.web_api_base_url` will be used.
    #[serde(default, deserialize_with = "GrpcConfig::deserialize_web_api_url")]
    pub web_api_url: Option<Uri>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LocalConfig {
    pub path: RichPathBuf,
}

impl Config {
    /// Load a configuration.
    ///
    /// # Errors
    ///
    /// If the configuration is incorrect, an error will be returned.
    pub fn load() -> Result<Self> {
        Ok(lgn_config::get("source_control")?.unwrap_or_default())
    }

    /// Instantiate a repository index.
    pub async fn instantiate_repository_index(&self) -> Result<Box<dyn RepositoryIndex>> {
        self.repository_index.instantiate().await
    }

    /// Load a configuration and instantiate a repository index.
    ///
    /// # Errors
    ///
    /// If the configuration is incorrect, an error will be returned.
    pub async fn load_and_instantiate_repository_index() -> Result<Box<dyn RepositoryIndex>> {
        Self::load()?.instantiate_repository_index().await
    }
}

impl Default for RepositoryIndexConfig {
    fn default() -> Self {
        Self::Grpc(GrpcConfig::default())
    }
}

impl RepositoryIndexConfig {
    /// Instantiate an index.
    ///
    /// If no repository name is specified, the one from the configuration will
    /// be used.
    ///
    /// If no default repository name is specified, "default" will be used.
    pub async fn instantiate(&self) -> Result<Box<dyn RepositoryIndex>> {
        match self {
            Self::Grpc(grpc_config) => {
                let online_config = lgn_online::Config::load()?;
                let client = online_config
                    .instantiate_web_api_client_with_url(grpc_config.web_api_url.as_ref(), &[])
                    .await?;
                let index = GrpcRepositoryIndex::new(client);
                Ok(Box::new(index))
            }
            Self::Local(local_config) => {
                let index = LocalRepositoryIndex::new(&local_config.path.as_ref()).await?;
                Ok(Box::new(index))
            }
        }
    }
}

impl GrpcConfig {
    fn deserialize_web_api_url<'de, D>(deserializer: D) -> Result<Option<Uri>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Deserialize::deserialize(deserializer)?;

        match s {
            Some(s) => s.parse().map_err(serde::de::Error::custom).map(Some),
            None => Ok(None),
        }
    }
}
