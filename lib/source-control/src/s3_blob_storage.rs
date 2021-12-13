use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::TryStreamExt;
use http::Uri;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{compute_file_hash, lz4_decompress, lz4_read, BlobStorage};

pub struct S3BlobStorage {
    bucket_name: String,
    root: PathBuf,
    client: aws_sdk_s3::Client,
    compressed_blob_cache: PathBuf,
}

impl S3BlobStorage {
    pub async fn new(s3uri: &str, compressed_blob_cache: PathBuf) -> Result<Self> {
        let uri = s3uri.parse::<Uri>().unwrap();
        let bucket_name = String::from(uri.host().unwrap());
        let root = PathBuf::from(uri.path().strip_prefix('/').unwrap());
        let config = aws_config::load_from_env().await;
        let client = aws_sdk_s3::Client::new(&config);

        let req = client.get_bucket_location().bucket(&bucket_name);

        req.send()
            .await
            .context(format!("failed to connect to bucket: {}", s3uri))?;

        Ok(Self {
            bucket_name,
            root,
            client,
            compressed_blob_cache,
        })
    }

    async fn blob_exists(&self, hash: &str) -> Result<bool> {
        let path = self.root.join(hash);
        let key = path.to_str().unwrap();
        //we fetch the acl to know if the object exists
        let req_acl = self
            .client
            .get_object_acl()
            .bucket(&self.bucket_name)
            .key(key);

        match req_acl.send().await {
            Ok(_acl) => Ok(true),
            Err(aws_sdk_s3::SdkError::ServiceError { err, raw: _ }) => {
                if let aws_sdk_s3::error::GetObjectAclErrorKind::NoSuchKey(_) = err.kind {
                    Ok(false)
                } else {
                    anyhow::bail!("error fetching acl for {:?}: {}", key, err);
                }
            }
            Err(e) => anyhow::bail!("error fetching acl for {:?}: {:?}", key, e),
        }
    }

    async fn download_blob_to_cache(&self, hash: &str) -> Result<PathBuf> {
        let cache_path = self.compressed_blob_cache.join(hash);
        if cache_path.exists() {
            //todo: validate the compressed file checksum
            return Ok(cache_path);
        }
        let path = self.root.join(hash);
        let s3key = path.to_str().unwrap();
        let req = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(s3key);

        let mut obj_output = req.send().await.context("error downloading blob")?;
        let mut output_file = std::fs::File::create(&cache_path)
            .context(format!("error creating file: {}", cache_path.display()))?;

        while let Some(bytes) = obj_output
            .body
            .try_next()
            .await
            .context("error reading blob stream")?
        {
            output_file
                .write(&bytes)
                .context("failed to write to temp buffer")?;
        }

        Ok(cache_path)
    }
}

#[async_trait]
impl BlobStorage for S3BlobStorage {
    async fn read_blob(&self, hash: &str) -> Result<String> {
        let cache_path = self.download_blob_to_cache(hash).await?;
        lz4_read(&cache_path)
    }

    async fn download_blob(&self, local_path: &Path, hash: &str) -> Result<()> {
        assert!(!hash.is_empty());
        let cache_path = self.download_blob_to_cache(hash).await?;
        lz4_decompress(&cache_path, local_path)?;
        let downloaded_hash = compute_file_hash(local_path)?;
        if hash != downloaded_hash {
            anyhow::bail!(
                "downloaded blob hash does not match for {}",
                local_path.display()
            );
        }

        Ok(())
    }

    async fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<()> {
        let path = self.root.join(hash);
        let key = path.to_str().unwrap();

        if self.blob_exists(hash).await? {
            return Ok(());
        }

        let req = self.client.put_object().bucket(&self.bucket_name).key(key);
        let mut buffer: Vec<u8> = Vec::new();

        let mut encoder = lz4::EncoderBuilder::new()
            .level(10)
            .build(&mut buffer)
            .context("error building encoder")?;
        encoder
            .write(contents)
            .context("error writing to encoder")?;

        encoder.finish().1.context("error finishing encoder")?;

        req.body(aws_sdk_s3::ByteStream::from(buffer))
            .send()
            .await
            .context("error writing to bucket")?;

        Ok(())
    }
}

pub async fn validate_connection_to_bucket(s3uri: &str) -> Result<()> {
    let bogus_blob_cache = std::path::PathBuf::new();
    let _storage = S3BlobStorage::new(s3uri, bogus_blob_cache).await?;
    Ok(())
}
