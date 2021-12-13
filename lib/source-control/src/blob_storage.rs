use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait BlobStorage {
    async fn read_blob(&self, hash: &str) -> Result<String>;
    async fn download_blob(&self, local_path: &Path, hash: &str) -> Result<()>;
    async fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<()>;
}
