use crate::*;
use sha2::{Digest, Sha256};
use std::collections::hash_map::HashMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use unicase::UniCase;

pub enum TreeNodeType {
    Directory = 1,
    File = 2,
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub hash: String,
}

impl TreeNode {
    pub fn new(name: String, hash: String) -> Self {
        Self { name, hash }
    }
}

#[derive(Debug, Clone)]
pub struct Tree {
    pub directory_nodes: Vec<TreeNode>,
    pub file_nodes: Vec<TreeNode>,
}

impl Tree {
    pub fn empty() -> Self {
        Self {
            directory_nodes: Vec::new(),
            file_nodes: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.directory_nodes.is_empty() && self.file_nodes.is_empty()
    }

    pub fn sort(&mut self) {
        self.directory_nodes.sort_by_key(|n| n.name.clone());
        self.file_nodes.sort_by_key(|n| n.name.clone());
    }

    pub fn hash(&self) -> String {
        //std::hash::Hasher is not right here because it supports only 64 bit hashes
        let mut hasher = Sha256::new();
        for node in &self.directory_nodes {
            hasher.update(node.name.as_bytes());
            hasher.update(&node.hash);
        }
        for node in &self.file_nodes {
            hasher.update(node.name.as_bytes());
            hasher.update(&node.hash);
        }
        format!("{:X}", hasher.finalize())
    }

    pub fn add_or_update_file_node(&mut self, node: TreeNode) {
        self.remove_file_node(&node.name);
        self.file_nodes.push(node);
    }

    pub fn add_or_update_dir_node(&mut self, node: TreeNode) {
        self.remove_dir_node(&node.name);
        self.directory_nodes.push(node);
    }

    pub fn find_dir_node(&self, specified: &str) -> Result<&TreeNode, String> {
        let name = UniCase::new(specified);
        for node in &self.directory_nodes {
            if UniCase::new(&node.name) == name {
                return Ok(node);
            }
        }
        Err(format!("could not find directory node {}", name))
    }

    pub fn find_file_node(&self, specified: &str) -> Option<&TreeNode> {
        let name = UniCase::new(specified);
        for node in &self.file_nodes {
            if UniCase::new(&node.name) == name {
                return Some(node);
            }
        }
        None
    }

    pub fn remove_file_node(&mut self, specified_name: &str) {
        let name = UniCase::new(specified_name);
        if let Some(index) = self
            .file_nodes
            .iter()
            .position(|x| UniCase::new(&x.name) == name)
        {
            self.file_nodes.swap_remove(index);
        }
    }

    pub fn remove_dir_node(&mut self, specified_name: &str) {
        let name = UniCase::new(specified_name);
        if let Some(index) = self
            .directory_nodes
            .iter()
            .position(|x| UniCase::new(&x.name) == name)
        {
            self.directory_nodes.swap_remove(index);
        }
    }
}

pub fn fetch_tree_subdir(
    connection: &mut RepositoryConnection,
    root: &Tree,
    subdir: &Path,
) -> Result<Tree, String> {
    let mut parent = root.clone();
    for component in subdir.components() {
        let component_name = component
            .as_os_str()
            .to_str()
            .expect("invalid path component name");
        match parent.find_dir_node(component_name) {
            Ok(node) => {
                parent = read_tree(connection, &node.hash)?;
            }
            Err(_) => {
                return Ok(Tree::empty()); //new directory
            }
        }
    }
    Ok(parent)
}

pub fn find_file_hash_in_tree(
    connection: &mut RepositoryConnection,
    relative_path: &Path,
    root_tree: &Tree,
) -> Result<Option<String>, String> {
    let parent_dir = relative_path.parent().expect("no parent to path provided");
    let dir_tree = fetch_tree_subdir(connection, root_tree, parent_dir)?;
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
pub fn update_tree_from_changes(
    previous_root: &Tree,
    local_changes: &[HashedChange],
    connection: &mut RepositoryConnection,
) -> Result<String, String> {
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
    //process leafs before parents to be able to patch parents with hash of children
    dir_to_update_by_length.sort_by_key(|a| core::cmp::Reverse(a.components().count()));
    for dir in dir_to_update_by_length {
        let mut tree = fetch_tree_subdir(connection, previous_root, &dir)?;
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

        save_tree(connection, &tree, &dir_hash)?;
        if dir.components().count() == 0 {
            return Ok(dir_hash);
        }
    }
    Err(String::from("root tree not processed"))
}

pub fn read_blob(connection: &mut RepositoryConnection, hash: &str) -> Result<String, String> {
    assert!(!hash.is_empty());
    let repo = connection.repository();
    let blob_path = repo.join(format!("blobs/{}", hash));
    lz4_read(&blob_path)
}

pub fn download_blob(
    connection: &mut RepositoryConnection,
    local_path: &Path,
    hash: &str,
) -> Result<(), String> {
    assert!(!hash.is_empty());
    let repo = connection.repository();
    let blob_path = repo.join(format!("blobs/{}", hash));
    lz4_decompress(&blob_path, local_path)
}

pub fn remove_dir_rec(
    connection: &mut RepositoryConnection,
    local_path: &Path,
    tree_hash: &str,
) -> Result<String, String> {
    let mut messages: Vec<String> = Vec::new();
    let tree = read_tree(connection, tree_hash)?;

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

    for dir_node in &tree.directory_nodes {
        let dir_path = local_path.join(&dir_node.name);
        let message = remove_dir_rec(connection, &dir_path, &dir_node.hash)?;
        if !message.is_empty() {
            messages.push(message);
        }
    }

    if let Err(e) = fs::remove_dir(&local_path) {
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

pub fn download_tree(
    connection: &mut RepositoryConnection,
    download_path: &Path,
    tree_hash: &str,
) -> Result<(), String> {
    let mut dir_to_process = Vec::from([TreeNode {
        name: String::from(download_path.to_str().expect("path is invalid string")),
        hash: String::from(tree_hash),
    }]);
    let mut errors: Vec<String> = Vec::new();
    while !dir_to_process.is_empty() {
        let dir_node = dir_to_process.pop().expect("empty dir_to_process");
        let tree = read_tree(connection, &dir_node.hash)?;
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
            if let Err(e) = download_blob(connection, &abs_path, &relative_file_node.hash) {
                errors.push(format!(
                    "Error downloading blob {} to {}: {}",
                    &relative_file_node.hash,
                    abs_path.display(),
                    e
                ));
            }
            if let Err(e) = make_file_read_only(&abs_path, true) {
                errors.push(e);
            }
        }
    }
    if !errors.is_empty() {
        let message = errors.join("\n");
        return Err(message);
    }
    Ok(())
}
