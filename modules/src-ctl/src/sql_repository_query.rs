use crate::{sql::*, *};
use async_trait::async_trait;
use sqlx::Row;

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

    async fn acquire(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>, String> {
        match self.pool.acquire().await {
            Ok(c) => Ok(c),
            Err(e) => Err(format!("Error acquiring sql connection: {}", e)),
        }
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

    async fn read_branch(&self, name: &str) -> Result<Branch, String> {
        let mut sql_connection = self.acquire().await?;
        match sqlx::query(
            "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
        )
        .bind(name)
        .fetch_one(&mut sql_connection)
        .await
        {
            Ok(row) => {
                let branch = Branch::new(
                    String::from(name),
                    row.get("head"),
                    row.get("parent"),
                    row.get("lock_domain_id"),
                );
                Ok(branch)
            }
            Err(e) => Err(format!("Error fetching branch {}: {}", name, e)),
        }
    }

    async fn read_commit(&self, id: &str) -> Result<Commit, String> {
        let mut sql_connection = self.acquire().await?;
        let mut changes: Vec<HashedChange> = Vec::new();

        match sqlx::query(
            "SELECT relative_path, hash, change_type
             FROM commit_changes
             WHERE commit_id = ?;",
        )
        .bind(id)
        .fetch_all(&mut sql_connection)
        .await
        {
            Ok(rows) => {
                for r in rows {
                    let change_type_int: i64 = r.get("change_type");
                    changes.push(HashedChange {
                        relative_path: r.get("relative_path"),
                        hash: r.get("hash"),
                        change_type: ChangeType::from_int(change_type_int).unwrap(),
                    });
                }
            }
            Err(e) => {
                return Err(format!("Error fetching changes for commit {}: {}", id, e));
            }
        }

        let mut parents: Vec<String> = Vec::new();
        match sqlx::query(
            "SELECT parent_id
             FROM commit_parents
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_all(&mut sql_connection)
        .await
        {
            Ok(rows) => {
                for r in rows {
                    parents.push(r.get("parent_id"));
                }
            }
            Err(e) => {
                return Err(format!("Error fetching parents for commit {}: {}", id, e));
            }
        }

        match sqlx::query(
            "SELECT owner, message, root_hash, date_time_utc 
             FROM commits
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&mut sql_connection)
        .await
        {
            Ok(row) => {
                let commit = Commit::new(
                    String::from(id),
                    row.get("owner"),
                    row.get("message"),
                    changes,
                    row.get("root_hash"),
                    parents,
                );
                Ok(commit)
            }
            Err(e) => Err(format!("Error fetching commit: {}", e)),
        }
    }
}
