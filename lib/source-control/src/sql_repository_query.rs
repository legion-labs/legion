use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use tokio::sync::Mutex;

use crate::{
    create_branches_table, init_commit_database, init_config_database, init_forest_database,
    init_lock_database, init_workspace_registrations_database, insert_config,
    sql::{create_database, drop_database, SqlConnectionPool},
    BlobStorageUrl, Branch, ChangeType, Commit, HashedChange, Lock, RepositoryQuery, Tree,
    TreeNode, TreeNodeType, WorkspaceRegistration,
};

pub enum DatabaseUri {
    Sqlite(String),
    Mysql(String),
}

// access to repository metadata inside a mysql or sqlite database
pub struct SqlRepositoryQuery {
    uri: DatabaseUri,
    pool: Mutex<Option<Arc<SqlConnectionPool>>>,
}

impl SqlRepositoryQuery {
    pub fn new(uri: DatabaseUri) -> Self {
        Self {
            uri,
            pool: Mutex::new(None),
        }
    }

    async fn get_conn(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>> {
        self.get_pool().await?.acquire().await
    }

    async fn get_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
        self.get_pool().await?.begin().await
    }

    async fn get_pool(&self) -> Result<Arc<SqlConnectionPool>> {
        let mut pool = self.pool.lock().await;

        if let Some(pool) = pool.as_ref() {
            Ok(Arc::clone(pool))
        } else {
            let new_pool = Arc::new(match &self.uri {
                DatabaseUri::Sqlite(uri) => SqlConnectionPool::new(uri).await?,
                DatabaseUri::Mysql(uri) => SqlConnectionPool::new(uri).await?,
            });

            *pool = Some(Arc::clone(&new_pool));

            Ok(new_pool)
        }
    }

    async fn initialize_database(
        conn: &mut sqlx::AnyConnection,
        blob_storage_url: &BlobStorageUrl,
    ) -> Result<()> {
        init_config_database(conn).await?;
        init_commit_database(conn).await?;
        init_forest_database(conn).await?;
        create_branches_table(conn).await?;
        init_workspace_registrations_database(conn).await?;
        init_lock_database(conn).await?;
        insert_config(conn, blob_storage_url).await?;

        Ok(())
    }

    async fn initialize_repository_data(&self) -> Result<()> {
        let lock_domain_id = uuid::Uuid::new_v4().to_string();
        let root_tree = Tree::empty();
        let root_hash = root_tree.hash();

        self.save_tree(&root_tree, &root_hash).await?;

        let id = uuid::Uuid::new_v4().to_string();
        let timestamp = Utc::now();
        let initial_commit = Commit::new(
            id,
            whoami::username(),
            String::from("initial commit"),
            Vec::new(),
            root_hash,
            Vec::new(),
            timestamp,
        );

        self.insert_commit(&initial_commit).await?;

        let main_branch = Branch::new(
            String::from("main"),
            initial_commit.id,
            String::new(),
            lock_domain_id,
        );

        self.insert_branch(&main_branch).await?;

        Ok(())
    }
}

#[async_trait]
impl RepositoryQuery for SqlRepositoryQuery {
    async fn ping(&self) -> Result<()> {
        self.get_conn().await?;

        Ok(())
    }

    async fn create_repository(
        &self,
        blob_storage_url: Option<BlobStorageUrl>,
    ) -> Result<BlobStorageUrl> {
        let blob_storage_url = match blob_storage_url {
            Some(blob_storage_url) => blob_storage_url,
            None => {
                return Err(anyhow::anyhow!(
                    "cannot create a SQL repository with no blob storage URL specified"
                ))
            }
        };

        match &self.uri {
            DatabaseUri::Sqlite(uri) => {
                create_database(uri).await?;
            }
            DatabaseUri::Mysql(uri) => {
                create_database(uri).await?;
            }
        }

        let mut conn = self.get_conn().await?;

        Self::initialize_database(&mut conn, &blob_storage_url).await?;
        self.initialize_repository_data().await?;

        Ok(blob_storage_url)
    }

    async fn destroy_repository(&self) -> Result<()> {
        match &self.uri {
            DatabaseUri::Sqlite(uri) => drop_database(uri).await,
            DatabaseUri::Mysql(uri) => drop_database(uri).await,
        }
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        let mut conn = self.get_conn().await?;

        sqlx::query("INSERT INTO workspace_registrations VALUES(?, ?);")
            .bind(workspace_registration.id.clone())
            .bind(workspace_registration.owner.clone())
            .execute(&mut conn)
            .await
            .context("error inserting into workspace_registrations")?;

        Ok(())
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        let mut conn = self.get_conn().await?;

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
        let mut transaction = self.get_transaction().await?;
        update_branch_tr(&mut transaction, branch).await?;

        transaction
            .commit()
            .await
            .context("error in transaction commit for update_branch")
    }

    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>> {
        let mut conn = self.get_conn().await?;

        match sqlx::query(
            "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
        )
        .bind(branch_name)
        .fetch_optional(&mut conn)
        .await
        .context(format!("error fetching branch `{}`", branch_name))?
        {
            None => Ok(None),
            Some(row) => {
                let branch = Branch::new(
                    String::from(branch_name),
                    row.get("head"),
                    row.get("parent"),
                    row.get("lock_domain_id"),
                );
                Ok(Some(branch))
            }
        }
    }

