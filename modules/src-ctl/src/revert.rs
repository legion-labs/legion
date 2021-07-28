use crate::*;
use std::path::Path;

pub fn revert_glob_command(pattern: &str) -> Result<(), String> {
    let mut nb_errors = 0;
    match glob::Pattern::new(pattern) {
        Ok(matcher) => {
            let current_dir = std::env::current_dir().unwrap();
            let workspace_root = find_workspace_root(&current_dir)?;
            let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
            for change in read_local_changes(&mut workspace_connection)? {
                if matcher.matches(&change.relative_path) {
                    println!("reverting {}", change.relative_path);
                    let local_file_path = workspace_root.join(change.relative_path);
                    if let Err(e) = revert_file_command(&local_file_path) {
                        println!("{}", e);
                        nb_errors += 1;
                    }
                }
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
    workspace_connection: &mut LocalWorkspaceConnection,
    repo_connection: &mut RepositoryConnection,
    path: &Path,
) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let relative_path = make_canonical_relative_path(&workspace_root, &abs_path)?;
    let local_change = match find_local_change(workspace_connection, &relative_path) {
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
    let workspace_branch = read_current_branch(&workspace_root)?;
    let current_commit = read_commit(repo_connection, &workspace_branch.head)?;
    let root_tree = read_tree(repo_connection, &current_commit.root_hash)?;
    let dir_tree = fetch_tree_subdir(repo_connection, &root_tree, parent_dir)?;

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
            .download_blob(&abs_path, &file_node.hash)
            .await?;
        make_file_read_only(&abs_path, true)?;
    }
    clear_local_change(workspace_connection, &local_change)?;
    match find_resolve_pending(workspace_connection, &relative_path) {
        Ok(Some(resolve_pending)) => clear_resolve_pending(workspace_connection, &resolve_pending),
        Err(e) => Err(format!(
            "Error finding resolve pending for file {}: {}",
            relative_path, e
        )),
        Ok(None) => Ok(()),
    }
}

pub fn revert_file_command(path: &Path) -> Result<(), String> {
    let abs_path = make_path_absolute(path);
    let workspace_root = find_workspace_root(&abs_path)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root)?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();
    let mut repo_connection = tokio_runtime.block_on(connect_to_server(&workspace_spec))?;
    tokio_runtime.block_on(revert_file(
        &mut workspace_connection,
        &mut repo_connection,
        path,
    ))
}
