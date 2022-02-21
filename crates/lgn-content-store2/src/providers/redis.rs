use async_trait::async_trait;
use redis::AsyncCommands;
use std::io::Cursor;

use crate::{
    ContentAsyncRead, ContentAsyncWrite, ContentReader, ContentWriter, Error, Identifier, Result,
};

use super::{Uploader, UploaderImpl};

pub struct RedisProvider {
    key_prefix: String,
    client: redis::Client,
}

impl RedisProvider {
    /// Generates a new Redis provider using the specified key prefix.
    ///
    /// # Errors
    ///
    /// If the specified Redis URL is invalid, an error is returned.
    pub async fn new(redis_url: impl Into<String>, key_prefix: impl Into<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url.into())
            .map_err(|err| anyhow::anyhow!("failed to instanciate a Redis client: {}", err))?;
        let key_prefix = key_prefix.into();

        Ok(Self { key_prefix, client })
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    pub async fn delete_content(&self, id: &Identifier) -> Result<()> {
        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let key = self.get_key(id);

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

    pub(crate) fn get_key(&self, id: &Identifier) -> String {
        Self::get_key_with_prefix(id, &self.key_prefix)
    }

    pub(crate) fn get_key_with_prefix(id: &Identifier, key_prefix: &str) -> String {
        if key_prefix.is_empty() {
            id.to_string()
        } else {
            format!("{}:{}", key_prefix, id)
        }
    }
}

#[async_trait]
impl ContentReader for RedisProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        let key = self.get_key(id);

        match con.get::<_, Option<Vec<u8>>>(&key).await {
            Ok(Some(value)) => Ok(Box::pin(Cursor::new(value))),
            Ok(None) => Err(Error::NotFound),
            Err(err) => Err(anyhow::anyhow!(
                "failed to get content from Redis for key `{}`: {}",
                key,
                err
            )
            .into()),
        }
    }
}

#[async_trait]
impl ContentWriter for RedisProvider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let key = self.get_key(id);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        match con.exists(&key).await {
            Ok(true) => Err(Error::AlreadyExists),
            Ok(false) => Ok(Box::pin(RedisUploader::new(
                id.clone(),
                RedisUploaderImpl {
                    client: self.client.clone(),
                    key_prefix: self.key_prefix.clone(),
                },
            ))),
            Err(err) => Err(anyhow::anyhow!(
                "failed to check if content exists for key `{}`: {}",
                key,
                err
            )
            .into()),
        }
    }
}

type RedisUploader = Uploader<RedisUploaderImpl>;

struct RedisUploaderImpl {
    client: redis::Client,
    key_prefix: String,
}

#[async_trait]
impl UploaderImpl for RedisUploaderImpl {
    async fn upload(self, data: Vec<u8>, id: Identifier) -> Result<()> {
        let key = RedisProvider::get_key_with_prefix(&id, &self.key_prefix);

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
