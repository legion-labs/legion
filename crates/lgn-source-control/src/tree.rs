use std::collections::hash_map::HashMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use async_recursion::async_recursion;

use crate::{
    make_file_read_only, ChangeType, HashedChange, IndexBackend, RepositoryConnection, Tree,
    TreeNode,
};

pub enum TreeNodeType {
    Directory = 1,
    File = 2,
}

pub async fn fetch_tree_subdir(
    query: &dyn IndexBackend,
    root: &Tree,
    subdir: &Path,
) -> Result<Tree> {
    let mut parent = root.clone();
    for component in subdir.components() {
        let component_name = component
            .as_os_str()
            .to_str()
            .expect("invalid path component name");
        match parent.find_dir_node(component_name) {
            Ok(node) => {
                parent = query.read_tree(&node.hash).await?;
            }
            Err(_) => {
                return Ok(Tree::empty()); //new directory
            }
        }
    }
    Ok(parent)
}

pub async fn find_file_hash_in_tree(
    connection: &RepositoryConnection,
    relative_path: &Path,
    root_tree: &Tree,
) -> Result<Option<String>> {
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let dir_tree = fetch_tree_subdir(connection.index_backend(), root_tree, parent_dir).await?;
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

// returns the hash of the updated root tree
pub async fn update_tree_from_changes(
    previous_root: &Tree,
    local_changes: &[HashedChange],
    connection: &RepositoryConnection,
) -> Result<String> {
    //scan changes to get the list of trees to update
    let mut dir_to_update = BTreeSet::new();
    for change in local_changes {
        let relative_path = Path::new(&change.relative_path);
        let parent = relative_path
            .parent()
            .expect("relative path with no parent");
        dir_to_update.insert(parent);
    }
    let root = Path::new("");
    //add ancestors
    for dir in dir_to_update.clone() {
        if let Some(mut parent) = dir.parent() {
            loop {
                dir_to_update.insert(parent);
                if parent == root {
                    break;
                }
                parent = parent.parent().expect("relative path with no parent");
            }
        }
    }
    let mut dir_to_update_by_length = Vec::<PathBuf>::new();
    for dir in dir_to_update {
        dir_to_update_by_length.push(dir.to_path_buf());
    }

    let mut parent_to_children_dir = HashMap::<PathBuf, Vec<TreeNode>>::new();
    //process leafs before parents to be able to patch parents with hash of
    // children
    dir_to_update_by_length.sort_by_key(|a| core::cmp::Reverse(a.components().count()));

    for dir in dir_to_update_by_length {
        let mut tree = fetch_tree_subdir(connection.index_backend(), previous_root, &dir).await?;
        for change in local_changes {
            let relative_path = Path::new(&change.relative_path);
            let parent = relative_path
                .parent()
                .expect("relative path with no parent");
            if dir == parent {
                let filename = String::from(
                    relative_path
                        .file_name()
                        .expect("error getting file name")
                        .to_str()
                        .expect("path is invalid string"),
                );
                if change.change_type == ChangeType::Delete {
                    tree.remove_file_node(&filename);
                } else {
                    tree.add_or_update_file_node(TreeNode {
                        name: filename,
                        hash: change.hash.clone(),
                    });
                }
            }
        }
        //find dir's children, add them to the current tree
        if let Some(v) = parent_to_children_dir.get(&dir) {
            for node in v {
                tree.add_or_update_dir_node(node.clone());
            }
        }

        tree.sort();
        let dir_hash = tree.hash(); //important not to modify tree beyond this point

        //save the child for the parent to find
        if let Some(dir_parent) = dir.parent() {
            //save the child for the parent to find
            let key = dir_parent.to_path_buf();
            let name = dir
                .strip_prefix(dir_parent)
                .expect("Error getting directory name");
            let dir_node = TreeNode {
                name: String::from(name.to_str().expect("path is invalid string")),
                hash: dir_hash.clone(),
            };
            match parent_to_children_dir.get_mut(&key) {
                Some(v) => {
                    v.push(dir_node);
                }
                None => {
                    parent_to_children_dir.insert(key, Vec::from([dir_node]));
                }
            }
        }

        connection
            .index_backend()
            .save_tree(&tree, &dir_hash)
            .await?;
        if dir.components().count() == 0 {
            return Ok(dir_hash);
        }
    }

    anyhow::bail!("root tree not processed");
}

#[async_recursion]
pub async fn remove_dir_rec(
    connection: Arc<RepositoryConnection>,
    local_path: &Path,
    tree_hash: &str,
) -> Result<String> {
    let mut messages: Vec<String> = Vec::new();
    let query = connection.index_backend();
    let tree = query.read_tree(tree_hash).await?;

    for file_node in &tree.file_nodes {
        let file_path = local_path.join(&file_node.name);
        make_file_read_only(&file_path, false)?;

        if let Err(e) = fs::remove_file(&file_path) {
            messages.push(format!(
                "Error deleting file {}: {}",
                file_path.display(),
                e
            ));
        } else {
            messages.push(format!("Deleted {}", file_path.display()));
        }
    }

    let connection = Arc::clone(&connection);
    for dir_node in tree.directory_nodes {
        let dir_path = local_path.join(&dir_node.name);
        let message = remove_dir_rec(Arc::clone(&connection), &dir_path, &dir_node.hash).await?;
        if !message.is_empty() {
            messages.push(message);
        }
    }

    if let Err(e) = fs::remove_dir(local_path) {
        messages.push(format!(
            "Error deleting directory {}: {}",
            local_path.display(),
            e
        ));
    } else {
        messages.push(format!("Deleted {}", local_path.display()));
    }

    Ok(messages.join("\n"))
}

pub async fn download_tree(
    connection: &RepositoryConnection,
    download_path: &Path,
    tree_hash: &str,
) -> Result<()> {
    let mut dir_to_process = Vec::from([TreeNode {
        name: String::from(download_path.to_str().expect("path is invalid string")),
        hash: String::from(tree_hash),
    }]);

    let mut errors: Vec<String> = Vec::new();

    while !dir_to_process.is_empty() {
        let dir_node = dir_to_process.pop().expect("empty dir_to_process");
        let tree = connection.index_backend().read_tree(&dir_node.hash).await?;

        for relative_subdir_node in tree.directory_nodes {
            let abs_subdir_node = TreeNode {
                name: format!("{}/{}", &dir_node.name, relative_subdir_node.name),
                hash: relative_subdir_node.hash,
            };
            match std::fs::create_dir_all(&abs_subdir_node.name) {
                Ok(_) => {
                    dir_to_process.push(abs_subdir_node);
                }
                Err(e) => {
                    errors.push(format!(
                        "Error creating directory {}: {}",
                        abs_subdir_node.name, e
                    ));
                }
            }
        }

        for relative_file_node in tree.file_nodes {
            let abs_path = PathBuf::from(&dir_node.name).join(relative_file_node.name);
            println!("writing {}", abs_path.display());
            if let Err(e) = connection
                .blob_storage()
                .download_blob(&abs_path, &relative_file_node.hash)
                .await
            {
                errors.push(format!(
                    "Error downloading blob {} to {}: {}",
                    &relative_file_node.hash,
                    abs_path.display(),
                    e
                ));
            }

            if let Err(e) = make_file_read_only(&abs_path, true) {
                errors.push(e.to_string());
            }
        }
    }

    if !errors.is_empty() {
        anyhow::bail!("failed to download tree:\n{}", errors.join("\n"));
    }

    Ok(())
}
