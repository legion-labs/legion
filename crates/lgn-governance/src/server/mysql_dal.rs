use std::{
    borrow::Cow,
    collections::{btree_map::Entry, BTreeMap},
};

use async_trait::async_trait;
use lgn_tracing::info;
use sqlx::{migrate::Migrator, mysql::MySqlRow, MySqlConnection, Row};

use super::PermissionsProvider;
use crate::types::{
    Permission, PermissionId, PermissionList, PermissionSet, Role, RoleAssignation, RoleId,
    RoleList, Space, SpaceId, SpaceUpdate, UserId,
};

use super::Result;

// The SQL migrations.
static MIGRATIONS: Migrator = sqlx::migrate!("migrations/mysql");

pub struct MySqlDal {
    sqlx_pool: sqlx::MySqlPool,
}

impl MySqlDal {
    pub async fn new(sqlx_pool: sqlx::MySqlPool) -> Result<Self> {
        MIGRATIONS.run(&sqlx_pool).await?;
        Self::sync_built_ins(&sqlx_pool).await?;

        Ok(Self { sqlx_pool })
    }

    pub async fn sync_built_ins(pool: &sqlx::MySqlPool) -> Result<()> {
        info!("Syncing built-in permissions and roles...");

        let mut tx = pool.begin().await?;

        for permission in Permission::BUILT_INS.iter().copied() {
            sqlx::query(
                "INSERT IGNORE INTO `permissions` (id, description, parent_id) VALUES (?, ?, ?)",
            )
            .bind(&permission.id)
            .bind(&permission.description)
            .bind(&permission.parent_id)
            .execute(&mut tx)
            .await?;
        }

        // Delete any roles/permissions associations that might exist for
        // the built-in roles: this is the only way to make sure the database really
        // reflects the built-ins.
        sqlx::query("DELETE FROM `roles_to_permissions` WHERE built_in = TRUE")
            .execute(&mut tx)
            .await?;

        for role in Role::get_built_ins().iter().copied() {
            sqlx::query("INSERT IGNORE INTO `roles` (id, description) VALUES (?, ?)")
                .bind(&role.id)
                .bind(&role.description)
                .execute(&mut tx)
                .await?;

            for permission_id in &role.permissions {
                sqlx::query(
                    "INSERT INTO `roles_to_permissions` (role_id, permission_id, built_in) VALUES (?, ?, TRUE)",
                )
                .bind(&role.id)
                .bind(&permission_id)
                .execute(&mut tx)
                .await?;
            }
        }

        tx.commit().await.map_err(Into::into)
    }

    pub async fn init_stack(&self, user_id: &UserId) -> Result<bool> {
        let mut tx = self.sqlx_pool.begin().await?;

        let result = if !sqlx::query("SELECT COUNT(1) FROM `users_to_roles` LIMIT 1 FOR UPDATE")
            .fetch_one(&mut tx)
            .await?
            .get::<bool, _>(0)
        {
            sqlx::query("INSERT INTO `users_to_roles` (user_id, role_id) VALUES (?, ?)")
                .bind(user_id)
                .bind(RoleId::SUPERADMIN)
                .execute(&mut tx)
                .await?;

            true
        } else {
            false
        };

        tx.commit().await.map_err(Into::into).map(|_| result)
    }

    pub async fn list_permissions(&self) -> Result<PermissionList> {
        sqlx::query("SELECT id, description, parent_id, created_at FROM `permissions`")
            .fetch_all(&self.sqlx_pool)
            .await?
            .into_iter()
            .map(|row| {
                Ok(Permission {
                    id: row.get::<&str, _>(0).parse()?,
                    description: Cow::Owned(row.get(1)),
                    parent_id: row.get::<Option<&str>, _>(2).map(str::parse).transpose()?,
                    created_at: row.get(3),
                })
            })
            .collect::<Result<_>>()
    }

    pub async fn get_permissions_with_parent_tx<'tx, Tx: sqlx::mysql::MySqlExecutor<'tx>>(
        &self,
        tx: Tx,
        permission_id: &PermissionId,
    ) -> Result<PermissionSet> {
        sqlx::query("SELECT id FROM `permissions` WHERE parent_id = ?")
            .bind(permission_id)
            .fetch_all(tx)
            .await?
            .into_iter()
            .map(|row| {
                row.get::<&str, _>(0)
                    .parse::<PermissionId>()
                    .map_err(Into::into)
            })
            .collect::<Result<_>>()
    }

    async fn list_permissions_for_role_tx<'tx, Tx: sqlx::mysql::MySqlExecutor<'tx>>(
        &self,
        tx: Tx,
        role_id: &RoleId,
    ) -> Result<PermissionSet> {
        sqlx::query("SELECT permission_id FROM `roles_to_permissions` WHERE `role_id` = ?")
            .bind(role_id)
            .fetch_all(tx)
            .await?
            .into_iter()
            .map(|row| row.get::<&str, _>(0).parse().map_err(Into::into))
            .collect::<Result<_>>()
    }

