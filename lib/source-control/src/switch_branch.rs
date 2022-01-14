use std::fs;
use std::path::Path;
use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use async_recursion::async_recursion;
use lgn_tracing::span_fn;

use crate::{
    connect_to_server, find_workspace_root, read_current_branch, read_workspace_spec,
    remove_dir_rec, sync_file, update_current_branch, LocalWorkspaceConnection,
    RepositoryConnection,
};

#[async_recursion]
pub async fn sync_tree_diff(
    connection: Arc<RepositoryConnection>,
    current_tree_hash: &str,
    new_tree_hash: &str,
    relative_path_tree: &Path,
    workspace_root: &Path,
) -> Result<()> {
    let mut files_present: BTreeMap<String, String> = BTreeMap::new();
    let mut dirs_present: BTreeMap<String, String> = BTreeMap::new();
    let query = connection.index_backend();

    if !current_tree_hash.is_empty() {
        let current_tree = query.read_tree(current_tree_hash).await?;
        for file_node in &current_tree.file_nodes {
            files_present.insert(file_node.name.clone(), file_node.hash.clone());
        }

        for dir_node in &current_tree.directory_nodes {
            dirs_present.insert(dir_node.name.clone(), dir_node.hash.clone());
        }
    }

    let new_tree = query.read_tree(new_tree_hash).await?;
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
            match sync_file(
                &connection,
                workspace_root
                    .join(relative_path_tree)
                    .join(&new_file_node.name),
                &new_file_node.hash,
            )
            .await
            {
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

        match sync_file(&connection, abs_path, "").await {
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
                Arc::clone(&connection),
                &present_hash,
                &new_dir_node.hash,
                &relative_sub_dir,
                workspace_root,
            )
            .await
            {
                println!("{}", e);
            }
        }
    }
    //delete the contents of the directories that were not matched
    for (name, hash) in dirs_present {
        let path = workspace_root.join(relative_path_tree).join(name);
        match remove_dir_rec(Arc::clone(&connection), &path, &hash).await {
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
#[span_fn]
pub async fn switch_branch_command(name: &str) -> Result<()> {
    let current_dir = std::env::current_dir().unwrap();
    let workspace_root = find_workspace_root(&current_dir)?;
    let mut workspace_connection = LocalWorkspaceConnection::new(&workspace_root).await?;
    let workspace_spec = read_workspace_spec(&workspace_root)?;
    let connection = connect_to_server(&workspace_spec).await?;
    let query = connection.index_backend();
    let (_current_branch_name, current_commit) =
        read_current_branch(workspace_connection.sql()).await?;
    let old_commit = query.read_commit(&current_commit).await?;
    let query = connection.index_backend();
    let new_branch = query.read_branch(name).await?;
    let new_commit = query.read_commit(&new_branch.head).await?;

    update_current_branch(
        workspace_connection.sql(),
        &new_branch.name,
        &new_branch.head,
    )
    .await?;

    sync_tree_diff(
        connection,
        &old_commit.root_hash,
        &new_commit.root_hash,
        Path::new(""),
        &workspace_root,
    )
    .await
}
