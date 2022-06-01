use sqlx::{migrate::Migrator, Row};

use crate::Space;

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

    pub async fn list_spaces(&self) -> Result<Vec<Space>> {
        Ok(
            sqlx::query("SELECT id, description, cordoned, created_at FROM spaces")
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
