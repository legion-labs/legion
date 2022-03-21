use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    AwsDynamoDbProvider, AwsS3Provider, AwsS3Url, CachingProvider, ContentProvider, GrpcProvider,
    LocalProvider, LruProvider, MemoryProvider, RedisProvider, Result, SmallContentProvider,
};

/// The configuration of the content-store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: ProviderConfig,
    pub caching_providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProviderConfig {
    pub path: Option<PathBuf>,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisProviderConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,

    #[serde(default)]
    pub key_prefix: String,
}

fn default_grpc_url() -> String {
    "://localhost:6379".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcProviderConfig {
    #[serde(default = "default_grpc_url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LruProviderConfig {
    #[serde(default)]
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsS3ProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsDynamoDbProviderConfig {
    pub table_name: String,
}

/// An environment variable that contains the default content-store section to
/// use.
pub const ENV_LGN_CONTENT_STORE_SECTION: &str = "LGN_CONTENT_STORE_SECTION";

impl Config {
    pub fn content_store_section() -> Option<String> {
        std::env::var(ENV_LGN_CONTENT_STORE_SECTION).ok()
    }

    /// Returns a new instance from the `legion.toml`, with the specified section.
    ///
    /// If the section is not found, the default section is used.
    pub fn from_legion_toml(section: Option<&str>) -> Self {
        let settings = lgn_config::Config::new();

        match section {
            None | Some("") => {
                if let Some(config) = settings.get::<Self>("content_store") {
                    config
                } else {
                    Self {
                        provider: ProviderConfig::default(),
                        caching_providers: vec![],
                    }
                }
            }
            Some(section) => {
                if let Some(config) = settings.get::<Self>(&format!("content_store.{}", section)) {
                    config
                } else {
                    Self::from_legion_toml(None)
                }
            }
        }
    }

    /// Returns a configuration of a local, disk-based content store operating at specified path.
    pub fn local(path: impl AsRef<Path>) -> Self {
        Self {
            provider: ProviderConfig::Local(LocalProviderConfig {
                path: Some(path.as_ref().to_owned()),
            }),
            caching_providers: vec![],
        }
    }

    /// Instanciate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instanciated.
    pub async fn instanciate_provider(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        let mut provider = self.provider.instanciate().await?;

        for caching_provider in &self.caching_providers {
            let caching_provider = caching_provider.instanciate().await?;
            provider = Box::new(CachingProvider::new(provider, caching_provider));
        }

        Ok(provider)
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
