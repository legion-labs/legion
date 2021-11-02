use std::path::Path;

use async_trait::async_trait;

#[async_trait]
pub trait BlobStorage {
    async fn read_blob(&self, hash: &str) -> Result<String, String>;
    async fn download_blob(&self, local_path: &Path, hash: &str) -> Result<(), String>;
    async fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<(), String>;
}
