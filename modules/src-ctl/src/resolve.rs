use crate::{sql::*, *};
use futures::executor::block_on;
use sqlx::Row;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Debug)]
pub struct ResolvePending {
    pub relative_path: String,
    pub base_commit_id: String,
    pub theirs_commit_id: String,
}

impl ResolvePending {
    pub fn new(
        canonical_relative_path: String,
        base_commit_id: String,
        theirs_commit_id: String,
    ) -> Self {
        Self {
            relative_path: canonical_relative_path,
            base_commit_id,
            theirs_commit_id,
        }
    }
}

pub async fn init_resolve_pending_database(
    workspace_connection: &mut LocalWorkspaceConnection,
) -> Result<(), String> {
    let sql_connection = workspace_connection.sql();
    let sql = "CREATE TABLE resolves_pending(relative_path VARCHAR(512) NOT NULL PRIMARY KEY, base_commit_id VARCHAR(255), theirs_commit_id VARCHAR(255));";
    if let Err(e) = execute_sql(sql_connection, sql).await {
        return Err(format!("Error creating resolves_pending table: {}", e));
    }
    Ok(())
}

pub fn save_resolve_pending(
    workspace_connection: &mut LocalWorkspaceConnection,
    resolve_pending: &ResolvePending,
) -> Result<(), String> {
    let sql_connection = workspace_connection.sql();
    if let Err(e) = block_on(
        sqlx::query("INSERT OR REPLACE into resolves_pending VALUES(?,?,?);")
            .bind(resolve_pending.relative_path.clone())
            .bind(resolve_pending.base_commit_id.clone())
            .bind(resolve_pending.theirs_commit_id.clone())
            .execute(&mut *sql_connection),
    ) {
        return Err(format!(
            "Error updating resolve pending {}: {}",
            resolve_pending.relative_path, e
        ));
    }
    Ok(())
}

pub fn clear_resolve_pending(
    workspace_connection: &mut LocalWorkspaceConnection,
    resolve_pending: &ResolvePending,
) -> Result<(), String> {
    let sql_connection = workspace_connection.sql();
    if let Err(e) = block_on(
        sqlx::query(
            "DELETE from resolves_pending
             WHERE relative_path=?;",
        )
        .bind(resolve_pending.relative_path.clone())
        .execute(&mut *sql_connection),
    ) {
        return Err(format!(
            "Error clearing resolve pending {}: {}",
            resolve_pending.relative_path, e
        ));
    }
    Ok(())
}

pub fn find_resolve_pending(
    workspace_connection: &mut LocalWorkspaceConnection,
    canonical_relative_path: &str,
) -> Result<Option<ResolvePending>, String> {
    let sql_connection = workspace_connection.sql();
    match block_on(
        sqlx::query(
            "SELECT base_commit_id, theirs_commit_id 
             FROM resolves_pending
             WHERE relative_path = ?;",
        )
        .bind(canonical_relative_path)
        .fetch_optional(&mut *sql_connection),
    ) {
        Ok(None) => Ok(None),
        Ok(Some(row)) => {
            let resolve_pending = ResolvePending::new(
                String::from(canonical_relative_path),
                row.get("base_commit_id"),
                row.get("theirs_commit_id"),
            );
            Ok(Some(resolve_pending))
        }
        Err(e) => Err(format!(
            "Error fetching resolve pending {}: {}",
            canonical_relative_path, e
        )),
    }
}

fn read_resolves_pending(
    workspace_connection: &mut LocalWorkspaceConnection,
) -> Result<Vec<ResolvePending>, String> {
    let sql_connection = workspace_connection.sql();
    let mut res = Vec::new();
    match block_on(
        sqlx::query(
            "SELECT relative_path, base_commit_id, theirs_commit_id 
             FROM resolves_pending;",
        )
        .fetch_all(&mut *sql_connection),
    ) {
        Ok(rows) => {
            for row in rows {
                let resolve_pending = ResolvePending::new(
                    row.get("relative_path"),
                    row.get("base_commit_id"),
                    row.get("theirs_commit_id"),
                );
                res.push(resolve_pending);
            }
            Ok(res)
        }
        Err(e) => Err(format!("Error fetching resolves pending: {}", e)),
    }
}

pub fn find_resolves_pending_command() -> Result<Vec<ResolvePending>, String> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    read_resolves_pending(&mut workspace_connection)
}

