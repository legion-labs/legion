use crate::*;
use futures::TryStreamExt;
use http::Uri;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct S3BlobStorage {
    bucket_name: String,
    root: PathBuf,
    client: s3::Client,
    tokio_runtime: tokio::runtime::Runtime,
    compressed_blob_cache: PathBuf,
}

impl S3BlobStorage {
    pub fn new(s3uri: &str, compressed_blob_cache: PathBuf) -> Result<Self, String> {
        let uri = s3uri.parse::<Uri>().unwrap();
        let bucket_name = String::from(uri.host().unwrap());
        let root = PathBuf::from(uri.path().strip_prefix("/").unwrap());
        let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
        let client = s3::Client::from_env();
        let req = client.get_bucket_location().bucket(&bucket_name);
        if let Err(e) = tokio_runtime.block_on(req.send()) {
            return Err(format!("Error connecting to bucket {}: {}", s3uri, e));
        }
        Ok(Self {
            bucket_name,
            root,
            client,
            tokio_runtime,
            compressed_blob_cache,
        })
    }

    fn blob_exists(&self, hash: &str) -> Result<bool, String> {
        let path = self.root.join(hash);
        let key = path.to_str().unwrap();
        //we fetch the acl to know if the object exists
        let req_acl = self
            .client
            .get_object_acl()
            .bucket(&self.bucket_name)
            .key(key);
        match self.tokio_runtime.block_on(req_acl.send()) {
            Ok(_acl) => Ok(true),
            Err(s3::SdkError::ServiceError { err, raw }) => match err.kind {
                s3::error::GetObjectAclErrorKind::NoSuchKey(_) => Ok(false),
                _ => {
                    let _dummy = raw;
                    Err(format!("error fetching acl: {}", err))
                }
            },
            Err(e) => Err(format!("error fetching acl: {:?}", e)),
        }
    }
}

impl BlobStorage for S3BlobStorage {
    fn read_blob(&self, _hash: &str) -> Result<String, String> {
        Err(String::from("not impl"))
    }

    fn download_blob(&self, local_path: &Path, hash: &str) -> Result<(), String> {
        assert!(!hash.is_empty());
        let path = self.root.join(hash);
        let s3key = path.to_str().unwrap();

        create_parent_directory(local_path)?;

        //todo: don't download files over and over
        let cache_path = self.compressed_blob_cache.join(hash);

        let req = self
            .client
            .get_object()
            .bucket(&self.bucket_name)
            .key(s3key);
        match self.tokio_runtime.block_on(req.send()) {
            Ok(mut obj_output) => match std::fs::File::create(&cache_path) {
                Ok(mut output_file) => loop {
                    match self.tokio_runtime.block_on(obj_output.body.try_next()) {
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
                        local_path.display(),
                        e
                    ));
                }
            },
            Err(e) => {
                return Err(format!("Error downloading blob: {}", e));
            }
        }
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

    fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<(), String> {
        let path = self.root.join(hash);
        let key = path.to_str().unwrap();
        if self.blob_exists(hash)? {
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
        if let Err(e) = self
            .tokio_runtime
            .block_on(req.body(s3::ByteStream::from(buffer)).send())
        {
            return Err(format!("Error writing to bucket {}", e));
        }

        Ok(())
    }
}

pub fn validate_connection_to_bucket(s3uri: &str) -> Result<(), String> {
    let bogus_blob_cache = std::path::PathBuf::new();
    let _storage = S3BlobStorage::new(s3uri, bogus_blob_cache)?;
    Ok(())
}
