use std::borrow::Cow;

use async_trait::async_trait;
use lgn_tracing::info;
use sqlx::{migrate::Migrator, Row};

use super::PermissionsProvider;
use crate::types::{
    Permission, PermissionId, PermissionList, PermissionSet, Role, RoleId, RoleList,
    RoleUserAssignation, Space, SpaceId, UserId,
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

    pub async fn get_permissions_with_parent(
        &self,
        permission_id: &PermissionId,
    ) -> Result<PermissionSet> {
        // TODO: This would highly benefit from caching.

        sqlx::query("SELECT id FROM `permissions` WHERE parent_id = ?")
            .bind(permission_id)
            .fetch_all(&self.sqlx_pool)
            .await?
            .into_iter()
            .map(|row| {
                row.get::<&str, _>(0)
                    .parse::<PermissionId>()
                    .map_err(Into::into)
            })
            .collect::<Result<_>>()
    }

    pub async fn list_permissions_for_role(&self, role_id: &RoleId) -> Result<PermissionSet> {
        sqlx::query("SELECT permission_id FROM `roles_to_permissions` WHERE `role_id` = ?")
            .bind(role_id)
            .fetch_all(&self.sqlx_pool)
            .await?
            .into_iter()
            .map(|row| row.get::<&str, _>(0).parse().map_err(Into::into))
            .collect::<Result<_>>()
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
            role.permissions =
                sqlx::query("SELECT permission_id FROM `roles_to_permissions` WHERE role_id = ?")
                    .bind(&role.id)
                    .fetch_all(&mut tx)
                    .await?
                    .into_iter()
                    .map(|row| Ok(row.get::<&str, _>(0).parse()?))
                    .collect::<Result<_>>()?;
        }

        tx.commit().await.map(|_| role_list).map_err(Into::into)
    }

    pub async fn list_roles_for_user(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
    ) -> Result<Vec<RoleUserAssignation>> {
        let query = if let Some(space_id) = space_id {
            sqlx::query("SELECT role_id, space_id FROM `users_to_roles` WHERE `user_id` = ? AND (`space_id` IS NULL OR `space_id` == ?)")
            .bind(user_id)
            .bind(space_id)
        } else {
            sqlx::query("SELECT role_id, space_id FROM `users_to_roles` WHERE `user_id` = ? AND `space_id` IS NULL")
            .bind(user_id)
        };

        query
            .fetch_all(&self.sqlx_pool)
            .await?
            .into_iter()
            .map(|row| {
                Ok(RoleUserAssignation {
                    user_id: user_id.clone(),
                    role_id: row.get::<&str, _>(0).parse()?,
                    space_id: row.get(1),
                })
            })
            .collect::<Result<_>>()
    }

    pub async fn list_spaces(&self) -> Result<Vec<Space>> {
        Ok(
            sqlx::query("SELECT id, description, cordoned, created_at FROM `spaces`")
                .fetch_all(&self.sqlx_pool)
                .await?
                .into_iter()
                .map(|row| Space {
                    id: row.get(0),
                    description: row.get(1),
                    cordoned: row.get(2),
                    created_at: row.get(3),
                })
                .collect::<Vec<_>>(),
        )
    }
}

#[async_trait]
impl PermissionsProvider for MySqlDal {
    async fn get_permissions_for_user(
        &self,
        user_id: &UserId,
        space_id: Option<&SpaceId>,
    ) -> Result<PermissionSet> {
        let mut permissions = PermissionSet::default();

        for role_assignation in self.list_roles_for_user(user_id, space_id).await? {
            permissions.extend(
                self.list_permissions_for_role(&role_assignation.role_id)
                    .await?,
            );
        }

        // Now we need to resolve parent permissions as well to make sure we
        // end-up with a complete permissions set.
        let mut unhandled = permissions.clone().into_iter().collect::<Vec<_>>();

        while let Some(permission_id) = unhandled.pop() {
            let child_permissions = self.get_permissions_with_parent(&permission_id).await?;

            for child_permission_id in &child_permissions {
                if !permissions.contains(child_permission_id) {
                    unhandled.push(child_permission_id.clone());
                }
            }

            permissions.extend(child_permissions);
        }

        Ok(permissions)
    }
}
