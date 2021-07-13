use crate::*;
use chrono::prelude::*;
use futures::executor::block_on;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct HashedChange {
    pub relative_path: String,
    pub hash: String,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone)]
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
}

pub fn init_commit_database(connection: &mut RepositoryConnection) -> Result<(), String> {
    let sql_connection = connection.sql();
    let sql = "CREATE TABLE commits(id VARCHAR(255), owner VARCHAR(255), message TEXT, root_hash CHAR(64), date_time_utc VARCHAR(255));
         CREATE UNIQUE INDEX commit_id on commits(id);
         CREATE TABLE commit_parents(id VARCHAR(255), parent_id TEXT);
         CREATE INDEX commit_parents_id on commit_parents(id);
         CREATE TABLE commit_changes(commit_id VARCHAR(255), relative_path TEXT, hash CHAR(64), change_type INTEGER);
         CREATE INDEX commit_changes_commit on commit_changes(commit_id);
        ";
    if let Err(e) = execute_sql(sql_connection, sql) {
        return Err(format!("Error creating commit tables and indices: {}", e));
    }
    Ok(())
}

pub fn save_commit(connection: &mut RepositoryConnection, commit: &Commit) -> Result<(), String> {
    let sql_connection = connection.sql();

    if let Err(e) = block_on(
        sqlx::query("INSERT INTO commits VALUES(?, ?, ?, ?, ?);")
            .bind(commit.id.clone())
            .bind(commit.owner.clone())
            .bind(commit.message.clone())
            .bind(commit.root_hash.clone())
            .bind(commit.date_time_utc.clone())
            .execute(&mut *sql_connection),
    ) {
        return Err(format!("Error inserting into commits: {}", e));
    }

    for parent_id in &commit.parents {
        if let Err(e) = block_on(
            sqlx::query("INSERT INTO commit_parents VALUES(?, ?);")
                .bind(commit.id.clone())
                .bind(parent_id.clone())
                .execute(&mut *sql_connection),
        ) {
            return Err(format!("Error inserting into commit_parents: {}", e));
        }
    }

    for change in &commit.changes {
        if let Err(e) = block_on(
            sqlx::query("INSERT INTO commit_changes VALUES(?, ?, ?, ?);")
                .bind(commit.id.clone())
                .bind(change.relative_path.clone())
                .bind(change.hash.clone())
                .bind(change.change_type.clone() as i64)
                .execute(&mut *sql_connection),
        ) {
            return Err(format!("Error inserting into commit_changes: {}", e));
        }
    }

    Ok(())
}

pub fn read_commit(connection: &mut RepositoryConnection, id: &str) -> Result<Commit, String> {
    let sql_connection = connection.sql();
    let mut changes: Vec<HashedChange> = Vec::new();

    match block_on(
        sqlx::query(
            "SELECT relative_path, hash, change_type
             FROM commit_changes
             WHERE commit_id = ?;",
        )
        .bind(id)
        .fetch_all(&mut *sql_connection),
    ) {
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
    match block_on(
        sqlx::query(
            "SELECT parent_id
             FROM commit_parents
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_all(&mut *sql_connection),
    ) {
        Ok(rows) => {
            for r in rows {
                parents.push(r.get("parent_id"));
            }
        }
        Err(e) => {
            return Err(format!("Error fetching parents for commit {}: {}", id, e));
        }
    }

    match block_on(
        sqlx::query(
            "SELECT owner, message, root_hash, date_time_utc 
             FROM commits
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&mut *sql_connection),
    ) {
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

pub fn commit_exists(connection: &mut RepositoryConnection, id: &str) -> bool {
    let sql_connection = connection.sql();
    let res = block_on(
        sqlx::query(
            "SELECT count(*) as count
             FROM commits
             WHERE id = ?;",
        )
        .bind(id)
        .fetch_one(&mut *sql_connection),
    );
    let row = res.unwrap();
    let count: i32 = row.get("count");
    count > 0
}

fn write_blob(file_path: &Path, contents: &[u8]) -> Result<(), String> {
    if fs::metadata(file_path).is_ok() {
        //blob already exists
        return Ok(());
    }

    lz4_compress_to_file(file_path, contents)
}

fn upload_localy_edited_blobs(
    workspace_root: &Path,
    repo_connection: &RepositoryConnection,
    local_changes: &[LocalChange],
) -> Result<Vec<HashedChange>, String> {
    let blob_dir = repo_connection.blob_directory();
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
            write_blob(&blob_dir.join(&hash), &local_file_contents)?;
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

pub fn commit_local_changes(
    workspace_connection: &mut LocalWorkspaceConnection,
    commit_id: &str,
    message: &str,
) -> Result<(), String> {
    let workspace_root = workspace_connection.workspace_path().to_path_buf();
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let mut current_branch = read_current_branch(&workspace_root)?;
    let mut connection = connect_to_server(&workspace_spec)?;
    let repo_branch = read_branch_from_repo(&mut connection, &current_branch.name)?;
    if repo_branch.head != current_branch.head {
        return Err(String::from("Workspace is not up to date, aborting commit"));
    }
    let local_changes = read_local_changes(workspace_connection)?;
    for change in &local_changes {
        let abs_path = workspace_root.join(&change.relative_path);
        assert_not_locked(&workspace_root, &abs_path)?;
    }
    let hashed_changes = upload_localy_edited_blobs(&workspace_root, &connection, &local_changes)?;

    let base_commit = read_commit(&mut connection, &current_branch.head)?;

    let new_root_hash = update_tree_from_changes(
        &read_tree(&mut connection, &base_commit.root_hash)?,
        &hashed_changes,
        &mut connection,
    )?;

    let mut parent_commits = Vec::from([base_commit.id]);
    for pending_branch_merge in read_pending_branch_merges(workspace_connection)? {
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
    save_commit(&mut connection, &commit)?;
    current_branch.head = commit.id;
    save_current_branch(&workspace_root, &current_branch)?;

    //todo: will need to lock to avoid races in updating branch in the database
    save_branch_to_repo(&mut connection, &current_branch)?;

    if let Err(e) = make_local_files_read_only(&workspace_root, &commit.changes) {
        println!("Error making local files read only: {}", e);
    }
    clear_local_changes(workspace_connection, &local_changes);
    if let Err(e) = clear_pending_branch_merges(workspace_connection) {
        println!("{}", e);
    }
    Ok(())
}

pub fn commit_command(message: &str) -> Result<(), String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let id = uuid::Uuid::new_v4().to_string();
    commit_local_changes(&mut workspace_connection, &id, message)
}

pub fn find_branch_commits(
    connection: &mut RepositoryConnection,
    branch: &Branch,
) -> Result<Vec<Commit>, String> {
    let mut commits = Vec::new();
    let mut c = read_commit(connection, &branch.head)?;
    println!("{}", c.id);
    commits.push(c.clone());
    while !c.parents.is_empty() {
        let id = &c.parents[0]; //first parent is assumed to be branch trunk
        c = read_commit(connection, id)?;
        println!("{}", c.id);
        commits.push(c.clone());
    }
    Ok(commits)
}
