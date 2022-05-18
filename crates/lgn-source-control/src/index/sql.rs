use async_trait::async_trait;
use chrono::DateTime;
use lgn_tracing::prelude::*;
use sqlx::{
    any::AnyPoolOptions, error::DatabaseError, migrate::MigrateDatabase, mysql::MySqlDatabaseError,
    Acquire, Executor, Pool, Row,
};
use std::collections::{BTreeSet, VecDeque};

use crate::{
    Branch, CanonicalPath, Change, ChangeType, Commit, CommitId, Error, Index, ListBranchesQuery,
    ListCommitsQuery, ListLocksQuery, Lock, MapOtherError, RepositoryIndex, RepositoryName, Result,
};

const TABLE_REPOSITORIES: &str = "repositories";
const TABLE_COMMITS: &str = "commits";
const TABLE_COMMIT_PARENTS: &str = "commit_parents";
const TABLE_COMMIT_CHANGES: &str = "commit_changes";
const TABLE_BRANCHES: &str = "branches";
const TABLE_LOCKS: &str = "locks";

#[derive(Debug, Clone)]
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
            Err(Error::Unspecified(format!(
                "unsupported SQL database driver: {}",
                url.split(':').next().unwrap_or_default()
            )))
        }
    }

    fn auto_increment(&self) -> &'static str {
        match &self {
            SqlDatabaseDriver::Sqlite(_) => "",
            SqlDatabaseDriver::Mysql(_) => "AUTO_INCREMENT",
        }
    }

    #[allow(clippy::borrowed_box)]
    fn is_unique_constraint_error(&self, db_err: &Box<dyn DatabaseError>) -> bool {
        match &self {
            Self::Sqlite(_) => db_err.code() == Some("2067".into()),
            Self::Mysql(_) => {
                db_err.code() == Some("23000".into())
                    && db_err
                        .try_downcast_ref::<MySqlDatabaseError>()
                        .filter(|e| e.number() == 1062)
                        .is_some()
            }
        }
    }

    async fn new_pool(&self) -> Result<Pool<sqlx::Any>> {
        Ok(match &self {
            Self::Sqlite(uri) => {
                AnyPoolOptions::new()
                    .connect(uri)
                    .await
                    .map_other_err(format!(
                        "failed to establish a SQLite connection pool to `{}`",
                        uri
                    ))?
            }
            Self::Mysql(uri) => AnyPoolOptions::new()
                .max_connections(10)
                .connect(uri)
                .await
                .map_other_err("failed to establish a MySQL connection pool".to_string())?,
        })
    }

    async fn create_database(&self) -> Result<()> {
        match &self {
            Self::Sqlite(uri) | Self::Mysql(uri) => sqlx::Any::create_database(uri)
                .await
                .map_other_err("failed to create database"),
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
#[derive(Debug, Clone)]
pub struct SqlRepositoryIndex {
    driver: SqlDatabaseDriver,
    pool: Pool<sqlx::Any>,
}

impl SqlRepositoryIndex {
    #[span_fn]
    pub async fn new(url: String) -> Result<Self> {
        let driver = SqlDatabaseDriver::new(url)?;

        info!("Connecting to SQL database...");

        // This should not be done in production, but it is useful for testing.
        if !driver.check_if_database_exists().await? {
            info!("Database does not exist: creating...");

            driver.create_database().await?;
        } else {
            info!("Database already exists: moving on...");
        }

        let pool = driver.new_pool().await?;

        let mut conn = pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")?;

        Self::initialize_database(&mut conn, &driver).await?;

        Ok(Self { driver, pool })
    }

    async fn get_repositories(conn: &mut sqlx::AnyConnection) -> Result<Vec<RepositoryName>> {
        let sql: &str = &format!("SELECT name from `{}` ORDER BY name;", TABLE_REPOSITORIES);

        conn.fetch_all(sql)
            .await
            .map_other_err("failed to list repositories")?
            .into_iter()
            .map(|row| row.get::<String, _>("name").parse())
            .collect()
    }

    async fn initialize_database(
        conn: &mut sqlx::AnyConnection,
        driver: &SqlDatabaseDriver,
    ) -> Result<()> {
        let sql: &str = &format!("SELECT 1 from `{}` LIMIT 1;", TABLE_REPOSITORIES);

        if conn.execute(sql).await.is_ok() {
            info!("Database already initialized");
        } else {
            info!("Database not initialized yet.");

            Self::create_repositories_table(conn, driver).await?;
            Self::create_commits_table(conn, driver).await?;
            Self::create_branches_table(conn).await?;
            Self::create_locks_table(conn).await?;
        }

        Ok(())
    }

    async fn create_repositories_table(
        conn: &mut sqlx::AnyConnection,
        driver: &SqlDatabaseDriver,
    ) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}` (id INTEGER NOT NULL {} PRIMARY KEY, name VARCHAR(255), CONSTRAINT unique_repository_name UNIQUE (name));",
            TABLE_REPOSITORIES,
            driver.auto_increment(),
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the commits table")
            .map(|_| ())
    }

    async fn create_commits_table(
        conn: &mut sqlx::AnyConnection,
        driver: &SqlDatabaseDriver,
    ) -> Result<()> {
        let sql: &str = &format!(
        "CREATE TABLE `{}` (repository_id INTEGER NOT NULL, id INTEGER NOT NULL {} PRIMARY KEY, owner VARCHAR(255), message TEXT, main_index_tree_id CHAR(64), path_index_tree_id CHAR(64), date_time_utc VARCHAR(255), FOREIGN KEY (repository_id) REFERENCES `{}`(id) ON DELETE CASCADE);
         CREATE INDEX repository_id_commit on `{}`(repository_id, id);
         CREATE TABLE `{}` (id INTEGER NOT NULL, parent_id INTEGER NOT NULL);
         CREATE INDEX commit_parents_id on `{}`(id);
         CREATE TABLE `{}` (commit_id INTEGER NOT NULL, canonical_path TEXT NOT NULL, old_cs_id VARCHAR(255), new_cs_id VARCHAR(255), FOREIGN KEY (commit_id) REFERENCES `{}`(id) ON DELETE CASCADE);
         CREATE INDEX commit_changes_commit on `{}`(commit_id);",
         TABLE_COMMITS,
         driver.auto_increment(),
         TABLE_REPOSITORIES,
         TABLE_COMMITS,
         TABLE_COMMIT_PARENTS,
         TABLE_COMMIT_PARENTS,
         TABLE_COMMIT_CHANGES,
         TABLE_COMMITS,
         TABLE_COMMIT_CHANGES
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the commits table")
            .map(|_| ())
    }

    async fn create_branches_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
        "CREATE TABLE `{}` (repository_id INTEGER NOT NULL, name VARCHAR(255), head INTEGER NOT NULL, lock_domain_id VARCHAR(64), PRIMARY KEY (repository_id, name), FOREIGN KEY (repository_id) REFERENCES `{}`(id) ON DELETE CASCADE, FOREIGN KEY (head) REFERENCES `{}`(id));
        CREATE INDEX branches_lock_domain_ids_index on `{}`(lock_domain_id);",
        TABLE_BRANCHES,
        TABLE_REPOSITORIES,
        TABLE_COMMITS,
        TABLE_BRANCHES
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the branches table and index")
            .map(|_| ())
    }

    async fn create_locks_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
        "CREATE TABLE `{}` (repository_id INTEGER NOT NULL, lock_domain_id VARCHAR(64), canonical_path VARCHAR(512), workspace_id VARCHAR(255), branch_name VARCHAR(255), UNIQUE (repository_id, lock_domain_id, canonical_path), FOREIGN KEY (repository_id) REFERENCES `{}`(id) ON DELETE CASCADE);
        ",
        TABLE_LOCKS,
        TABLE_REPOSITORIES,
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the locks table and index")
            .map(|_| ())
    }
}

#[async_trait]
impl RepositoryIndex for SqlRepositoryIndex {
    async fn create_repository(&self, repository_name: &RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("SqlRepositoryIndex::create_repository");

        info!("Creating repository `{}` in SQL index", repository_name);

        let index = SqlIndex::init(self.driver.clone(), self.pool.clone(), repository_name).await?;

        Ok(Box::new(index))
    }

    async fn destroy_repository(&self, repository_name: &RepositoryName) -> Result<()> {
        async_span_scope!("SqlRepositoryIndex::destroy_repository");

        info!("Destroying repository `{}` in SQL index", repository_name);

        let index = SqlIndex::load(self.driver.clone(), self.pool.clone(), repository_name).await?;

        index.cleanup_repository_data().await
    }

    async fn load_repository(&self, repository_name: &RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("SqlRepositoryIndex::load_repository");

        info!("Loading repository `{}` in SQL index", repository_name);

        let index = SqlIndex::load(self.driver.clone(), self.pool.clone(), repository_name).await?;

        Ok(Box::new(index))
    }

    async fn list_repositories(&self) -> Result<Vec<RepositoryName>> {
        async_span_scope!("SqlRepositoryIndex::list_repositories");

        let mut conn = self
            .pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")?;

        Self::get_repositories(&mut conn).await
    }
}

pub struct SqlIndex {
    driver: SqlDatabaseDriver,
    pool: Pool<sqlx::Any>,
    repository_name: RepositoryName,
    repository_id: i64,
}

impl core::fmt::Debug for SqlIndex {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SqlIndex")
            .field("driver", &self.driver)
            .finish()
    }
}

impl SqlIndex {
    async fn init(
        driver: SqlDatabaseDriver,
        pool: Pool<sqlx::Any>,
        repository_name: &RepositoryName,
    ) -> Result<Self> {
        let conn = pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")?;

        let repository_id =
            Self::initialize_repository_data(conn, repository_name, &driver).await?;

        Ok(Self {
            driver,
            pool,
            repository_name: repository_name.clone(),
            repository_id,
        })
    }

    async fn load(
        driver: SqlDatabaseDriver,
        pool: Pool<sqlx::Any>,
        repository_name: &RepositoryName,
    ) -> Result<Self> {
        let mut conn = pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")?;

        let repository_id = Self::get_repository_id(&mut conn, repository_name).await?;

        Ok(Self {
            driver,
            pool,
            repository_name: repository_name.clone(),
            repository_id,
        })
    }

    async fn get_conn(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>> {
        info!(
            "Acquiring SQL connections: pool currently has {} connections",
            self.pool.size()
        );

        self.pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")
    }

    async fn get_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
        info!(
            "Acquiring SQL transaction: pool currently has {} connections",
            self.pool.size()
        );

        self.pool
            .begin()
            .await
            .map_other_err("failed to acquire SQL transaction")
    }

    async fn initialize_repository_data(
        mut conn: sqlx::pool::PoolConnection<sqlx::Any>,
        repository_name: &RepositoryName,
        driver: &SqlDatabaseDriver,
    ) -> Result<i64> {
        let mut transaction = conn
            .begin()
            .await
            .map_other_err("failed to acquire SQL transaction")?;

        let repository_id = Self::insert_repository_transactional(
            &mut transaction,
            repository_name.clone(),
            driver,
        )
        .await?;

        let empty_tree_id = {
            use lgn_content_store::{
                indexing::{Tree, TreeWriter},
                Provider,
            };

            let provider = Provider::new_in_memory();
            provider.write_tree(&Tree::default()).await.unwrap()
        };

        let initial_commit = Commit::new_unique_now(
            whoami::username(),
            String::from("initial commit"),
            BTreeSet::new(),
            empty_tree_id.clone(),
            empty_tree_id,
            BTreeSet::new(),
        );

        let commit_id =
            Self::insert_commit_transactional(&mut transaction, repository_id, &initial_commit)
                .await?;

        let main_branch = Branch::new(String::from("main"), commit_id);

        Self::insert_branch_transactional(&mut transaction, repository_id, &main_branch).await?;

        transaction.commit().await.map_other_err(format!(
            "failed to commit transaction when creating repository `{}`",
            repository_name,
        ))?;

        Ok(repository_id)
    }

    async fn cleanup_repository_data(self) -> Result<()> {
        let mut transaction = self.get_transaction().await?;

        Self::delete_repository_transactional(&mut transaction, self.repository_id).await?;

        transaction.commit().await.map_other_err(format!(
            "failed to commit transaction when delete repository `{}`",
            self.repository_name,
        ))?;

        Ok(())
    }

    #[span_fn]
    async fn read_branch_for_update<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<Branch> {
        let query = match &self.driver {
            SqlDatabaseDriver::Sqlite(_) => format!(
                "SELECT head, lock_domain_id
                     FROM `{}`
                     WHERE repository_id=?
                     AND name=?;",
                TABLE_BRANCHES
            ),
            SqlDatabaseDriver::Mysql(_) => format!(
                "SELECT head, lock_domain_id
                     FROM `{}`
                     WHERE repository_id=?
                     AND name = ?
                     FOR UPDATE;",
                TABLE_BRANCHES
            ),
        };

        let row = sqlx::query(&query)
            .bind(self.repository_id)
            .bind(name)
            .fetch_one(executor)
            .await
            .map_other_err(format!(
                "failed to read the branch for repository {}",
                self.repository_id
            ))?;

        Ok(Branch {
            name: name.to_string(),
            head: CommitId(
                row.get::<i64, _>("head")
                    .try_into()
                    .map_other_err("failed to read the head")?,
            ),
            lock_domain_id: row.get("lock_domain_id"),
        })
    }

    #[span_fn]
    async fn insert_repository_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_name: RepositoryName,
        driver: &SqlDatabaseDriver,
    ) -> Result<i64> {
        let result = match sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(NULL, ?);",
            TABLE_REPOSITORIES
        ))
        .bind(repository_name.to_string())
        .execute(transaction)
        .await
        {
            Ok(result) => result,
            Err(sqlx::Error::Database(db_err)) => {
                return if driver.is_unique_constraint_error(&db_err) {
                    Err(Error::RepositoryAlreadyExists { repository_name })
                } else {
                    Err(Error::Unspecified(format!(
                        "failed to insert the repository `{}`: {}",
                        repository_name, db_err,
                    )))
                };
            }
            Err(err) => {
                return Err(err).map_other_err(&format!(
                    "failed to insert the repository `{}`",
                    repository_name,
                ))
            }
        };

        result.last_insert_id().ok_or_else(|| {
            Error::Unspecified(format!(
                "failed to get the last insert id when inserting repository `{}`",
                repository_name
            ))
        })
    }

    #[span_fn]
    async fn delete_repository_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
    ) -> Result<()> {
        sqlx::query(&format!("DELETE FROM `{}` WHERE id=?;", TABLE_REPOSITORIES))
            .bind(repository_id)
            .execute(transaction)
            .await
            .map_other_err(&format!(
                "failed to delete the repository {}",
                repository_id,
            ))
            .map(|_| ())
    }

    #[span_fn]
    async fn get_repository_id(
        conn: &mut sqlx::pool::PoolConnection<sqlx::Any>,
        repository_name: &RepositoryName,
    ) -> Result<i64> {
        match sqlx::query(&format!(
            "SELECT id
             FROM `{}`
             WHERE name = ?;",
            TABLE_REPOSITORIES,
        ))
        .bind(repository_name.to_string())
        .fetch_one(conn)
        .await
        {
            Ok(row) => Ok(row.get::<i64, _>("id")),
            Err(sqlx::Error::RowNotFound) => {
                Err(Error::repository_does_not_exist(repository_name.clone()))
            }
            Err(err) => {
                Err(err).map_other_err(format!("failed to fetch repository `{}`", repository_name))
            }
        }
    }

    #[span_fn]
    async fn insert_branch_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
        branch: &Branch,
    ) -> Result<()> {
        let head: i64 = branch
            .head
            .0
            .try_into()
            .map_other_err("failed to convert the head")?;

        sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, ?, ?, ?);",
            TABLE_BRANCHES
        ))
        .bind(repository_id)
        .bind(&branch.name)
        .bind(head)
        .bind(&branch.lock_domain_id)
        .execute(transaction)
        .await
        .map_other_err(&format!(
            "failed to insert the branch `{}` in repository {}",
            &branch.name, repository_id
        ))
        .map(|_| ())
    }

    #[span_fn]
    async fn list_commits_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
        query: &ListCommitsQuery,
    ) -> Result<Vec<Commit>> {
        let mut result = Vec::new();
        result.reserve(1024);

        let mut next_commit_ids: VecDeque<i64> = query
            .commit_ids
            .iter()
            .map(|id| {
                id.0.try_into()
                    .map_other_err("failed to convert the commit id")
            })
            .collect::<Result<VecDeque<_>>>()?;

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
                "SELECT canonical_path, old_cs_id, new_cs_id
             FROM `{}`
             WHERE commit_id=?;",
                TABLE_COMMIT_CHANGES,
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
                    let old_cs_id: String = row.get("old_cs_id");

                    let old_cs_id = if !old_cs_id.is_empty() {
                        match old_cs_id
                            .parse()
                            .map_other_err("failed to parse the old chunk id")
                        {
                            Ok(id) => Some(id),
                            Err(err) => return Some(Err(err)),
                        }
                    } else {
                        None
                    };

                    let new_cs_id: String = row.get("new_cs_id");
                    let new_cs_id = if !new_cs_id.is_empty() {
                        match new_cs_id
                            .parse()
                            .map_other_err("failed to parse the new chunk id")
                        {
                            Ok(id) => Some(id),
                            Err(err) => return Some(Err(err)),
                        }
                    } else {
                        None
                    };

                    ChangeType::new(old_cs_id, new_cs_id)
                        .map(|change_type| Ok(Change::new(canonical_path, change_type)))
                } else {
                    None
                }
            })
            .collect::<Result<BTreeSet<_>>>()?;

            let parents: BTreeSet<i64> = sqlx::query(&format!(
                "SELECT parent_id
                FROM `{}`
                WHERE id = ?;",
                TABLE_COMMIT_PARENTS,
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

            next_commit_ids.extend(&parents);

            result.push(
                match sqlx::query(&format!(
                    "SELECT owner, message, main_index_tree_id, path_index_tree_id, date_time_utc 
             FROM `{}`
             WHERE repository_id=?
             AND id=?;",
                    TABLE_COMMITS
                ))
                .bind(repository_id)
                .bind(&commit_id)
                .fetch_one(&mut *transaction)
                .await
                {
                    Ok(row) => {
                        let timestamp = DateTime::parse_from_rfc3339(row.get("date_time_utc"))
                            .unwrap()
                            .into();

                        let parents = parents
                            .into_iter()
                            .map(|id| {
                                Ok(CommitId(
                                    id.try_into().map_other_err("failed to convert commit id")?,
                                ))
                            })
                            .collect::<Result<BTreeSet<_>>>()?;

                        Commit::new(
                            CommitId(
                                commit_id
                                    .try_into()
                                    .map_other_err("failed to convert the commit id")?,
                            ),
                            row.get("owner"),
                            row.get("message"),
                            changes,
                            row.get::<String, &str>("main_index_tree_id")
                                .parse()
                                .unwrap(),
                            row.get::<String, &str>("path_index_tree_id")
                                .parse()
                                .unwrap(),
                            parents,
                            timestamp,
                        )
                    }
                    Err(sqlx::Error::RowNotFound) => {
                        return Err(Error::commit_not_found(CommitId(
                            commit_id.try_into().unwrap_or_default(),
                        )));
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

    #[span_fn]
    async fn insert_commit_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
        commit: &Commit,
    ) -> Result<CommitId> {
        let result = sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, NULL, ?, ?, ?, ?, ?);",
            TABLE_COMMITS
        ))
        .bind(repository_id)
        .bind(commit.owner.clone())
        .bind(commit.message.clone())
        .bind(commit.main_index_tree_id.to_string())
        .bind(commit.path_index_tree_id.to_string())
        .bind(commit.timestamp.to_rfc3339())
        .execute(&mut *transaction)
        .await
        .map_other_err(format!(
            "failed to insert the commit in repository {}",
            repository_id
        ))?;

        let commit_id = result.last_insert_id().unwrap();

        for parent_id in &commit.parents {
            let parent_id: i64 = parent_id
                .0
                .try_into()
                .map_other_err("failed to convert commit id")?;

            sqlx::query(&format!(
                "INSERT INTO `{}` VALUES(?, ?);",
                TABLE_COMMIT_PARENTS
            ))
            .bind(commit_id)
            .bind(parent_id)
            .execute(&mut *transaction)
            .await
            .map_other_err(format!(
                "failed to insert the commit parent `{}` for commit `{}`",
                parent_id, &commit.id
            ))?;
        }

        for change in &commit.changes {
            sqlx::query(&format!(
                "INSERT INTO `{}` VALUES(?, ?, ?, ?);",
                TABLE_COMMIT_CHANGES
            ))
            .bind(commit_id)
            .bind(change.canonical_path().to_string())
            .bind(
                change
                    .change_type()
                    .old_id()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
            )
            .bind(
                change
                    .change_type()
                    .new_id()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
            )
            .execute(&mut *transaction)
            .await
            .map_other_err(format!(
                "failed to insert the commit change for commit `{}`",
                &commit.id
            ))?;
        }

        Ok(CommitId(
            commit_id
                .try_into()
                .map_other_err("failed to convert commit id")?,
        ))
    }

    async fn update_branch_transactional<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        executor: E,
        repository_id: i64,
        branch: &Branch,
    ) -> Result<()> {
        let head: i64 = branch
            .head
            .0
            .try_into()
            .map_other_err("failed to convert commit id")?;
        sqlx::query(&format!(
            "UPDATE `{}` SET head=?, lock_domain_id=?
             WHERE repository_id=? AND name=?;",
            TABLE_BRANCHES
        ))
        .bind(head)
        .bind(branch.lock_domain_id.clone())
        .bind(repository_id)
        .bind(branch.name.clone())
        .execute(executor)
        .await
        .map_other_err(format!(
            "failed to update the `{}` branch in repository {}",
            &branch.name, repository_id
        ))?;

        Ok(())
    }

    async fn get_lock_transactional<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        executor: E,
        repository_id: i64,
        lock_domain_id: &str,
        canonical_path: &CanonicalPath,
    ) -> Result<Lock> {
        match sqlx::query(&format!(
            "SELECT workspace_id, branch_name
             FROM `{}`
             WHERE repository_id=?
             AND lock_domain_id=?
             AND canonical_path=?;",
            TABLE_LOCKS,
        ))
        .bind(repository_id)
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
        self.pool.close().await;
    }
}

#[async_trait]
impl Index for SqlIndex {
    fn repository_name(&self) -> &RepositoryName {
        &self.repository_name
    }

    async fn insert_branch(&self, branch: &Branch) -> Result<()> {
        async_span_scope!("SqlIndex::insert_branch");
        let mut transaction = self.get_transaction().await?;

        Self::insert_branch_transactional(&mut transaction, self.repository_id, branch).await?;

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
        async_span_scope!("SqlIndex::update_branch");
        let mut transaction = self.get_transaction().await?;

        Self::update_branch_transactional(&mut transaction, self.repository_id, branch).await?;

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
        async_span_scope!("SqlIndex::get_branch");
        let mut conn = self.get_conn().await?;

        match sqlx::query(&format!(
            "SELECT head, lock_domain_id 
             FROM `{}`
             WHERE repository_id=?
             AND name = ?;",
            TABLE_BRANCHES
        ))
        .bind(self.repository_id)
        .bind(branch_name)
        .fetch_optional(&mut conn)
        .await
        .map_other_err(format!("error fetching branch `{}`", branch_name))?
        {
            None => Err(Error::branch_not_found(branch_name.to_string())),
            Some(row) => Ok(Branch {
                name: branch_name.to_string(),
                head: CommitId(
                    row.get::<i64, _>("head")
                        .try_into()
                        .map_other_err("failed to convert head")?,
                ),
                lock_domain_id: row.get("lock_domain_id"),
            }),
        }
    }

    async fn list_branches(&self, query: &ListBranchesQuery<'_>) -> Result<Vec<Branch>> {
        async_span_scope!("SqlIndex::list_branches");
        let mut conn = self.get_conn().await?;

        match query.lock_domain_id {
            Some(lock_domain_id) => sqlx::query(&format!(
                "SELECT name, head 
             FROM `{}`
             WHERE repository_id=?
             AND lock_domain_id=?;",
                TABLE_BRANCHES
            ))
            .bind(self.repository_id)
            .bind(lock_domain_id)
            .fetch_all(&mut conn)
            .await
            .map_other_err(&format!(
                "error fetching branches in lock domain `{}`",
                lock_domain_id
            ))?
            .into_iter()
            .map(|row| {
                Ok(Branch {
                    name: row.get("name"),
                    head: CommitId(
                        row.get::<i64, _>("head")
                            .try_into()
                            .map_other_err("failed to convert head")?,
                    ),
                    lock_domain_id: lock_domain_id.to_string(),
                })
            })
            .collect(),
            None => sqlx::query(&format!(
                "SELECT name, head, lock_domain_id 
             FROM `{}`
             WHERE repository_id=?;",
                TABLE_BRANCHES
            ))
            .bind(self.repository_id)
            .fetch_all(&mut conn)
            .await
            .map_other_err("error fetching branches")?
            .into_iter()
            .map(|row| {
                Ok(Branch {
                    name: row.get("name"),
                    head: CommitId(
                        row.get::<i64, _>("head")
                            .try_into()
                            .map_other_err("failed to convert head")?,
                    ),
                    lock_domain_id: row.get("lock_domain_id"),
                })
            })
            .collect(),
        }
    }

    async fn list_commits(&self, query: &ListCommitsQuery) -> Result<Vec<Commit>> {
        async_span_scope!("SqlIndex::list_commits");
        let mut transaction = self.get_transaction().await?;

        let result =
            Self::list_commits_transactional(&mut transaction, self.repository_id, query).await?;

        transaction
            .commit()
            .await
            .map_other_err("failed to commit transaction while listing commits")?;

        Ok(result)
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<CommitId> {
        async_span_scope!("SqlIndex::commit_to_branch");
        let mut transaction = self.get_transaction().await?;

        let stored_branch = self
            .read_branch_for_update(&mut transaction, &branch.name)
            .await?;

        if &stored_branch != branch {
            return Err(Error::stale_branch(stored_branch));
        }

        let new_branch = branch.advance(
            Self::insert_commit_transactional(&mut transaction, self.repository_id, commit).await?,
        );

        Self::update_branch_transactional(&mut transaction, self.repository_id, &new_branch)
            .await?;

        transaction.commit().await.map_other_err(&format!(
            "failed to commit transaction while committing commit `{}` to branch `{}`",
            &commit.id, &branch.name
        ))?;

        Ok(new_branch.head)
    }

    async fn lock(&self, lock: &Lock) -> Result<()> {
        async_span_scope!("SqlIndex::lock");
        let mut transaction = self.get_transaction().await?;

        match Self::get_lock_transactional(
            &mut transaction,
            self.repository_id,
            &lock.lock_domain_id,
            &lock.canonical_path,
        )
        .await
        {
            Ok(lock) => Err(Error::lock_already_exists(lock)),
            Err(Error::LockNotFound { .. }) => {
                sqlx::query(&format!(
                    "INSERT INTO `{}` VALUES(?, ?, ?, ?, ?);",
                    TABLE_LOCKS
                ))
                .bind(self.repository_id)
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
        async_span_scope!("SqlIndex::get_lock");
        let mut conn = self.get_conn().await?;

        Self::get_lock_transactional(
            &mut conn,
            self.repository_id,
            lock_domain_id,
            canonical_path,
        )
        .await
    }

    async fn list_locks(&self, query: &ListLocksQuery<'_>) -> Result<Vec<Lock>> {
        async_span_scope!("SqlIndex::list_locks");
        let mut conn = self.get_conn().await?;

        if !query.lock_domain_ids.is_empty() {
            let mut locks = Vec::new();

            for lock_domain_id in &query.lock_domain_ids {
                locks.extend(
                    sqlx::query(&format!(
                        "SELECT canonical_path, workspace_id, branch_name
                        FROM `{}`
                        WHERE repository_id=?
                        AND lock_domain_id=?;",
                        TABLE_LOCKS,
                    ))
                    .bind(self.repository_id)
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
                FROM `{}`
                WHERE repository_id=?;",
                TABLE_LOCKS,
            ))
            .bind(self.repository_id)
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
        async_span_scope!("SqlIndex::unlock");
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "DELETE FROM `{}`
            WHERE repository_id=?
            AND canonical_path=?
            AND lock_domain_id=?;",
            TABLE_LOCKS
        ))
        .bind(self.repository_id)
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
        async_span_scope!("SqlIndex::count_locks");
        let mut conn = self.get_conn().await?;

        if !query.lock_domain_ids.is_empty() {
            let mut result = 0;
            for lock_domain_id in &query.lock_domain_ids {
                result += sqlx::query(&format!(
                    "SELECT count(*) as count
                    FROM `{}`
                    WHERE repository_id=?
                    AND lock_domain_id = ?;",
                    TABLE_LOCKS,
                ))
                .bind(self.repository_id)
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
                FROM `{}`
                WHERE repository_id=?;",
                TABLE_LOCKS,
            ))
            .bind(self.repository_id)
            .fetch_one(&mut conn)
            .await
            .map_other_err("failed to count locks")
            .map(|row| row.get::<i32, _>("count"))
        }
    }
}
