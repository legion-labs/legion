use crate::{sql::execute_sql, *};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HashedChange {
    pub relative_path: String,
    pub hash: String,
    pub change_type: ChangeType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Commit {
    pub id: String,
    pub owner: String,
    pub message: String,
    pub changes: Vec<HashedChange>,
    pub root_hash: String,
    pub parents: Vec<String>,
    pub date_time_utc: String,
}

impl Commit {
    pub fn new(
        id: String,
        owner: String,
        message: String,
        changes: Vec<HashedChange>,
        root_hash: String,
        parents: Vec<String>,
    ) -> Self {
        let date_time_utc = Utc::now().to_rfc3339();
        assert!(!parents.contains(&id));
        Self {
            id,
            owner,
            message,
            changes,
            root_hash,
            parents,
            date_time_utc,
        }
    }

    pub fn from_json(contents: &str) -> Result<Self, String> {
        let parsed: serde_json::Result<Self> = serde_json::from_str(contents);
        match parsed {
            Ok(obj) => Ok(obj),
            Err(e) => Err(format!("Error parsing commit: {}", e)),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        match serde_json::to_string(&self) {
            Ok(json) => Ok(json),
            Err(e) => Err(format!("Error formatting commit {:?}: {}", self.id, e)),
        }
    }
}

pub async fn init_commit_database(sql_connection: &mut sqlx::AnyConnection) -> Result<(), String> {
    let sql = "CREATE TABLE commits(id VARCHAR(255), owner VARCHAR(255), message TEXT, root_hash CHAR(64), date_time_utc VARCHAR(255));
         CREATE UNIQUE INDEX commit_id on commits(id);
         CREATE TABLE commit_parents(id VARCHAR(255), parent_id TEXT);
         CREATE INDEX commit_parents_id on commit_parents(id);
         CREATE TABLE commit_changes(commit_id VARCHAR(255), relative_path TEXT, hash CHAR(64), change_type INTEGER);
         CREATE INDEX commit_changes_commit on commit_changes(commit_id);
        ";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating commit tables and indices: {}", e));
    }
    Ok(())
}

async fn upload_localy_edited_blobs(
    workspace_root: &Path,
    repo_connection: &RepositoryConnection,
    local_changes: &[LocalChange],
) -> Result<Vec<HashedChange>, String> {
    let mut res = Vec::<HashedChange>::new();
    for local_change in local_changes {
        if local_change.change_type == ChangeType::Delete {
            res.push(HashedChange {
                relative_path: local_change.relative_path.clone(),
                hash: String::from(""),
                change_type: local_change.change_type.clone(),
            });
        } else {
            let local_path = workspace_root.join(&local_change.relative_path);
            let local_file_contents = read_bin_file(&local_path)?;
            let hash = format!("{:X}", Sha256::digest(&local_file_contents));
            repo_connection
                .blob_storage()
                .await?
                .write_blob(&hash, &local_file_contents)
                .await?;
            res.push(HashedChange {
                relative_path: local_change.relative_path.clone(),
                hash: hash.clone(),
                change_type: local_change.change_type.clone(),
            });
        }
    }
    Ok(res)
}

fn make_local_files_read_only(
    workspace_root: &Path,
    changes: &[HashedChange],
) -> Result<(), String> {
    for change in changes {
        if change.change_type != ChangeType::Delete {
            let full_path = workspace_root.join(&change.relative_path);
            make_file_read_only(&full_path, true)?;
        }
    }
    Ok(())
}

pub async fn commit_local_changes(
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    commit_id: &str,
    message: &str,
) -> Result<(), String> {
    let workspace_spec = read_workspace_spec(workspace_root)?;
    let (current_branch_name, current_workspace_commit) =
        read_current_branch(workspace_transaction).await?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.query();
    let mut repo_branch = query.read_branch(&current_branch_name).await?;

    if repo_branch.head != current_workspace_commit {
        // Check early to save work, but the real transaction lock will happen later.
        // Don't want to lock too early because a slow client would block everyone.
        return Err(String::from("Workspace is not up to date, aborting commit"));
    }
    let local_changes = read_local_changes(workspace_transaction).await?;
    for change in &local_changes {
        let abs_path = workspace_root.join(&change.relative_path);
        assert_not_locked(query, workspace_root, workspace_transaction, &abs_path).await?;
    }
    let hashed_changes =
        upload_localy_edited_blobs(workspace_root, &connection, &local_changes).await?;

    let base_commit = query.read_commit(&current_workspace_commit).await?;

    let new_root_hash = update_tree_from_changes(
        &query.read_tree(&base_commit.root_hash).await?,
        &hashed_changes,
        &connection,
    )
    .await?;

    let mut parent_commits = Vec::from([base_commit.id]);
    for pending_branch_merge in read_pending_branch_merges(workspace_transaction).await? {
        parent_commits.push(pending_branch_merge.head.clone());
    }

    let commit = Commit::new(
        String::from(commit_id),
        whoami::username(),
        String::from(message),
        hashed_changes,
        new_root_hash,
        parent_commits,
    );
    query.insert_commit(&commit).await?;
    repo_branch.head = commit.id.clone();
    update_current_branch(workspace_transaction, &current_branch_name, &commit.id).await?;

    //todo: will need to lock to avoid races in updating branch in the database
    query.update_branch(&repo_branch).await?;

    if let Err(e) = make_local_files_read_only(workspace_root, &commit.changes) {
        println!("Error making local files read only: {}", e);
    }
    clear_local_changes(workspace_transaction, &local_changes).await;
    if let Err(e) = clear_pending_branch_merges(workspace_transaction).await {
        println!("{}", e);
    }
    Ok(())
}

pub async fn commit_command(message: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let id = uuid::Uuid::new_v4().to_string();
    commit_local_changes(&workspace_root, &mut workspace_transaction, &id, message).await?;
    if let Err(e) = workspace_transaction.commit().await {
        return Err(format!(
            "Error in transaction commit for commit_command: {}",
            e
        ));
    }
    Ok(())
}

pub async fn find_branch_commits(
    connection: &RepositoryConnection,
    branch: &Branch,
) -> Result<Vec<Commit>, String> {
    let mut commits = Vec::new();
    let query = connection.query();
    let mut c = query.read_commit(&branch.head).await?;
    println!("{}", c.id);
    commits.push(c.clone());
    while !c.parents.is_empty() {
        let id = &c.parents[0]; //first parent is assumed to be branch trunk
        c = query.read_commit(id).await?;
        println!("{}", c.id);
        commits.push(c.clone());
    }
    Ok(commits)
}
