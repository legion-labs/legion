use async_trait::async_trait;
use lgn_tracing::async_span_scope;
use redis::AsyncCommands;
use std::{fmt::Display, io::Write};

use super::{AliasReader, AliasWriter, Error, Result};
use crate::Identifier;

/// A provider that stores content in Redis.
#[derive(Debug, Clone)]
pub struct RedisAliasProvider {
    key_prefix: String,
    client: redis::Client,
    host: String,
}

impl RedisAliasProvider {
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

    pub(crate) fn get_alias_key(&self, key: &[u8]) -> String {
        Self::get_alias_key_with_prefix(key, &self.key_prefix)
    }

    pub(crate) fn get_alias_key_with_prefix(key: &[u8], key_prefix: &str) -> String {
        let mut enc = base64::write::EncoderStringWriter::new(base64::URL_SAFE_NO_PAD);
        enc.write_all(key).expect("base64 encoding failed");
        let key = enc.into_inner();

        if key_prefix.is_empty() {
            format!("alias:{}", key)
        } else {
            format!("{}:alias:{}", key_prefix, key)
        }
    }
}

impl Display for RedisAliasProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Redis (host: {}, key prefix: {})",
            self.host, self.key_prefix
        )
    }
}

#[async_trait]
impl AliasReader for RedisAliasProvider {
    async fn resolve_alias(&self, key: &[u8]) -> Result<Identifier> {
        async_span_scope!("RedisAliasProvider::resolve_alias");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let k = self.get_alias_key(key);

        match con.get::<_, Option<Vec<u8>>>(&k).await {
            Ok(Some(value)) => Ok(Identifier::read_from(std::io::Cursor::new(value))?),
            Ok(None) => Err(Error::AliasNotFound(key.into())),
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
impl AliasWriter for RedisAliasProvider {
    async fn register_alias(&self, key: &[u8], id: &Identifier) -> Result<Identifier> {
        async_span_scope!("RedisAliasProvider::regiser_alias");

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let k = self.get_alias_key(key);

        match con.exists(&k).await {
            Ok(true) => Err(Error::AliasAlreadyExists(key.into())),
            Ok(false) => match con.set_nx(&k, id.as_vec()).await {
                Ok(()) => Ok(Identifier::new_alias(key.into())),
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

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_redis_alias_provider() {
        let docker = testcontainers::clients::Cli::default();
        let redis =
            testcontainers::Docker::run(&docker, testcontainers::images::redis::Redis::default());

        let redis_host = format!("localhost:{}", redis.get_host_port(6379).unwrap());
        let redis_url = format!("redis://{}", redis_host);
        let key_prefix = "content-store";
        let alias_provider = RedisAliasProvider::new(redis_url.clone(), key_prefix)
            .await
            .expect("failed to create Redis provider");

        let uid = uuid::Uuid::new_v4();
        let key = uid.as_bytes();

        crate::alias_providers::test_alias_provider(&alias_provider, key).await;
    }
}
