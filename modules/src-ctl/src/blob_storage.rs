use std::path::Path;

pub trait BlobStorage {
    fn read_blob(&self, hash: &str) -> Result<String, String>;
    fn download_blob(&self, local_path: &Path, hash: &str) -> Result<(), String>;
    fn write_blob(&self, hash: &str, contents: &[u8]) -> Result<(), String>;
}
