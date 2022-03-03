use async_trait::async_trait;
use aws_sdk_s3::presigning::config::PresigningConfig;
use bytes::Bytes;
use pin_project::pin_project;
use std::collections::{BTreeMap, BTreeSet};
use std::future::Future;
use std::str::FromStr;
use std::sync::Mutex;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{fmt::Display, pin::Pin};
use tokio::io::AsyncWrite;
use tokio_stream::Stream;
use tokio_util::io::StreamReader;

use crate::traits::get_content_readers_impl;
use crate::{
    ContentAddressReader, ContentAddressWriter, ContentAsyncRead, ContentAsyncWrite, ContentReader,
    ContentWriter, Error, Identifier, Result,
};

#[derive(Debug, Clone)]
pub struct AwsS3Provider {
    url: AwsS3Url,
    client: aws_sdk_s3::Client,
    validity_duration: Duration,
}

impl AwsS3Provider {
    /// Generates a new AWS S3 provider using the specified bucket and root.
    ///
    /// The default AWS configuration is used.
    pub async fn new(url: AwsS3Url) -> Self {
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);

        Self {
            url,
            client,
            validity_duration: Duration::from_secs(60 * 30),
        }
    }

    /// Set the validity duration for presigned URLs.
    pub fn with_validity_duration(self, validity_duration: Duration) -> Self {
        Self {
            validity_duration,
            ..self
        }
    }

    fn blob_key(&self, id: &Identifier) -> String {
        format!("{}/{}", self.url.root, id)
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    pub async fn delete_content(&self, id: &Identifier) -> Result<()> {
        let key = self.blob_key(id);

        match self
            .client
            .delete_object()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                Err(
                    anyhow::anyhow!("failed to delete object `{}` from AWS S3: {}", key, err)
                        .into(),
                )
            }
        }
    }

    /// Check whether an object exists with the specified identifier.
    ///
    /// # Errors
    ///
    /// Only if an object's existence cannot be determined, an error is returned.
    pub async fn check_object_existence(&self, id: &Identifier) -> Result<bool> {
        let key = self.blob_key(id);

        match self
            .client
            .get_object_acl()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(aws_sdk_s3::SdkError::ServiceError { err, raw: _ }) => {
                if let aws_sdk_s3::error::GetObjectAclErrorKind::NoSuchKey(_) = err.kind {
                    Ok(false)
                } else {
                    Err(
                        anyhow::anyhow!("failed to get object acl `{}` from AWS S3: {}", key, err)
                            .into(),
                    )
                }
            }
            Err(err) => {
                Err(
                    anyhow::anyhow!("failed to get object acl `{}` from AWS S3: {}", key, err)
                        .into(),
                )
            }
        }
    }
}

#[pin_project]
#[derive(Debug)]
struct ByteStreamReader(#[pin] aws_sdk_s3::ByteStream);

impl Stream for ByteStreamReader {
    type Item = std::result::Result<Bytes, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project()
            .0
            .poll_next(cx)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Interrupted, e))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

struct ByteStreamWriter {
    client: aws_sdk_s3::Client,
    bucket_name: String,
    key: String,
    // TODO: Rewrite this type with pin_project and avoid the mutex.
    state: Mutex<ByteStreamWriterState>,
}

type ByteStreamWriterBoxedFuture = Box<
    dyn Future<
            Output = std::result::Result<
                aws_sdk_s3::output::PutObjectOutput,
                aws_sdk_s3::SdkError<aws_sdk_s3::error::PutObjectError>,
            >,
        > + Send
        + 'static,
>;

enum ByteStreamWriterState {
    Writing(Vec<u8>),
    Uploading(Pin<ByteStreamWriterBoxedFuture>),
}

impl ByteStreamWriter {
    fn new(client: aws_sdk_s3::Client, bucket_name: String, key: String) -> Self {
        Self {
            client,
            bucket_name,
            key,
            state: Mutex::new(ByteStreamWriterState::Writing(Vec::new())),
        }
    }

    fn poll_write_impl(&self, buf: &[u8]) -> Poll<std::result::Result<usize, std::io::Error>> {
        match &mut *self.state.lock().unwrap() {
            ByteStreamWriterState::Writing(buffer) => {
                buffer.extend_from_slice(buf);

                Poll::Ready(Ok(buf.len()))
            }
            ByteStreamWriterState::Uploading(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot write to an uploading stream",
            ))),
        }
    }

    fn poll_flush_impl(&self) -> Poll<std::result::Result<(), std::io::Error>> {
        match &*self.state.lock().unwrap() {
            ByteStreamWriterState::Writing(_) => Poll::Ready(Ok(())),
            ByteStreamWriterState::Uploading(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot flush an uploading stream",
            ))),
        }
    }

    fn poll_shutdown_impl(
        &self,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let mut state = self.state.lock().unwrap();

        let fut = match &mut *state {
            ByteStreamWriterState::Writing(buffer) => {
                let body = aws_sdk_s3::ByteStream::from(std::mem::take(buffer));

                let fut = self
                    .client
                    .put_object()
                    .bucket(&self.bucket_name)
                    .key(&self.key)
                    .body(body)
                    .send();

                *state = ByteStreamWriterState::Uploading(Box::pin(fut));

                if let ByteStreamWriterState::Uploading(fut) = &mut *state {
                    fut
                } else {
                    unreachable!()
                }
            }
            ByteStreamWriterState::Uploading(fut) => fut,
        };

        match fut.as_mut().poll(cx) {
            Poll::Ready(Ok(_)) => Poll::Ready(Ok(())),
            Poll::Ready(Err(err)) => {
                Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, err)))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for ByteStreamWriter {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        self.poll_write_impl(buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        self.poll_flush_impl()
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        self.poll_shutdown_impl(cx)
    }
}

