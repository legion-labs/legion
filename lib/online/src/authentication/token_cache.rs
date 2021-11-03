use async_trait::async_trait;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::Context;
use directories::ProjectDirs;
use log::{debug, warn};

use super::jwt::signature_validation::SignatureValidation;
use super::jwt::Validation;
use super::Authenticator;
use super::ClientTokenSet;

/// A `TokenCache` stores authentication tokens and handles their lifetime.
pub struct TokenCache<A, V> {
    authenticator: A,
    project_dirs: ProjectDirs,
    validation: Validation<'static, V>,
}

impl<A, V> TokenCache<A, V>
where
    A: Authenticator,
    V: SignatureValidation,
{
    /// Instanciate a new `TokenCache`.
    pub fn new(authenticator: A, project_dirs: ProjectDirs) -> Self {
        Self {
            authenticator,
            project_dirs,
            validation: Validation::default(),
        }
    }

    /// Get the `Authenticator` used by this `TokenCache`.
    pub fn authenticator(&self) -> &A {
        &self.authenticator
    }

    fn get_tokens_file_path(&self) -> PathBuf {
        self.project_dirs.cache_dir().join("tokens.json")
    }

    /// Read the token set from the cache.
    pub fn read_token_set_from_cache(&self) -> anyhow::Result<ClientTokenSet> {
        let path = self.get_tokens_file_path();

        let file = File::open(&path)
            .with_context(|| format!("reading tokens files from {}", path.display()))?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(Into::into)
    }

    // Write the access token to the cache.
    pub fn write_token_set_to_cache(&self, token_set: &ClientTokenSet) -> anyhow::Result<()> {
        let path = self.get_tokens_file_path();
        let parent_path = path.parent().unwrap();

        std::fs::create_dir_all(parent_path)
            .with_context(|| format!("creating cache directory at {}", parent_path.display()))?;
        let file = File::create(&path)
            .with_context(|| format!("creating tokens file at {}", path.display()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &token_set).map_err(Into::into)
    }

    // Delete the cache.
    pub fn delete_cache(&self) -> anyhow::Result<()> {
        let path = self.get_tokens_file_path();
        match std::fs::remove_file(&path) {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e).with_context(|| format!("deleting cache file at {}", path.display()))
                }
            }
        }
    }
}

#[async_trait]
impl<T, V> Authenticator for TokenCache<T, V>
where
    T: Authenticator + Send + Sync,
    V: SignatureValidation + Send + Sync,
{
    /// Get the access token from the cache if it exists, or performs an implicit refresh.
    ///
    /// If that fails to, the call will fall back to the `Authenticator`'s
    /// `get_access_token_interactive` method, which may prompt the user for credentials.
    ///
    /// If the tokens end up being refreshed, they will be stored in the cache.
    async fn login(&self) -> anyhow::Result<ClientTokenSet> {
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
                                return self.refresh_login(refresh_token).await;
                            }

                            self.authenticator.login().await?
                        } else {
                            debug!("Reusing cached access token.");

                            // Bail out immediately because we don't need to refresh the token and write it
                            // to the cache in this case.
                            return Ok(token_set);
                        }
                    }
                    Err(e) => {
                        warn!("invalid access token ({}): cache will be deleted", e);

                        self.delete_cache()?;

                        self.authenticator.login().await?
                    }
                }
            }
            Err(err) => {
                warn!("Failed to read access token from cache: {}", err);

                self.authenticator.login().await?
            }
        };

        // If we can't write the token to the cache, we can't do anything about it but warn the user.
        if let Err(err) = self.write_token_set_to_cache(&token_set) {
            warn!("Failed to write access token to cache: {}", err);
        }

        Ok(token_set)
    }

    async fn refresh_login(&self, refresh_token: &str) -> anyhow::Result<ClientTokenSet> {
        self.authenticator
            .refresh_login(refresh_token)
            .await
            .map(|token_set| {
                if let Err(err) = self.write_token_set_to_cache(&token_set) {
                    warn!("Failed to write access token to cache: {}", err);
                }

                token_set
            })
    }

    /// Perform a logout, delegating its execution to the owned `Authenticator` and clearing the cache.
    async fn logout(&self) -> anyhow::Result<()> {
        self.delete_cache()?;

        self.authenticator.logout().await
    }
}
