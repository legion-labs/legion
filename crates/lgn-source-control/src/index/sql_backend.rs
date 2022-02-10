use async_recursion::async_recursion;
use async_trait::async_trait;
use chrono::DateTime;
use reqwest::Url;
use sqlx::{migrate::MigrateDatabase, Acquire, Executor, Row};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};
use tokio::sync::Mutex;

use crate::{
    sql::SqlConnectionPool, BlobStorageUrl, Branch, CanonicalPath, Change, ChangeType, Commit,
    Error, FileInfo, IndexBackend, ListBranchesQuery, ListCommitsQuery, ListLocksQuery, Lock,
    MapOtherError, Result, Tree, WorkspaceRegistration,
};

#[derive(Debug)]
enum SqlDatabaseDriver {
    Sqlite(String),
    Mysql(String),
}

impl SqlDatabaseDriver {
    fn new(url: String) -> Result<Self> {
        if url.starts_with("mysql://") {
            Ok(Self::Mysql(url))
        } else if url.starts_with("sqlite://") {
            Ok(Self::Sqlite(url))
        } else {
            Err(Error::invalid_index_url(
                url.clone(),
                anyhow::anyhow!(
                    "unsupported SQL database driver: {}",
                    url.split(':').next().unwrap_or_default()
                ),
            ))
        }
    }

    fn url(&self) -> &str {
        match self {
            Self::Sqlite(url) | Self::Mysql(url) => url,
        }
    }

    async fn new_pool(&self) -> Result<SqlConnectionPool> {
        Ok(match &self {
            Self::Sqlite(uri) => SqlConnectionPool::new(uri).await.map_other_err(format!(
                "failed to establish a SQLite connection pool to `{}`",
                uri
            ))?,
            Self::Mysql(uri) => SqlConnectionPool::new(uri).await.map_other_err(format!(
                "failed to establish a MySQL connection pool to `{}`",
                uri
            ))?,
        })
    }

    async fn create_database(&self) -> Result<()> {
        match &self {
            Self::Sqlite(uri) | Self::Mysql(uri) => sqlx::Any::create_database(uri)
                .await
                .map_other_err("failed to create database"),
        }
    }

    async fn drop_database(&self) -> Result<()> {
        match &self {
            Self::Sqlite(uri) | Self::Mysql(uri) => sqlx::Any::drop_database(uri)
                .await
                .map_other_err("failed to drop database"),
        }
    }

    async fn check_if_database_exists(&self) -> Result<bool> {
        match &self {
            Self::Sqlite(uri) | Self::Mysql(uri) => sqlx::Any::database_exists(uri)
                .await
                .map_other_err("failed to check if database exists"),
        }
    }
}

// access to repository metadata inside a mysql or sqlite database
pub struct SqlIndexBackend {
    driver: SqlDatabaseDriver,
    pool: Mutex<Option<Arc<SqlConnectionPool>>>,
    blob_storage_url: BlobStorageUrl,
}

impl core::fmt::Debug for SqlIndexBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SqlIndexBackend")
            .field("driver", &self.driver)
            .field("blob_storage_url", &self.blob_storage_url)
            .finish()
    }
}

impl SqlIndexBackend {
    const TABLE_COMMITS: &'static str = "commits";
    const TABLE_COMMIT_PARENTS: &'static str = "commit_parents";
    const TABLE_COMMIT_CHANGES: &'static str = "commit_changes";
    const TABLE_FOREST: &'static str = "forest";
    const TABLE_FOREST_LINKS: &'static str = "forest_links";
    const TABLE_BRANCHES: &'static str = "branches";
    const TABLE_WORKSPACE_REGISTRATIONS: &'static str = "workspace_registrations";
    const TABLE_LOCKS: &'static str = "locks";

