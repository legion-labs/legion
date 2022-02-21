use async_trait::async_trait;
use futures::Future;
use pin_project::pin_project;
use redis::AsyncCommands;
use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ContentReader, ContentWriter, Error, Identifier, Result};

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
        if self.key_prefix.is_empty() {
            id.to_string()
        } else {
            format!("{}:{}", self.key_prefix, id)
        }
    }
}

#[async_trait]
impl ContentReader for RedisProvider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncRead + Send>>> {
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
    async fn get_content_writer(&self, id: &Identifier) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
        let key = self.get_key(id);

        let mut con = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| anyhow::anyhow!("failed to get connection to Redis: {}", err))?;

        match con.exists(&key).await {
            Ok(true) => Err(Error::AlreadyExists),
            Ok(false) => Ok(Box::pin(RedisUploader::new(
                self.client.clone(),
                key,
                id.clone(),
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

#[pin_project]
struct RedisUploader {
    #[pin]
    state: RedisUploaderState,
}

#[allow(clippy::type_complexity)]
enum RedisUploaderState {
    Writing(Option<(std::io::Cursor<Vec<u8>>, Identifier, redis::Client, String)>),
    Uploading(Pin<Box<dyn Future<Output = Result<(), std::io::Error>> + Send + 'static>>),
}

impl RedisUploader {
    pub fn new(client: redis::Client, key: String, id: Identifier) -> Self {
        let state =
            RedisUploaderState::Writing(Some((std::io::Cursor::new(Vec::new()), id, client, key)));

        Self { state }
    }

    async fn upload(
        data: Vec<u8>,
        id: Identifier,
        client: redis::Client,
        key: String,
    ) -> Result<(), std::io::Error> {
        id.matches(&data).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("the data does not match the specified id: {}", err),
            )
        })?;

        let mut con = client.get_async_connection().await.map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("failed to get connection to Redis: {}", err),
            )
        })?;

        match con.set_nx(&key, data).await {
            Ok(()) => Ok(()),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                anyhow::anyhow!("failed to set content in Redis for key `{}`: {}", key, err),
            )),
        }
    }
}

impl AsyncWrite for RedisUploader {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        if let RedisUploaderState::Writing(Some((cursor, _, _, _))) = this.state.get_mut() {
            Pin::new(cursor).poll_write(cx, buf)
        } else {
            panic!("HttpUploader::poll_write called after completion")
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        if let RedisUploaderState::Writing(Some((cursor, _, _, _))) = this.state.get_mut() {
            Pin::new(cursor).poll_flush(cx)
        } else {
            panic!("HttpUploader::poll_flush called after completion")
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let state = this.state.get_mut();

        loop {
            *state = match state {
                RedisUploaderState::Writing(args) => {
                    let res = Pin::new(&mut args.as_mut().unwrap().0).poll_shutdown(cx);

                    match res {
                        Poll::Ready(Ok(())) => {
                            let (cursor, id, client, key) = args.take().unwrap();

                            RedisUploaderState::Uploading(Box::pin(Self::upload(
                                cursor.into_inner(),
                                id,
                                client,
                                key,
                            )))
                        }
                        p => return p,
                    }
                }
                RedisUploaderState::Uploading(call) => return Pin::new(call).poll(cx),
            };
        }
    }
}
