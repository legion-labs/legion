use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::{
    connect_to_server, find_workspace_root, read_current_branch, read_workspace_spec,
    remove_dir_rec, sync_file, trace_scope, update_current_branch, LocalWorkspaceConnection,
    RepositoryConnection,
};

pub fn sync_tree_diff(
    runtime: &tokio::runtime::Runtime,
    connection: &RepositoryConnection,
    current_tree_hash: &str,
    new_tree_hash: &str,
    relative_path_tree: &Path,
    workspace_root: &Path,
) -> Result<(), String> {
    let mut files_present: BTreeMap<String, String> = BTreeMap::new();
    let mut dirs_present: BTreeMap<String, String> = BTreeMap::new();
    let query = connection.query();
    if !current_tree_hash.is_empty() {
        let current_tree = runtime.block_on(query.read_tree(current_tree_hash))?;
        for file_node in &current_tree.file_nodes {
            files_present.insert(file_node.name.clone(), file_node.hash.clone());
        }

        for dir_node in &current_tree.directory_nodes {
            dirs_present.insert(dir_node.name.clone(), dir_node.hash.clone());
        }
    }

    let new_tree = runtime.block_on(query.read_tree(new_tree_hash))?;
    for new_file_node in &new_tree.file_nodes {
        let present_hash = match files_present.get(&new_file_node.name) {
            Some(hash) => {
                let res = hash.clone();
                files_present.remove(&new_file_node.name);
                res
            }
            None => String::new(),
        };
        if new_file_node.hash != present_hash {
            match runtime.block_on(sync_file(
                connection,
                &workspace_root
                    .join(relative_path_tree)
                    .join(&new_file_node.name),
                &new_file_node.hash,
            )) {
                Ok(message) => {
                    println!("{}", message);
                }
                Err(e) => {
                    println!("{}", e);
                }
            }
        }
    }

    //those files were not matched, delete them
    for k in files_present.keys() {
        let abs_path = workspace_root.join(relative_path_tree).join(&k);
        match runtime.block_on(sync_file(connection, &abs_path, "")) {
            Ok(message) => {
                println!("{}", message);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    for new_dir_node in &new_tree.directory_nodes {
        let present_hash = match dirs_present.get(&new_dir_node.name) {
            Some(hash) => {
                let res = hash.clone();
                dirs_present.remove(&new_dir_node.name);
                res
            }
            None => String::new(),
        };
        let relative_sub_dir = relative_path_tree.join(&new_dir_node.name);
        let abs_dir = workspace_root.join(&relative_sub_dir);
        if !abs_dir.exists() {
            if let Err(e) = fs::create_dir(&abs_dir) {
                println!("Error creating directory {}: {}", abs_dir.display(), e);
            }
        }
        if new_dir_node.hash != present_hash {
            if let Err(e) = sync_tree_diff(
                runtime,
                connection,
                &present_hash,
                &new_dir_node.hash,
                &relative_sub_dir,
                workspace_root,
            ) {
                println!("{}", e);
            }
        }
    }
    //delete the contents of the directories that were not matched
    for (name, hash) in dirs_present {
        let path = workspace_root.join(&relative_path_tree).join(name);
        match remove_dir_rec(runtime, connection, &path, &hash) {
            Ok(messages) => {
                if !messages.is_empty() {
                    println!("{}", messages);
                }
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    Ok(())
}

// not yet async because of sync_tree_diff
pub fn switch_branch_command(runtime: &tokio::runtime::Runtime, name: &str) -> Result<(), String> {
    trace_scope!();
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection =
        runtime.block_on(LocalWorkspaceConnection::new(&workspace_root))?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = runtime.block_on(connect_to_server(&workspace_spec))?;
    let query = connection.query();
    let (_current_branch_name, current_commit) =
        runtime.block_on(read_current_branch(workspace_connection.sql()))?;
    let old_commit = runtime.block_on(query.read_commit(&current_commit))?;
    let query = connection.query();
    let new_branch = runtime.block_on(query.read_branch(name))?;
    let new_commit = runtime.block_on(query.read_commit(&new_branch.head))?;
    runtime.block_on(update_current_branch(
        workspace_connection.sql(),
        &new_branch.name,
        &new_branch.head,
    ))?;
    sync_tree_diff(
        runtime,
        &connection,
        &old_commit.root_hash,
        &new_commit.root_hash,
        Path::new(""),
        &workspace_root,
    )
}
