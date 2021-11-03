mod aws_cognito_signature_validation;
mod rsa_signature_validation;

pub use aws_cognito_signature_validation::AwsCognitoSignatureValidation;
pub use rsa_signature_validation::RsaSignatureValidation;

/// `ValidationResult` represents the result of a validation.
#[derive(Debug)]
pub enum ValidationResult {
    /// The signature is valid.
    Valid,
    /// The signature is invalid.
    Invalid(anyhow::Error),
    /// The signature has an unsupported format.
    Unsupported,
}

impl ValidationResult {
    /// Returns the current validation result or calls the specified function if the result is
    /// `ValidationResult::Unsupported`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use legion_online::authentication::jwt::signature_validation::ValidationResult::{Valid, Invalid, Unsupported};
    ///
    /// assert!(Unsupported.or_else(|| Valid).is_valid());
    /// assert!(Invalid(anyhow::anyhow!("error")).or_else(|| Valid).is_invalid());
    /// assert!(Valid.or_else(|| Invalid(anyhow::anyhow!("error"))).is_valid());
    /// ```
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        if let Self::Unsupported = self {
            f()
        } else {
            self
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, Self::Invalid(_))
    }

    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported)
    }

    /// Returns the current validation result as a standard result.
    ///
    /// # Example
    ///
    /// ```rust
    /// use legion_online::authentication::jwt::signature_validation::ValidationResult::{Valid, Invalid, Unsupported};
    ///
    /// assert!(Valid.ok().is_ok());
    /// assert!(Invalid(anyhow::anyhow!("error")).ok().is_err());
    /// assert!(Unsupported.ok().is_err());
    /// ```
    pub fn ok(self) -> Result<(), anyhow::Error> {
        match self {
            Self::Valid => Ok(()),
            Self::Invalid(e) => Err(e),
            Self::Unsupported => Err(anyhow::anyhow!("unsupported")),
        }
    }
}

/// A type implementing `SignatureValidation` is able to validate the signature of a JWT.
pub trait SignatureValidation {
    fn validate_signature(
        &self,
        alg: &str,
        kid: Option<&str>,
        message: &str,
        signature: &[u8],
    ) -> ValidationResult;
}

/// Chains two `SignatureValidation`s that will be tried in sequence.
///
/// If the first `SignatureValidation` returns `ValidationResult::Unsupported`, the second one will
/// be tried.
pub struct SignatureValidationChain<First, Second> {
    first: First,
    second: Second,
}

impl<First, Second> SignatureValidationChain<First, Second>
where
    First: SignatureValidation,
    Second: SignatureValidation,
{
    pub fn new(first: First, second: Second) -> Self {
        Self { first, second }
    }
}

impl<First, Second> SignatureValidation for SignatureValidationChain<First, Second>
where
    First: SignatureValidation,
    Second: SignatureValidation,
{
    fn validate_signature(
        &self,
        alg: &str,
        kid: Option<&str>,
        message: &str,
        signature: &[u8],
    ) -> ValidationResult {
        self.first
            .validate_signature(alg, kid, message, signature)
            .or_else(|| self.second.validate_signature(alg, kid, message, signature))
    }
}

/// Chains any number  of `SignatureValidation`s that will be tried in sequence.
///
/// The first `SignatureValidation` that doesn't return `ValidationResult::Unsupported` will stop
/// the chain.
#[macro_export]
macro_rules! chain {
    ($x:expr) => {
        $x
    };
    ($x:expr, $($y:expr),*) => {
        legion_online::authentication::jwt::signature_validation::SignatureValidationChain::new(
            $x,
            chain!($($y),*),
        )
    };
}

pub use chain;
