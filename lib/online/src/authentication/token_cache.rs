use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use async_trait::async_trait;
use directories::ProjectDirs;
use lgn_telemetry::{debug, warn};
use tokio::sync::{Mutex, MutexGuard};

use super::{jwt::UnsecureValidation, Authenticator, ClientTokenSet, Error, Result};

/// A `TokenCache` stores authentication tokens and handles their lifetime.
pub struct TokenCache<A> {
    project_dirs: ProjectDirs,
    validation: UnsecureValidation<'static>,
    authenticator: Mutex<A>,
}

impl<A> TokenCache<A>
where
    A: Authenticator,
{
    /// Instanciate a new `TokenCache`
    pub fn new(authenticator: A, project_dirs: ProjectDirs) -> Self {
        Self {
            project_dirs,
            validation: UnsecureValidation::default(),
            authenticator: Mutex::new(authenticator),
        }
    }

    /// Get the `Authenticator` used by this `TokenCache`.
    pub async fn authenticator(&self) -> MutexGuard<'_, A> {
        self.authenticator.lock().await
    }

    fn get_tokens_file_path(&self) -> PathBuf {
        self.project_dirs.cache_dir().join("tokens.json")
    }

    /// Read the token set from the cache.
    pub fn read_token_set_from_cache(&self) -> Result<ClientTokenSet> {
        let path = self.get_tokens_file_path();

        let file = File::open(&path).map_err(|e| {
            Error::Internal(format!(
                "reading tokens files from {}: {}",
                path.display(),
                e
            ))
        })?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| {
            Error::Internal(format!("failed to parse JSON token set from cache: {}", e))
        })
    }

    // Write the access token to the cache.
    pub fn write_token_set_to_cache(&self, token_set: &ClientTokenSet) -> Result<()> {
        let path = self.get_tokens_file_path();
        let parent_path = path.parent().unwrap();

        std::fs::create_dir_all(parent_path).map_err(|e| {
            Error::Internal(format!(
                "creating cache directory at {}: {}",
                parent_path.display(),
                e
            ))
        })?;
        let file = File::create(&path).map_err(|e| {
            Error::Internal(format!("creating tokens file at {}: {}", path.display(), e))
        })?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &token_set)
            .map_err(|e| Error::Internal(format!("failed to write JSON token set to cache: {}", e)))
    }

    // Delete the cache.
    pub fn delete_cache(&self) -> Result<()> {
        let path = self.get_tokens_file_path();
        match std::fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(Error::Internal(format!(
                        "deleting cache file at {}: {}",
                        path.display(),
                        e,
                    )))
                }
            }
        }
    }

    async fn refresh_login_with(
        &self,
        refresh_token: &str,
        authenticator: &A,
    ) -> Result<ClientTokenSet> {
        authenticator
            .refresh_login(refresh_token)
            .await
            .map(|token_set| {
                if let Err(err) = self.write_token_set_to_cache(&token_set) {
                    warn!("Failed to write access token to cache: {}", err);
                }

                token_set
            })
    }
}

#[async_trait]
impl<T> Authenticator for TokenCache<T>
where
    T: Authenticator + Send + Sync,
{
    /// Get the access token from the cache if it exists, or performs an
    /// implicit refresh.
    ///
    /// If that fails to, the call will fall back to the `Authenticator`'s
    /// `login` method, which may prompt the user for credentials.
    ///
    /// If the tokens end up being refreshed, they will be stored in the cache.
    async fn login(&self) -> Result<ClientTokenSet> {
        let authenticator = self.authenticator().await;

        let token_set = match self.read_token_set_from_cache() {
            Ok(token_set) => {
                let access_token = &token_set.access_token[..];
                match access_token.try_into() {
                    Ok(access_token) => {
                        if let Err(err) = self.validation.validate_claims(&access_token) {
                            warn!(
                                "Cached access token is invalid ({}): refreshing login...",
                                err
                            );

                            if let Some(refresh_token) = &token_set.refresh_token {
                                return self
                                    .refresh_login_with(refresh_token, &authenticator)
                                    .await;
                            }

                            authenticator.login().await?
                        } else {
                            debug!("Reusing cached access token.");

                            // Bail out immediately because we don't need to refresh the token and
                            // write it to the cache in this case.
                            return Ok(token_set);
                        }
                    }
                    Err(e) => {
                        warn!("invalid access token ({}): cache will be deleted", e);

                        self.delete_cache()?;

                        authenticator.login().await?
                    }
                }
            }
            Err(err) => {
                warn!("Failed to read access token from cache: {}", err);

                authenticator.login().await?
            }
        };

        // If we can't write the token to the cache, we can't do anything about it but
        // warn the user.
        if let Err(err) = self.write_token_set_to_cache(&token_set) {
            warn!("Failed to write access token to cache: {}", err);
        }

        Ok(token_set)
    }

    async fn refresh_login(&self, refresh_token: &str) -> Result<ClientTokenSet> {
        let authenticator = self.authenticator().await;

        self.refresh_login_with(refresh_token, &authenticator).await
    }

    /// Perform a logout, delegating its execution to the owned `Authenticator`
    /// and clearing the cache.
    async fn logout(&self) -> Result<()> {
        let authenticator = self.authenticator().await;

        self.delete_cache()?;

        authenticator.logout().await
    }
}