    async fn list_all_permissions_for_role_tx(
        &self,
        tx: &mut MySqlConnection,
        role_id: &RoleId,
    ) -> Result<PermissionSet> {
        let mut permissions = self.list_permissions_for_role_tx(&mut *tx, role_id).await?;

        // Now we need to resolve parent permissions as well to make sure we
        // end-up with a complete permissions set.
        let mut unhandled = permissions.clone().into_iter().collect::<Vec<_>>();

        while let Some(permission_id) = unhandled.pop() {
            let child_permissions = self
                .get_permissions_with_parent_tx(&mut *tx, &permission_id)
                .await?;

            for child_permission_id in &child_permissions {
                if !permissions.contains(child_permission_id) {
                    unhandled.push(child_permission_id.clone());
                }
            }

            permissions.extend(child_permissions);
        }

        Ok(permissions)
    }

    pub async fn list_roles(&self) -> Result<RoleList> {
        let mut tx = self.sqlx_pool.begin().await?;

        let mut role_list = sqlx::query("SELECT id, description, created_at FROM `roles`")
            .fetch_all(&mut tx)
            .await?
            .into_iter()
            .map(|row| {
                Ok(Role {
                    id: row.get::<&str, _>(0).parse()?,
                    description: Cow::Owned(row.get(1)),
                    created_at: row.get(2),
                    permissions: PermissionSet::default(),
                })
            })
            .collect::<Result<RoleList>>()?;

        for role in role_list.iter_mut() {
            role.permissions = self.list_permissions_for_role_tx(&mut tx, &role.id).await?;
        }

        tx.commit().await.map(|_| role_list).map_err(Into::into)
    }

    pub async fn list_all_roles_for_user(&self, user_id: &UserId) -> Result<Vec<RoleAssignation>> {
        self.list_all_roles_for_user_tx(&self.sqlx_pool, user_id)
            .await
    }

    async fn list_roles_for_user_tx<'tx, Tx: sqlx::mysql::MySqlExecutor<'tx>>(
        &self,
        tx: Tx,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
    ) -> Result<Vec<RoleAssignation>> {
        let query = if let Some(space_id) = space_id {
            sqlx::query("SELECT role_id, space_id FROM `users_to_roles` WHERE `user_id` = ? AND (`space_id` IS NULL OR `space_id` = ?)")
            .bind(user_id)
            .bind(space_id)
        } else {
            sqlx::query("SELECT role_id, space_id FROM `users_to_roles` WHERE `user_id` = ? AND `space_id` IS NULL")
            .bind(user_id)
        };

        query
            .fetch_all(tx)
            .await?
            .into_iter()
            .map(|row| {
                Ok(RoleAssignation {
                    role_id: row.get::<&str, _>(0).parse()?,
                    space_id: row.get(1),
                })
            })
            .collect::<Result<_>>()
    }

    async fn list_all_roles_for_user_tx<'tx, Tx: sqlx::mysql::MySqlExecutor<'tx>>(
        &self,
        tx: Tx,
        user_id: &UserId,
    ) -> Result<Vec<RoleAssignation>> {
        sqlx::query("SELECT role_id, space_id FROM `users_to_roles` WHERE `user_id` = ?")
            .bind(user_id)
            .fetch_all(tx)
            .await?
            .into_iter()
            .map(|row| {
                Ok(RoleAssignation {
                    role_id: row.get::<&str, _>(0).parse()?,
                    space_id: row.get(1),
                })
            })
            .collect::<Result<_>>()
    }

    async fn list_all_permissions_by_space_for_user_tx(
        &self,
        tx: &mut MySqlConnection,
        user_id: &UserId,
    ) -> Result<(PermissionSet, BTreeMap<SpaceId, PermissionSet>)> {
        let mut global_permissions = PermissionSet::default();
        let mut permissions_by_space = BTreeMap::new();

        for role_assignation in self.list_all_roles_for_user_tx(&mut *tx, user_id).await? {
            let permissions = self
                .list_all_permissions_for_role_tx(&mut *tx, &role_assignation.role_id)
                .await?;

            if let Some(space_id) = role_assignation.space_id {
                match permissions_by_space.entry(space_id) {
                    Entry::Vacant(entry) => {
                        entry.insert(permissions);
                    }
                    Entry::Occupied(entry) => {
                        entry.into_mut().extend(permissions);
                    }
                }
            } else {
                global_permissions.extend(permissions);
            }
        }

        Ok((global_permissions, permissions_by_space))
    }

    #[allow(clippy::needless_pass_by_value)]
    fn row_to_space(row: MySqlRow) -> Result<Space> {
        Ok(Space {
            id: row.try_get(0)?,
            description: row.try_get(1)?,
            cordoned: row.try_get(2)?,
            created_at: row.try_get(3)?,
        })
    }

