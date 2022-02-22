use std::path::PathBuf;

use serde::Deserialize;

use crate::{
    ContentProvider, GrpcProvider, LocalProvider, MemoryProvider, RedisProvider, Result,
    SmallContentProvider,
};

/// The configuration of the content-store.
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub provider: ProviderConfig,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ProviderConfig {
    Memory {},
    Local(LocalProviderConfig),
    Redis(RedisProviderConfig),
    Grpc(GrpcProviderConfig),
}

#[derive(Deserialize, Debug)]
pub struct LocalProviderConfig {
    pub path: PathBuf,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

#[derive(Deserialize, Debug)]
pub struct RedisProviderConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,

    #[serde(default)]
    pub key_prefix: String,
}

fn default_grpc_url() -> String {
    "://localhost:6379".to_string()
}

#[derive(Deserialize, Debug)]
pub struct GrpcProviderConfig {
    #[serde(default = "default_grpc_url")]
    pub url: String,
}

impl Config {
    /// Create a new configuration by reading the available configuration files.
    pub fn new() -> Self {
        let settings = lgn_config::Config::new();

        if let Some(config) = settings.get::<Self>("content_store") {
            config
        } else {
            Self::default()
        }
    }
}

impl ProviderConfig {
    /// Instanciate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instanciated.
    pub async fn new_provider(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        Ok(match self {
            Self::Memory {} => Box::new(SmallContentProvider::new(MemoryProvider::new())),
            Self::Local(ref config) => Box::new(SmallContentProvider::new(
                LocalProvider::new(config.path.clone()).await?,
            )),
            Self::Redis(ref config) => Box::new(SmallContentProvider::new(
                RedisProvider::new(config.url.clone(), config.key_prefix.clone()).await?,
            )),
            Self::Grpc(ref config) => {
                let uri = config
                    .url
                    .parse()
                    .map_err(|err| anyhow::anyhow!("failed to parse gRPC url: {}", err))?;
                let client = lgn_online::grpc::GrpcClient::new(uri);

                Box::new(SmallContentProvider::new(GrpcProvider::new(client).await))
            }
        })
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::Memory {}
    }
}
