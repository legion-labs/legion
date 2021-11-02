use std::fs;
use std::path::Path;

use crate::{
    assert_not_locked, connect_to_server, find_local_change, find_workspace_root,
    make_canonical_relative_path, make_file_read_only, make_path_absolute, read_workspace_spec,
    save_local_change, trace_scope, ChangeType, LocalChange, LocalWorkspaceConnection,
};

pub async fn delete_local_file(
    workspace_root: &Path,
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    path_specified: &Path,
) -> Result<(), String> {
    let abs_path = make_path_absolute(path_specified);
    if !abs_path.exists() {
        return Err(format!("Error: file not found {}", abs_path.display()));
    }
    let workspace_spec = read_workspace_spec(workspace_root)?;
    let repo_connection = connect_to_server(&workspace_spec).await?;
    assert_not_locked(
        repo_connection.query(),
        workspace_root,
        workspace_transaction,
        &abs_path,
    )
    .await?;

    let relative_path = make_canonical_relative_path(workspace_root, &abs_path)?;

    match find_local_change(workspace_transaction, &relative_path).await {
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
    save_local_change(workspace_transaction, &local_change).await?;

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

pub async fn delete_file_command(path_specified: &Path) -> Result<(), String> {
    trace_scope!();
    let abs_path = make_path_absolute(path_specified);
    if !abs_path.exists() {
        return Err(format!("Error: file not found {}", abs_path.display()));
    }
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    delete_local_file(&workspace_root, &mut workspace_transaction, path_specified).await?;
    if let Err(e) = workspace_transaction.commit().await {
        return Err(format!(
            "Error in transaction commit for delete_file_command: {}",
            e
        ));
    }
    Ok(())
}
