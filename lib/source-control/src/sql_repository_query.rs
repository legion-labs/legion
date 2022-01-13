use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use tokio::sync::Mutex;

use crate::{
    blob_storage::BlobStorageUrl,
    sql::{create_database, drop_database, SqlConnectionPool},
    Branch, ChangeType, Commit, Error, HashedChange, Lock, MapOtherError, RepositoryQuery, Result,
    Tree, TreeNode, TreeNodeType, WorkspaceRegistration,
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
    const TABLE_CONFIGURATION: &'static str = "configuration";
    const TABLE_COMMITS: &'static str = "commits";
    const TABLE_FOREST: &'static str = "forest";
    const TABLE_BRANCHES: &'static str = "branches";
    const TABLE_WORKSPACE_REGISTRATIONS: &'static str = "workspace_registrations";
    const TABLE_LOCKS: &'static str = "locks";

    pub fn new(uri: DatabaseUri) -> Self {
        Self {
            uri,
            pool: Mutex::new(None),
        }
    }

    async fn get_conn(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>> {
        self.get_pool()
            .await?
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")
    }

    async fn get_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
        self.get_pool()
            .await?
            .begin()
            .await
            .map_other_err("failed to acquire SQL transaction")
    }

    async fn get_pool(&self) -> Result<Arc<SqlConnectionPool>> {
        let mut pool = self.pool.lock().await;

        if let Some(pool) = pool.as_ref() {
            Ok(Arc::clone(pool))
        } else {
            let new_pool = Arc::new(match &self.uri {
                DatabaseUri::Sqlite(uri) => SqlConnectionPool::new(uri).await.map_other_err(
                    format!("failed to establish a SQLite connection pool to `{}`", uri),
                )?,
                DatabaseUri::Mysql(uri) => SqlConnectionPool::new(uri).await.map_other_err(
                    format!("failed to establish a MySQL connection pool to `{}`", uri),
                )?,
            });

            *pool = Some(Arc::clone(&new_pool));

            Ok(new_pool)
        }
    }

    async fn initialize_database(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        blob_storage_url: &BlobStorageUrl,
    ) -> Result<()> {
        Self::create_configuration_table(transaction).await?;
        Self::create_commits_database(transaction).await?;
        Self::create_forest_database(transaction).await?;
        Self::create_branches_table(transaction).await?;
        Self::create_workspace_registrations_table(transaction).await?;
        Self::create_locks_table(transaction).await?;

        Self::insert_configuration(transaction, blob_storage_url).await?;

        Ok(())
    }

    async fn create_configuration_table(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        sqlx::query(&format!(
            "CREATE TABLE {}(blob_storage_url TEXT);",
            Self::TABLE_CONFIGURATION
        ))
        .execute(&mut *transaction)
        .await
        .map_other_err("failed to create the configuration table")
        .map(|_| ())
    }

    async fn create_commits_database(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        sqlx::query(&format!("CREATE TABLE {}(id VARCHAR(255), owner VARCHAR(255), message TEXT, root_hash CHAR(64), date_time_utc VARCHAR(255));
         CREATE UNIQUE INDEX commit_id on commits(id);
         CREATE TABLE commit_parents(id VARCHAR(255), parent_id TEXT);
         CREATE INDEX commit_parents_id on commit_parents(id);
         CREATE TABLE commit_changes(commit_id VARCHAR(255), relative_path TEXT, hash CHAR(64), change_type INTEGER);
         CREATE INDEX commit_changes_commit on commit_changes(commit_id);
        ", Self::TABLE_COMMITS))
        .execute(&mut *transaction)
        .await
        .map_other_err("failed to create the commits table")
        .map(|_| ())
    }

    async fn create_forest_database(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        sqlx::query(&format!(
        "CREATE TABLE {} (name VARCHAR(255), hash CHAR(64), parent_tree_hash CHAR(64), node_type INTEGER);
         CREATE INDEX tree on {}(parent_tree_hash);", Self::TABLE_FOREST, Self::TABLE_FOREST))
        .execute(&mut *transaction)
            .await
        .map_other_err("failed to create the forest table and tree index")
        .map(|_| ())
    }

    async fn create_branches_table(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        sqlx::query(&format!("CREATE TABLE {}(name VARCHAR(255), head VARCHAR(255), parent VARCHAR(255), lock_domain_id VARCHAR(64));
         CREATE UNIQUE INDEX branch_name on {}(name);
        ", Self::TABLE_BRANCHES, Self::TABLE_BRANCHES))
        .execute(&mut *transaction)
            .await
        .map_other_err("failed to create the branches table and index")
        .map(|_| ())
    }

    async fn create_workspace_registrations_table(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        sqlx::query(&format!(
            "CREATE TABLE {}(id VARCHAR(255), owner VARCHAR(255));
               CREATE UNIQUE INDEX workspace_registration_id on {}(id);",
            Self::TABLE_WORKSPACE_REGISTRATIONS,
            Self::TABLE_WORKSPACE_REGISTRATIONS
        ))
        .execute(&mut *transaction)
        .await
        .map_other_err("failed to create the workspace registrations table and index")
        .map(|_| ())
    }

    async fn create_locks_table(transaction: &mut sqlx::Transaction<'_, sqlx::Any>) -> Result<()> {
        sqlx::query(&format!(
        "CREATE TABLE {}(relative_path VARCHAR(512), lock_domain_id VARCHAR(64), workspace_id VARCHAR(255), branch_name VARCHAR(255));
         CREATE UNIQUE INDEX lock_key on {}(relative_path, lock_domain_id);
        ", Self::TABLE_LOCKS, Self::TABLE_LOCKS))
        .execute(&mut *transaction)
            .await
        .map_other_err("failed to create the locks table and index")
        .map(|_| ())
    }

    async fn insert_configuration(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        blob_storage: &BlobStorageUrl,
    ) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO {} VALUES(?);",
            Self::TABLE_CONFIGURATION
        ))
        .bind(blob_storage.to_string())
        .execute(&mut *transaction)
        .await
        .map_other_err("failed to insert the configuration")
        .map(|_| ())
    }

    async fn initialize_repository_data(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        let lock_domain_id = uuid::Uuid::new_v4().to_string();
        let root_tree = Tree::empty();
        let root_hash = root_tree.hash();

        Self::save_tree_transactional(transaction, &root_tree, &root_hash).await?;

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

        Self::insert_commit_transactional(transaction, &initial_commit).await?;

        let main_branch = Branch::new(
            String::from("main"),
            initial_commit.id,
            String::new(),
            lock_domain_id,
        );

        Self::insert_branch_transactional(transaction, &main_branch).await?;

        Ok(())
    }

    async fn read_branch_sqlite(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        name: &str,
    ) -> Result<Branch> {
        let row = sqlx::query(&format!(
            "SELECT head, parent, lock_domain_id
             FROM {}
             WHERE name = ?;",
            Self::TABLE_BRANCHES
        ))
        .bind(name)
        .fetch_one(transaction)
        .await
        .map_other_err("failed to read the branch from SQLite")?;

        Ok(Branch::new(
            String::from(name),
            row.get("head"),
            row.get("parent"),
            row.get("lock_domain_id"),
        ))
    }

    async fn read_branch_mysql<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        executor: E,
        name: &str,
    ) -> Result<Branch> {
        let row = sqlx::query(&format!(
            "SELECT head, parent, lock_domain_id
             FROM {}
             WHERE name = ?
             FOR UPDATE;",
            Self::TABLE_BRANCHES
        ))
        .bind(name)
        .fetch_one(executor)
        .await
        .map_other_err("failed to read the branch from MySQL")?;

        Ok(Branch::new(
            String::from(name),
            row.get("head"),
            row.get("parent"),
            row.get("lock_domain_id"),
        ))
    }

    async fn insert_branch_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        branch: &Branch,
    ) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO {} VALUES(?, ?, ?, ?);",
            Self::TABLE_BRANCHES
        ))
        .bind(branch.name.clone())
        .bind(branch.head.clone())
        .bind(branch.parent.clone())
        .bind(branch.lock_domain_id.clone())
        .execute(transaction)
        .await
        .map_other_err(&format!("failed to insert the branch `{}`", &branch.name))
        .map(|_| ())
    }

    async fn insert_commit_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        commit: &Commit,
    ) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO {} VALUES(?, ?, ?, ?, ?);",
            Self::TABLE_COMMITS
        ))
        .bind(commit.id.clone())
        .bind(commit.owner.clone())
        .bind(commit.message.clone())
        .bind(commit.root_hash.clone())
        .bind(commit.timestamp.to_rfc3339())
        .execute(&mut *transaction)
        .await
        .map_other_err(format!("failed to insert the commit `{}`", &commit.id))?;

        for parent_id in &commit.parents {
            sqlx::query("INSERT INTO commit_parents VALUES(?, ?);")
                .bind(commit.id.clone())
                .bind(parent_id.clone())
                .execute(&mut *transaction)
                .await
                .map_other_err(format!(
                    "failed to insert the commit parent `{}` for commit `{}`",
                    parent_id, &commit.id
                ))?;
        }

        for change in &commit.changes {
            sqlx::query("INSERT INTO commit_changes VALUES(?, ?, ?, ?);")
                .bind(commit.id.clone())
                .bind(change.relative_path.clone())
                .bind(change.hash.clone())
                .bind(change.change_type.clone() as i64)
                .execute(&mut *transaction)
                .await
                .map_other_err(format!(
                    "failed to insert the commit change for commit `{}`",
                    &commit.id
                ))?;
        }

        Ok(())
    }

    async fn update_branch_transactional<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        executor: E,
        branch: &Branch,
    ) -> Result<()> {
        sqlx::query(&format!(
            "UPDATE {} SET head=?, parent=?, lock_domain_id=?
             WHERE name=?;",
            Self::TABLE_BRANCHES
        ))
        .bind(branch.head.clone())
        .bind(branch.parent.clone())
        .bind(branch.lock_domain_id.clone())
        .bind(branch.name.clone())
        .execute(executor)
        .await
        .map_other_err(format!("failed to update the `{}` branch", &branch.name))?;

        Ok(())
    }

    async fn save_tree_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        tree: &Tree,
        hash: &str,
    ) -> Result<()> {
        let tree_in_db = Self::read_tree_transactional(&mut *transaction, hash).await?;

        if !tree.is_empty() && !tree_in_db.is_empty() {
            return Ok(());
        }

        for file_node in &tree.file_nodes {
            sqlx::query(&format!(
                "INSERT INTO {} VALUES(?, ?, ?, ?);",
                Self::TABLE_FOREST,
            ))
            .bind(file_node.name.clone())
            .bind(file_node.hash.clone())
            .bind(hash)
            .bind(TreeNodeType::File as i64)
            .execute(&mut *transaction)
            .await
            .map_other_err(&format!(
                "failed to insert file node `{}` into tree `{}`",
                file_node.name, hash
            ))?;
        }

        for dir_node in &tree.directory_nodes {
            sqlx::query(&format!(
                "INSERT INTO {} VALUES(?, ?, ?, ?);",
                Self::TABLE_FOREST
            ))
            .bind(dir_node.name.clone())
            .bind(dir_node.hash.clone())
            .bind(hash)
            .bind(TreeNodeType::Directory as i64)
            .execute(&mut *transaction)
            .await
            .map_other_err(&format!(
                "failed to insert directory node `{}` into tree `{}`",
                dir_node.name, hash
            ))?;
        }

        Ok(())
    }

    async fn read_tree_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        tree_hash: &str,
    ) -> Result<Tree> {
        let mut directory_nodes: Vec<TreeNode> = Vec::new();
        let mut file_nodes: Vec<TreeNode> = Vec::new();

        let rows = sqlx::query(&format!(
            "SELECT name, hash, node_type
             FROM {}
             WHERE parent_tree_hash = ?
             ORDER BY name;",
            Self::TABLE_FOREST
        ))
        .bind(tree_hash)
        .fetch_all(transaction)
        .await
        .map_other_err(&format!(
            "failed to fetch tree nodes for tree `{}`",
            tree_hash
        ))?;

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

    async fn find_lock_transactional<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        executor: E,
        lock_domain_id: &str,
        relative_path: &str,
    ) -> Result<Option<Lock>> {
        Ok(sqlx::query(&format!(
            "SELECT workspace_id, branch_name
             FROM {}
             WHERE lock_domain_id=?
             AND relative_path=?;",
            Self::TABLE_LOCKS,
        ))
        .bind(lock_domain_id)
        .bind(relative_path)
        .fetch_optional(executor)
        .await
        .map_other_err(&format!(
            "failed to find lock `{}` in domain `{}`",
            relative_path, lock_domain_id,
        ))?
        .map(|row| Lock {
            relative_path: String::from(relative_path),
            lock_domain_id: String::from(lock_domain_id),
            workspace_id: row.get("workspace_id"),
            branch_name: row.get("branch_name"),
        }))
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
            None => return Err(Error::NoBlobStorageUrl),
        };

        match &self.uri {
            DatabaseUri::Sqlite(uri) => {
                create_database(uri).await?;
            }
            DatabaseUri::Mysql(uri) => {
                create_database(uri).await?;
            }
        }

        let mut transaction = self.get_transaction().await?;

        Self::initialize_database(&mut transaction, &blob_storage_url).await?;
        Self::initialize_repository_data(&mut transaction).await?;

        transaction
            .commit()
            .await
            .map_other_err("failed to commit transaction when creating repository")?;

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

        sqlx::query(&format!(
            "INSERT INTO {} VALUES(?, ?);",
            Self::TABLE_WORKSPACE_REGISTRATIONS
        ))
        .bind(workspace_registration.id.clone())
        .bind(workspace_registration.owner.clone())
        .execute(&mut conn)
        .await
        .map_other_err(format!(
            "failed to register the workspace `{}` for user `{}`",
            &workspace_registration.id, &workspace_registration.owner,
        ))
        .map(|_| ())
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        Self::insert_branch_transactional(&mut transaction, branch).await?;

        transaction
            .commit()
            .await
            .map_other_err(format!(
                "failed to commit transaction when inserting branch `{}`",
                branch.name
            ))
            .map(|_| ())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        Self::update_branch_transactional(&mut transaction, branch).await?;

        transaction
            .commit()
            .await
            .map_other_err(&format!(
                "failed to commit transaction while updating branch `{}`",
                &branch.name
            ))
            .map(|_| ())
    }

    async fn find_branch(&self, branch_name: &str) -> Result<Option<Branch>> {
        let mut conn = self.get_conn().await?;

        match sqlx::query(&format!(
            "SELECT head, parent, lock_domain_id 
             FROM {}
             WHERE name = ?;",
            Self::TABLE_BRANCHES
        ))
        .bind(branch_name)
        .fetch_optional(&mut conn)
        .await
        .map_other_err(format!("error fetching branch `{}`", branch_name))?
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

        Ok(sqlx::query(&format!(
            "SELECT name, head, parent 
             FROM {}
             WHERE lock_domain_id = ?;",
            Self::TABLE_BRANCHES
        ))
        .bind(lock_domain_id)
        .fetch_all(&mut conn)
        .await
        .map_other_err(&format!(
            "error fetching branches in lock domain `{}`",
            lock_domain_id
        ))?
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

        Ok(sqlx::query(&format!(
            "SELECT name, head, parent, lock_domain_id 
             FROM {};",
            Self::TABLE_BRANCHES
        ))
        .fetch_all(&mut conn)
        .await
        .map_other_err("error fetching branches")?
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
        .map_other_err(format!(
            "failed to fetch commit changes for commit `{}`",
            id
        ))?
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
        .map_other_err(format!("failed to fetch parents for commit `{}`", id))?
        .into_iter()
        .map(|row| row.get("parent_id"))
        .collect();

        sqlx::query(&format!(
            "SELECT owner, message, root_hash, date_time_utc 
             FROM {}
             WHERE id = ?;",
            Self::TABLE_COMMITS
        ))
        .bind(id)
        .fetch_one(&mut conn)
        .await
        .map_other_err(&format!("failed to fetch commit `{}`", id))
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

        Self::insert_commit_transactional(&mut transaction, commit).await?;

        transaction.commit().await.map_other_err(&format!(
            "failed to commit transaction while inserting commit `{}`",
            &commit.id
        ))
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        let stored_branch = match self.uri {
            DatabaseUri::Sqlite(_) => {
                Self::read_branch_sqlite(&mut transaction, &branch.name).await?
            }
            DatabaseUri::Mysql(_) => {
                Self::read_branch_mysql(&mut transaction, &branch.name).await?
            }
        };

        if &stored_branch != branch {
            return Err(Error::stale_branch(stored_branch));
        }

        Self::insert_commit_transactional(&mut transaction, commit).await?;

        let mut new_branch = branch.clone();
        new_branch.head = commit.id.clone();

        Self::update_branch_transactional(&mut transaction, &new_branch).await?;

        transaction.commit().await.map_other_err(&format!(
            "failed to commit transaction while committing commit `{}` to branch `{}`",
            &commit.id, &branch.name
        ))
    }

    async fn commit_exists(&self, id: &str) -> Result<bool> {
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "SELECT count(*) as count
             FROM {}
             WHERE id = ?;",
            Self::TABLE_COMMITS
        ))
        .bind(id)
        .fetch_one(&mut conn)
        .await
        .map_other_err(&format!("failed to check if commit `{}` exists", id))
        .map(|row| row.get::<i32, _>("count") > 0)
    }

    async fn read_tree(&self, tree_hash: &str) -> Result<Tree> {
        let mut conn = self.get_conn().await?;
        let mut directory_nodes: Vec<TreeNode> = Vec::new();
        let mut file_nodes: Vec<TreeNode> = Vec::new();

        let rows = sqlx::query(&format!(
            "SELECT name, hash, node_type
             FROM {}
             WHERE parent_tree_hash = ?
             ORDER BY name;",
            Self::TABLE_FOREST
        ))
        .bind(tree_hash)
        .fetch_all(&mut conn)
        .await
        .map_other_err(&format!(
            "failed to fetch tree nodes for tree `{}`",
            tree_hash
        ))?;

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
        let mut transaction = self.get_transaction().await?;

        Self::save_tree_transactional(&mut transaction, tree, hash).await?;

        transaction.commit().await.map_other_err(&format!(
            "failed to commit transaction while saving tree `{}`",
            hash
        ))
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        if let Some(lock) = Self::find_lock_transactional(
            &mut transaction,
            &lock.lock_domain_id,
            &lock.relative_path,
        )
        .await?
        {
            return Err(Error::lock_already_exists(lock));
        }

        sqlx::query(&format!(
            "INSERT INTO {} VALUES(?, ?, ?, ?);",
            Self::TABLE_LOCKS
        ))
        .bind(lock.relative_path.clone())
        .bind(lock.lock_domain_id.clone())
        .bind(lock.workspace_id.clone())
        .bind(lock.branch_name.clone())
        .execute(&mut transaction)
        .await
        .map_other_err(&format!(
            "failed to insert lock `{}` in domain `{}`",
            lock.relative_path, lock.lock_domain_id,
        ))?;

        transaction
            .commit()
            .await
            .map_other_err(&format!(
                "failed to commit transaction while inserting lock `{}`",
                lock.relative_path
            ))
            .map(|_| ())
    }

    async fn find_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<Option<Lock>> {
        let mut conn = self.get_conn().await?;

        Self::find_lock_transactional(&mut conn, lock_domain_id, relative_path).await
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>> {
        let mut conn = self.get_conn().await?;

        Ok(sqlx::query(&format!(
            "SELECT relative_path, workspace_id, branch_name
             FROM {}
             WHERE lock_domain_id=?;",
            Self::TABLE_LOCKS,
        ))
        .bind(lock_domain_id)
        .fetch_all(&mut conn)
        .await
        .map_other_err(&format!(
            "failed to find locks in domain `{}`",
            lock_domain_id,
        ))?
        .into_iter()
        .map(|row| Lock {
            relative_path: row.get("relative_path"),
            lock_domain_id: String::from(lock_domain_id),
            workspace_id: row.get("workspace_id"),
            branch_name: row.get("branch_name"),
        })
        .collect())
    }

    async fn clear_lock(&self, lock_domain_id: &str, relative_path: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "DELETE from {} WHERE relative_path=? AND lock_domain_id=?;",
            Self::TABLE_LOCKS
        ))
        .bind(relative_path)
        .bind(lock_domain_id)
        .execute(&mut conn)
        .await
        .map_other_err(&format!(
            "failed to clear lock `{}` in domain `{}`",
            relative_path, lock_domain_id,
        ))
        .map(|_| ())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32> {
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "SELECT count(*) as count
             FROM {}
             WHERE lock_domain_id = ?;",
            Self::TABLE_LOCKS,
        ))
        .bind(lock_domain_id)
        .fetch_one(&mut conn)
        .await
        .map_other_err(&format!(
            "failed to count locks in domain `{}`",
            lock_domain_id,
        ))
        .map(|row| row.get::<i32, _>("count"))
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        let mut conn = self.get_conn().await?;

        let row = sqlx::query(&format!(
            "SELECT blob_storage_url
             FROM {};",
            Self::TABLE_CONFIGURATION
        ))
        .fetch_one(&mut conn)
        .await
        .map_other_err("failed to get blob storage url")?;

        row.get::<&str, _>("blob_storage_url")
            .parse()
            .map_other_err("failed to parse blob storage url")
    }
}
