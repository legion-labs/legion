use std::borrow::Cow;

use sqlx::{migrate::Migrator, Row};

use crate::{types::Permission, PermissionList, Role, RoleList, Space};

use super::Result;

// The SQL migrations.
static MIGRATIONS: Migrator = sqlx::migrate!("migrations/mysql");

pub struct MySqlDal {
    sqlx_pool: sqlx::MySqlPool,
}

impl MySqlDal {
    pub async fn new(sqlx_pool: sqlx::MySqlPool) -> Result<Self> {
        MIGRATIONS.run(&sqlx_pool).await?;

        Ok(Self { sqlx_pool })
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
            .map(|mut permission_list| {
                let mut r = PermissionList::new_built_in();
                r.append(&mut permission_list);
                r
            })
    }

    pub async fn list_roles(&self) -> Result<RoleList> {
        sqlx::query("SELECT id, description, created_at FROM `roles`")
            .fetch_all(&self.sqlx_pool)
            .await?
            .into_iter()
            .map(|row| {
                Ok(Role {
                    id: row.get::<&str, _>(0).parse()?,
                    description: Cow::Owned(row.get(1)),
                    created_at: row.get(3),
                })
            })
            .collect::<Result<_>>()
            .map(|mut role_list| {
                let mut r = RoleList::new_built_in();
                r.append(&mut role_list);
                r
            })
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