pub async fn find_file_hash_at_commit(
    connection: &RepositoryConnection,
    relative_path: &Path,
    commit_id: &str,
) -> Result<Option<String>, String> {
    let query = connection.query();
    let commit = query.read_commit(commit_id).await?;
    let root_tree = query.read_tree(&commit.root_hash).await?;
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let dir_tree = fetch_tree_subdir(query, &root_tree, parent_dir).await?;
    match dir_tree.find_file_node(
        relative_path
            .file_name()
            .expect("no file name in path specified")
            .to_str()
            .expect("invalid file name"),
    ) {
        Some(file_node) => Ok(Some(file_node.hash.clone())),
        None => Ok(None),
    }
}

fn run_merge_program(
    relative_path: &Path,
    abs_path: &str,
    theirs_path: &str,
    base_path: &str,
    output_path: &str,
) -> Result<(), String> {
    let config = Config::read_config()?;
    match config.find_merge_command(relative_path) {
        Some(mut external_command_vec) => {
            for item in &mut external_command_vec[..] {
                *item = item.replace("%local", abs_path);
                *item = item.replace("%theirs", theirs_path);
                *item = item.replace("%base", base_path);
                *item = item.replace("%output", output_path);
            }

            match Command::new(&external_command_vec[0])
                .args(&external_command_vec[1..])
                .output()
            {
                Ok(output) => {
                    let mut out = std::io::stdout();
                    out.write_all(&output.stdout).unwrap();
                    out.flush().unwrap();

                    let mut err = std::io::stderr();
                    err.write_all(&output.stderr).unwrap();
                    err.flush().unwrap();
                }
                Err(e) => {
                    return Err(format!(
                        "Error executing external command {:?}: {}",
                        external_command_vec, e
                    ));
                }
            }
        }
        None => {
            return Err(format!(
                "No merge command corresponding to {} was found in {}",
                relative_path.display(),
                Config::config_file_path().unwrap().display()
            ));
        }
    }
    Ok(())
}

fn run_diffy_merge(yours_path: &Path, theirs_path: &Path, base_path: &Path) -> Result<(), String> {
    let yours = read_bin_file(yours_path)?;
    let theirs = read_bin_file(theirs_path)?;
    let base = read_bin_file(base_path)?;
    match diffy::merge_bytes(&base, &yours, &theirs) {
        Ok(merged_contents) => {
            write_file(yours_path, &merged_contents)?;
            println!("Merge completed, {} updated", yours_path.display());
        }
        Err(conflicts) => {
            write_file(yours_path, &conflicts)?;
            println!(
                "Merge *not* completed, please resolve conflicts in {}",
                yours_path.display()
            );
        }
    }
    Ok(())
}

pub async fn resolve_file_command(p: &Path, allow_tools: bool) -> Result<(), String> {
    let abs_path = make_path_absolute(p);
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let relative_path = make_canonical_relative_path(&workspace_root, p)?;
    match find_resolve_pending(&mut workspace_connection, &relative_path) {
        Err(e) => {
            return Err(format!(
                "Error finding resolve pending for file {}: {}",
                p.display(),
                e
            ));
        }
        Ok(None) => {
            return Err(format!(
                "Pending resolve for file {} not found",
                p.display()
            ));
        }
        Ok(Some(resolve_pending)) => {
            let base_file_hash = find_file_hash_at_commit(
                &connection,
                Path::new(&relative_path),
                &resolve_pending.base_commit_id,
            )
            .await?
            .unwrap();
            let base_temp_file =
                download_temp_file(&connection, &workspace_root, &base_file_hash).await?;
            let theirs_file_hash = find_file_hash_at_commit(
                &connection,
                Path::new(&relative_path),
                &resolve_pending.theirs_commit_id,
            )
            .await?
            .unwrap();
            let theirs_temp_file =
                download_temp_file(&connection, &workspace_root, &theirs_file_hash).await?;
            let tmp_dir = workspace_root.join(".lsc/tmp");
            let output_temp_file = TempPath {
                path: tmp_dir.join(format!("merge_output_{}", uuid::Uuid::new_v4().to_string())),
            };
            if !allow_tools {
                run_diffy_merge(&abs_path, &theirs_temp_file.path, &base_temp_file.path)?;
                clear_resolve_pending(&mut workspace_connection, &resolve_pending)?;
                return Ok(());
            }

            run_merge_program(
                Path::new(&relative_path),
                abs_path.to_str().unwrap(),
                theirs_temp_file.path.to_str().unwrap(),
                base_temp_file.path.to_str().unwrap(),
                output_temp_file.path.to_str().unwrap(),
            )?;
            if let Err(e) = fs::copy(&output_temp_file.path, &abs_path) {
                return Err(format!(
                    "Error copying {} to {}: {}",
                    output_temp_file.path.display(),
                    abs_path.display(),
                    e
                ));
            }
            println!("Merge accepted, {} updated", abs_path.display());
            clear_resolve_pending(&mut workspace_connection, &resolve_pending)?;
        }
    }
    Ok(())
}