#[async_trait]
impl ContentReader for AwsS3Provider {
    async fn get_content_reader(&self, id: &Identifier) -> Result<ContentAsyncRead> {
        let key = self.blob_key(id);

        let object = match self
            .client
            .get_object()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(object) => Ok(object),
            Err(aws_sdk_s3::SdkError::ServiceError { err, raw: _ }) => {
                if let aws_sdk_s3::error::GetObjectErrorKind::NoSuchKey(_) = err.kind {
                    Err(Error::NotFound)
                } else {
                    Err(
                        anyhow::anyhow!("failed to get object `{}` from AWS S3: {}", key, err)
                            .into(),
                    )
                }
            }
            Err(err) => {
                Err(anyhow::anyhow!("failed to get object `{}` from AWS S3: {}", key, err).into())
            }
        }?;

        let bytestream = ByteStreamReader(object.body);
        let stream = StreamReader::new(bytestream);

        Ok(Box::pin(stream))
    }

    async fn get_content_readers<'ids>(
        &self,
        ids: &'ids BTreeSet<Identifier>,
    ) -> Result<BTreeMap<&'ids Identifier, Result<ContentAsyncRead>>> {
        get_content_readers_impl(self, ids).await
    }
}

#[async_trait]
impl ContentWriter for AwsS3Provider {
    async fn get_content_writer(&self, id: &Identifier) -> Result<ContentAsyncWrite> {
        let key = self.blob_key(id);

        match self
            .client
            .get_object()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Err(Error::AlreadyExists {}),
            Err(aws_sdk_s3::SdkError::ServiceError { err, raw: _ }) => {
                if let aws_sdk_s3::error::GetObjectErrorKind::NoSuchKey(_) = err.kind {
                    Ok(())
                } else {
                    Err(
                        anyhow::anyhow!("failed to get object `{}` from AWS S3: {}", key, err)
                            .into(),
                    )
                }
            }
            Err(err) => {
                Err(anyhow::anyhow!("failed to get object `{}` from AWS S3: {}", key, err).into())
            }
        }?;

        let writer = ByteStreamWriter::new(self.client.clone(), self.url.bucket_name.clone(), key);

        Ok(Box::pin(writer))
    }
}

#[async_trait]
impl ContentAddressReader for AwsS3Provider {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        if !self.check_object_existence(id).await? {
            return Err(Error::NotFound);
        }

        let key = self.blob_key(id);

        Ok(self
            .client
            .get_object()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .presigned(
                PresigningConfig::expires_in(self.validity_duration)
                    .map_err(|err| anyhow::anyhow!("failed to create presigned URL: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to generate AWS S3 get signature for object `{}`: {}",
                    key,
                    err
                )
            })?
            .uri()
            .to_string())
    }
}

#[async_trait]
impl ContentAddressWriter for AwsS3Provider {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        if self.check_object_existence(id).await? {
            return Err(Error::AlreadyExists);
        }

        let key = self.blob_key(id);

        Ok(self
            .client
            .put_object()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .presigned(
                PresigningConfig::expires_in(self.validity_duration)
                    .map_err(|err| anyhow::anyhow!("failed to create presigned URL: {}", err))?,
            )
            .await
            .map_err(|err| {
                anyhow::anyhow!(
                    "failed to generate AWS S3 put signature for object `{}`: {}",
                    key,
                    err
                )
            })?
            .uri()
            .to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AwsS3Url {
    pub bucket_name: String,
    pub root: String,
}

impl Display for AwsS3Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.root.is_empty() {
            write!(f, "s3://{}", self.bucket_name)
        } else {
            write!(f, "s3://{}/{}", self.bucket_name, self.root)
        }
    }
}

impl FromStr for AwsS3Url {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if !s.starts_with("s3://") {
            return Err(
                anyhow::anyhow!("invalid S3 URL: should start with `s3://` in `{}`", s).into(),
            );
        }

        let mut splitter = s[5..].splitn(2, '/');

        Ok(Self {
            bucket_name: splitter
                .next()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| anyhow::anyhow!("invalid S3 URL: missing bucket name in `{}`", s))?
                .into(),
            root: splitter.next().unwrap_or_default().to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_s3_from_url() {
        assert_eq!(
            AwsS3Url {
                bucket_name: "test-bucket".into(),
                root: "test-root".into(),
            },
            "s3://test-bucket/test-root".parse().unwrap()
        );
        assert_eq!(
            AwsS3Url {
                bucket_name: "test-bucket".into(),
                root: "test/root".into(),
            },
            "s3://test-bucket/test/root".parse().unwrap()
        );
    }

    #[test]
    fn test_aws_s3_from_url_invalid() {
        assert!("s3:///test-root/".parse::<AwsS3Url>().is_err());
        assert!("s3://".parse::<AwsS3Url>().is_err());
    }

    #[test]
    fn test_aws_s3_display() {
        assert_eq!(
            "s3://test-bucket",
            &AwsS3Url {
                bucket_name: "test-bucket".into(),
                root: "".into(),
            }
            .to_string()
        );
        assert_eq!(
            "s3://test-bucket/test-root",
            &AwsS3Url {
                bucket_name: "test-bucket".into(),
                root: "test-root".into(),
            }
            .to_string()
        );
        assert_eq!(
            "s3://test-bucket/test/root",
            &AwsS3Url {
                bucket_name: "test-bucket".into(),
                root: "test/root".into(),
            }
            .to_string()
        );
    }
}
