use anyhow::Context;
use serde::{Deserialize, Serialize};

/// A JWT 'JOSE' header, as defined by RFC 7519.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub alg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typ: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
}

impl Header {
    /// Creates a new JWT header from its base64 representation.
    ///
    /// # Arguments
    ///
    /// * `base64` - The base64 representation of the header, without its trailing '.'.
    ///
    /// # Errors
    ///
    /// * `base64::DecodeError` - If the base64 representation is invalid.
    /// * `serde_json::Error` - If the header could not be deserialized.
    ///
    /// # Examples
    ///
    /// ```
    /// use legion_online::authentication::jwt::Header;
    ///
    /// let header = Header::from_base64("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9").unwrap();
    ///
    /// assert_eq!(header, Header {
    ///     alg: "HS256".to_string(),
    ///     typ: Some("JWT".to_string()),
    ///     kid: None,
    /// });
    /// ```
    pub fn from_base64(base64: &str) -> anyhow::Result<Self> {
        serde_json::from_slice(
            base64::decode(base64)
                .context("failed to decode base64 JWT header")?
                .as_slice(),
        )
        .context("failed to parse JSON JWT header")
    }

    /// Returns the base64 representation of the header.
    ///
    /// # Examples
    ///
    /// ```
    /// use legion_online::authentication::jwt::Header;
    ///
    /// let header = Header {
    ///     alg: "HS256".to_string(),
    ///     typ: Some("JWT".to_string()),
    ///     kid: None,
    /// };
    ///
    /// assert_eq!(header.to_base64().unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    /// ```
    pub fn to_base64(&self) -> anyhow::Result<String> {
        Ok(base64::encode(
            serde_json::to_string(self).context("failed to serialize header")?,
        ))
    }
}
