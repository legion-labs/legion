use std::path::PathBuf;

use serde::Deserialize;

use crate::{
    AwsDynamoDbProvider, AwsS3Provider, AwsS3Url, CachingProvider, ContentProvider, GrpcProvider,
    LocalProvider, LruProvider, MemoryProvider, RedisProvider, Result, SmallContentProvider,
};

/// The configuration of the content-store.
#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub provider: ProviderConfig,
    pub caching_provider: Option<ProviderConfig>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ProviderConfig {
    Memory {},
    Lru(LruProviderConfig),
    Local(LocalProviderConfig),
    Redis(RedisProviderConfig),
    Grpc(GrpcProviderConfig),
    AwsS3(AwsS3ProviderConfig),
    AwsDynamoDb(AwsDynamoDbProviderConfig),
}

#[derive(Deserialize, Debug)]
pub struct LocalProviderConfig {
    pub path: Option<PathBuf>,
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

#[derive(Deserialize, Debug)]
pub struct LruProviderConfig {
    #[serde(default)]
    pub size: usize,
}

#[derive(Deserialize, Debug)]
pub struct AwsS3ProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,
}

#[derive(Deserialize, Debug)]
pub struct AwsDynamoDbProviderConfig {
    pub table_name: String,
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

    /// Instanciate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instanciated.
    pub async fn instanciate_provider(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        let provider = self.provider.instanciate().await?;

        if let Some(caching_provider) = &self.caching_provider {
            let caching_provider = caching_provider.instanciate().await?;
            Ok(Box::new(CachingProvider::new(provider, caching_provider)))
        } else {
            Ok(provider)
        }
    }
}

impl ProviderConfig {
    /// Instanciate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instanciated.
    pub async fn instanciate(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        Ok(match self {
            Self::Memory {} => Box::new(SmallContentProvider::new(MemoryProvider::new())),
            Self::Lru(config) => Box::new(SmallContentProvider::new(LruProvider::new(config.size))),
            Self::Local(config) => {
                let path = match &config.path {
                    Some(path) => path.clone(),
                    None => std::env::temp_dir().join("lgn-content-store"),
                };

                Box::new(SmallContentProvider::new(LocalProvider::new(path).await?))
            }
            Self::Redis(config) => Box::new(SmallContentProvider::new(
                RedisProvider::new(config.url.clone(), config.key_prefix.clone()).await?,
            )),
            Self::AwsS3(config) => Box::new(SmallContentProvider::new(
                AwsS3Provider::new(AwsS3Url {
                    bucket_name: config.bucket_name.clone(),
                    root: config.root.clone(),
                })
                .await,
            )),
            Self::AwsDynamoDb(config) => Box::new(SmallContentProvider::new(
                AwsDynamoDbProvider::new(config.table_name.clone()).await,
            )),
            Self::Grpc(config) => {
                let uri = config
                    .url
                    .parse()
                    .map_err(|err| anyhow::anyhow!("failed to parse gRPC url: {}", err))?;
                let client = lgn_online::grpc::GrpcClient::new(uri);
                let authenticator_config = lgn_online::authentication::AuthenticatorConfig::new()
                    .map_err(|err| {
                    anyhow::anyhow!("failed to create authenticator config: {}", err)
                })?;
                let authenticator = authenticator_config.authenticator().await.map_err(|err| {
                    anyhow::anyhow!("failed to instanciate an authenticator: {}", err)
                })?;

                let client = lgn_online::grpc::AuthenticatedClient::new(client, authenticator, &[]);

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
