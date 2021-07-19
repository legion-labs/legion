use crate::*;
use http::Uri;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct S3BlobStorage {
    bucket_name: String,
    root: PathBuf,
    client: s3::Client,
    tokio_runtime: tokio::runtime::Runtime,
}

impl S3BlobStorage {
    pub fn new(s3uri: &str) -> Result<Self, String> {
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

    fn download_blob(&self, _local_path: &Path, hash: &str) -> Result<(), String> {
        assert!(!hash.is_empty());
        Err(String::from("not impl"))
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
    let _storage = S3BlobStorage::new(s3uri)?;
    Ok(())
}
