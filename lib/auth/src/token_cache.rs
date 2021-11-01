use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use anyhow::Context;
use directories::ProjectDirs;
use log::{debug, info, warn};

use super::Authenticator;
use super::TokenSet;

/// A `TokenCache` stores authentication tokens and handles their lifetime.
pub struct TokenCache {
    authenticator: Authenticator,
    project_dirs: ProjectDirs,
    validation: jsonwebtoken::Validation,
}

impl TokenCache {
    /// Instanciate a new `TokenCache`.
    pub fn new(authenticator: Authenticator, project_dirs: ProjectDirs) -> Self {
        //let project_dirs = ProjectDirs::from("com", "legionlabs", "").unwrap();
        let validation = jsonwebtoken::Validation {
            validate_exp: true,
            algorithms: vec![
                jsonwebtoken::Algorithm::HS256,
                jsonwebtoken::Algorithm::HS384,
                jsonwebtoken::Algorithm::HS512,
                jsonwebtoken::Algorithm::ES256,
                jsonwebtoken::Algorithm::ES384,
                jsonwebtoken::Algorithm::RS256,
                jsonwebtoken::Algorithm::RS384,
                jsonwebtoken::Algorithm::RS512,
                jsonwebtoken::Algorithm::PS256,
                jsonwebtoken::Algorithm::PS384,
                jsonwebtoken::Algorithm::PS512,
            ],
            ..jsonwebtoken::Validation::default()
        };

        Self {
            authenticator,
            project_dirs,
            validation,
        }
    }

    /// Get the `Authenticator` used by this `TokenCache`.
    pub fn authenticator(&self) -> &Authenticator {
        &self.authenticator
    }

    /// Get the access token from the cache if it exists, or performs an implicit refresh.
    ///
    /// If that fails to, the call will fall back to the `Authenticator`'s
    /// `get_access_token_interactive` method, which may prompt the user for credentials.
    ///
    /// If the tokens end up being refreshed, they will be stored in the cache.
    pub async fn get_access_token(&self) -> anyhow::Result<TokenSet> {
        let token_set = match self.read_access_token_from_cache() {
            Ok(token_set) => {
                if self.has_valid_access_token(&token_set) {
                    debug!("Using cached access token");

                    // Bail out immediately because we don't need to refresh the token and write it
                    // to the cache in this case.
                    return Ok(token_set);
                } else if let Some(refresh_token) = &token_set.refresh_token {
                    info!("Refreshing expired access token...");

                    self.authenticator
                        .get_access_token_from_refresh_token(refresh_token)
                        .await?
                } else {
                    info!("No refresh token found, falling back to interactive login");

                    self.authenticator.get_access_token_interactive().await?
                }
            }
            Err(err) => {
                warn!("Failed to read access token from cache: {}", err);

                self.authenticator.get_access_token_interactive().await?
            }
        };

        // If we can't write the token to the cache, we can't do anything about it but warn the user.
        if let Err(err) = self.write_access_token_to_cache(&token_set) {
            warn!("Failed to write access token to cache: {}", err);
        }

        Ok(token_set)
    }

    fn get_tokens_file_path(&self) -> PathBuf {
        self.project_dirs.cache_dir().join("tokens.json")
    }

    /// Read the tokens from the cache.
    pub fn read_access_token_from_cache(&self) -> anyhow::Result<TokenSet> {
        let path = self.get_tokens_file_path();

        let file = File::open(&path)
            .with_context(|| format!("reading tokens files from {}", path.display()))?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(Into::into)
    }

    // Write the access token to the cache.
    pub fn write_access_token_to_cache(&self, token_set: &TokenSet) -> anyhow::Result<()> {
        let path = self.get_tokens_file_path();
        let parent_path = path.parent().unwrap();

        std::fs::create_dir_all(parent_path)
            .with_context(|| format!("creating cache directory at {}", parent_path.display()))?;
        let file = File::create(&path)
            .with_context(|| format!("creating tokens file at {}", path.display()))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &token_set).map_err(Into::into)
    }

    fn has_valid_access_token(&self, token_set: &TokenSet) -> bool {
        match self.validate_token(&token_set.access_token) {
            Ok(_) => true,
            Err(err) => {
                warn!("Failed to validate access token: {}", err);
                false
            }
        }
    }

    fn validate_token(&self, token: &str) -> anyhow::Result<()> {
        // We care about the expiration date but not the signature here.
        jsonwebtoken::dangerous_insecure_decode_with_validation::<Claims>(token, &self.validation)
            .map(|_| ())
            .map_err(Into::into)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {}
