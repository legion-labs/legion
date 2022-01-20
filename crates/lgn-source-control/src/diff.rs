use std::io::Write;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::{
    find_file_hash_at_commit, make_path_absolute, path_relative_to, read_text_file, Config,
    Workspace,
};

async fn reference_version_name_as_commit_id(
    workspace: &Workspace,
    reference_version_name: &str,
) -> Result<String> {
    match reference_version_name {
        "base" => {
            let (_branch_name, commit_id) = workspace.backend.get_current_branch().await?;
            Ok(commit_id)
        }
        "latest" => {
            let (branch_name, _commit_id) = workspace.backend.get_current_branch().await?;
            let branch = workspace.index_backend.read_branch(&branch_name).await?;
            Ok(branch.head)
        }
        _ => Ok(String::from(reference_version_name)),
    }
}

async fn print_diff(workspace: &Workspace, local_path: &Path, ref_file_hash: &str) -> Result<()> {
    let base_version_contents = workspace.blob_storage.read_blob(ref_file_hash).await?;
    let base_version_contents =
        String::from_utf8(base_version_contents).context("error reading base version contents")?;

    let local_version_contents = read_text_file(local_path)?;
    let patch = diffy::create_patch(&base_version_contents, &local_version_contents);
    println!("{}", patch);
    Ok(())
}

pub async fn diff_file_command(
    path: impl AsRef<Path>,
    reference_version_name: &str,
    allow_tools: bool,
) -> Result<()> {
    let abs_path = make_path_absolute(path)?;
    let workspace = Workspace::find(&abs_path).await?;
    let relative_path = path_relative_to(&abs_path, &workspace.root)?;
    let ref_commit_id =
        reference_version_name_as_commit_id(&workspace, reference_version_name).await?;

    let ref_file_hash = find_file_hash_at_commit(&workspace, &relative_path, &ref_commit_id)
        .await?
        .unwrap();

    if !allow_tools {
        return print_diff(&workspace, &abs_path, &ref_file_hash).await;
    }

    let config = Config::read_config()?;

    match config.find_diff_command(&relative_path) {
        Some(mut external_command_vec) => {
            let ref_temp_file = workspace.download_temporary_file(&ref_file_hash).await?;
            let ref_path_str = ref_temp_file.to_str().unwrap();
            let local_file = abs_path.to_str().unwrap();
            for item in &mut external_command_vec[..] {
                *item = item.replace("%ref", ref_path_str);
                *item = item.replace("%local", local_file);
            }
            let output = Command::new(&external_command_vec[0])
                .args(&external_command_vec[1..])
                .output()
                .context(format!(
                    "Failed to execute external diff command: {:?}",
                    external_command_vec
                ))?;

            let mut out = std::io::stdout();
            out.write_all(&output.stdout).unwrap();
            out.flush().unwrap();

            let mut err = std::io::stderr();
            err.write_all(&output.stderr).unwrap();
            err.flush().unwrap();
        }
        None => {
            return print_diff(&workspace, &abs_path, &ref_file_hash).await;
        }
    }

    Ok(())
}
