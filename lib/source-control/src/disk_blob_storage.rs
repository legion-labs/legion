use anyhow::Result;
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{lz4_compress_to_file, lz4_decompress, lz4_read, BlobStorage};

pub struct DiskBlobStorage {
    pub blob_directory: PathBuf,
}

#[async_trait]
impl BlobStorage for DiskBlobStorage {
    async fn read_blob(&self, hash: &str) -> Result<String> {
        let blob_path = self.blob_directory.join(hash);
        lz4_read(&blob_path)
    }

    async fn download_blob(&self, local_path: &Path, hash: &str) -> Result<()> {
        assert!(!hash.is_empty());
        let blob_path = self.blob_directory.join(hash);
        lz4_decompress(&blob_path, local_path)
    }

    async fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<()> {
        let path = self.blob_directory.join(hash);
        write_blob_to_disk(&path, contents)
    }
}

fn write_blob_to_disk(file_path: &Path, contents: &[u8]) -> Result<()> {
    if fs::metadata(file_path).is_ok() {
        //blob already exists
        return Ok(());
    }

    lz4_compress_to_file(file_path, contents)
}