    async fn list_spaces_tx(&self, tx: &mut MySqlConnection) -> Result<Vec<Space>> {
        sqlx::query("SELECT id, description, cordoned, created_at FROM `spaces`")
            .fetch_all(tx)
            .await?
            .into_iter()
            .map(Self::row_to_space)
            .collect::<Result<_>>()
    }

    pub async fn list_spaces_for_user(&self, user_id: &UserId) -> Result<Vec<Space>> {
        let mut tx = self.sqlx_pool.begin().await?;

        let (global_permissions, permissions_by_space) = self
            .list_all_permissions_by_space_for_user_tx(&mut *tx, user_id)
            .await?;

        let spaces = if global_permissions.contains(&PermissionId::SPACE_READ) {
            self.list_spaces_tx(&mut *tx).await?
        } else {
            let mut spaces = Vec::with_capacity(permissions_by_space.len());

            for (space_id, permissions) in permissions_by_space {
                if permissions.contains(&PermissionId::SPACE_READ) {
                    spaces.push(self.get_space_tx(&mut *tx, &space_id).await?);
                }
            }

            spaces
        };

        tx.commit().await.map(|_| spaces).map_err(Into::into)
    }

    async fn get_space_tx<'tx, Tx: sqlx::mysql::MySqlExecutor<'tx>>(
        &self,
        tx: Tx,
        space_id: &SpaceId,
    ) -> Result<Space> {
        sqlx::query("SELECT id, description, cordoned, created_at FROM `spaces` WHERE id = ?")
            .bind(space_id)
            .fetch_one(tx)
            .await
            .map(Self::row_to_space)?
    }

    pub async fn get_space(&self, space_id: &SpaceId) -> Result<Space> {
        self.get_space_tx(&self.sqlx_pool, space_id).await
    }

    pub async fn create_space(&self, space_id: &SpaceId, description: &str) -> Result<Space> {
        let mut tx = self.sqlx_pool.begin().await?;

        sqlx::query("INSERT INTO `spaces` (id, description) VALUES (?, ?)")
            .bind(space_id)
            .bind(description)
            .execute(&mut tx)
            .await?;

        let space = self.get_space_tx(&mut tx, space_id).await?;

        tx.commit().await.map(|_| space).map_err(Into::into)
    }

    pub async fn update_space(&self, space_id: &SpaceId, update: SpaceUpdate) -> Result<Space> {
        let mut tx = self.sqlx_pool.begin().await?;

        if let Some(description) = update.description {
            sqlx::query("UPDATE `spaces` SET `description`=? WHERE `id` = ?")
                .bind(description)
                .bind(space_id)
                .execute(&mut tx)
                .await?;
        }

        let space = self.get_space_tx(&mut tx, space_id).await?;

        tx.commit().await.map(|_| space).map_err(Into::into)
    }

    pub async fn delete_space(&self, space_id: &SpaceId) -> Result<Space> {
        let mut tx = self.sqlx_pool.begin().await?;

        let space = self.get_space_tx(&mut tx, space_id).await?;

        sqlx::query("DELETE FROM `spaces` WHERE `id` = ?")
            .bind(space_id)
            .execute(&mut tx)
            .await?;

        tx.commit().await.map(|_| space).map_err(Into::into)
    }

    pub async fn cordon_space(&self, space_id: &SpaceId) -> Result<Space> {
        self.set_space_cordoned(space_id, true).await
    }

    pub async fn uncordon_space(&self, space_id: &SpaceId) -> Result<Space> {
        self.set_space_cordoned(space_id, false).await
    }

    async fn set_space_cordoned(&self, space_id: &SpaceId, cordoned: bool) -> Result<Space> {
        let mut tx = self.sqlx_pool.begin().await?;

        sqlx::query("UPDATE `spaces` SET `cordoned`=? WHERE `id` = ?")
            .bind(cordoned)
            .bind(space_id)
            .execute(&mut tx)
            .await?;

        let space = self.get_space_tx(&mut tx, space_id).await?;

        tx.commit().await.map(|_| space).map_err(Into::into)
    }
}

#[async_trait]
impl PermissionsProvider for MySqlDal {
    async fn get_permissions_for_user(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
    ) -> Result<PermissionSet> {
        let mut tx = self.sqlx_pool.begin().await?;

        let mut permissions = PermissionSet::default();

        for role_assignation in self
            .list_roles_for_user_tx(&mut tx, user_id, space_id)
            .await?
        {
            permissions.extend(
                self.list_all_permissions_for_role_tx(&mut tx, &role_assignation.role_id)
                    .await?,
            );
        }

        tx.commit().await.map(|_| permissions).map_err(Into::into)
    }
}
