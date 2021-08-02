use crate::*;
use std::fs;
use std::path::Path;

pub fn delete_file_command(path_specified: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path_specified);
    if !abs_path.exists() {
        return Err(format!("Error: file not found {}", abs_path.display()));
    }
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo_connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    tokio_runtime.block_on(assert_not_locked(
        &repo_connection,
        &workspace_root,
        &abs_path,
    ))?;

    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;

    match find_local_change(&mut connection, &relative_path) {
        Ok(Some(change)) => {
            return Err(format!(
                "Error: {} already tracked for {:?}",
                change.relative_path, change.change_type
            ));
        }
        Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        Ok(None) => { //all is good
        }
    }

    //todo: lock file
    let local_change = LocalChange::new(&relative_path, ChangeType::Delete);
    save_local_change(&mut connection, &local_change)?;

    make_file_read_only(&abs_path, false)?;
    if let Err(e) = fs::remove_file(&abs_path) {
        return Err(format!(
            "Error deleting local file {}: {}",
            abs_path.display(),
            e
        ));
    }
    Ok(())
}
