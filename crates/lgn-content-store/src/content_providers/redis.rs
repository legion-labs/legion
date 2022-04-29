use async_trait::async_trait;
use lgn_tracing::{async_span_scope, debug, error, span_fn, warn};
use redis::AsyncCommands;
use std::{fmt::Display, io::Cursor};

use super::{
    ContentAsyncReadWithOriginAndSize, ContentAsyncWrite, ContentReader, ContentWriter, Error,
    HashRef, Origin, Result, WithOriginAndSize,
};

use super::{Uploader, UploaderImpl};

/// A provider that stores content in Redis.
#[derive(Debug, Clone)]
pub struct RedisContentProvider {
    key_prefix: String,
    client: redis::Client,
    host: String,
}

impl RedisContentProvider {
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
    #[span_fn]
    pub async fn delete_content(&self, id: &HashRef) -> Result<()> {
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

    pub(crate) fn get_content_key(&self, id: &HashRef) -> String {
        Self::get_content_key_with_prefix(id, &self.key_prefix)
    }

    pub(crate) fn get_content_key_with_prefix(id: &HashRef, key_prefix: &str) -> String {
        if key_prefix.is_empty() {
            format!("content:{}", id)
        } else {
            format!("{}:content:{}", key_prefix, id)
        }
    }
}

impl Display for RedisContentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Redis (host: {}, key prefix: {})",
            self.host, self.key_prefix
        )
    }
}

#[async_trait]
impl ContentReader for RedisContentProvider {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("RedisContentProvider::get_content_reader");

        debug!("RedisContentProvider::get_content_reader({})", id);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let key = self.get_content_key(id);

        match con.get::<_, Option<Vec<u8>>>(&key).await {
            Ok(Some(value)) => {
                debug!(
                    "RedisContentProvider::get_content_reader({}) -> found item with key `{}`",
                    id, key
                );

                let origin = Origin::Redis {
                    host: self.host.clone(),
                    key,
                };

                Ok(Cursor::new(value).with_origin_and_size(origin, id.data_size()))
            }
            Ok(None) => {
                warn!(
                    "RedisContentProvider::get_content_reader({}) -> item with key `{}` was not found",
                    id, key
                );

                Err(Error::HashRefNotFound(id.clone()))
            }
            Err(err) => {
                error!(
                    "RedisContentProvider::get_content_reader({}) -> failed to read item with key `{}`: {}",
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
}

#[async_trait]
impl ContentWriter for RedisContentProvider {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("RedisContentProvider::get_content_writer");

        debug!("RedisContentProvider::get_content_writer({})", id);

        let key = self.get_content_key(id);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        match con.exists(&key).await {
            Ok(true) => {
                debug!(
                    "RedisContentProvider::get_content_writer({}) -> item with key `{}` already exists",
                    id, key
                );

                Err(Error::HashRefAlreadyExists(id.clone()))
            }
            Ok(false) => {
                debug!(
                    "RedisContentProvider::get_content_writer({}) -> item with key `{}` does not exist: writer created",
                    id, key
                );

                Ok(Box::pin(RedisUploader::new(RedisUploaderImpl {
                    client: self.client.clone(),
                    key_prefix: self.key_prefix.clone(),
                })))
            }
            Err(err) => {
                error!(
                    "RedisContentProvider::get_content_writer({}) -> failed to check key `{}`: {}",
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
}

type RedisUploader = Uploader<RedisUploaderImpl>;

#[derive(Debug)]
struct RedisUploaderImpl {
    client: redis::Client,
    key_prefix: String,
}

#[async_trait]
impl UploaderImpl for RedisUploaderImpl {
    async fn upload(self, data: Vec<u8>) -> Result<()> {
        async_span_scope!("RedisContentProvider::upload");

        let id = HashRef::new_from_data(&data);
        let key = RedisContentProvider::get_content_key_with_prefix(&id, &self.key_prefix);

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

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_redis_content_provider() {
        let docker = testcontainers::clients::Cli::default();
        let redis =
            testcontainers::Docker::run(&docker, testcontainers::images::redis::Redis::default());

        let redis_host = format!("localhost:{}", redis.get_host_port(6379).unwrap());
        let redis_url = format!("redis://{}", redis_host);
        let key_prefix = "content-store";
        let content_provider = RedisContentProvider::new(redis_url.clone(), key_prefix)
            .await
            .expect("failed to create Redis provider");

        let data: &[u8; 128] = &[0x41; 128];

        let origin = Origin::Redis {
            host: redis_host,
            key: format!("{}:content:{}", key_prefix, HashRef::new_from_data(data)),
        };

        crate::content_providers::test_content_provider(&content_provider, data, origin).await;
    }
}
