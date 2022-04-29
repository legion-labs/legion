use crate::{
    AliasProvider, AliasProviderCache, AwsDynamoDbAliasProvider, AwsDynamoDbContentProvider,
    AwsS3ContentProvider, AwsS3Url, ContentAddressProvider, ContentProvider, ContentProviderCache,
    DataSpace, Error, GrpcAliasProvider, GrpcContentProvider, LocalAliasProvider,
    LocalContentProvider, LruAliasProvider, LruContentProvider, MemoryAliasProvider,
    MemoryContentProvider, Provider, RedisAliasProvider, RedisContentProvider, Result,
};
use http::Uri;
use lgn_config::RichPathBuf;
use serde::Deserialize;

/// The configuration of the content-store.
#[derive(Debug, Clone, Default, Deserialize, PartialEq)]
pub struct Config {
    pub content_provider: ContentProviderConfig,
    pub alias_provider: AliasProviderConfig,

    #[serde(default)]
    pub caching_content_providers: Vec<ContentProviderConfig>,
    #[serde(default)]
    pub caching_alias_providers: Vec<AliasProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentProviderConfig {
    Memory {},
    Lru(LruContentProviderConfig),
    Local(LocalContentProviderConfig),
    Redis(RedisContentProviderConfig),
    Grpc(GrpcContentProviderConfig),
    AwsS3(AwsS3ContentProviderConfig),
    AwsDynamoDb(AwsDynamoDbContentProviderConfig),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AliasProviderConfig {
    Memory {},
    Lru(LruAliasProviderConfig),
    Local(LocalAliasProviderConfig),
    Redis(RedisAliasProviderConfig),
    Grpc(GrpcAliasProviderConfig),
    AwsDynamoDb(AwsDynamoDbAliasProviderConfig),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AddressProviderConfig {
    AwsS3(AwsS3AddressProviderConfig),
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalContentProviderConfig {
    pub path: RichPathBuf,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LocalAliasProviderConfig {
    pub path: RichPathBuf,
}

fn default_redis_url() -> Uri {
    "redis://localhost:6379".parse().unwrap()
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RedisContentProviderConfig {
    #[serde(default = "default_redis_url", with = "http_serde::uri")]
    pub url: Uri,

    #[serde(default)]
    pub key_prefix: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RedisAliasProviderConfig {
    #[serde(default = "default_redis_url", with = "http_serde::uri")]
    pub url: Uri,

    #[serde(default)]
    pub key_prefix: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GrpcContentProviderConfig {
    #[serde(default, with = "option_uri")]
    pub api_url: Option<Uri>,
    pub data_space: DataSpace,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct GrpcAliasProviderConfig {
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
pub struct LruContentProviderConfig {
    #[serde(default)]
    pub size: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LruAliasProviderConfig {
    #[serde(default)]
    pub size: usize,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsS3ContentProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsS3AddressProviderConfig {
    pub bucket_name: String,

    #[serde(default)]
    pub root: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsDynamoDbContentProviderConfig {
    pub region: Option<String>,
    pub table_name: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct AwsDynamoDbAliasProviderConfig {
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
    pub async fn instantiate_provider(&self) -> Result<Provider> {
        let mut content_provider = self.content_provider.instantiate().await?;

        for caching_content_provider in &self.caching_content_providers {
            let caching_content_provider = caching_content_provider.instantiate().await?;
            content_provider = Box::new(ContentProviderCache::new(
                content_provider,
                caching_content_provider,
            ));
        }

        let mut alias_provider = self.alias_provider.instantiate().await?;

        for caching_alias_provider in &self.caching_alias_providers {
            let caching_alias_provider = caching_alias_provider.instantiate().await?;
            alias_provider = Box::new(AliasProviderCache::new(
                alias_provider,
                caching_alias_provider,
            ));
        }

        let provider = Provider::new(content_provider, alias_provider);

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
    pub async fn load_and_instantiate_provider(section: &str) -> Result<Provider> {
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
    pub async fn load_and_instantiate_persistent_provider() -> Result<Provider> {
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
    pub async fn load_and_instantiate_volatile_provider() -> Result<Provider> {
        let config = Self::load_volatile()?;
        config.instantiate_provider().await
    }
}

impl ContentProviderConfig {
    /// Instantiate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instantiated.
    pub async fn instantiate(&self) -> Result<Box<dyn ContentProvider>> {
        Ok(match self {
            Self::Memory {} => Box::new(MemoryContentProvider::new()),
            Self::Lru(config) => Box::new(LruContentProvider::new(config.size)),
            Self::Local(config) => Box::new(LocalContentProvider::new(config.path.as_ref()).await?),
            Self::Redis(config) => Box::new(
                RedisContentProvider::new(config.url.to_string(), config.key_prefix.clone())
                    .await?,
            ),
            Self::AwsS3(config) => Box::new(
                AwsS3ContentProvider::new(AwsS3Url {
                    bucket_name: config.bucket_name.clone(),
                    root: config.root.clone(),
                })
                .await,
            ),
            Self::AwsDynamoDb(config) => Box::new(
                AwsDynamoDbContentProvider::new(config.region.clone(), config.table_name.clone())
                    .await?,
            ),
            Self::Grpc(config) => {
                let client = lgn_online::Config::load()?
                    .instantiate_api_client_with_url(config.api_url.as_ref(), &[])
                    .await?;

                Box::new(GrpcContentProvider::new(client, config.data_space.clone()).await)
            }
        })
    }
}

impl Default for ContentProviderConfig {
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
    pub async fn instantiate(&self) -> Result<Box<dyn ContentAddressProvider>> {
        Ok(match self {
            Self::AwsS3(config) => Box::new(
                AwsS3ContentProvider::new(AwsS3Url {
                    bucket_name: config.bucket_name.clone(),
                    root: config.root.clone(),
                })
                .await,
            ),
        })
    }
}

impl AliasProviderConfig {
    /// Instantiate the provider for the configuration.
    ///
    /// # Errors
    ///
    /// This function will return an error if the provider cannot be instantiated.
    pub async fn instantiate(&self) -> Result<Box<dyn AliasProvider>> {
        Ok(match self {
            Self::Memory {} => Box::new(MemoryAliasProvider::new()),
            Self::Lru(config) => Box::new(LruAliasProvider::new(config.size)),
            Self::Local(config) => Box::new(LocalAliasProvider::new(config.path.as_ref()).await?),
            Self::Redis(config) => Box::new(
                RedisAliasProvider::new(config.url.to_string(), config.key_prefix.clone()).await?,
            ),
            Self::AwsDynamoDb(config) => Box::new(
                AwsDynamoDbAliasProvider::new(config.region.clone(), config.table_name.clone())
                    .await?,
            ),
            Self::Grpc(config) => {
                let client = lgn_online::Config::load()?
                    .instantiate_api_client_with_url(config.api_url.as_ref(), &[])
                    .await?;

                Box::new(GrpcAliasProvider::new(client, config.data_space.clone()).await)
            }
        })
    }
}

impl Default for AliasProviderConfig {
    fn default() -> Self {
        Self::Memory {}
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
            [content_store.content_provider]
            type = "local"
            path = "./test/content"

            [content_store.alias_provider]
            type = "local"
            path = "./test/aliases"
            "#,
        );

        let config: Config = config
            .get("content_store")
            .expect("failed to read configuration")
            .expect("option was none");

        assert_eq!(
            config,
            Config {
                content_provider: ContentProviderConfig::Local(LocalContentProviderConfig {
                    path: PathBuf::from_str("./test/content").unwrap().into(),
                }),
                alias_provider: AliasProviderConfig::Local(LocalAliasProviderConfig {
                    path: PathBuf::from_str("./test/aliases").unwrap().into(),
                }),
                caching_content_providers: vec![],
                caching_alias_providers: vec![],
            }
        );
    }

    #[test]
    fn test_parse_config_with_caching() {
        let config = lgn_config::Config::from_toml(
            r#"
            [content_store.content_provider]
            type = "local"
            path = "./test/content"

            [content_store.alias_provider]
            type = "local"
            path = "./test/aliases"

            [[content_store.caching_content_providers]]
            type = "lru"
            size = 100

            [[content_store.caching_content_providers]]
            type = "memory"

            [[content_store.caching_alias_providers]]
            type = "lru"
            size = 100

            [[content_store.caching_alias_providers]]
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
                content_provider: ContentProviderConfig::Local(LocalContentProviderConfig {
                    path: PathBuf::from_str("./test/content").unwrap().into(),
                }),
                alias_provider: AliasProviderConfig::Local(LocalAliasProviderConfig {
                    path: PathBuf::from_str("./test/aliases").unwrap().into(),
                }),
                caching_content_providers: vec![
                    ContentProviderConfig::Lru(LruContentProviderConfig { size: 100 }),
                    ContentProviderConfig::Memory {}
                ],
                caching_alias_providers: vec![
                    AliasProviderConfig::Lru(LruAliasProviderConfig { size: 100 }),
                    AliasProviderConfig::Memory {}
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
            ContentProviderConfig::Local(LocalContentProviderConfig {
                path: PathBuf::from_str("./test").unwrap().into(),
            }),
            config.get("provider1").unwrap().unwrap(),
        );
        assert_eq!(
            ContentProviderConfig::Memory {},
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

        let config: LocalContentProviderConfig =
            config.get("content_store.provider").unwrap().unwrap();

        assert_eq!(
            config,
            LocalContentProviderConfig {
                path: PathBuf::from_str("./test").unwrap().into(),
            }
        );
    }
}
