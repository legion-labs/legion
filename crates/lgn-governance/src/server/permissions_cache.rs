use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::types::{PermissionId, PermissionSet, SpaceId, UserId};

use super::{Error, Result};

/// A cache for permissions.
pub struct PermissionsCache<Provider> {
    provider: Provider,
    cache: Mutex<lru_time_cache::LruCache<(UserId, Option<SpaceId>), PermissionSet>>,
}

#[async_trait]
pub trait PermissionsProvider {
    async fn get_permissions_for_user(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
    ) -> Result<PermissionSet>;
}

#[async_trait]
impl<Provider: PermissionsProvider + Send + Sync> PermissionsProvider for Arc<Provider> {
    async fn get_permissions_for_user(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
    ) -> Result<PermissionSet> {
        self.as_ref()
            .get_permissions_for_user(user_id, space_id)
            .await
    }
}

impl<Provider> PermissionsCache<Provider> {
    pub fn new(provider: Provider) -> Self {
        Self {
            provider,
            cache: Mutex::new(lru_time_cache::LruCache::with_expiry_duration_and_capacity(
                std::time::Duration::from_secs(600), // We keep credentials for 10 minutes.
                1000, // We keep at most 1000 credentials in the cache. This could probably be buffed-up if required.
            )),
        }
    }

    pub async fn clear(&self) {
        self.cache.lock().await.clear();
    }
}

impl<Provider: PermissionsProvider> PermissionsCache<Provider> {
    /// Checks that a given user has the appropriate permissions, for an
    /// optionally-specified space.
    ///
    /// # Errors
    ///
    /// Returns an error if the user does not have the appropriate permissions.
    pub async fn get_missing_user_permissions(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
        required_permissions: &[PermissionId],
    ) -> Result<PermissionSet> {
        let key = (user_id.clone(), space_id.cloned());

        let mut cache = self.cache.lock().await;

        let user_permissions = match cache.get(&key) {
            Some(permissions) => permissions,
            None => {
                let permissions = self
                    .provider
                    .get_permissions_for_user(user_id, space_id)
                    .await?;

                cache.entry(key).or_insert(permissions)
            }
        };

        let mut missing = PermissionSet::default();

        for permission_id in required_permissions {
            if !user_permissions.contains(permission_id) {
                missing.insert(permission_id.clone());
            }
        }

        Ok(missing)
    }

    /// Returns whether a given user has the appropriate permissions, for an
    /// optionally-specified space.
    ///
    /// # Errors
    ///
    /// Returns an error if the user does not have the appropriate permissions.
    pub async fn user_has_permissions(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
        required_permissions: &[PermissionId],
    ) -> Result<bool> {
        let missing = self
            .get_missing_user_permissions(user_id, space_id, required_permissions)
            .await?;

        Ok(missing.is_empty())
    }

    /// Checks that a given user has the appropriate permissions, for an
    /// optionally-specified space.
    ///
    /// # Errors
    ///
    /// Returns an error if the user does not have the appropriate permissions.
    pub async fn check_user_permissions(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
        required_permissions: &[PermissionId],
    ) -> Result<()> {
        if let Some(first) = self
            .get_missing_user_permissions(user_id, space_id, required_permissions)
            .await?
            .into_iter()
            .next()
        {
            Err(Error::PermissionDenied(
                user_id.clone(),
                first,
                space_id.cloned(),
            ))
        } else {
            Ok(())
        }
    }
}
