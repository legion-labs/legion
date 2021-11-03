use std::time;

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};

use super::{signature_validation::SignatureValidation, Token};

/// Provides JWT validation.
pub struct Validation<'a, T> {
    /// A tolerance for the not-before and expiry times.
    leeway: time::Duration,

    /// The signature validation method.
    ///
    /// If `None`, the signature is not verified which is not recommended.
    signature_validation: Option<T>,

    /// A function that returns the current time.
    time_fn: fn() -> time::SystemTime,

    /// Whether to validate the expiration.
    validate_exp: bool,

    /// Whether to validate the not-before.
    validate_nbf: bool,

    /// A issuer to check.
    iss: Option<&'a str>,

    /// A subject identifier to check.
    sub: Option<&'a str>,

    /// A audience to check.
    aud: Option<&'a str>,
}

impl<T> Default for Validation<'_, T> {
    fn default() -> Self {
        Self {
            leeway: time::Duration::from_secs(0),
            signature_validation: None,
            time_fn: std::time::SystemTime::now,
            validate_exp: true,
            validate_nbf: true,
            iss: None,
            sub: None,
            aud: None,
        }
    }
}

impl<'a, T> Validation<'a, T>
where
    T: SignatureValidation,
{
    /// Sets the leeway for the not-before and expiry times.
    pub fn with_leeway(mut self, leeway: time::Duration) -> Self {
        self.leeway = leeway;
        self
    }

    /// Sets the signature validation method.
    pub fn with_signature_validation(mut self, signature_validation: T) -> Self {
        self.signature_validation = Some(signature_validation);
        self
    }

    /// Sets the time function.
    pub fn with_time_fn(mut self, time_fn: fn() -> time::SystemTime) -> Self {
        self.time_fn = time_fn;
        self
    }

    /// Disables the validation of the expiration.
    pub fn disable_exp_validation(mut self) -> Self {
        self.validate_exp = false;
        self
    }

    /// Disables the validation of the not-before.
    pub fn disable_nbf_validation(mut self) -> Self {
        self.validate_nbf = false;
        self
    }

    /// Sets the issuer to check.
    pub fn with_iss(mut self, iss: &'a str) -> Self {
        self.iss = Some(iss);
        self
    }

    /// Sets the subject identifier to check.
    pub fn with_sub(mut self, sub: &'a str) -> Self {
        self.sub = Some(sub);
        self
    }

    /// Sets the audience to check.
    pub fn with_aud(mut self, aud: &'a str) -> Self {
        self.aud = Some(aud);
        self
    }

    /// Validate the specified token's signature.
    pub(crate) fn validate_signature(&self, token: &Token<'_>) -> anyhow::Result<()> {
        self.signature_validation
            .as_ref()
            .map_or(Ok(()), |signature_validation| {
                token.validate_signature(signature_validation)
            })
    }

    /// Validate the specified token's claims.
    pub(crate) fn validate_claims(&self, token: &Token<'_>) -> anyhow::Result<()> {
        let now = (self.time_fn)();
        let claims: Claims = token
            .to_claims_unsafe()
            .context("failed to read common claims")?;

        if self.validate_exp {
            let exp = time::UNIX_EPOCH
                .checked_add(time::Duration::from_secs(claims.exp))
                .ok_or_else(|| anyhow::anyhow!("invalid exp"))?;

            if exp + self.leeway < now {
                bail!("the token has already expired: {:#?} < {:#?}", exp, now);
            }
        }

        if self.validate_nbf {
            if let Some(nbf) = claims.nbf {
                let nbf = time::UNIX_EPOCH
                    .checked_add(time::Duration::from_secs(nbf))
                    .ok_or_else(|| anyhow::anyhow!("invalid nbf"))?;

                if nbf > now + self.leeway {
                    bail!("the token is not yet valid: {:#?} > {:#?}", nbf, now);
                }
            }
        }

        if let Some(iss) = self.iss {
            match &claims.iss {
                Some(claims_iss) => {
                    if iss != claims_iss {
                        bail!(
                            "the token's issuer is invalid: got `{}` when `{}` was expected",
                            claims_iss,
                            iss,
                        );
                    }
                }
                _ => bail!("the token has no issuer but `{}` was expected", iss),
            }
        }

        if let Some(sub) = self.sub {
            match &claims.sub {
                Some(claims_sub) => {
                    if sub != claims_sub {
                        bail!(
                            "the token's subject identifier is invalid: got `{}` when `{}` was expected",
                            claims_sub,
                            sub,
                        );
                    }
                }
                _ => bail!(
                    "the token has no subject identifier but `{}` was expected",
                    sub
                ),
            }
        }

        if let Some(aud) = self.aud {
            match &claims.aud {
                Some(claims_aud) => {
                    if aud != claims_aud {
                        bail!(
                            "the token's audience is invalid: got `{}` when `{}` was expected",
                            claims_aud,
                            aud,
                        );
                    }
                }
                _ => bail!("the token has no audience but `{}` was expected", aud),
            }
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
struct Claims {
    pub exp: u64,
    pub nbf: Option<u64>,
    pub sub: Option<String>,
    pub iss: Option<String>,
    pub aud: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::authentication::jwt::signature_validation::RsaSignatureValidation;

    use super::*;

    #[derive(Deserialize, PartialEq, Eq, Debug, Clone)]
    struct MyClaims {}

    #[test]
    fn test_validation() {
        let signature_validation = RsaSignatureValidation::new_from_components(
         "6S7asUuzq5Q_3U9rbs-PkDVIdjgmtgWreG5qWPsC9xXZKiMV1AiV9LXyqQsAYpCqEDM3XbfmZqGb48yLhb_XqZaKgSYaC_h2DjM7lgrIQAp9902Rr8fUmLN2ivr5tnLxUUOnMOc2SQtr9dgzTONYW5Zu3PwyvAWk5D6ueIUhLtYzpcB-etoNdL3Ir2746KIy_VUsDwAM7dhrqSK8U2xFCGlau4ikOTtvzDownAMHMrfE7q1B6WZQDAQlBmxRQsyKln5DIsKv6xauNsHRgBAKctUxZG8M4QJIx3S6Aughd3RZC4Ca5Ae9fd8L8mlNYBCrQhOZ7dS0f4at4arlLcajtw",
         "AQAB",
    ).unwrap();

        let validation = Validation::default()
            .with_signature_validation(signature_validation)
            .with_time_fn(|| {
                time::UNIX_EPOCH
                    .checked_add(time::Duration::from_secs(1635964695))
                    .unwrap()
            })
            .with_sub("1234567890");

        let token: Token<'_> = "eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiYWRtaW4iOnRydWUsImlhdCI6MTYzNTk2NDY5NSwiZXhwIjoxNjM1OTY4Mjk1fQ.PombMRzfZo00dGKI6KGqdBMcojVulZSJYRrXytpkPjGoz8A_74rnXmOZGq9UhJobOF8zsek9e4mYTcO30FzBwAXygTKB3S4x7kRnaw6YCzkPo5gzGiPWiNRtSByn58OTU7xYNikA4JJBCSyT6U6nV0yoTjg-sMC2yg7YlvXMRvKL9JIBDdA7xdIWHw7krigG-hpL75WLqAXTzffuFgCAsh739e06ukdacL3QiZ36dqYGXh3A-QA5Ls9xw4ZW_AISO8QoZ4gKUIpWbftl36lvl5ZQht0bO7g9J3RMJ1GuwWj82SCCii7w15oR5o5HVfETvTxPDzy6fnBDkclel1VvKg".try_into().unwrap();

        let claims: MyClaims = token.into_claims(&validation).unwrap();

        assert_eq!(claims, MyClaims {});
    }
}
