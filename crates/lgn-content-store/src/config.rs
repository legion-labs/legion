use crate::{
    AwsAggregatorProvider, AwsDynamoDbProvider, AwsS3Provider, AwsS3Url, CachingProvider,
    ContentAddressProvider, ContentProvider, DataSpace, Error, GrpcProvider, LocalProvider,
    LruProvider, MemoryProvider, RedisProvider, Result, SmallContentProvider,
};
use http::Uri;
use lgn_config::RichPathBuf;
use serde::Deserialize;

/// The configuration of the content-store.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct Config {
    pub provider: ProviderConfig,

    #[serde(default)]
    pub caching_providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConfig {
    Memory {},
    Lru(LruProviderConfig),
    Local(LocalProviderConfig),
    Redis(RedisProviderConfig),
    Grpc(GrpcProviderConfig),
    AwsS3(AwsS3ProviderConfig),
    AwsDynamoDb(AwsDynamoDbProviderConfig),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AddressProviderConfig {
    AwsS3(AwsS3AddressProviderConfig),
}
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalProviderConfig {
    pub path: RichPathBuf,
}

fn default_redis_url() -> Uri {
    "redis://localhost:6379".parse().unwrap()
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RedisProviderConfig {
    #[serde(default = "default_redis_url", with = "http_serde::uri")]
    pub url: Uri,

    #[serde(default)]
    pub key_prefix: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GrpcProviderConfig {
    #[serde(default, with = "option_uri")]
    pub api_url: Option<Uri>,
    pub data_space: DataSpace,
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
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LruProviderConfig {
    #[serde(default)]
    pub size: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsS3ProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,

    // When using S3, we must provide a DynamoDb table along to handle aliases.
    pub dynamodb: AwsDynamoDbProviderConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsS3AddressProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsDynamoDbProviderConfig {
    pub region: Option<String>,
    pub table_name: String,
}

impl Config {
    /// The default name for the persistent content-store configuration.
    pub const SECTION_PERSISTENT: &'static str = "persistent";
    /// The default name for the volatile content-store configuration.
    pub const SECTION_VOLATILE: &'static str = "volatile";

    /// Returns the default configuration for the persistent content-store.
    ///
    /// # Errors
    ///
    /// If the specified configuration section does not exist,
    /// `Error::MissingConfigurationSection` is returned.
    ///
    /// If the configuration section is invalid, `Error::Configuration` is
    /// returned.
    pub fn load_persistent() -> Result<Self> {
        Self::load(Self::SECTION_PERSISTENT)
    }

    /// Returns the default configuration for the volatile content-store.
    ///
    /// # Errors
    ///
    /// If the specified configuration section does not exist,
    /// `Error::MissingConfigurationSection` is returned.
    ///
    /// If the configuration section is invalid, `Error::Configuration` is
    /// returned.
    pub fn load_volatile() -> Result<Self> {
        Self::load(Self::SECTION_VOLATILE)
    }

    /// Returns a new instance from the `legion.toml`, with the specified section.
    ///
    /// # Errors
    ///
    /// If the specified configuration section does not exist,
    /// `Error::MissingConfigurationSection` is returned.
    ///
    /// If the configuration section is invalid, `Error::Configuration` is
    /// returned.
    pub fn load(section: &str) -> Result<Self> {
        match lgn_config::get(&format!("content_store.{}", section))
            .map_err(Error::Configuration)?
        {
            Some(config) => Ok(config),
            None => Err(Error::MissingConfigurationSection {
                section: section.to_string(),
            }),
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

    /// Load the configuration from the specified section and immediately instantiate the provider.
    ///
    /// This is a convenience method.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration cannot be read
    /// or the provider cannot be instantiated.
    pub async fn load_and_instantiate_provider(
        section: &str,
    ) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        let config = Self::load(section)?;
        config.instantiate_provider().await
    }

    /// Load the persistent configuration and immediately instantiate the provider.
    ///
    /// This is a convenience method.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration cannot be read
    /// or the provider cannot be instantiated.
    pub async fn load_and_instantiate_persistent_provider(
    ) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        let config = Self::load_persistent()?;
        config.instantiate_provider().await
    }

    /// Load the volatile configuration and immediately instantiate the provider.
    ///
    /// This is a convenience method.
    ///
    /// # Errors
    ///
    /// This function will return an error if the configuration cannot be read
    /// or the provider cannot be instantiated.
    pub async fn load_and_instantiate_volatile_provider(
    ) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        let config = Self::load_volatile()?;
        config.instantiate_provider().await
    }
}

impl ProviderConfig {
    /// Instantiate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instantiated.
    pub async fn instantiate(&self) -> Result<Box<dyn ContentProvider + Send + Sync>> {
        Ok(match self {
            Self::Memory {} => Box::new(SmallContentProvider::new(MemoryProvider::new())),
            Self::Lru(config) => Box::new(SmallContentProvider::new(LruProvider::new(config.size))),
            Self::Local(config) => Box::new(SmallContentProvider::new(
                LocalProvider::new(config.path.as_ref()).await?,
            )),
            Self::Redis(config) => Box::new(SmallContentProvider::new(
                RedisProvider::new(config.url.to_string(), config.key_prefix.clone()).await?,
            )),
            Self::AwsS3(config) => Box::new(SmallContentProvider::new(AwsAggregatorProvider::new(
                AwsS3Provider::new(AwsS3Url {
                    bucket_name: config.bucket_name.clone(),
                    root: config.root.clone(),
                })
                .await,
                AwsDynamoDbProvider::new(
                    config.dynamodb.region.clone(),
                    config.dynamodb.table_name.clone(),
                )
                .await?,
            ))),
            Self::AwsDynamoDb(config) => Box::new(SmallContentProvider::new(
                AwsDynamoDbProvider::new(config.region.clone(), config.table_name.clone()).await?,
            )),
            Self::Grpc(config) => {
                let client = lgn_online::Config::load()?
                    .instantiate_api_client_with_url(config.api_url.as_ref(), &[])
                    .await?;

                Box::new(SmallContentProvider::new(
                    GrpcProvider::new(client, config.data_space.clone()).await,
                ))
            }
        })
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self::Memory {}
    }
}

impl AddressProviderConfig {
    /// Instantiate the address provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instantiated.
    pub async fn instantiate(&self) -> Result<Box<dyn ContentAddressProvider + Send + Sync>> {
        Ok(match self {
            Self::AwsS3(config) => Box::new(
                AwsS3Provider::new(AwsS3Url {
                    bucket_name: config.bucket_name.clone(),
                    root: config.root.clone(),
                })
                .await,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use super::*;

    #[test]
    fn test_parse_config_without_caching() {
        let config = lgn_config::Config::from_toml(
            r#"
            [content_store.provider]
            type = "local"
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
                    path: PathBuf::from_str("./test").unwrap().into(),
                }),
                caching_providers: vec![],
            }
        );
    }

    #[test]
    fn test_parse_config_with_caching() {
        let config = lgn_config::Config::from_toml(
            r#"
            [content_store.provider]
            type = "local"
            path = "./test"

            [[content_store.caching_providers]]
            type = "lru"
            size = 100

            [[content_store.caching_providers]]
            type = "memory"
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
                    path: PathBuf::from_str("./test").unwrap().into(),
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
            [provider1]
            type = "local"
            path = "./test"

            [provider2]
            type = "memory"
            "#,
        );

        assert_eq!(
            ProviderConfig::Local(LocalProviderConfig {
                path: PathBuf::from_str("./test").unwrap().into(),
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
            [content_store.provider]
            type = "local"
            path = "./test"
            "#,
        );

        let config: LocalProviderConfig = config.get("content_store.provider").unwrap().unwrap();

        assert_eq!(
            config,
            LocalProviderConfig {
                path: PathBuf::from_str("./test").unwrap().into(),
            }
        );
    }
}
