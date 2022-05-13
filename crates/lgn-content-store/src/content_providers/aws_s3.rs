use async_trait::async_trait;
use aws_sdk_s3::presigning::config::PresigningConfig;
use bytes::Bytes;
use lgn_tracing::{async_span_scope, span_fn};
use pin_project::pin_project;
use std::future::Future;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{fmt::Display, pin::Pin};
use tokio::io::AsyncWrite;
use tokio_stream::Stream;
use tokio_util::io::StreamReader;

use super::{
    ContentAddressReader, ContentAddressWriter, ContentAsyncReadWithOriginAndSize,
    ContentAsyncWrite, ContentReader, ContentWriter, Error, HashRef, Origin, Result,
    WithOriginAndSize,
};

#[derive(Debug, Clone)]
pub struct AwsS3ContentProvider {
    url: AwsS3Url,
    client: aws_sdk_s3::Client,
    validity_duration: Duration,
}

impl AwsS3ContentProvider {
    /// Generates a new AWS S3 provider using the specified bucket and root.
    ///
    /// The default AWS configuration is used.
    #[span_fn]
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
    #[must_use]
    pub fn with_validity_duration(self, validity_duration: Duration) -> Self {
        Self {
            validity_duration,
            ..self
        }
    }

    fn blob_key(&self, id: &HashRef) -> String {
        if self.url.root.is_empty() {
            id.to_string()
        } else {
            format!("{}/{}", self.url.root, id)
        }
    }

