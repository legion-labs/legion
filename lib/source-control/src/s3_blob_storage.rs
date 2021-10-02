use crate::*;
use async_trait::async_trait;
use futures::TryStreamExt;
use http::Uri;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct S3BlobStorage {
    bucket_name: String,
    root: PathBuf,
    client: s3::Client,
    compressed_blob_cache: PathBuf,
}

impl S3BlobStorage {
    pub async fn new(s3uri: &str, compressed_blob_cache: PathBuf) -> Result<Self, String> {
        let uri = s3uri.parse::<Uri>().unwrap();
        let bucket_name = String::from(uri.host().unwrap());
        let root = PathBuf::from(uri.path().strip_prefix('/').unwrap());
        let config = aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

        let req = client.get_bucket_location().bucket(&bucket_name);
        if let Err(e) = req.send().await {
            return Err(format!("Error connecting to bucket {}: {}", s3uri, e));
        }
        Ok(Self {
            bucket_name,
            root,
            client,
            compressed_blob_cache,
        })
    }

    async fn blob_exists(&self, hash: &str) -> Result<bool, String> {
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
            Err(s3::SdkError::ServiceError { err, raw }) => {
                if let s3::error::GetObjectAclErrorKind::NoSuchKey(_) = err.kind {
                    Ok(false)
                } else {
                    let _dummy = raw;
                    Err(format!("error fetching acl for {:?}: {}", key, err))
                }
            }
            Err(e) => Err(format!("error fetching acl for {:?}: {:?}", key, e)),
        }
    }

    async fn download_blob_to_cache(&self, hash: &str) -> Result<PathBuf, String> {
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
        match req.send().await {
            Ok(mut obj_output) => match std::fs::File::create(&cache_path) {
                Ok(mut output_file) => loop {
                    match obj_output.body.try_next().await {
                        Ok(Some(bytes)) => {
                            if let Err(e) = output_file.write(&bytes) {
                                return Err(format!("Error writing to temp buffer: {}", e));
                            }
                        }
                        Ok(None) => {
                            break;
                        }
                        Err(e) => {
                            return Err(format!("Error reading blob stream: {}", e));
                        }
                    }
                },
                Err(e) => {
                    return Err(format!(
                        "Error creating file {}: {}",
                        cache_path.display(),
                        e
                    ));
                }
            },
            Err(e) => {
                return Err(format!("Error downloading blob: {}", e));
            }
        }
        Ok(cache_path)
    }
}

#[async_trait]
impl BlobStorage for S3BlobStorage {
    async fn read_blob(&self, hash: &str) -> Result<String, String> {
        let cache_path = self.download_blob_to_cache(hash).await?;
        lz4_read(&cache_path)
    }

    async fn download_blob(&self, local_path: &Path, hash: &str) -> Result<(), String> {
        assert!(!hash.is_empty());
        let cache_path = self.download_blob_to_cache(hash).await?;
        lz4_decompress(&cache_path, local_path)?;
        let downloaded_hash = compute_file_hash(local_path)?;
        if hash != downloaded_hash {
            return Err(format!(
                "Downloaded blob hash does not match for {}",
                local_path.display()
            ));
        }
        Ok(())
    }

    async fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<(), String> {
        let path = self.root.join(hash);
        let key = path.to_str().unwrap();
        if self.blob_exists(hash).await? {
            return Ok(());
        }

        let req = self.client.put_object().bucket(&self.bucket_name).key(key);
        let mut buffer: Vec<u8> = Vec::new();
        match lz4::EncoderBuilder::new().level(10).build(&mut buffer) {
            Ok(mut encoder) => {
                if let Err(e) = encoder.write(contents) {
                    return Err(format!("Error writing to lz4 encoder: {}", e));
                }
                if let (_w, Err(e)) = encoder.finish() {
                    return Err(format!("Error closing lz4 encoder: {}", e));
                }
            }
            Err(e) => {
                return Err(format!("Error making lz4 encoder: {}", e));
            }
        }
        if let Err(e) = req.body(s3::ByteStream::from(buffer)).send().await {
            return Err(format!("Error writing to bucket {}", e));
        }

        Ok(())
    }
}

pub async fn validate_connection_to_bucket(s3uri: &str) -> Result<(), String> {
    let bogus_blob_cache = std::path::PathBuf::new();
    let _storage = S3BlobStorage::new(s3uri, bogus_blob_cache).await?;
    Ok(())
}
