use std::borrow::Cow;

use lgn_tracing::info;
use sqlx::{migrate::Migrator, Row};

use crate::{types::Permission, PermissionList, PermissionSet, Role, RoleList, Space};

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
