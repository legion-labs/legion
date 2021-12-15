use anyhow::{Context, Result};
use std::sync::Arc;

use async_trait::async_trait;
use sqlx::Row;

use crate::{
    sql::SqlConnectionPool, BlobStorageUrl, Branch, ChangeType, Commit, HashedChange, Lock,
    RepositoryQuery, Tree, TreeNode, TreeNodeType, Workspace,
};

pub enum Databases {
    Sqlite,
    Mysql,
}

// access to repository metadata inside a mysql or sqlite database
pub struct SqlRepositoryQuery {
    pool: Arc<SqlConnectionPool>,
    database_kind: Databases,
}

impl SqlRepositoryQuery {
    pub fn new(pool: Arc<SqlConnectionPool>, database_kind: Databases) -> Self {
        Self {
            pool,
            database_kind,
        }
    }
}

#[async_trait]
impl RepositoryQuery for SqlRepositoryQuery {
    async fn insert_workspace(&self, workspace: &Workspace) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query("INSERT INTO workspaces VALUES(?, ?, ?);")
            .bind(workspace.id.clone())
            .bind(workspace.root.clone())
            .bind(workspace.owner.clone())
            .execute(&mut conn)
            .await
            .context("error inserting into workspaces")?;

        Ok(())
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        sqlx::query("INSERT INTO branches VALUES(?, ?, ?, ?);")
            .bind(branch.name.clone())
            .bind(branch.head.clone())
            .bind(branch.parent.clone())
            .bind(branch.lock_domain_id.clone())
            .execute(&mut conn)
            .await
            .context("error inserting into branches")?;

        Ok(())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        let mut transaction = self.pool.begin().await?;
        update_branch_tr(&mut transaction, branch).await?;