    /// Delete the content with the specified identifier.
    ///
    /// # Errors
    ///
    /// Otherwise, any other error is returned.
    #[span_fn]
    pub async fn delete_content(&self, id: &HashRef) -> Result<()> {
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
    #[span_fn]
    pub async fn check_object_existence(&self, id: &HashRef) -> Result<bool> {
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
            Err(aws_sdk_s3::types::SdkError::ServiceError { err, raw: _ }) => {
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

impl Display for AwsS3ContentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AWS S3 (bucket: {}, root: {}, signed URLs validity duration: {})",
            self.url.bucket_name,
            self.url.root,
            duration_string::DurationString::from(self.validity_duration)
        )
    }
}

#[pin_project]
#[derive(Debug)]
struct ByteStreamReader(#[pin] aws_sdk_s3::types::ByteStream);

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

#[pin_project]
struct ByteStreamWriter {
    client: aws_sdk_s3::Client,
    bucket_name: String,
    key: String,
    #[pin]
    state: ByteStreamWriterState,
}

type ByteStreamWriterBoxedFuture = Box<
    dyn Future<
            Output = std::result::Result<
                aws_sdk_s3::output::PutObjectOutput,
                aws_sdk_s3::types::SdkError<aws_sdk_s3::error::PutObjectError>,
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
            state: ByteStreamWriterState::Writing(Vec::new()),
        }
    }

    fn poll_write_impl(
        self: Pin<&mut Self>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        let this = self.project();

        match &mut this.state.get_mut() {
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

    fn poll_flush_impl(self: Pin<&mut Self>) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();

        match this.state.get_mut() {
            ByteStreamWriterState::Writing(_) => Poll::Ready(Ok(())),
            ByteStreamWriterState::Uploading(_) => Poll::Ready(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot flush an uploading stream",
            ))),
        }
    }

    fn poll_shutdown_impl(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<std::result::Result<(), std::io::Error>> {
        let this = self.project();
        let state = this.state.get_mut();

        let fut = match &mut *state {
            ByteStreamWriterState::Writing(buffer) => {
                let body = aws_sdk_s3::types::ByteStream::from(std::mem::take(buffer));

                let fut = this
                    .client
                    .put_object()
                    .bucket((*this.bucket_name).clone())
                    .key((*this.key).clone())
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
impl ContentReader for AwsS3ContentProvider {
    async fn get_content_reader(&self, id: &HashRef) -> Result<ContentAsyncReadWithOriginAndSize> {
        async_span_scope!("AwsS3Provider::get_content_reader");

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
            Err(aws_sdk_s3::types::SdkError::ServiceError { err, raw: _ }) => {
                if let aws_sdk_s3::error::GetObjectErrorKind::NoSuchKey(_) = err.kind {
                    Err(Error::HashRefNotFound(id.clone()))
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
        let origin = Origin::AwsS3 {
            bucket_name: self.url.bucket_name.clone(),
            key: key.clone(),
        };

        Ok(stream.with_origin_and_size(origin, id.data_size()))
    }
}

#[async_trait]
impl ContentWriter for AwsS3ContentProvider {
    async fn get_content_writer(&self, id: &HashRef) -> Result<ContentAsyncWrite> {
        async_span_scope!("AwsS3Provider::get_content_writer");

        let key = self.blob_key(id);

        match self
            .client
            .get_object()
            .bucket(&self.url.bucket_name)
            .key(&key)
            .send()
            .await
        {
            Ok(_) => Err(Error::HashRefAlreadyExists(id.clone())),
            Err(aws_sdk_s3::types::SdkError::ServiceError { err, raw: _ }) => {
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
impl ContentAddressReader for AwsS3ContentProvider {
    async fn get_content_read_address_with_origin(&self, id: &HashRef) -> Result<(String, Origin)> {
        async_span_scope!("AwsS3Provider::get_content_read_address_with_origin");

        if !self.check_object_existence(id).await? {
            return Err(Error::HashRefNotFound(id.clone()));
        }

        let key = self.blob_key(id);

        Ok((
            self.client
                .get_object()
                .bucket(&self.url.bucket_name)
                .key(&key)
                .presigned(
                    PresigningConfig::expires_in(self.validity_duration).map_err(|err| {
                        anyhow::anyhow!("failed to create presigned URL: {}", err)
                    })?,
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
                .to_string(),
            Origin::AwsS3 {
                bucket_name: self.url.bucket_name.clone(),
                key,
            },
        ))
    }
}

#[async_trait]
impl ContentAddressWriter for AwsS3ContentProvider {
    async fn get_content_write_address(&self, id: &HashRef) -> Result<String> {
        async_span_scope!("AwsS3Provider::get_content_write_address");

        if self.check_object_existence(id).await? {
            return Err(Error::HashRefAlreadyExists(id.clone()));
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

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_aws_s3_content_provider() {
        let s3_prefix = format!(
            "lgn-content-store/test_aws_s3_provider/{}",
            uuid::Uuid::new_v4()
        );
        let aws_s3_url: AwsS3Url = format!("s3://legionlabs-ci-tests/{}", s3_prefix)
            .parse()
            .unwrap();

        let content_provider = AwsS3ContentProvider::new(aws_s3_url.clone()).await;

        let data = &*{
            let mut data = Vec::new();
            let uid = uuid::Uuid::new_v4();

            const BIG_DATA_A: [u8; 128] = [0x41; 128];
            std::io::Write::write_all(&mut data, &BIG_DATA_A).unwrap();
            std::io::Write::write_all(&mut data, uid.as_bytes()).unwrap();

            data
        };

        let origin = Origin::AwsS3 {
            bucket_name: aws_s3_url.bucket_name.clone(),
            key: format!("{}/{}", s3_prefix, HashRef::new_from_data(data)),
        };

        let id = crate::content_providers::test_content_provider(
            &content_provider,
            data,
            origin.clone(),
        )
        .await;

        // Additional tests for the address provider aspect.
        let (read_url, new_origin) = content_provider
            .get_content_read_address_with_origin(&id)
            .await
            .unwrap();

        assert_eq!(origin, new_origin);

        let new_data = reqwest::get(read_url)
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .bytes()
            .await
            .unwrap();

        assert_eq!(new_data, data);

        // Test reading from an address that doesn't exist yet.
        let data = &*{
            let mut data = Vec::new();
            let uid = uuid::Uuid::new_v4();

            const BIG_DATA_B: [u8; 128] = [0x42; 128];
            std::io::Write::write_all(&mut data, &BIG_DATA_B).unwrap();
            std::io::Write::write_all(&mut data, uid.as_bytes()).unwrap();

            data
        };

        let id = HashRef::new_from_data(data);

        match content_provider
            .get_content_read_address_with_origin(&id)
            .await
        {
            Err(Error::HashRefNotFound(err_id)) => assert_eq!(id, err_id),
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(..) => panic!("expected error"),
        }

        let write_url = content_provider
            .get_content_write_address(&id)
            .await
            .unwrap();

        reqwest::Client::new()
            .put(write_url)
            .body(data.to_vec())
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();

        let new_data = crate::ContentReaderExt::read_content(&content_provider, &id)
            .await
            .unwrap();

        assert_eq!(new_data, data);

        // This write should fail as the value already exists.
        match content_provider.get_content_write_address(&id).await {
            Err(Error::HashRefAlreadyExists(err_id)) => assert_eq!(id, err_id),
            Err(e) => panic!("unexpected error: {:?}", e),
            Ok(..) => panic!("expected error"),
        }

        content_provider
            .delete_content(&id)
            .await
            .expect("failed to delete content");
    }
}
