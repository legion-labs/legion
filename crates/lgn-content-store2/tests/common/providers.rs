use async_trait::async_trait;
use lgn_content_store2::{ContentAddressReader, ContentAddressWriter, Identifier, Result};

pub struct FakeContentAddressProvider {}

impl FakeContentAddressProvider {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_address(id: &Identifier, suffix: &str) -> String {
        format!("http://{}/{}", id, suffix)
    }
}

#[async_trait]
impl ContentAddressReader for FakeContentAddressProvider {
    async fn get_content_read_address(&self, id: &Identifier) -> Result<String> {
        Ok(Self::get_address(id, "read"))
    }
}

#[async_trait]
impl ContentAddressWriter for FakeContentAddressProvider {
    async fn get_content_write_address(&self, id: &Identifier) -> Result<String> {
        Ok(Self::get_address(id, "write"))
    }
}
