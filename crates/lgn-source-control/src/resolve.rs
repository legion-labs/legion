use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use lgn_tracing::span_fn;

use crate::{
    fetch_tree_subdir, make_canonical_relative_path, make_path_absolute, read_bin_file, write_file,
    Config, Workspace,
};

#[span_fn]
pub async fn find_resolves_pending_command() -> Result<Vec<ResolvePending>> {
    let workspace = Workspace::find_in_current_directory().await?;
    workspace
        .backend
        .read_resolves_pending()
        .await
        .map_err(Into::into)
}

pub async fn find_file_hash_at_commit(
    workspace: &Workspace,
    relative_path: &Path,
    commit_id: &str,
) -> Result<Option<String>> {
    let commit = workspace.index_backend.read_commit(commit_id).await?;
    let root_tree = workspace
        .index_backend
        .read_tree(&commit.root_tree_id)
        .await?;
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let dir_tree = fetch_tree_subdir(workspace, &root_tree, parent_dir).await?;
    //TODO: Commented this out during refactoring.
    //match dir_tree.find_file_node(
    //    relative_path
    //        .file_name()
    //        .expect("no file name in path specified")
    //        .to_str()
    //        .expect("invalid file name"),
    //) {
    //    Some(file_node) => Ok(Some(file_node.hash.clone())),
    //    None => Ok(None),
    //}
    Ok(None)
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
pub async fn resolve_file_command(p: impl AsRef<Path>, allow_tools: bool) -> Result<()> {
    let abs_path = make_path_absolute(p.as_ref())?;
    let workspace = Workspace::find(&abs_path).await?;
    let relative_path = make_canonical_relative_path(&workspace.root, p.as_ref())?;

    let resolve_pending = workspace
        .backend
        .find_resolve_pending(&relative_path)
        .await
        .context(format!(
            "error finding resolve pending for {}",
            p.as_ref().display()
        ))?
        .ok_or_else(|| anyhow::anyhow!("no resolve pending found for {}", p.as_ref().display(),))?;

    let base_file_hash = find_file_hash_at_commit(
        &workspace,
        Path::new(&relative_path),
        &resolve_pending.base_commit_id,
    )
    .await?
    .unwrap();
    let base_temp_file = workspace.download_temporary_file(&base_file_hash).await?;
    let theirs_file_hash = find_file_hash_at_commit(
        &workspace,
        Path::new(&relative_path),
        &resolve_pending.theirs_commit_id,
    )
    .await?
    .unwrap();
    let theirs_temp_file = workspace.download_temporary_file(&theirs_file_hash).await?;
    let tmp_dir = workspace.root.join(".lsc/tmp");
    let output_temp_file = tempfile::NamedTempFile::new_in(&tmp_dir)?.into_temp_path();

    if !allow_tools {
        run_diffy_merge(
            &abs_path,
            &theirs_temp_file.to_path_buf(),
            &base_temp_file.to_path_buf(),
        )?;

        return workspace
            .backend
            .clear_resolve_pending(&resolve_pending)
            .await
            .map_err(Into::into);
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

    workspace
        .backend
        .clear_resolve_pending(&resolve_pending)
        .await
        .map_err(Into::into)
}
