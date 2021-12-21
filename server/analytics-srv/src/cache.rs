use anyhow::Context;
use anyhow::Result;
use std::path::PathBuf;

pub struct DiskCache {
    _directory: PathBuf,
}

impl DiskCache {
    pub fn new() -> Result<Self> {
        let folder = std::env::var("LEGION_TELEMETRY_CACHE_DIRECTORY").with_context(|| {
            String::from("Error reading env variable LEGION_TELEMETRY_CACHE_DIRECTORY")
        })?;
        let directory = PathBuf::from(folder);
        Ok(Self {
            _directory: directory,
        })
    }

    // pub fn put(&self, name: &str, buffer: &[u8]) {
    //     dbg!((name, buffer.len()));
    // }
}
