use crate::*;
use std::io::Write;
use std::path::Path;
use std::process::Command;

async fn reference_version_name_as_commit_id(
    repo_query: &dyn RepositoryQuery,
    workspace_root: &Path,
    reference_version_name: &str,
) -> Result<String, String> {
    match reference_version_name {
        "base" => {
            let workspace_branch = read_current_branch(workspace_root)?;
            Ok(workspace_branch.head)
        }
        "latest" => {
            let workspace_branch = read_current_branch(workspace_root)?;
            let branch = repo_query.read_branch(&workspace_branch.name).await?;
            Ok(branch.head)
        }
        _ => Ok(String::from(reference_version_name)),
    }
}

async fn print_diff(
    connection: &mut RepositoryConnection,
    local_path: &Path,
    ref_file_hash: &str,
) -> Result<(), String> {
    let base_version_contents = connection
        .blob_storage()
        .await?
        .read_blob(ref_file_hash)
        .await?;
    let local_version_contents = read_text_file(local_path)?;
    let patch = diffy::create_patch(&base_version_contents, &local_version_contents);
    println!("{}", patch);
    Ok(())
}

pub fn diff_file_command(
    path: &Path,
    reference_version_name: &str,
    allow_tools: bool,
) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    let relative_path = path_relative_to(&abs_path, &workspace_root)?;
    let ref_commit_id = tokio_runtime.block_on(reference_version_name_as_commit_id(
        connection.query(),
        &workspace_root,
        reference_version_name,
    ))?;
    let ref_file_hash = tokio_runtime
        .block_on(find_file_hash_at_commit(
            &mut connection,
            &relative_path,
            &ref_commit_id,
        ))?
        .unwrap();

    if !allow_tools {
        return tokio_runtime.block_on(print_diff(&mut connection, &abs_path, &ref_file_hash));
    }

    let config = Config::read_config()?;
    match config.find_diff_command(&relative_path) {
        Some(mut external_command_vec) => {
            let ref_temp_file = tokio_runtime.block_on(download_temp_file(
                &mut connection,
                &workspace_root,
                &ref_file_hash,
            ))?;
            let ref_path_str = ref_temp_file.path.to_str().unwrap();
            let local_file = abs_path.to_str().unwrap();
            for item in &mut external_command_vec[..] {
                *item = item.replace("%ref", ref_path_str);
                *item = item.replace("%local", local_file);
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
            return tokio_runtime.block_on(print_diff(&mut connection, &abs_path, &ref_file_hash));
        }
    }

    Ok(())
}
