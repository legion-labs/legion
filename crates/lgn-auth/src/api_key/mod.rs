mod errors;
mod memory_validation;
mod request_authorizer;

use std::fmt::Formatter;

use async_trait::async_trait;
pub use errors::{Error, Result};
pub use memory_validation::MemoryValidation;
pub use request_authorizer::RequestAuthorizer;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct ApiKey(String);

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey(**redacted**)")
    }
}

impl From<String> for ApiKey {
    fn from(api_key: String) -> Self {
        Self(api_key)
    }
}

/// A trait for types that can validate API keys.
#[async_trait]
pub trait ApiKeyValidator {
    async fn validate_api_key(&self, api_key: ApiKey) -> Result<()>;
}
