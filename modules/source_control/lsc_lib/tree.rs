use crate::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::hash_map::HashMap;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TreeNode {
    pub name: PathBuf,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tree {
    pub directory_nodes: Vec<TreeNode>,
    pub file_nodes: Vec<TreeNode>,
}

impl Tree {
    pub fn empty() -> Tree {
        Tree {
            directory_nodes: Vec::new(),
            file_nodes: Vec::new(),
        }
    }

    pub fn hash(&self) -> String {
        //std::hash::Hasher is not right here because it supports only 64 bit hashes
        let mut hasher = Sha256::new();
        for node in &self.directory_nodes{
            hasher.update( node.name.to_str().expect("invalid node name").as_bytes() );
            hasher.update( &node.hash );
        }
        for node in &self.file_nodes{
            hasher.update( node.name.to_str().expect("invalid node name").as_bytes() );
            hasher.update( &node.hash );
        }
        format!("{:X}", hasher.finalize())
    }

    pub fn add_or_update_file_node(&mut self, node: TreeNode) {
        self.file_nodes.push(node);
    }

    pub fn add_or_update_dir_node(&mut self, node: TreeNode) {
        self.directory_nodes.push(node);
    }

    pub fn remove_file_node(&mut self, node_name: &Path) {
        if let Some(index) = self.file_nodes.iter().position(|x| x.name == node_name) {
            self.file_nodes.swap_remove(index);
        }
    }
}

pub fn update_tree_from_changes(
    previous_version: Tree,
    local_changes: &[HashedChange],
) -> Result<Tree, String> {
    //scan changes to get the list of trees to update
    let mut dir_to_update = BTreeSet::new();
    for change in local_changes {
        let parent = change
            .relative_path
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
    dir_to_update_by_length.sort_by(|b, a| a.components().count().cmp(&b.components().count()));
    for dir in dir_to_update_by_length {
        let mut tree = Tree::empty(); //todo: fetch previous version
        for change in local_changes {
            let parent = change
                .relative_path
                .parent()
                .expect("relative path with no parent");
            if dir == parent {
                //todo: handle edit & delete
                tree.add_or_update_file_node(TreeNode {
                    name: PathBuf::from(
                        change
                            .relative_path
                            .file_name()
                            .expect("error getting file name"),
                    ),
                    hash: change.hash.clone(),
                });
            }
        }
        //find dir's children, add them to the current tree
        if let Some(v) = parent_to_children_dir.get(&dir) {
            for node in v {
                tree.add_or_update_dir_node(node.clone());
            }
        }

        //save the child for the parent to find
        if let Some(dir_parent) = dir.parent() {
            //save the child for the parent to find
            let key = dir_parent.to_path_buf();
            let name = dir
                .strip_prefix(dir_parent)
                .expect("Error getting directory name");
            let dir_hash = tree.hash(); //important not to modify tree beyond this point
            let dir_node = TreeNode {
                name: name.to_path_buf(),
                hash: dir_hash,
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
        

        //todo: add to database
        println!("tree {}: {:?}", dir.display(), tree);
    }
    return Ok(previous_version.clone());
}
