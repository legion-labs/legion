use crate::{sql::*, *};
use async_trait::async_trait;
use sqlx::Row;
use std::sync::Arc;

// access to repository metadata inside a mysql or sqlite database
pub struct SqlRepositoryQuery {
    pool: Arc<SqlConnectionPool>,
}

impl SqlRepositoryQuery {
    pub fn new(pool: Arc<SqlConnectionPool>) -> Self {
        Self { pool }
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

    async fn insert_branch(&self, branch: &Branch) -> Result<(), String> {
        let mut sql_connection = self.pool.acquire().await?;
        if let Err(e) = sqlx::query("INSERT INTO branches VALUES(?, ?, ?, ?);")
            .bind(branch.name.clone())
            .bind(branch.head.clone())
            .bind(branch.parent.clone())
            .bind(branch.lock_domain_id.clone())
            .execute(&mut sql_connection)
            .await
        {
            return Err(format!("Error inserting into branches: {}", e));
        }
        Ok(())
    }

    async fn update_branch(&self, branch: &Branch) -> Result<(), String> {
        let mut transaction = self.pool.begin().await?;
        update_branch_tr(&mut transaction, branch).await?;
        if let Err(e) = transaction.commit().await {
            return Err(format!(
                "Error in transaction commit for update_branch: {}",
                e
            ));
        }
        Ok(())
    }

    async fn read_branch(&self, name: &str) -> Result<Branch, String> {
        match self.find_branch(name).await {
            Ok(Some(branch)) => Ok(branch),
            Ok(None) => Err(format!("branch not found {}", name)),
            Err(e) => Err(e),
        }
    }

    async fn find_branch(&self, name: &str) -> Result<Option<Branch>, String> {
        let mut sql_connection = self.pool.acquire().await?;
        match sqlx::query(
            "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
        )
        .bind(name)
        .fetch_optional(&mut sql_connection)
        .await
        {
            Ok(None) => Ok(None),
            Ok(Some(row)) => {
                let branch = Branch::new(
                    String::from(name),
                    row.get("head"),
                    row.get("parent"),
                    row.get("lock_domain_id"),
                );
                Ok(Some(branch))
            }
            Err(e) => Err(format!("Error fetching branch {}: {}", name, e)),
        }
    }

    async fn find_branches_in_lock_domain(
        &self,
        lock_domain_id: &str,
    ) -> Result<Vec<Branch>, String> {
        let mut sql_connection = self.pool.acquire().await?;
        let mut res = Vec::new();
        match sqlx::query(
            "SELECT name, head, parent 
             FROM branches
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut sql_connection)
        .await
        {
            Ok(rows) => {
                for r in rows {
                    let branch = Branch::new(
                        r.get("name"),
                        r.get("head"),
                        r.get("parent"),
                        String::from(lock_domain_id),
                    );
                    res.push(branch);
                }
                Ok(res)
            }
            Err(e) => Err(format!("Error fetching branches: {}", e)),
        }
    }

    async fn read_branches(&self) -> Result<Vec<Branch>, String> {
        let mut sql_connection = self.pool.acquire().await?;
        let mut res = Vec::new();
        match sqlx::query(
            "SELECT name, head, parent, lock_domain_id 
             FROM branches;",
        )
        .fetch_all(&mut sql_connection)
        .await
        {
            Ok(rows) => {
                for r in rows {
                    let branch = Branch::new(
                        r.get("name"),
                        r.get("head"),
                        r.get("parent"),
                        r.get("lock_domain_id"),
                    );
                    res.push(branch);
                }
                Ok(res)
            }
            Err(e) => Err(format!("Error fetching branches: {}", e)),
        }
    }

    async fn read_commit(&self, id: &str) -> Result<Commit, String> {
        let mut sql_connection = self.pool.acquire().await?;
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

    async fn insert_commit(&self, commit: &Commit) -> Result<(), String> {
        let mut transaction = self.pool.begin().await?;
        insert_commit_tr(&mut transaction, commit).await?;
        if let Err(e) = transaction.commit().await {
            return Err(format!(
                "Error in transaction commit for insert_commit: {}",
                e
            ));
        }
        Ok(())
    }

    async fn commit_to_branch(&self, commit: &Commit, branch: &Branch) -> Result<(), String> {
        let mut transaction = self.pool.begin().await?;
        let stored_branch = read_branch_tr(&mut transaction, &branch.name).await?;
        if &stored_branch != branch {
            //rollback is implicit but there is bug in sqlx: https://github.com/launchbadge/sqlx/issues/1358
            if let Err(e) = transaction.rollback().await {
                println!("Error in rollback: {}", e);
            }
            return Err(format!(
                "Error: commit on stale branch. Branch {} is now at commit {}",
                branch.name, stored_branch.head
            ));
        }
        insert_commit_tr(&mut transaction, commit).await?;
        let mut new_branch = branch.clone();
        new_branch.head = commit.id.clone();
        update_branch_tr(&mut transaction, &new_branch).await?;
        if let Err(e) = transaction.commit().await {
            return Err(format!(
                "Error in transaction commit for commit_to_branch: {}",
                e
            ));
        }
        Ok(())
    }

    async fn commit_exists(&self, id: &str) -> Result<bool, String> {
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

    async fn read_tree(&self, hash: &str) -> Result<Tree, String> {
        let mut sql_connection = self.pool.acquire().await?;
        let mut directory_nodes: Vec<TreeNode> = Vec::new();
        let mut file_nodes: Vec<TreeNode> = Vec::new();

        match sqlx::query(
            "SELECT name, hash, node_type
             FROM tree_nodes
             WHERE parent_tree_hash = ?
             ORDER BY name;",
        )
        .bind(hash)
        .fetch_all(&mut sql_connection)
        .await
        {
            Ok(rows) => {
                for r in rows {
                    let name: String = r.get("name");
                    let node_hash: String = r.get("hash");
                    let node_type: i64 = r.get("node_type");
                    let node = TreeNode::new(name, node_hash);
                    if node_type == TreeNodeType::Directory as i64 {
                        directory_nodes.push(node);
                    } else if node_type == TreeNodeType::File as i64 {
                        file_nodes.push(node);
                    }
                }
            }
            Err(e) => {
                return Err(format!("Error fetching tree nodes for {}: {}", hash, e));
            }
        }

        Ok(Tree {
            directory_nodes,
            file_nodes,
        })
    }

    async fn save_tree(&self, tree: &Tree, hash: &str) -> Result<(), String> {
        let mut sql_connection = self.pool.acquire().await?;
        let tree_in_db = self.read_tree(hash).await?;
        if !tree.is_empty() && !tree_in_db.is_empty() {
            return Ok(());
        }
        for file_node in &tree.file_nodes {
            if let Err(e) = sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(file_node.name.clone())
                .bind(file_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::File as i64)
                .execute(&mut sql_connection)
                .await
            {
                return Err(format!("Error inserting into tree_nodes: {}", e));
            }
        }
        for dir_node in &tree.directory_nodes {
            if let Err(e) = sqlx::query("INSERT INTO tree_nodes VALUES(?, ?, ?, ?);")
                .bind(dir_node.name.clone())
                .bind(dir_node.hash.clone())
                .bind(hash)
                .bind(TreeNodeType::Directory as i64)
                .execute(&mut sql_connection)
                .await
            {
                return Err(format!("Error inserting into tree_nodes: {}", e));
            }
        }
        Ok(())
    }

    async fn insert_lock(&self, lock: &Lock) -> Result<(), String> {
        let mut sql_connection = self.pool.acquire().await?;
        match sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE relative_path = ?
             AND lock_domain_id = ?;",
        )
        .bind(lock.relative_path.clone())
        .bind(lock.lock_domain_id.clone())
        .fetch_one(&mut sql_connection)
        .await
        {
            Err(e) => {
                return Err(format!("Error counting locks: {}", e));
            }
            Ok(row) => {
                let count: i32 = row.get("count");
                if count > 0 {
                    return Err(format!(
                        "Lock {} already exists in domain {}",
                        lock.relative_path, lock.lock_domain_id
                    ));
                }
            }
        }
        if let Err(e) = sqlx::query("INSERT INTO locks VALUES(?, ?, ?, ?);")
            .bind(lock.relative_path.clone())
            .bind(lock.lock_domain_id.clone())
            .bind(lock.workspace_id.clone())
            .bind(lock.branch_name.clone())
            .execute(&mut sql_connection)
            .await
        {
            return Err(format!("Error inserting into locks: {}", e));
        }
        Ok(())
    }

    async fn find_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<Option<Lock>, String> {
        let mut sql_connection = self.pool.acquire().await?;
        match sqlx::query(
            "SELECT workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?
             AND relative_path=?;",
        )
        .bind(lock_domain_id)
        .bind(canonical_relative_path)
        .fetch_optional(&mut sql_connection)
        .await
        {
            Ok(None) => Ok(None),
            Ok(Some(row)) => Ok(Some(Lock {
                relative_path: String::from(canonical_relative_path),
                lock_domain_id: String::from(lock_domain_id),
                workspace_id: row.get("workspace_id"),
                branch_name: row.get("branch_name"),
            })),
            Err(e) => Err(format!("Error fetching lock: {}", e)),
        }
    }

    async fn find_locks_in_domain(&self, lock_domain_id: &str) -> Result<Vec<Lock>, String> {
        let mut sql_connection = self.pool.acquire().await?;
        match sqlx::query(
            "SELECT relative_path, workspace_id, branch_name
             FROM locks
             WHERE lock_domain_id=?;",
        )
        .bind(lock_domain_id)
        .fetch_all(&mut sql_connection)
        .await
        {
            Ok(rows) => {
                let mut locks = Vec::new();
                for r in rows {
                    locks.push(Lock {
                        relative_path: r.get("relative_path"),
                        lock_domain_id: String::from(lock_domain_id),
                        workspace_id: r.get("workspace_id"),
                        branch_name: r.get("branch_name"),
                    });
                }
                Ok(locks)
            }
            Err(e) => Err(format!("Error listing locks: {}", e)),
        }
    }

    async fn clear_lock(
        &self,
        lock_domain_id: &str,
        canonical_relative_path: &str,
    ) -> Result<(), String> {
        let mut sql_connection = self.pool.acquire().await?;
        if let Err(e) = sqlx::query("DELETE from locks WHERE relative_path=? AND lock_domain_id=?;")
            .bind(canonical_relative_path)
            .bind(lock_domain_id)
            .execute(&mut sql_connection)
            .await
        {
            return Err(format!("Error clearing lock: {}", e));
        }
        Ok(())
    }

    async fn count_locks_in_domain(&self, lock_domain_id: &str) -> Result<i32, String> {
        let mut sql_connection = self.pool.acquire().await?;
        match sqlx::query(
            "SELECT count(*) as count
             FROM locks
             WHERE lock_domain_id = ?;",
        )
        .bind(lock_domain_id)
        .fetch_one(&mut sql_connection)
        .await
        {
            Err(e) => Err(format!("Error counting locks: {}", e)),
            Ok(row) => {
                let count: i32 = row.get("count");
                Ok(count)
            }
        }
    }

    async fn read_blob_storage_spec(&self) -> Result<BlobStorageSpec, String> {
        let mut sql_connection = self.pool.acquire().await?;
        match sqlx::query(
            "SELECT blob_storage_spec 
             FROM config;",
        )
        .fetch_one(&mut sql_connection)
        .await
        {
            Ok(row) => BlobStorageSpec::from_json(row.get("blob_storage_spec")),
            Err(e) => Err(format!("Error fetching blob storage spec: {}", e)),
        }
    }
}

async fn insert_commit_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    commit: &Commit,
) -> Result<(), String> {
    if let Err(e) = sqlx::query("INSERT INTO commits VALUES(?, ?, ?, ?, ?);")
        .bind(commit.id.clone())
        .bind(commit.owner.clone())
        .bind(commit.message.clone())
        .bind(commit.root_hash.clone())
        .bind(commit.date_time_utc.clone())
        .execute(&mut *tr)
        .await
    {
        return Err(format!("Error inserting into commits: {}", e));
    }
    for parent_id in &commit.parents {
        if let Err(e) = sqlx::query("INSERT INTO commit_parents VALUES(?, ?);")
            .bind(commit.id.clone())
            .bind(parent_id.clone())
            .execute(&mut *tr)
            .await
        {
            return Err(format!("Error inserting into commit_parents: {}", e));
        }
    }
    for change in &commit.changes {
        if let Err(e) = sqlx::query("INSERT INTO commit_changes VALUES(?, ?, ?, ?);")
            .bind(commit.id.clone())
            .bind(change.relative_path.clone())
            .bind(change.hash.clone())
            .bind(change.change_type.clone() as i64)
            .execute(&mut *tr)
            .await
        {
            return Err(format!("Error inserting into commit_changes: {}", e));
        }
    }
    Ok(())
}

async fn update_branch_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    branch: &Branch,
) -> Result<(), String> {
    if let Err(e) = sqlx::query(
        "UPDATE branches SET head=?, parent=?, lock_domain_id=?
             WHERE name=?;",
    )
    .bind(branch.head.clone())
    .bind(branch.parent.clone())
    .bind(branch.lock_domain_id.clone())
    .bind(branch.name.clone())
    .execute(tr)
    .await
    {
        return Err(format!("Error updating branch {}: {}", branch.name, e));
    }
    Ok(())
}

//read_branch_tr: locks the row
async fn read_branch_tr(
    tr: &mut sqlx::Transaction<'_, sqlx::Any>,
    name: &str,
) -> Result<Branch, String> {
    //mysql version:
    // match sqlx::query(
    //     "SELECT head, parent, lock_domain_id
    //          FROM branches
    //          WHERE name = ?
    //          FOR UPDATE;",
    // )
    //sqlite version:
    match sqlx::query(
        "SELECT head, parent, lock_domain_id 
             FROM branches
             WHERE name = ?;",
    )
    .bind(name)
    .fetch_one(tr)
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