    /// Instanciate a new SQL index backend.
    ///
    /// The url is expected to contain a `blob_storage_url` parameter.
    pub fn new(url: String) -> Result<Self> {
        let mut sql_url = Url::parse(&url).map_other_err(format!("invalid SQL URL: {}", url))?;

        let blob_storage_url = sql_url
            .query_pairs()
            .find(|(k, _)| k == "blob_storage_url")
            .ok_or_else(|| {
                Error::invalid_index_url(
                    url.clone(),
                    anyhow::anyhow!("missing `blob_storage_url` parameter in SQL index URL"),
                )
            })
            .map(|(_, v)| v.to_string())?;

        let blob_storage_url = blob_storage_url.parse().map_other_err(format!(
            "failed to parse `blob_storage_url` parameter in SQL index URL: {}",
            blob_storage_url
        ))?;

        let old_sql_url = sql_url.clone();

        sql_url.query_pairs_mut().clear().extend_pairs(
            old_sql_url
                .query_pairs()
                .filter(|(k, _)| k != "blob_storage_url"),
        );

        let url = if let Some(idx) = url.find('?') {
            format!(
                "{}?{}",
                url.split_at(idx).0,
                sql_url.query().unwrap_or_default(),
            )
        } else {
            url
        }
        .trim_end_matches('?')
        .to_string();

        let driver = SqlDatabaseDriver::new(url)?;

        Ok(Self {
            driver,
            pool: Mutex::new(None),
            blob_storage_url,
        })
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
            let new_pool = Arc::new(self.driver.new_pool().await?);

            *pool = Some(Arc::clone(&new_pool));

            Ok(new_pool)
        }
    }

    async fn initialize_database(conn: &mut sqlx::AnyConnection) -> Result<()> {
        Self::create_commits_table(conn).await?;
        Self::create_forest_table(conn).await?;
        Self::create_branches_table(conn).await?;
        Self::create_workspace_registrations_table(conn).await?;
        Self::create_locks_table(conn).await?;

        Ok(())
    }

    async fn create_commits_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
        "CREATE TABLE `{}` (id VARCHAR(255), owner VARCHAR(255), message TEXT, root_hash CHAR(64), date_time_utc VARCHAR(255), UNIQUE (id));
         CREATE TABLE `{}` (id VARCHAR(255), parent_id TEXT);
         CREATE INDEX commit_parents_id on `{}`(id);
         CREATE TABLE `{}` (commit_id VARCHAR(255) NOT NULL, canonical_path TEXT NOT NULL, old_hash VARCHAR(255), new_hash VARCHAR(255), old_size INTEGER, new_size INTEGER);
         CREATE INDEX commit_changes_commit on `{}`(commit_id);",
         Self::TABLE_COMMITS,
         Self::TABLE_COMMIT_PARENTS,
         Self::TABLE_COMMIT_PARENTS,
         Self::TABLE_COMMIT_CHANGES,
         Self::TABLE_COMMIT_CHANGES
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the commits table")
            .map(|_| ())
    }

    async fn create_forest_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}` (id VARCHAR(255) PRIMARY KEY, name VARCHAR(255), hash VARCHAR(255), size INTEGER);
            CREATE TABLE `{}` (id VARCHAR(255), child_id VARCHAR(255) NOT NULL, CONSTRAINT unique_link UNIQUE (id, child_id), FOREIGN KEY (id) REFERENCES `{}`(id), FOREIGN KEY (child_id) REFERENCES `{}`(id));
            CREATE INDEX forest_links_index on `{}`(id);",
            Self::TABLE_FOREST,
            Self::TABLE_FOREST_LINKS,
            Self::TABLE_FOREST,
            Self::TABLE_FOREST,
            Self::TABLE_FOREST_LINKS,
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the forest table and tree index")
            .map(|_| ())
    }

    async fn create_branches_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
        "CREATE TABLE `{}` (name VARCHAR(255) PRIMARY KEY, head VARCHAR(255), lock_domain_id VARCHAR(64));
        CREATE INDEX branches_lock_domain_ids_index on `{}`(lock_domain_id);",
        Self::TABLE_BRANCHES,
        Self::TABLE_BRANCHES
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the branches table and index")
            .map(|_| ())
    }

    async fn create_workspace_registrations_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}` (id VARCHAR(255), owner VARCHAR(255), UNIQUE (id));",
            Self::TABLE_WORKSPACE_REGISTRATIONS,
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the workspace registrations table and index")
            .map(|_| ())
    }

    async fn create_locks_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
        "CREATE TABLE `{}` (lock_domain_id VARCHAR(64), canonical_path VARCHAR(512), workspace_id VARCHAR(255), branch_name VARCHAR(255), UNIQUE (lock_domain_id, canonical_path));
        ",
        Self::TABLE_LOCKS
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the locks table and index")
            .map(|_| ())
    }

    async fn initialize_repository_data(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    ) -> Result<()> {
        let tree = Tree::empty();
        let tree_id = Self::save_tree_transactional(transaction, &tree).await?;

        let initial_commit = Commit::new_unique_now(
            whoami::username(),
            String::from("initial commit"),
            BTreeSet::new(),
            tree_id,
            BTreeSet::new(),
        );

        Self::insert_commit_transactional(transaction, &initial_commit).await?;

        let main_branch = Branch::new(String::from("main"), initial_commit.id);

        Self::insert_branch_transactional(transaction, &main_branch).await?;

        Ok(())
    }

    async fn read_branch_for_update<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<Branch> {
        let query = match &self.driver {
            SqlDatabaseDriver::Sqlite(_) => format!(
                "SELECT head, lock_domain_id
                     FROM `{}`
                     WHERE name = ?;",
                Self::TABLE_BRANCHES
            ),
            SqlDatabaseDriver::Mysql(_) => format!(
                "SELECT head, lock_domain_id
                     FROM `{}`
                     WHERE name = ?
                     FOR UPDATE;",
                Self::TABLE_BRANCHES
            ),
        };

        let row = sqlx::query(&query)
            .bind(name)
            .fetch_one(executor)
            .await
            .map_other_err("failed to read the branch from MySQL")?;

        Ok(Branch {
            name: name.to_string(),
            head: row.get("head"),
            lock_domain_id: row.get("lock_domain_id"),
        })
    }

    async fn insert_branch_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        branch: &Branch,
    ) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, ?, ?);",
            Self::TABLE_BRANCHES
        ))
        .bind(&branch.name)
        .bind(&branch.head)
        .bind(&branch.lock_domain_id)
        .execute(transaction)
        .await
        .map_other_err(&format!("failed to insert the branch `{}`", &branch.name))
        .map(|_| ())
    }

    async fn list_commits_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        query: &ListCommitsQuery<'_>,
    ) -> Result<Vec<Commit>> {
        let mut result = Vec::new();
        result.reserve(1024);
        let mut next_commit_ids: VecDeque<String> =
            query.commit_ids.iter().copied().map(Into::into).collect();

        let mut depth = Some(query.depth).filter(|d| *d > 0);

        while let Some(commit_id) = next_commit_ids.pop_front() {
            if result.len() >= result.capacity() {
                break;
            }

            if let Some(depth) = &mut depth {
                if *depth == 0 {
                    break;
                }

                *depth -= 1;
            }

            let changes = sqlx::query(&format!(
                "SELECT canonical_path, old_hash, new_hash, old_size, new_size
             FROM `{}`
             WHERE commit_id = ?;",
                Self::TABLE_COMMIT_CHANGES,
            ))
            .bind(&commit_id)
            .fetch_all(&mut *transaction)
            .await
            .map_other_err(format!(
                "failed to fetch commit changes for commit `{}`",
                &commit_id
            ))?
            .into_iter()
            .filter_map(|row| {
                if let Ok(canonical_path) = CanonicalPath::new(row.get("canonical_path")) {
                    let old_hash: String = row.get("old_hash");

                    let old_info = if !old_hash.is_empty() {
                        let old_size: i64 = row.get("old_size");

                        Some(FileInfo {
                            hash: old_hash,
                            size: old_size as u64,
                        })
                    } else {
                        None
                    };

                    let new_hash: String = row.get("new_hash");
                    let new_info = if !new_hash.is_empty() {
                        let new_size: i64 = row.get("new_size");

                        Some(FileInfo {
                            hash: new_hash,
                            size: new_size as u64,
                        })
                    } else {
                        None
                    };

                    ChangeType::new(old_info, new_info)
                        .map(|change_type| Change::new(canonical_path, change_type))
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>();

            let parents: BTreeSet<String> = sqlx::query(&format!(
                "SELECT parent_id
             FROM `{}`
             WHERE id = ?;",
                Self::TABLE_COMMIT_PARENTS,
            ))
            .bind(&commit_id)
            .fetch_all(&mut *transaction)
            .await
            .map_other_err(format!(
                "failed to fetch parents for commit `{}`",
                &commit_id
            ))?
            .into_iter()
            .map(|row| row.get("parent_id"))
            .collect();

            next_commit_ids.extend(parents.iter().cloned());

            result.push(
                match sqlx::query(&format!(
                    "SELECT owner, message, root_hash, date_time_utc 
             FROM `{}`
             WHERE id = ?;",
                    Self::TABLE_COMMITS
                ))
                .bind(&commit_id)
                .fetch_one(&mut *transaction)
                .await
                {
                    Ok(row) => {
                        let timestamp = DateTime::parse_from_rfc3339(row.get("date_time_utc"))
                            .unwrap()
                            .into();

                        Commit::new(
                            String::from(&commit_id),
                            row.get("owner"),
                            row.get("message"),
                            changes,
                            row.get("root_hash"),
                            parents,
                            timestamp,
                        )
                    }
                    Err(sqlx::Error::RowNotFound) => {
                        return Err(Error::commit_not_found(commit_id));
                    }
                    Err(err) => {
                        return Err(err)
                            .map_other_err(format!("failed to fetch commit `{}`", commit_id));
                    }
                },
            );
        }

        Ok(result)
    }

    async fn insert_commit_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        commit: &Commit,
    ) -> Result<()> {
        sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, ?, ?, ?, ?);",
            Self::TABLE_COMMITS
        ))
        .bind(commit.id.clone())
        .bind(commit.owner.clone())
        .bind(commit.message.clone())
        .bind(commit.root_tree_id.clone())
        .bind(commit.timestamp.to_rfc3339())
        .execute(&mut *transaction)
        .await
        .map_other_err(format!("failed to insert the commit `{}`", &commit.id))?;

        for parent_id in &commit.parents {
            sqlx::query(&format!(
                "INSERT INTO `{}` VALUES(?, ?);",
                Self::TABLE_COMMIT_PARENTS
            ))
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
            sqlx::query(&format!(
                "INSERT INTO `{}` VALUES(?, ?, ?, ?, ?, ?);",
                Self::TABLE_COMMIT_CHANGES
            ))
            .bind(commit.id.clone())
            .bind(change.canonical_path().to_string())
            .bind(
                change
                    .change_type()
                    .old_info()
                    .map(|info| info.hash.as_str())
                    .unwrap_or_default(),
            )
            .bind(
                change
                    .change_type()
                    .new_info()
                    .map(|info| info.hash.as_str())
                    .unwrap_or_default(),
            )
            .bind(
                change
                    .change_type()
                    .old_info()
                    .map(|info| i64::try_from(info.size).unwrap_or(0))
                    .unwrap_or_default(),
            )
            .bind(
                change
                    .change_type()
                    .new_info()
                    .map(|info| i64::try_from(info.size).unwrap_or(0))
                    .unwrap_or_default(),
            )
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
            "UPDATE `{}` SET head=?, lock_domain_id=?
             WHERE name=?;",
            Self::TABLE_BRANCHES
        ))
        .bind(branch.head.clone())
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
    ) -> Result<String> {
        Self::save_tree_node_transactional(transaction, None, tree).await
    }

    async fn tree_node_exists(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        id: &str,
    ) -> Result<bool> {
        Ok(sqlx::query(&format!(
            "SELECT COUNT(1) FROM `{}` WHERE id = ?",
            Self::TABLE_FOREST
        ))
        .bind(&id)
        .fetch_one(&mut *transaction)
        .await
        .map_other_err(format!("failed to check for tree node `{}` existence", id))
        .map(|row| row.get::<i32, _>(0))?
            > 0)
    }

    #[async_recursion]
    async fn save_tree_node_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        parent_id: Option<&'async_recursion str>,
        tree: &Tree,
    ) -> Result<String> {
        let id = tree.id();

        // We only insert the tree node if it doesn't exist already.
        if !Self::tree_node_exists(&mut *transaction, &id).await? {
            let sql = &format!(
                "INSERT INTO `{}` (id, name, hash, size) VALUES(?, ?, ?, ?);",
                Self::TABLE_FOREST,
            );

            match tree {
                Tree::Directory { name, children } => {
                    sqlx::query(sql)
                        .bind(&id)
                        .bind(name)
                        .bind(Option::<String>::None)
                        .bind(Option::<i64>::None)
                        .execute(&mut *transaction)
                        .await
                        .map_other_err(&format!(
                            "failed to insert tree directory node `{}`",
                            &id
                        ))?;

                    for child in children.values() {
                        Self::save_tree_node_transactional(transaction, Some(&id), child).await?;
                    }
                }
                Tree::File { name, info } => {
                    sqlx::query(sql)
                        .bind(&id)
                        .bind(name)
                        .bind(Some(&info.hash))
                        .bind(Some(i64::try_from(info.size).unwrap_or(0)))
                        .execute(&mut *transaction)
                        .await
                        .map_other_err(&format!("failed to insert tree file node `{}`", &id))?;
                }
            }
        }

        // Even if the tree node existed, the relationship should not exist already.
        if let Some(parent_id) = parent_id {
            sqlx::query(&format!(
                "INSERT INTO `{}` (id, child_id) VALUES(?, ?);",
                Self::TABLE_FOREST_LINKS,
            ))
            .bind(parent_id)
            .bind(&id)
            .execute(&mut *transaction)
            .await
            .map_other_err(format!(
                "failed to create link between node {} and its parent {}",
                id, parent_id
            ))?;
        }

        Ok(id)
    }

    async fn get_tree_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        id: &str,
    ) -> Result<Tree> {
        let row = sqlx::query(&format!(
            "SELECT name, hash, size
             FROM `{}`
             WHERE id = ?;",
            Self::TABLE_FOREST
        ))
        .bind(id)
        .fetch_one(&mut *transaction)
        .await
        .map_other_err(format!("failed to fetch tree node `{}`", id))?;

        let name = row.get("name");
        let hash: String = row.get("hash");

        let info = if !hash.is_empty() {
            let size: i64 = row.get("size");

            Some(FileInfo {
                hash,
                size: size as u64,
            })
        } else {
            None
        };

        let tree = Self::read_tree_node_transactional(transaction, id, name, info).await?;

        #[cfg(debug_assertions)]
        assert!(
            !(tree.id() != id),
            "tree node `{}` was not saved correctly",
            id
        );

        Ok(tree)
    }

    #[async_recursion]
    async fn read_tree_node_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        id: &str,
        name: String,
        info: Option<FileInfo>,
    ) -> Result<Tree> {
        Ok(if let Some(info) = info {
            Tree::File { info, name }
        } else {
            let child_ids = sqlx::query(&format!(
                "SELECT child_id
                 FROM `{}`
                 WHERE id = ?;",
                Self::TABLE_FOREST_LINKS
            ))
            .bind(&id)
            .fetch_all(&mut *transaction)
            .await
            .map_other_err(format!("failed to fetch children for tree node `{}`", &id))?
            .into_iter()
            .map(|row| row.get::<String, _>("child_id"))
            .collect::<Vec<String>>();

            let mut children = BTreeMap::new();

            for child_id in child_ids {
                let row = sqlx::query(&format!(
                    "SELECT name, hash, size
                    FROM `{}`
                    WHERE id = ?;",
                    Self::TABLE_FOREST
                ))
                .bind(&child_id)
                .fetch_one(&mut *transaction)
                .await
                .map_other_err(format!(
                    "failed to fetch children for tree node data `{}`",
                    &child_id
                ))?;

                let name: String = row.get("name");
                let hash: String = row.get("hash");

                let info = if !hash.is_empty() {
                    let size: i64 = row.get("size");

                    Some(FileInfo {
                        hash,
                        size: size as u64,
                    })
                } else {
                    None
                };

                let child =
                    Self::read_tree_node_transactional(&mut *transaction, &child_id, name, info)
                        .await?;

                children.insert(child.name().to_string(), child);
            }

            Tree::Directory { name, children }
        })
    }

    async fn get_lock_transactional<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        executor: E,
        lock_domain_id: &str,
        canonical_path: &CanonicalPath,
    ) -> Result<Lock> {
        match sqlx::query(&format!(
            "SELECT workspace_id, branch_name
             FROM `{}`
             WHERE lock_domain_id=?
             AND canonical_path=?;",
            Self::TABLE_LOCKS,
        ))
        .bind(lock_domain_id)
        .bind(canonical_path.to_string())
        .fetch_optional(executor)
        .await
        .map_other_err(&format!(
            "failed to find lock `{}/{}`",
            lock_domain_id, canonical_path,
        ))?
        .map(|row| Lock {
            canonical_path: canonical_path.clone(),
            lock_domain_id: lock_domain_id.to_string(),
            workspace_id: row.get("workspace_id"),
            branch_name: row.get("branch_name"),
        }) {
            Some(lock) => Ok(lock),
            None => Err(Error::lock_not_found(
                lock_domain_id.to_string(),
                canonical_path.clone(),
            )),
        }
    }

    pub async fn close(&mut self) {
        if let Some(pool) = self.pool.lock().await.take() {
            pool.close().await;
        }
    }
}

