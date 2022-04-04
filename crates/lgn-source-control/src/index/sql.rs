use async_recursion::async_recursion;
use async_trait::async_trait;
use chrono::DateTime;
use lgn_content_store2::ChunkIdentifier;
use lgn_tracing::prelude::*;
use sqlx::{
    any::AnyPoolOptions, error::DatabaseError, migrate::MigrateDatabase, Acquire, Executor, Pool,
    Row,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    Branch, CanonicalPath, Change, ChangeType, Commit, CommitId, Error, Index, ListBranchesQuery,
    ListCommitsQuery, ListLocksQuery, Lock, MapOtherError, RepositoryIndex, RepositoryName, Result,
    Tree, WorkspaceRegistration,
};

const TABLE_REPOSITORIES: &str = "repositories";
const TABLE_COMMITS: &str = "commits";
const TABLE_COMMIT_PARENTS: &str = "commit_parents";
const TABLE_COMMIT_CHANGES: &str = "commit_changes";
const TABLE_FOREST: &str = "forest";
const TABLE_FOREST_LINKS: &str = "forest_links";
const TABLE_BRANCHES: &str = "branches";
const TABLE_WORKSPACE_REGISTRATIONS: &str = "workspace_registrations";
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
            Self::Mysql(_) => db_err.code() == Some("1062".into()),
        }
    }

    async fn new_pool(&self) -> Result<Pool<sqlx::Any>> {
        Ok(match &self {
            Self::Sqlite(uri) => AnyPoolOptions::new()
                .connect(uri)
                .await
                .map_other_err("failed to establish a SQLite connection pool".to_string())?,
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
}

impl SqlRepositoryIndex {
    pub async fn new(url: String) -> Result<Self> {
        async_span_scope!("SqlRepositoryIndex::new");
        let driver = SqlDatabaseDriver::new(url)?;

        // This should not be done in production, but it is useful for testing.
        if !driver.check_if_database_exists().await? {
            driver.create_database().await?;

            let pool = driver.new_pool().await?;
            let mut conn = pool
                .acquire()
                .await
                .map_other_err("failed to acquire SQL connection")?;

            Self::initialize_database(&mut conn, &driver).await?;
        }

        Ok(Self { driver })
    }

    async fn initialize_database(
        conn: &mut sqlx::AnyConnection,
        driver: &SqlDatabaseDriver,
    ) -> Result<()> {
        Self::create_repositories_table(conn, driver).await?;
        Self::create_commits_table(conn, driver).await?;
        Self::create_forest_table(conn).await?;
        Self::create_branches_table(conn).await?;
        Self::create_workspace_registrations_table(conn).await?;
        Self::create_locks_table(conn).await?;

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
        "CREATE TABLE `{}` (repository_id INTEGER NOT NULL, id INTEGER NOT NULL {} PRIMARY KEY, owner VARCHAR(255), message TEXT, root_hash CHAR(64), date_time_utc VARCHAR(255), FOREIGN KEY (repository_id) REFERENCES `{}`(id) ON DELETE CASCADE);
         CREATE INDEX repository_id_commit on `{}`(repository_id, id);
         CREATE TABLE `{}` (id INTEGER NOT NULL, parent_id INTEGER NOT NULL);
         CREATE INDEX commit_parents_id on `{}`(id);
         CREATE TABLE `{}` (commit_id INTEGER NOT NULL, canonical_path TEXT NOT NULL, old_chunk_id VARCHAR(255), new_chunk_id VARCHAR(255), FOREIGN KEY (commit_id) REFERENCES `{}`(id) ON DELETE CASCADE);
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

    async fn create_forest_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}` (id VARCHAR(255) PRIMARY KEY, name VARCHAR(255), chunk_id VARCHAR(255));
            CREATE TABLE `{}` (id VARCHAR(255), child_id VARCHAR(255) NOT NULL, CONSTRAINT unique_link UNIQUE (id, child_id), FOREIGN KEY (id) REFERENCES `{}`(id) ON DELETE CASCADE, FOREIGN KEY (child_id) REFERENCES `{}`(id) ON DELETE CASCADE);
            CREATE INDEX forest_links_index on `{}`(id);",
            TABLE_FOREST,
            TABLE_FOREST_LINKS,
            TABLE_FOREST,
            TABLE_FOREST,
            TABLE_FOREST_LINKS,
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the forest table and tree index")
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

    async fn create_workspace_registrations_table(conn: &mut sqlx::AnyConnection) -> Result<()> {
        let sql: &str = &format!(
            "CREATE TABLE `{}` (id VARCHAR(255), owner VARCHAR(255), UNIQUE (id));",
            TABLE_WORKSPACE_REGISTRATIONS,
        );

        conn.execute(sql)
            .await
            .map_other_err("failed to create the workspace registrations table and index")
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
    async fn create_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("SqlRepositoryIndex::create_repository");

        let pool = self.driver.new_pool().await?;
        let index = SqlIndex::init(self.driver.clone(), pool, repository_name).await?;

        Ok(Box::new(index))
    }

    async fn destroy_repository(&self, repository_name: RepositoryName) -> Result<()> {
        async_span_scope!("SqlRepositoryIndex::destroy_repository");

        let pool = self.driver.new_pool().await?;
        let index = SqlIndex::load(self.driver.clone(), pool, repository_name).await?;

        index.cleanup_repository_data().await
    }

    async fn load_repository(&self, repository_name: RepositoryName) -> Result<Box<dyn Index>> {
        async_span_scope!("SqlRepositoryIndex::load_repository");

        let pool = self.driver.new_pool().await?;
        let index = SqlIndex::load(self.driver.clone(), pool, repository_name).await?;

        Ok(Box::new(index))
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
        repository_name: RepositoryName,
    ) -> Result<Self> {
        let conn = pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")?;

        let repository_id =
            Self::initialize_repository_data(conn, &repository_name, &driver).await?;

        Ok(Self {
            driver,
            pool,
            repository_name,
            repository_id,
        })
    }

    async fn load(
        driver: SqlDatabaseDriver,
        pool: Pool<sqlx::Any>,
        repository_name: RepositoryName,
    ) -> Result<Self> {
        let mut conn = pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")?;

        let repository_id = Self::get_repository_id(&mut conn, repository_name.clone()).await?;

        Ok(Self {
            driver,
            pool,
            repository_name,
            repository_id,
        })
    }

    async fn get_conn(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Any>> {
        self.pool
            .acquire()
            .await
            .map_other_err("failed to acquire SQL connection")
    }

    async fn get_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Any>> {
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

        let tree = Tree::empty();
        let tree_id = Self::save_tree_transactional(&mut transaction, &tree).await?;

        let initial_commit = Commit::new_unique_now(
            whoami::username(),
            String::from("initial commit"),
            BTreeSet::new(),
            tree_id,
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

    async fn read_branch_for_update<'e, E: sqlx::Executor<'e, Database = sqlx::Any>>(
        &self,
        executor: E,
        name: &str,
    ) -> Result<Branch> {
        async_span_scope!("SqlIndex::read_branch_for_update");

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

    async fn insert_repository_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_name: RepositoryName,
        driver: &SqlDatabaseDriver,
    ) -> Result<i64> {
        async_span_scope!("SqlIndex::insert_repository_transactional");

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

    async fn delete_repository_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
    ) -> Result<()> {
        async_span_scope!("SqlIndex::delete_repository_transactional");

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

    async fn get_repository_id(
        conn: &mut sqlx::pool::PoolConnection<sqlx::Any>,
        repository_name: RepositoryName,
    ) -> Result<i64> {
        async_span_scope!("SqlIndex::get_repository_id");

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
            Err(sqlx::Error::RowNotFound) => Err(Error::repository_does_not_exist(repository_name)),
            Err(err) => {
                Err(err).map_other_err(format!("failed to fetch repository `{}`", repository_name))
            }
        }
    }

    async fn insert_branch_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
        branch: &Branch,
    ) -> Result<()> {
        async_span_scope!("SqlIndex::insert_branch_transactional");

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

    async fn list_commits_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
        query: &ListCommitsQuery,
    ) -> Result<Vec<Commit>> {
        async_span_scope!("SqlIndex::list_commits_transactional");

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
                "SELECT canonical_path, old_chunk_id, new_chunk_id
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
                    let old_chunk_id: String = row.get("old_chunk_id");

                    let old_chunk_id = if !old_chunk_id.is_empty() {
                        match old_chunk_id
                            .parse()
                            .map_other_err("failed to parse the old chunk id")
                        {
                            Ok(id) => Some(id),
                            Err(err) => return Some(Err(err)),
                        }
                    } else {
                        None
                    };

                    let new_chunk_id: String = row.get("new_chunk_id");
                    let new_chunk_id = if !new_chunk_id.is_empty() {
                        match new_chunk_id
                            .parse()
                            .map_other_err("failed to parse the new chunk id")
                        {
                            Ok(id) => Some(id),
                            Err(err) => return Some(Err(err)),
                        }
                    } else {
                        None
                    };

                    ChangeType::new(old_chunk_id, new_chunk_id)
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
                    "SELECT owner, message, root_hash, date_time_utc 
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
                            row.get("root_hash"),
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

    async fn insert_commit_transactional(
        transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
        repository_id: i64,
        commit: &Commit,
    ) -> Result<CommitId> {
        async_span_scope!("SqlIndex::insert_commit_transactional");

        let result = sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, NULL, ?, ?, ?, ?);",
            TABLE_COMMITS
        ))
        .bind(repository_id)
        .bind(commit.owner.clone())
        .bind(commit.message.clone())
        .bind(commit.root_tree_id.clone())
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
                    .old_chunk_id()
                    .map(std::string::ToString::to_string)
                    .unwrap_or_default(),
            )
            .bind(
                change
                    .change_type()
                    .new_chunk_id()
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
            TABLE_FOREST
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
                "INSERT INTO `{}` (id, name, chunk_id) VALUES(?, ?, ?);",
                TABLE_FOREST,
            );

            match tree {
                Tree::Directory { name, children } => {
                    sqlx::query(sql)
                        .bind(&id)
                        .bind(name)
                        .bind(Option::<String>::None)
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
                Tree::File { name, chunk_id } => {
                    sqlx::query(sql)
                        .bind(&id)
                        .bind(name)
                        .bind(Some(&chunk_id.to_string()))
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
                TABLE_FOREST_LINKS,
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
            "SELECT name, chunk_id
             FROM `{}`
             WHERE id = ?;",
            TABLE_FOREST
        ))
        .bind(id)
        .fetch_one(&mut *transaction)
        .await
        .map_other_err(format!("failed to fetch tree node `{}`", id))?;

        let name = row.get("name");
        let chunk_id: Option<String> = row.get("chunk_id");

        let chunk_id = match chunk_id.as_deref() {
            None | Some("") => None,
            Some(chunk_id) => Some(
                chunk_id
                    .parse()
                    .map_other_err(format!("failed to parse chunk id for tree node `{}`", id))?,
            ),
        };

        let tree = Self::read_tree_node_transactional(transaction, id, name, chunk_id).await?;

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
        chunk_id: Option<ChunkIdentifier>,
    ) -> Result<Tree> {
        Ok(if let Some(chunk_id) = chunk_id {
            Tree::File { chunk_id, name }
        } else {
            let child_ids = sqlx::query(&format!(
                "SELECT child_id
                 FROM `{}`
                 WHERE id = ?;",
                TABLE_FOREST_LINKS
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
                    "SELECT name, chunk_id
                    FROM `{}`
                    WHERE id = ?;",
                    TABLE_FOREST
                ))
                .bind(&child_id)
                .fetch_one(&mut *transaction)
                .await
                .map_other_err(format!(
                    "failed to fetch children for tree node data `{}`",
                    &child_id
                ))?;

                let name: String = row.get("name");
                let chunk_id: String = row.get("chunk_id");

                let chunk_id = if !chunk_id.is_empty() {
                    Some(chunk_id.parse().map_other_err(format!(
                        "failed to parse chunk id for tree node `{}`",
                        &child_id
                    ))?)
                } else {
                    None
                };

                let child = Self::read_tree_node_transactional(
                    &mut *transaction,
                    &child_id,
                    name,
                    chunk_id,
                )
                .await?;

                children.insert(child.name().to_string(), child);
            }

            Tree::Directory { name, children }
        })
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

    async fn register_workspace(
        &self,
        workspace_registration: &WorkspaceRegistration,
    ) -> Result<()> {
        async_span_scope!("SqlIndex::register_workspace");
        let mut conn = self.get_conn().await?;

        sqlx::query(&format!(
            "INSERT INTO `{}` VALUES(?, ?);",
            TABLE_WORKSPACE_REGISTRATIONS
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

    async fn get_tree(&self, id: &str) -> Result<Tree> {
        async_span_scope!("SqlIndex::get_tree");
        let mut transaction = self.get_transaction().await?;

        Self::get_tree_transactional(&mut transaction, id).await
    }

    async fn save_tree(&self, tree: &Tree) -> Result<String> {
        async_span_scope!("SqlIndex::save_tree");
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
