use crate::{sql::*, *};
use async_trait::async_trait;

// access to repository metadata inside a mysql or sqlite database
pub struct SqlRepositoryQuery {
    pool: sqlx::AnyPool,
}

impl SqlRepositoryQuery {
    pub fn new(db_uri: &str) -> Result<Self, String> {
        Ok(Self {
            pool: alloc_sql_pool(db_uri)?,
        })
    }
}

#[async_trait]
impl RepositoryQuery for SqlRepositoryQuery {
    async fn insert_workspace(&self, workspace: &Workspace) -> Result<(), String> {
        match self.pool.acquire().await {
            Ok(mut connection) => {
                if let Err(e) = sqlx::query("INSERT INTO workspaces VALUES(?, ?, ?);")
                    .bind(workspace.id.clone())
                    .bind(workspace.root.clone())
                    .bind(workspace.owner.clone())
                    .execute(&mut connection)
                    .await
                {
                    Err(format!("Error inserting into workspaces: {}", e))
                } else {
                    Ok(())
                }
            }
            Err(e) => Err(format!("Error acquiring sql connection: {}", e)),
        }
    }
}