        transaction
            .commit()
            .await
            .context("error in transaction commit for update_branch")
    }

    async fn read_branch(&self, name: &str) -> Result<Branch> {
        self.find_branch(name)
            .await?
            .ok_or_else(|| anyhow::anyhow!("branch `{}` not found", name))
    }

    async fn find_branch(&self, name: &str) -> Result<Option<Branch>> {
        let mut conn = self.pool.acquire().await?;

        match sqlx::query(
            "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
        )
        .bind(name)
        .fetch_optional(&mut conn)
        .await
        .context(format!("error fetching branch `{}`", name))?
        {
            None => Ok(None),
            Some(row) => {
                let branch = Branch::new(
                    String::from(name),
                    row.get("head"),
                    row.get("parent"),
                    row.get("lock_domain_id"),
                );
                Ok(Some(branch))
            }
        }
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        let mut sql_connection = self.pool.acquire().await?;

        Ok(sqlx::query(
            "SELECT name, head, parent 
             FROM branches
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut sql_connection)
        .await
        .context("error fetching branches")?
        .into_iter()
        .map(|row| {
            Branch::new(
                row.get("name"),
                row.get("head"),
                row.get("parent"),
                String::from(lock_domain_id),
            )
        })
        .collect())
    }

    async fn read_branches(&self) -> Result<Vec<Branch>> {
        let mut conn = self.pool.acquire().await?;

        Ok(sqlx::query(
            "SELECT name, head, parent, lock_domain_id 
             FROM branches;",
        )
        .fetch_all(&mut conn)
        .await
        .context("error fetching branches")?
        .into_iter()
        .map(|row| {
            Branch::new(
                row.get("name"),
                row.get("head"),
                row.get("parent"),
                row.get("lock_domain_id"),
            )
        })
        .collect())
    }

    async fn read_commit(&self, id: &str) -> Result<Commit> {
        let mut sql_connection = self.pool.acquire().await?;

        let changes = sqlx::query(
            "SELECT relative_path, hash, change_type
             FROM commit_changes
             WHERE commit_id = ?;",
        )
        .bind(id)
        .fetch_all(&mut sql_connection)
        .await
        .context(format!("error fetching commit changes for commit `{}`", id))?
        .into_iter()
        .map(|row| {
            let change_type_int: i64 = row.get("change_type");
            HashedChange {
                relative_path: row.get("relative_path"),
                hash: row.get("hash"),
                change_type: ChangeType::from_int(change_type_int).unwrap(),
            }
        })
        .collect();

        let parents = sqlx::query(
            "SELECT parent_id
             FROM commit_parents
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_all(&mut sql_connection)
        .await
        .context(format!("error fetching parents for commit {}", id))?
        .into_iter()
        .map(|row| row.get("parent_id"))
        .collect();

        sqlx::query(
            "SELECT owner, message, root_hash, date_time_utc 
             FROM commits
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&mut sql_connection)
        .await
        .context("error fetching commit")
        .map(|row| {
            Commit::new(
                String::from(id),
                row.get("owner"),
                row.get("message"),
                changes,
                row.get("root_hash"),
                parents,
            )
        })
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        let mut transaction = self.pool.begin().await?;
        insert_commit_tr(&mut transaction, commit).await?;
        transaction
            .commit()
            .await
            .context("error in transaction commit for insert_commit")
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        let mut transaction = self.pool.begin().await?;
        let stored_branch = match self.database_kind {
            Databases::Sqlite => sqlite_read_branch_tr(&mut transaction, &branch.name).await?,
            Databases::Mysql => mysql_read_branch_tr(&mut transaction, &branch.name).await?,
        };

        if &stored_branch != branch {
            //rollback is implicit but there is bug in sqlx: https://github.com/launchbadge/sqlx/issues/1358
            if let Err(e) = transaction.rollback().await {
                println!("Error in rollback: {}", e);
            }

            anyhow::bail!(
                "commit on stale branch. Branch {} is now at commit {}",
                branch.name,
                stored_branch.head
            );
        }

        insert_commit_tr(&mut transaction, commit).await?;
        let mut new_branch = branch.clone();
        new_branch.head = commit.id.clone();
        update_branch_tr(&mut transaction, &new_branch).await?;

        transaction
            .commit()
            .await
            .context("error in transaction commit for commit_to_branch")
    }

    async fn commit_exists(&self, id: &str) -> Result<bool> {
        let mut sql_connection = self.pool.acquire().await?;
        let res = sqlx::query(
            "SELECT count(*) as count
             FROM commits
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&mut sql_connection)
        .await;
        let row = res.unwrap();
        let count: i32 = row.get("count");
        Ok(count > 0)
    }

    async fn read_tree(&self, hash: &str) -> Result<Tree> {
        let mut sql_connection = self.pool.acquire().await?;
        let mut directory_nodes: Vec<TreeNode> = Vec::new();
        let mut file_nodes: Vec<TreeNode> = Vec::new();

        let rows = sqlx::query(
            "SELECT name, hash, node_type
             FROM tree_nodes
             WHERE parent_tree_hash = ?
             ORDER BY name;",
        )
        .bind(hash)
        .fetch_all(&mut sql_connection)
        .await
        .context(format!("error fetching tree nodes for {}", hash))?;

        for row in rows {
            let name: String = row.get("name");
            let node_hash: String = row.get("hash");
            let node_type: i64 = row.get("node_type");
            let node = TreeNode::new(name, node_hash);

            if node_type == TreeNodeType::Directory as i64 {
                directory_nodes.push(node);
            } else if node_type == TreeNodeType::File as i64 {
                file_nodes.push(node);
            }
        }

        Ok(Tree {
            directory_nodes,
            file_nodes,
        })
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<()> {
        let mut sql_connection = self.pool.acquire().await?;
        let tree_in_db = self.read_tree(hash).await?;

        if !tree.is_empty() && !tree_in_db.is_empty() {
            return Ok(());
        }

        for file_node in &tree.file_nodes {
            sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(file_node.name.clone())
                .bind(file_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::File as i64)
                .execute(&mut sql_connection)
                .await
                .context("error inserting into tree_nodes")?;
        }

        for dir_node in &tree.directory_nodes {
            sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(dir_node.name.clone())
                .bind(dir_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::Directory as i64)
                .execute(&mut sql_connection)
                .await
                .context("error inserting into tree_nodes")?;
        }

        Ok(())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        let mut sql_connection = self.pool.acquire().await?;
        let row = sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE relative_path = ?
             AND lock_domain_id = ?;",
        )
        .bind(lock.relative_path.clone())
        .bind(lock.lock_domain_id.clone())
        .fetch_one(&mut sql_connection)
        .await
        .context("error counting locks")?;

        let count: i32 = row.get("count");
        if count > 0 {
            anyhow::bail!(
                "lock {} already exists in domain {}",
                lock.relative_path,
                lock.lock_domain_id
            );
        }

        sqlx::query("INSERT INTO locks VALUES(?, ?, ?, ?);")
            .bind(lock.relative_path.clone())
            .bind(lock.lock_domain_id.clone())
            .bind(lock.workspace_id.clone())
            .bind(lock.branch_name.clone())
            .execute(&mut sql_connection)
            .await
            .context("error inserting into locks")?;

        Ok(())
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>> {
        let mut sql_connection = self.pool.acquire().await?;
        Ok(sqlx::query(
            "SELECT workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?
             AND relative_path=?;",
        )
        .bind(lock_domain_id)
        .bind(canonical_relative_path)
        .fetch_optional(&mut sql_connection)
        .await
        .context("error fetching lock")?
        .map(|row| Lock {
            relative_path: String::from(canonical_relative_path),
            lock_domain_id: String::from(lock_domain_id),
            workspace_id: row.get("workspace_id"),
            branch_name: row.get("branch_name"),
        }))
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        let mut sql_connection = self.pool.acquire().await?;
        Ok(sqlx::query(
            "SELECT relative_path, workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut sql_connection)
        .await
        .context("error listing locks")?
        .into_iter()
        .map(|row| Lock {
            relative_path: row.get("relative_path"),
            lock_domain_id: String::from(lock_domain_id),
            workspace_id: row.get("workspace_id"),
            branch_name: row.get("branch_name"),
        })
        .collect())
    }

    async fn clear_lock(&self, lock_domain_id: &str, canonical_relative_path: &str) -> Result<()> {
        let mut sql_connection = self.pool.acquire().await?;

        sqlx::query("DELETE from locks WHERE relative_path=? AND lock_domain_id=?;")
            .bind(canonical_relative_path)
            .bind(lock_domain_id)
            .execute(&mut sql_connection)
            .await
            .context("error clearing lock")?;

        Ok(())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        let mut sql_connection = self.pool.acquire().await?;
        let row = sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_one(&mut sql_connection)
        .await
        .context("error counting locks")?;

        Ok(row.get("count"))
    }

    async fn read_blob_storage_spec(&self) -> Result<BlobStorageUrl> {
        let mut sql_connection = self.pool.acquire().await?;
        let row = sqlx::query(
            "SELECT blob_storage_spec 
             FROM config;",
        )
        .fetch_one(&mut sql_connection)
        .await
        .context("error fetching blob storage spec")?;

        row.get::<&str, _>("blob_storage_spec").parse()
    }
}

async fn insert_commit_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    commit: &Commit,
) -> Result<()> {
    sqlx::query("INSERT INTO commits VALUES(?, ?, ?, ?, ?);")
        .bind(commit.id.clone())
        .bind(commit.owner.clone())
        .bind(commit.message.clone())
        .bind(commit.root_hash.clone())
        .bind(commit.date_time_utc.clone())
        .execute(&mut *tr)
        .await
        .context("error inserting into commits")?;

    for parent_id in &commit.parents {
        sqlx::query("INSERT INTO commit_parents VALUES(?, ?);")
            .bind(commit.id.clone())
            .bind(parent_id.clone())
            .execute(&mut *tr)
            .await
            .context("error inserting into commit_parents")?;
    }

    for change in &commit.changes {
        sqlx::query("INSERT INTO commit_changes VALUES(?, ?, ?, ?);")
            .bind(commit.id.clone())
            .bind(change.relative_path.clone())
            .bind(change.hash.clone())
            .bind(change.change_type.clone() as i64)
            .execute(&mut *tr)
            .await
            .context("error inserting into commit_changes")?;
    }

    Ok(())
}

async fn update_branch_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    branch: &Branch,
) -> Result<()> {
    sqlx::query(
        "UPDATE branches SET head=?, parent=?, lock_domain_id=?
             WHERE name=?;",
    )
    .bind(branch.head.clone())
    .bind(branch.parent.clone())
    .bind(branch.lock_domain_id.clone())
    .bind(branch.name.clone())
    .execute(tr)
    .await
    .context("error updating branch")?;

    Ok(())
}

async fn sqlite_read_branch_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    name: &str,
) -> Result<Branch> {
    let row = sqlx::query(
        "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
    )
    .bind(name)
    .fetch_one(tr)
    .await
    .context("error fetching branch")?;

    Ok(Branch::new(
        String::from(name),
        row.get("head"),
        row.get("parent"),
        row.get("lock_domain_id"),
    ))
}

async fn mysql_read_branch_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    name: &str,
) -> Result<Branch> {
    let row = sqlx::query(
        "SELECT head, parent, lock_domain_id
             FROM branches
             WHERE name = ?
             FOR UPDATE;",
    )
    .bind(name)
    .fetch_one(tr)
    .await
    .context("error fetching branch")?;

    Ok(Branch::new(
        String::from(name),
        row.get("head"),
        row.get("parent"),
        row.get("lock_domain_id"),
    ))
}
