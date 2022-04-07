use std::{fmt::Display, sync::Arc};

use async_trait::async_trait;
use lgn_content_store::{ContentAddressReader, ContentAddressWriter, Error, Identifier, Result};
use tokio::sync::Mutex;

pub struct FakeContentAddressProvider {
    base_url: String,
    already_exists: Arc<Mutex<bool>>,
}

impl Display for FakeContentAddressProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fake provider (base_url: {})", self.base_url)
    }
}

impl FakeContentAddressProvider {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            already_exists: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_address(&self, id: &Identifier, suffix: &str) -> String {
        format!("{}{}/{}", self.base_url, id, suffix)
    }

    pub async fn set_already_exists(&self, exists: bool) {
        *self.already_exists.lock().await = exists;
    }
}

#[async_trait]
impl ContentAddressReader for FakeContentAddressProvider {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        Ok(self.get_address(id, "read"))
    }
}

#[async_trait]
impl ContentAddressWriter for FakeContentAddressProvider {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        if *self.already_exists.lock().await {
            Err(Error::IdentifierAlreadyExists(id.clone()))
        } else {
            Ok(self.get_address(id, "write"))
        }
    }
}
