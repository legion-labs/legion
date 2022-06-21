use http::Uri;
use lgn_config::RichPathBuf;
use serde::{Deserialize, Deserializer};

use crate::{
    ApiRepositoryIndex, LocalRepositoryIndex, RepositoryIndex, Result, SqlRepositoryIndex,
};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    repository_index: RepositoryIndexConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RepositoryIndexConfig {
    Api(ApiConfig),
    Sql(SqlConfig),
    Local(LocalConfig),
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApiConfig {
    /// The API URL to use for requests.
    ///
    /// If not specified, the global `online.api_base_url` will be used.
    #[serde(default, deserialize_with = "ApiConfig::deserialize_web_api_url")]
    pub base_url: Option<Uri>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct SqlConfig {
    pub connection_string: String,
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
        Self::Api(ApiConfig::default())
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
            Self::Api(api_config) => api_config.instantiate().await,
            Self::Sql(sql_config) => {
                let index = sql_config.instantiate().await?;
                Ok(Box::new(index))
            }
            Self::Local(local_config) => {
                let index = local_config.instantiate().await?;
                Ok(Box::new(index))
            }
        }
    }
}

impl ApiConfig {
    pub async fn instantiate(&self) -> Result<Box<dyn RepositoryIndex>> {
        let online_config = lgn_online::Config::load()?;
        let base_url = self
            .base_url
            .as_ref()
            .unwrap_or(&online_config.api_base_url)
            .clone();
        let client = online_config.instantiate_client(&[]).await?;

        Ok(Box::new(ApiRepositoryIndex::new(client, base_url)))
    }
}

impl SqlConfig {
    pub async fn instantiate(&self) -> Result<SqlRepositoryIndex> {
        SqlRepositoryIndex::new(self.connection_string.clone()).await
    }
}

impl LocalConfig {
    pub async fn instantiate(&self) -> Result<LocalRepositoryIndex> {
        LocalRepositoryIndex::new(&self.path.as_ref()).await
    }
}

impl ApiConfig {
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
