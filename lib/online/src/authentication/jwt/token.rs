use anyhow::{anyhow, Context};
use serde::{de::DeserializeOwned, Serialize};

use std::str::FromStr;

use super::Header;

/// A JWT token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<Claims> {
    pub header: Header,
    pub payload: Claims,
    pub signature: Vec<u8>,
}

impl<Claims> FromStr for Token<Claims>
where
    Claims: DeserializeOwned,
{
    type Err = anyhow::Error;

    /// Parses a JWT token from a string.
    ///
    /// The string must be in the format `<header>.<payload>.<signature>`.
    ///
    /// The header and payload are base64 encoded.
    /// The signature is base64 encoded and must be a valid signature for the header and payload.
    ///
    /// Note: The signature is **NOT** verified.
    ///
    /// The Claims type must implement `DeserializeOwned`.
    ///
    /// # Example
    /// ```
    /// use serde::Deserialize;
    ///
    /// use legion_online::authentication::jwt::{Header, Token};
    ///
    /// #[derive(Deserialize, Debug, Eq, PartialEq)]
    /// struct Claims {
    ///     sub: String,
    ///     name: String,
    ///     iat: u64,
    /// }
    ///
    /// let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".parse::<Token<Claims>>().unwrap();
    ///
    /// assert_eq!(
    ///     token,
    ///     Token{
    ///         header: Header{
    ///             alg: "HS256".to_string(),
    ///             typ: Some("JWT".to_string()),
    ///             kid: None,
    ///         },
    ///         payload: Claims{
    ///             sub: "1234567890".to_string(),
    ///             name: "John Doe".to_string(),
    ///             iat: 1516239022,
    ///         },
    ///         signature: vec![
    ///             73, 249, 74, 199, 4, 73, 72, 199,
    ///             138, 40, 93, 144, 79, 135, 240, 164,
    ///             199, 137, 127, 126, 143, 58, 78, 178,
    ///             37, 95, 218, 117, 11, 44, 195, 151,
    ///         ],
    ///     }
    /// )
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(3, '.').collect();

        if parts.len() != 3 {
            return Err(anyhow!("invalid token: {}", s));
        }

        let header = Header::from_base64(parts[0])?;
        let payload = serde_json::from_slice(
            base64::decode_config(parts[1], base64::URL_SAFE_NO_PAD)
                .context("failed to decode base64 JWT payload")?
                .as_slice(),
        )
        .context("failed to decode JSON JWT payload")?;
        let signature = base64::decode_config(parts[2], base64::URL_SAFE_NO_PAD)
            .context("failed to decode base64 JWT signature")?;

        Ok(Self {
            header,
            payload,
            signature,
        })
    }
}

impl<Claims> ToString for Token<Claims>
where
    Claims: Serialize,
{
    /// Serializes a JWT token to a string.
    ///
    /// The string is in the format `<header>.<payload>.<signature>`.
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Serialize;
    ///
    /// use legion_online::authentication::jwt::{Header, Token};
    ///
    /// #[derive(Serialize, Debug, Eq, PartialEq)]
    /// struct Claims {
    ///     sub: String,
    ///     name: String,
    ///     iat: u64,
    /// }
    ///
    /// let token = Token{
    ///     header: Header{
    ///         alg: "HS256".to_string(),
    ///         typ: Some("JWT".to_string()),
    ///         kid: None,
    ///     },
    ///     payload: Claims{
    ///         sub: "1234567890".to_string(),
    ///         name: "John Doe".to_string(),
    ///         iat: 1516239022,
    ///     },
    ///     signature: vec![
    ///         73, 249, 74, 199, 4, 73, 72, 199,
    ///         138, 40, 93, 144, 79, 135, 240, 164,
    ///         199, 137, 127, 126, 143, 58, 78, 178,
    ///         37, 95, 218, 117, 11, 44, 195, 151,
    ///     ],
    /// };
    ///
    /// assert_eq!(
    ///     token.to_string(),
    ///     "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
    /// )
    /// ```
    fn to_string(&self) -> String {
        format!(
            "{}.{}.{}",
            self.header.to_base64(),
            base64::encode_config(
                serde_json::to_string(&self.payload).expect("failed to encode JSON JWT payload"),
                base64::URL_SAFE_NO_PAD
            ),
            base64::encode_config(&self.signature, base64::URL_SAFE_NO_PAD),
        )
    }
}