#[async_trait]
impl IndexBackend for SqlIndexBackend {
    fn url(&self) -> &str {
        self.driver.url()
    }

    async fn create_index(&self) -> Result<BlobStorageUrl> {
        if self.driver.check_if_database_exists().await? {
            return Err(Error::index_already_exists(self.url()));
        }

        self.driver.create_database().await?;

        let mut conn = self.get_conn().await?;

        Self::initialize_database(&mut conn).await?;

        let mut transaction = conn
            .begin()
            .await
            .map_other_err("failed to acquire SQL transaction")?;

        Self::initialize_repository_data(&mut transaction).await?;

        transaction
            .commit()
            .await
            .map_other_err("failed to commit transaction when creating repository")?;

        Ok(self.blob_storage_url.clone())
    }

    async fn destroy_index(&self) -> Result<()> {
        if !self.driver.check_if_database_exists().await? {
            return Err(Error::index_does_not_exist(self.url()));
        }

        if let Some(pool) = self.pool.lock().await.take() {
            pool.close().await;
        }

        self.driver.drop_database().await
    }

    async fn index_exists(&self) -> Result<bool> {
        if !self.driver.check_if_database_exists().await? {
            return Ok(false);
        }

        Ok(self.get_conn().await.is_ok())
    }

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, ?);",
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

    async fn get_branch(&self, branch_name: &str) -> Result<Branch> {
        let mut conn = self.get_conn().await?;

        match sqlx::query(&format!(
            "SELECT head, lock_domain_id 
             FROM `{}`
             WHERE name = ?;",
            Self::TABLE_BRANCHES
        ))
        .bind(branch_name)
        .fetch_optional(&mut conn)
        .await
        .map_other_err(format!("error fetching branch `{}`", branch_name))?
        {
            None => Err(Error::branch_not_found(branch_name.to_string())),
            Some(row) => Ok(Branch {
                name: branch_name.to_string(),
                head: row.get("head"),
                lock_domain_id: row.get("lock_domain_id"),
            }),
        }
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        let mut conn = self.get_conn().await?;

        Ok(match query.lock_domain_id {
            Some(lock_domain_id) => sqlx::query(&format!(
                "SELECT name, head 
             FROM `{}`
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
            .map(|row| Branch {
                name: row.get("name"),
                head: row.get("head"),
                lock_domain_id: lock_domain_id.to_string(),
            })
            .collect(),
            None => sqlx::query(&format!(
                "SELECT name, head, lock_domain_id 
             FROM `{}`;",
                Self::TABLE_BRANCHES
            ))
            .fetch_all(&mut conn)
            .await
            .map_other_err("error fetching branches")?
            .into_iter()
            .map(|row| Branch {
                name: row.get("name"),
                head: row.get("head"),
                lock_domain_id: row.get("lock_domain_id"),
            })
            .collect(),
        })
    }

    async fn list_commits(&self, query: &ListCommitsQuery<'_>) -> Result<Vec<Commit>> {
        let mut transaction = self.get_transaction().await?;

        let result = Self::list_commits_transactional(&mut transaction, query).await?;

        transaction
            .commit()
            .await
            .map_other_err("failed to commit transaction while listing commits")?;

        Ok(result)
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        let stored_branch = self
            .read_branch_for_update(&mut transaction, &branch.name)
            .await?;

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

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        let mut transaction = self.get_transaction().await?;

        Self::get_tree_transactional(&mut transaction, id).await
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        let mut transaction = self.get_transaction().await?;

        let tree_id = Self::save_tree_transactional(&mut transaction, tree).await?;

        transaction
            .commit()
            .await
            .map_other_err(&format!(
                "failed to commit transaction while saving tree `{}`",
                &tree_id,
            ))
            .map(|_| tree_id)
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        match Self::get_lock_transactional(
            &mut transaction,
            &lock.lock_domain_id,
            &lock.canonical_path,
        )
        .await
        {
            Ok(lock) => Err(Error::lock_already_exists(lock)),
            Err(Error::LockNotFound { .. }) => {
                sqlx::query(&format!(
                    "INSERT INTO `{}` VALUES(?, ?, ?, ?);",
                    Self::TABLE_LOCKS
                ))
                .bind(lock.canonical_path.to_string())
                .bind(lock.lock_domain_id.clone())
                .bind(lock.workspace_id.clone())
                .bind(lock.branch_name.clone())
                .execute(&mut transaction)
                .await
                .map_other_err(&format!("failed to lock `{}`", lock))?;

                transaction
                    .commit()
                    .await
                    .map_other_err(&format!(
                        "failed to commit transaction while inserting lock `{}`",
                        lock
                    ))
                    .map(|_| ())
            }
            Err(err) => Err(err),
        }
    }

    async fn get_lock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<Lock> {
        let mut conn = self.get_conn().await?;

        Self::get_lock_transactional(&mut conn, lock_domain_id, canonical_path).await
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        let mut conn = self.get_conn().await?;

        if !query.lock_domain_ids.is_empty() {
            let mut locks = Vec::new();

            for lock_domain_id in &query.lock_domain_ids {
                locks.extend(
                    sqlx::query(&format!(
                        "SELECT canonical_path, workspace_id, branch_name
                        FROM `{}`
                        WHERE lock_domain_id=?;",
                        Self::TABLE_LOCKS,
                    ))
                    .bind(*lock_domain_id)
                    .fetch_all(&mut conn)
                    .await
                    .map_other_err(&format!(
                        "failed to find locks in domain `{}`",
                        *lock_domain_id,
                    ))?
                    .into_iter()
                    .map(|row| {
                        Ok(Lock {
                            canonical_path: CanonicalPath::new(
                                &row.get::<String, _>("canonical_path"),
                            )?,
                            lock_domain_id: (*lock_domain_id).to_string(),
                            workspace_id: row.get("workspace_id"),
                            branch_name: row.get("branch_name"),
                        })
                    }),
                );
            }

            locks.into_iter().collect()
        } else {
            sqlx::query(&format!(
                "SELECT lock_domain_id, canonical_path, workspace_id, branch_name
                FROM `{}`;",
                Self::TABLE_LOCKS,
            ))
            .fetch_all(&mut conn)
            .await
            .map_other_err("failed to list locks")?
            .into_iter()
            .map(|row| {
                Ok(Lock {
                    canonical_path: CanonicalPath::new(&row.get::<String, _>("canonical_path"))?,
                    lock_domain_id: row.get("lock_domain_id"),
                    workspace_id: row.get("workspace_id"),
                    branch_name: row.get("branch_name"),
                })
            })
            .collect()
        }
    }

    async fn unlock(&self, lock_domain_id: &str, canonical_path: &CanonicalPath) -> Result<()> {
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "DELETE from `{}` WHERE canonical_path=? AND lock_domain_id=?;",
            Self::TABLE_LOCKS
        ))
        .bind(canonical_path.to_string())
        .bind(lock_domain_id)
        .execute(&mut conn)
        .await
        .map_other_err(&format!(
            "failed to clear lock `{}/{}`",
            lock_domain_id, canonical_path,
        ))
        .map(|_| ())
    }

    async fn count_locks(&self, query: &ListLocksQuery<'_>) -> Result<i32> {
        let mut conn = self.get_conn().await?;

        if !query.lock_domain_ids.is_empty() {
            let mut result = 0;
            for lock_domain_id in &query.lock_domain_ids {
                result += sqlx::query(&format!(
                    "SELECT count(*) as count
                    FROM `{}`
                    WHERE lock_domain_id = ?;",
                    Self::TABLE_LOCKS,
                ))
                .bind(*lock_domain_id)
                .fetch_one(&mut conn)
                .await
                .map_other_err(&format!(
                    "failed to count locks in domain `{}`",
                    *lock_domain_id,
                ))
                .map(|row| row.get::<i32, _>("count"))?;
            }

            Ok(result)
        } else {
            sqlx::query(&format!(
                "SELECT count(*) as count
                FROM `{}`;",
                Self::TABLE_LOCKS,
            ))
            .fetch_one(&mut conn)
            .await
            .map_other_err("failed to count locks")
            .map(|row| row.get::<i32, _>("count"))
        }
    }

    async fn get_blob_storage_url(&self) -> Result<BlobStorageUrl> {
        Ok(self.blob_storage_url.clone())
    }
}
