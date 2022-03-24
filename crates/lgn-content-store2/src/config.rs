use crate::{
    AwsDynamoDbProvider, AwsS3Provider, AwsS3Url, CachingProvider, ContentProvider, GrpcProvider,
    LocalProvider, LruProvider, MemoryProvider, RedisProvider, Result, SmallContentProvider,
};
use lgn_config::RelativePathBuf;
use serde::{Deserialize, Serialize};

/// The configuration of the content-store.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub provider: ProviderConfig,

    #[serde(default)]
    pub caching_providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalProviderConfig {
    #[serde(serialize_with = "RelativePathBuf::serialize_relative")]
    pub path: RelativePathBuf,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RedisProviderConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,

    #[serde(default)]
    pub key_prefix: String,
}

fn default_grpc_url() -> String {
    "://localhost:6379".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrpcProviderConfig {
    #[serde(default = "default_grpc_url")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LruProviderConfig {
    #[serde(default)]
    pub size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AwsS3ProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        match section {
            None | Some("") => lgn_config::get_or_default("content_store")
                .expect("failed to load content_store config"),
            Some(section) => lgn_config::get_or_else(&format!("content_store.{}", section), || {
                Self::from_legion_toml(None)
            })
            .unwrap(),
        }
    }

    /// Instantiate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instantiated.
    pub async fn instantiate_provider(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        let mut provider = self.provider.instantiate().await?;

        for caching_provider in &self.caching_providers {
            let caching_provider = caching_provider.instantiate().await?;
            provider = Box::new(CachingProvider::new(provider, caching_provider));
        }

        Ok(provider)
    }
}

impl ProviderConfig {
    /// Instantiate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instanciated.
    pub async fn instantiate(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        Ok(match self {
            Self::Memory {} => Box::new(SmallContentProvider::new(MemoryProvider::new())),
            Self::Lru(config) => Box::new(SmallContentProvider::new(LruProvider::new(config.size))),
            Self::Local(config) => Box::new(SmallContentProvider::new(
                LocalProvider::new(config.path.relative()).await?,
            )),
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
                    anyhow::anyhow!("failed to instantiate an authenticator: {}", err)
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_parse_config_without_caching() {
        let config = lgn_config::Config::from_toml(
            r#"
            [content_store.provider.local]
            path = "./test"
            "#,
        );

        let config: Config = config
            .get("content_store")
            .expect("failed to read configuration")
            .expect("option was none");

        assert_eq!(
            config,
            Config {
                provider: ProviderConfig::Local(LocalProviderConfig {
                    path: Path::new("./test").into(),
                }),
                caching_providers: vec![],
            }
        );
    }

    #[test]
    fn test_parse_config_with_caching() {
        let config = lgn_config::Config::from_toml(
            r#"
            [content_store.provider.local]
            path = "./test"

            [[content_store.caching_providers]]
            [content_store.caching_providers.lru]
            size = 100

            [[content_store.caching_providers]]
            [content_store.caching_providers.memory]
            "#,
        );

        let config: Config = config
            .get("content_store")
            .expect("failed to read configuration")
            .expect("option was none");

        assert_eq!(
            config,
            Config {
                provider: ProviderConfig::Local(LocalProviderConfig {
                    path: Path::new("./test").into(),
                }),
                caching_providers: vec![
                    ProviderConfig::Lru(LruProviderConfig { size: 100 }),
                    ProviderConfig::Memory {}
                ],
            }
        );
    }

    #[test]
    fn test_parse_provider_config() {
        let config = lgn_config::Config::from_toml(
            r#"
            [provider1.local]
            path = "./test"

            [provider2.memory]
            "#,
        );

        assert_eq!(
            ProviderConfig::Local(LocalProviderConfig {
                path: Path::new("./test").into(),
            }),
            config.get("provider1").unwrap().unwrap(),
        );
        assert_eq!(
            ProviderConfig::Memory {},
            config.get("provider2").unwrap().unwrap(),
        );
    }

    #[test]
    fn test_parse_local_provider_config() {
        let config = lgn_config::Config::from_toml(
            r#"
            [content_store.provider.local]
            path = "./test"
            "#,
        );

        let config: LocalProviderConfig =
            config.get("content_store.provider.local").unwrap().unwrap();

        assert_eq!(
            config,
            LocalProviderConfig {
                path: Path::new("./test").into(),
            }
        );
    }
}
