use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, error, warn};
use redis::AsyncCommands;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    io::Cursor,
};

use crate::{
    traits::{get_content_readers_impl, WithOrigin},
    ContentAsyncReadWithOrigin, ContentAsyncWrite, ContentReader, ContentWriter, Error, Identifier,
    Origin, Result,
};

use super::{Uploader, UploaderImpl};

/// A provider that stores content in Redis.
#[derive(Debug, Clone)]
pub struct RedisProvider {
    key_prefix: String,
    client: redis::Client,
    host: String,
}

impl RedisProvider {
    /// Generates a new Redis provider using the specified key prefix.
    ///
    /// # Errors
    ///
    /// If the specified Redis URL is invalid, an error is returned.
    pub async fn new(redis_url: impl Into<String>, key_prefix: impl Into<String>) -> Result<Self> {
        let url = redis_url.into();
        let client = redis::Client::open(url.clone())
            .map_err(|err| anyhow::anyhow!("failed to instantiate a Redis client: {}", err))?;
        let key_prefix = key_prefix.into();
        let host = url
            .parse::<http::Uri>()
            .map_err(|err| anyhow::anyhow!("failed to parse Redis URL: {}", err))?
            .authority()
            .ok_or_else(|| anyhow::anyhow!("Redis URL must contain an authority"))?
            .to_string();

        Ok(Self {
            key_prefix,
            client,
            host,
        })
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    pub async fn delete_content(&self, id: &Identifier) -> Result<()> {
        async_span_scope!("RedisProvider::delete_content");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let key = self.get_content_key(id);

        match con.del::<_, bool>(&key).await {
            Ok(true) => Ok(()),
            Ok(false) => Err(anyhow::anyhow!(
                "could not delete non-existing content from Redis for key `{}`",
                key,
            )
            .into()),
            Err(err) => Err(anyhow::anyhow!(
                "failed to delete content from Redis for key `{}`: {}",
                key,
                err
            )
            .into()),
        }
    }

    pub(crate) fn get_content_key(&self, id: &Identifier) -> String {
        Self::get_content_key_with_prefix(id, &self.key_prefix)
    }

    pub(crate) fn get_content_key_with_prefix(id: &Identifier, key_prefix: &str) -> String {
        if key_prefix.is_empty() {
            format!("content:{}", id)
        } else {
            format!("{}:content:{}", key_prefix, id)
        }
    }

    pub(crate) fn get_alias_key(&self, key_space: &str, key: &str) -> String {
        Self::get_alias_key_with_prefix(key_space, key, &self.key_prefix)
    }

    pub(crate) fn get_alias_key_with_prefix(
        key_space: &str,
        key: &str,
        key_prefix: &str,
    ) -> String {
        if key_prefix.is_empty() {
            format!("alias:{}:{}", key_space, key)
        } else {
            format!("{}:alias:{}:{}", key_prefix, key_space, key)
        }
    }
}

impl Display for RedisProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Redis (host: {}, key prefix: {})",
            self.host, self.key_prefix
        )
    }
}

#[async_trait]
impl ContentReader for RedisProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncReadWithOrigin> {
        async_span_scope!("RedisProvider::get_content_reader");

        debug!("RedisProvider::get_content_reader({})", id);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let key = self.get_content_key(id);

        match con.get::<_, Option<Vec<u8>>>(&key).await {
            Ok(Some(value)) => {
                debug!(
                    "RedisProvider::get_content_reader({}) -> found item with key `{}`",
                    id, key
                );

                let origin = Origin::Redis {
                    host: self.host.clone(),
                    key,
                };

                Ok(Cursor::new(value).with_origin(origin))
            }
            Ok(None) => {
                warn!(
                    "RedisProvider::get_content_reader({}) -> item with key `{}` was not found",
                    id, key
                );

                Err(Error::IdentifierNotFound(id.clone()))
            }
            Err(err) => {
                error!(
                    "RedisProvider::get_content_reader({}) -> failed to read item with key `{}`: {}",
                    id, key, err
                );

                Err(anyhow::anyhow!(
                    "failed to get content from Redis for key `{}`: {}",
                    key,
                    err
                )
                .into())
            }
        }
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncReadWithOrigin>>> {
        async_span_scope!("RedisProvider::get_content_readers");

        debug!("RedisProvider::get_content_readers({:?})", ids);

        get_content_readers_impl(self, ids).await
    }

    async fn resolve_alias(&self, key_space: &str, key: &str) -> Result<Identifier> {
        async_span_scope!("RedisProvider::resolve_alias");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let k = self.get_alias_key(key_space, key);

        match con.get::<_, Option<Vec<u8>>>(&k).await {
            Ok(Some(value)) => Identifier::read_from(std::io::Cursor::new(value)),
            Ok(None) => Err(Error::AliasNotFound {
                key_space: key_space.to_string(),
                key: key.to_string(),
            }),
            Err(err) => Err(anyhow::anyhow!(
                "failed to resolve alias from Redis for key `{}`: {}",
                k,
                err
            )
            .into()),
        }
    }
}

#[async_trait]
impl ContentWriter for RedisProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        async_span_scope!("RedisProvider::get_content_writer");

        debug!("RedisProvider::get_content_writer({})", id);

        let key = self.get_content_key(id);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        match con.exists(&key).await {
            Ok(true) => {
                debug!(
                    "RedisProvider::get_content_writer({}) -> item with key `{}` already exists",
                    id, key
                );

                Err(Error::IdentifierAlreadyExists(id.clone()))
            }
            Ok(false) => {
                debug!(
                    "RedisProvider::get_content_writer({}) -> item with key `{}` does not exist: writer created",
                    id, key
                );

                Ok(Box::pin(RedisUploader::new(
                    id.clone(),
                    RedisUploaderImpl {
                        client: self.client.clone(),
                        key_prefix: self.key_prefix.clone(),
                    },
                )))
            }
            Err(err) => {
                error!(
                    "RedisProvider::get_content_writer({}) -> failed to check key `{}`: {}",
                    id, key, err
                );

                Err(anyhow::anyhow!(
                    "failed to check if content exists for key `{}`: {}",
                    key,
                    err
                )
                .into())
            }
        }
    }

    async fn register_alias(&self, key_space: &str, key: &str, id: &Identifier) -> Result<()> {
        async_span_scope!("RedisProvider::regiser_alias");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let k = self.get_alias_key(key_space, key);

        match con.exists(&k).await {
            Ok(true) => Err(Error::AliasAlreadyExists {
                key_space: key_space.to_string(),
                key: key.to_string(),
            }),
            Ok(false) => match con.set_nx(&k, id.as_vec()).await {
                Ok(()) => Ok(()),
                Err(err) => Err(anyhow::anyhow!(
                    "failed to register alias in Redis for key `{}`: {}",
                    k,
                    err
                )
                .into()),
            },
            Err(err) => {
                Err(
                    anyhow::anyhow!("failed to check if alias exists for key `{}`: {}", k, err)
                        .into(),
                )
            }
        }
    }
}

type RedisUploader = Uploader<RedisUploaderImpl>;

#[derive(Debug)]
struct RedisUploaderImpl {
    client: redis::Client,
    key_prefix: String,
}

#[async_trait]
impl UploaderImpl for RedisUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        async_span_scope!("RedisProvider::upload");

        let key = RedisProvider::get_content_key_with_prefix(&id, &self.key_prefix);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        match con.set_nx(&key, data).await {
            Ok(()) => Ok(()),
            Err(err) => {
                Err(
                    anyhow::anyhow!("failed to set content in Redis for key `{}`: {}", key, err)
                        .into(),
                )
            }
        }
    }
}
