use std::time;

/// Provides JWT validation.
pub struct Validation<KeyStore> {
    /// A tolerance for the not-before and expiry times.
    pub leeway: time::Duration,

    // The key store to use for verifying the signature.
    pub key_store: KeyStore,
}

impl<KeyStore> Validation<KeyStore> {
    /// Creates a new validation instance.
    pub fn new(key_store: KeyStore) -> Self {
        Self {
            leeway: time::Duration::from_secs(0),
            key_store,
        }
    }

    /// Sets the leeway for the not-before and expiry times.
    pub fn with_leeway(mut self, leeway: time::Duration) -> Self {
        self.leeway = leeway;
        self
    }
}
