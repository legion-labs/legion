use crate::*;
use std::path::Path;

pub async fn revert_glob_command(pattern: &str) -> Result<(), String> {
    let mut nb_errors = 0;
    match glob::Pattern::new(pattern) {
        Ok(matcher) => {
            let current_dir = std::env::current_dir().unwrap();
            let workspace_root = find_workspace_root(&current_dir)?;
            let workspace_spec = read_workspace_spec(&workspace_root)?;
            let connection = connect_to_server(&workspace_spec).await?;
            let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
            let mut workspace_transaction = workspace_connection.begin().await?;
            for change in read_local_changes(&mut workspace_transaction).await? {
                if matcher.matches(&change.relative_path) {
                    println!("reverting {}", change.relative_path);
                    let local_file_path = workspace_root.join(change.relative_path);
                    if let Err(e) =
                        revert_file(&mut workspace_transaction, &connection, &local_file_path).await
                    {
                        println!("{}", e);
                        nb_errors += 1;
                    }
                }
            }
            if let Err(e) = workspace_transaction.commit().await {
                return Err(format!(
                    "Error in transaction commit for revert_glob_command: {}",
                    e
                ));
            }
        }
        Err(e) => {
            return Err(format!("Error parsing glob pattern: {}", e));
        }
    }
    if nb_errors == 0 {
        Ok(())
    } else {
        Err(format!("{} errors", nb_errors))
    }
}

pub async fn revert_file(
    workspace_transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
    repo_connection: &RepositoryConnection,
    path: &Path,
) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;
    let local_change = match find_local_change(workspace_transaction, &relative_path).await {
        Ok(Some(change)) => change,
        Err(e) => {
            return Err(format!("Error searching in local changes: {}", e));
        }
        Ok(None) => {
            return Err(format!("Error local change {} not found", relative_path));
        }
    };
    let parent_dir = Path::new(&relative_path)
        .parent()
        .expect("no parent to path provided");
    let (_branch_name, current_commit) = read_current_branch(workspace_transaction).await?;
    let query = repo_connection.query();
    let current_commit = query.read_commit(&current_commit).await?;
    let root_tree = query.read_tree(&current_commit.root_hash).await?;
    let dir_tree = fetch_tree_subdir(query, &root_tree, parent_dir).await?;

    if local_change.change_type != ChangeType::Add {
        let file_node;
        match dir_tree.find_file_node(
            abs_path
                .file_name()
                .expect("no file name in path specified")
                .to_str()
                .expect("invalid file name"),
        ) {
            Some(node) => {
                file_node = node;
            }
            None => {
                return Err(String::from("Original file not found in tree"));
            }
        }
        repo_connection
            .blob_storage()
            .await?
            .download_blob(&abs_path, &file_node.hash)
            .await?;
        make_file_read_only(&abs_path, true)?;
    }
    clear_local_change(workspace_transaction, &local_change).await?;
    match find_resolve_pending(workspace_transaction, &relative_path).await {
        Ok(Some(resolve_pending)) => {
            clear_resolve_pending(workspace_transaction, &resolve_pending).await
        }
        Err(e) => Err(format!(
            "Error finding resolve pending for file {}: {}",
            relative_path, e
        )),
        Ok(None) => Ok(()),
    }
}

pub async fn revert_file_command(path: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let mut workspace_transaction = workspace_connection.begin().await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let repo_connection = connect_to_server(&workspace_spec).await?;
    revert_file(&mut workspace_transaction, &repo_connection, path).await?;
    if let Err(e) = workspace_transaction.commit().await {
        return Err(format!(
            "Error in transaction commit for revert_file_command: {}",
            e
        ));
    }
    Ok(())
}
