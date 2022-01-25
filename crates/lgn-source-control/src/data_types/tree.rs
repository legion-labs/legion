use sha2::{Digest, Sha256};
use unicase::UniCase;

use super::TreeNode;

#[derive(Debug, Clone)]
pub struct Tree {
    pub directory_nodes: Vec<TreeNode>,
    pub file_nodes: Vec<TreeNode>,
}

impl From<Tree> for lgn_source_control_proto::Tree {
    fn from(tree: Tree) -> Self {
        Self {
            directory_nodes: tree.directory_nodes.into_iter().map(Into::into).collect(),
            file_nodes: tree.file_nodes.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<lgn_source_control_proto::Tree> for Tree {
    fn from(tree: lgn_source_control_proto::Tree) -> Self {
        Self {
            directory_nodes: tree.directory_nodes.into_iter().map(Into::into).collect(),
            file_nodes: tree.file_nodes.into_iter().map(Into::into).collect(),
        }
    }
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

    pub fn find_dir_node(&self, specified: &str) -> anyhow::Result<&TreeNode> {
        let name = UniCase::new(specified);
        for node in &self.directory_nodes {
            if UniCase::new(&node.name) == name {
                return Ok(node);
            }
        }

        anyhow::bail!("could not find directory node {}", name);
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
