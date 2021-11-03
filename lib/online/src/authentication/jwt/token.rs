use std::ops::Deref;

use anyhow::{anyhow, Context};

use super::Header;

/// A JWT.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'a> {
    raw: &'a str,
    signed: &'a str,
    header: &'a str,
    payload: &'a str,
    signature: &'a str,
}

impl<'a> Token<'a> {
    /// Get the header of the JWT.
    ///
    /// This may fail if the header is not valid.
    pub fn header(&self) -> anyhow::Result<Header> {
        Header::from_base64(self.header).context("invalid base64 JWT header")
    }

    // Get the signature of the JWT.
    //
    // This may fail if the signature is not valid.
    pub fn signature(&self) -> anyhow::Result<Vec<u8>> {
        base64::decode_config(self.signature, base64::URL_SAFE_NO_PAD)
            .context("invalid base64 JWT signature")
    }

    /// Convert the token into its claims.
    ///
    /// This method does not validate the signature and should **NOT** be used most of the time.
    pub fn into_claims_unsafe<Claims>(self) -> anyhow::Result<Claims>
    where
        Claims: serde::de::Deserialize<'a>,
    {
        serde_json::from_str(self.payload).map_err(Into::into)
    }

    /// Convert the token into its claims.
    ///
    /// This method validates the signature and is recommended.
    pub fn into_claims<Claims, SignatureValidation>(
        self,
        signature_validation: &SignatureValidation,
    ) -> anyhow::Result<Claims>
    where
        Claims: serde::de::Deserialize<'a>,
        SignatureValidation: super::signature_validation::SignatureValidation,
    {
        let header = self.header()?;
        let signature = self.signature()?;

        signature_validation
            .validate_signature(&header.alg, self.signed, &signature)
            .ok()?;

        self.into_claims_unsafe()
    }

    /// Validate the signature on the token without consuming it.
    pub fn validate_signature<SignatureValidation>(
        &self,
        signature_validation: &SignatureValidation,
    ) -> anyhow::Result<()>
    where
        SignatureValidation: super::signature_validation::SignatureValidation,
    {
        let header = self.header()?;
        let signature = self.signature()?;

        signature_validation
            .validate_signature(&header.alg, self.signed, &signature)
            .ok()
    }

    pub fn as_str(&self) -> &str {
        self.raw
    }
}

impl<'a> TryFrom<&'a str> for Token<'a> {
    type Error = anyhow::Error;

    /// Parses a JWT from a string.
    ///
    /// The string must be in the format `<header>.<payload>.<signature>`.
    ///
    /// The header, payload and signature are base64 encoded, as per RFC 7519.
    ///
    /// Note: The signature is **NOT** verified.
    ///
    /// # Example
    /// ```
    /// use legion_online::authentication::jwt::Token;
    ///
    /// let raw_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
    /// let token: Token = raw_token.clone().try_into().unwrap();
    ///
    /// assert_eq!(
    ///     token.as_str(),
    ///     raw_token,
    /// );
    /// ```
    fn try_from(raw: &'a str) -> Result<Self, Self::Error> {
        let (signed, signature) = raw.rsplit_once('.').ok_or_else(|| anyhow!("invalid JWT"))?;
        let mut parts = signed.splitn(2, '.');
        let header = parts.next().ok_or_else(|| anyhow!("missing header"))?;
        let payload = parts.next().ok_or_else(|| anyhow!("missing payload"))?;

        Ok(Self {
            raw,
            signed,
            header,
            payload,
            signature,
        })
    }
}

impl<'a> From<Token<'a>> for &'a str {
    fn from(token: Token<'a>) -> Self {
        token.raw
    }
}

impl Deref for Token<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        self.raw
    }
}
