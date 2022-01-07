use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;
use sqlx::Row;

use crate::{
    connect_to_server, download_temp_file, fetch_tree_subdir, find_workspace_root,
    make_canonical_relative_path, make_path_absolute, read_bin_file, read_workspace_spec,
    sql::execute_sql, write_file, Config, LocalWorkspaceConnection, RepositoryConnection,
};

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
) -> Result<()> {
    let sql_connection = workspace_connection.sql();
    let sql = "CREATE TABLE resolves_pending(relative_path VARCHAR(512) NOT NULL PRIMARY KEY, base_commit_id VARCHAR(255), theirs_commit_id VARCHAR(255));";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating resolves_pending table")
}

pub async fn save_resolve_pending(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    resolve_pending: &ResolvePending,
) -> Result<()> {
    sqlx::query("INSERT OR REPLACE into resolves_pending VALUES(?,?,?);")
        .bind(resolve_pending.relative_path.clone())
        .bind(resolve_pending.base_commit_id.clone())
        .bind(resolve_pending.theirs_commit_id.clone())
        .execute(workspace_transaction)
        .await
        .context(format!(
            "error saving resolve pending for {}",
            resolve_pending.relative_path
        ))?;

    Ok(())
}

pub async fn clear_resolve_pending(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    resolve_pending: &ResolvePending,
) -> Result<()> {
    sqlx::query(
        "DELETE from resolves_pending
             WHERE relative_path=?;",
    )
    .bind(resolve_pending.relative_path.clone())
    .execute(workspace_transaction)
    .await
    .context(format!(
        "error clearing resolve pending for {}",
        resolve_pending.relative_path
    ))?;

    Ok(())
}

pub async fn find_resolve_pending(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    canonical_relative_path: &str,
) -> Result<Option<ResolvePending>> {
    Ok(sqlx::query(
        "SELECT base_commit_id, theirs_commit_id 
             FROM resolves_pending
             WHERE relative_path = ?;",
    )
    .bind(canonical_relative_path)
    .fetch_optional(workspace_transaction)
    .await
    .context(format!(
        "error finding resolve pending for {}",
        canonical_relative_path
    ))?
    .map(|row| {
        ResolvePending::new(
            String::from(canonical_relative_path),
            row.get("base_commit_id"),
            row.get("theirs_commit_id"),
        )
    }))
}

async fn read_resolves_pending(
    workspace_connection: &mut LocalWorkspaceConnection,
) -> Result<Vec<ResolvePending>> {
    let sql_connection = workspace_connection.sql();

    Ok(sqlx::query(
        "SELECT relative_path, base_commit_id, theirs_commit_id 
             FROM resolves_pending;",
    )
    .fetch_all(&mut *sql_connection)
    .await
    .context("error fetching resolves pending")?
    .into_iter()
    .map(|row| {
        ResolvePending::new(
            row.get("relative_path"),
            row.get("base_commit_id"),
            row.get("theirs_commit_id"),
        )
    })
    .collect())
}

#[span_fn]
pub async fn find_resolves_pending_command() -> Result<Vec<ResolvePending>> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    read_resolves_pending(&mut workspace_connection).await
}

pub async fn find_file_hash_at_commit(
    connection: &RepositoryConnection,
    relative_path: &Path,
    commit_id: &str,
) -> Result<Option<String>> {
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

#[span_fn]
fn run_merge_program(
    relative_path: &Path,
    abs_path: &str,
    theirs_path: &str,
    base_path: &str,
    output_path: &str,
) -> Result<()> {
    let config = Config::read_config()?;
    let external_command_vec: Vec<_> = config
        .find_merge_command(relative_path)
        .context(format!(
            "error finding merge command for {} in {}",
            relative_path.display(),
            Config::config_file_path().unwrap().display()
        ))?
        .into_iter()
        .map(|item| {
            item.replace("%local", abs_path)
                .replace("%theirs", theirs_path)
                .replace("%base", base_path)
                .replace("%output", output_path)
        })
        .collect();

    let output = Command::new(&external_command_vec[0])
        .args(&external_command_vec[1..])
        .output()
        .context("error running external merge program")?;

    let mut out = std::io::stdout();
    out.write_all(&output.stdout).unwrap();
    out.flush().unwrap();

    let mut err = std::io::stderr();
    err.write_all(&output.stderr).unwrap();
    err.flush().unwrap();

    Ok(())
}

fn run_diffy_merge(yours_path: &Path, theirs_path: &Path, base_path: &Path) -> Result<()> {
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

#[span_fn]
pub async fn resolve_file_command(p: &Path, allow_tools: bool) -> Result<()> {
    let abs_path = make_path_absolute(p);
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let relative_path = make_canonical_relative_path(&workspace_root, p)?;

    let resolve_pending = find_resolve_pending(&mut workspace_transaction, &relative_path)
        .await
        .context(format!("error finding resolve pending for {}", p.display()))?
        .ok_or_else(|| anyhow::anyhow!("no resolve pending found for {}", p.display(),))?;

    let base_file_hash = find_file_hash_at_commit(
        &connection,
        Path::new(&relative_path),
        &resolve_pending.base_commit_id,
    )
    .await?
    .unwrap();
    let base_temp_file = download_temp_file(&connection, &workspace_root, &base_file_hash).await?;
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
    let output_temp_file = tempfile::NamedTempFile::new_in(&tmp_dir)?.into_temp_path();

    if !allow_tools {
        run_diffy_merge(
            &abs_path,
            &theirs_temp_file.to_path_buf(),
            &base_temp_file.to_path_buf(),
        )?;
        clear_resolve_pending(&mut workspace_transaction, &resolve_pending).await?;

        return workspace_transaction
            .commit()
            .await
            .context("error in transaction commit for resolve_file_command");
    }

    run_merge_program(
        Path::new(&relative_path),
        abs_path.to_str().unwrap(),
        theirs_temp_file.to_path_buf().to_str().unwrap(),
        base_temp_file.to_path_buf().to_str().unwrap(),
        output_temp_file.to_str().unwrap(),
    )?;

    tokio::fs::copy(output_temp_file.to_path_buf(), &abs_path)
        .await
        .context(format!(
            "error copying {} to {}",
            output_temp_file.to_path_buf().display(),
            abs_path.display()
        ))?;

    println!("Merge accepted, {} updated", abs_path.display());
    clear_resolve_pending(&mut workspace_transaction, &resolve_pending).await?;

    workspace_transaction
        .commit()
        .await
        .context("eroor in transaction commit for resolve_file_command")
}