    async fn find_branches_in_lock_domain(&self, lock_domain_id: &str) -> Result<Vec<Branch>> {
        let mut conn = self.get_conn().await?;

        Ok(sqlx::query(
            "SELECT name, head, parent 
             FROM branches
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut conn)
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
        let mut conn = self.get_conn().await?;

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
        let mut conn = self.get_conn().await?;

        let changes = sqlx::query(
            "SELECT relative_path, hash, change_type
             FROM commit_changes
             WHERE commit_id = ?;",
        )
        .bind(id)
        .fetch_all(&mut conn)
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
        .fetch_all(&mut conn)
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
        .fetch_one(&mut conn)
        .await
        .context("error fetching commit")
        .map(|row| {
            let timestamp = DateTime::parse_from_rfc3339(row.get("date_time_utc"))
                .unwrap()
                .into();

            Commit::new(
                String::from(id),
                row.get("owner"),
                row.get("message"),
                changes,
                row.get("root_hash"),
                parents,
                timestamp,
            )
        })
    }

    async fn insert_commit(&self, commit: &Commit) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        insert_commit_tr(&mut transaction, commit).await?;

        transaction
            .commit()
            .await
            .context("error in transaction commit for insert_commit")
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        let stored_branch = match self.uri {
            DatabaseUri::Sqlite(_) => sqlite_read_branch_tr(&mut transaction, &branch.name).await?,
            DatabaseUri::Mysql(_) => mysql_read_branch_tr(&mut transaction, &branch.name).await?,
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
        let mut conn = self.get_conn().await?;

        let res = sqlx::query(
            "SELECT count(*) as count
             FROM commits
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&mut conn)
        .await;
        let row = res.unwrap();
        let count: i32 = row.get("count");
        Ok(count > 0)
    }

    async fn read_tree(&self, tree_hash: &str) -> Result<Tree> {
        let mut conn = self.get_conn().await?;
        let mut directory_nodes: Vec<TreeNode> = Vec::new();
        let mut file_nodes: Vec<TreeNode> = Vec::new();

        let rows = sqlx::query(
            "SELECT name, hash, node_type
             FROM tree_nodes
             WHERE parent_tree_hash = ?
             ORDER BY name;",
        )
        .bind(tree_hash)
        .fetch_all(&mut conn)
        .await
        .context(format!("error fetching tree nodes for {}", tree_hash))?;

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
        let mut conn = self.get_conn().await?;
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
                .execute(&mut conn)
                .await
                .context("error inserting into tree_nodes")?;
        }

        for dir_node in &tree.directory_nodes {
            sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(dir_node.name.clone())
                .bind(dir_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::Directory as i64)
                .execute(&mut conn)
                .await
                .context("error inserting into tree_nodes")?;
        }

        Ok(())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        let mut conn = self.get_conn().await?;

        let row = sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE relative_path = ?
             AND lock_domain_id = ?;",
        )
        .bind(lock.relative_path.clone())
        .bind(lock.lock_domain_id.clone())
        .fetch_one(&mut conn)
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
            .execute(&mut conn)
            .await
            .context("error inserting into locks")?;

        Ok(())
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>> {
        let mut conn = self.get_conn().await?;

        Ok(sqlx::query(
            "SELECT workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?
             AND relative_path=?;",
        )
        .bind(lock_domain_id)
        .bind(canonical_relative_path)
        .fetch_optional(&mut conn)
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
        let mut conn = self.get_conn().await?;

        Ok(sqlx::query(
            "SELECT relative_path, workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut conn)
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
        let mut conn = self.get_conn().await?;

        sqlx::query("DELETE from locks WHERE relative_path=? AND lock_domain_id=?;")
            .bind(canonical_relative_path)
            .bind(lock_domain_id)
            .execute(&mut conn)
            .await
            .context("error clearing lock")?;

        Ok(())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        let mut conn = self.get_conn().await?;

        let row = sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_one(&mut conn)
        .await
        .context("error counting locks")?;

        Ok(row.get("count"))
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        let mut conn = self.get_conn().await?;

        let row = sqlx::query(
            "SELECT blob_storage_spec 
             FROM config;",
        )
        .fetch_one(&mut conn)
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
        .bind(commit.timestamp.to_rfc3339())
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
