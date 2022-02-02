use std::collections::HashMap;

use lgn_source_control::{CanonicalPath, Tree};

pub struct InodeIndex {
    tree: Tree,
    next_inode: u64,
    inode_to_path: HashMap<u64, CanonicalPath>,
    path_to_inode: HashMap<CanonicalPath, u64>,
}

impl InodeIndex {
    pub fn new(tree: Tree) -> Self {
        let mut next_inode: u64 = 1;
        let inode_to_path: HashMap<u64, CanonicalPath> = tree
            .iter()
            .map(|(path, _)| {
                let inode = next_inode;
                next_inode += 1;
                (inode, path)
            })
            .collect();

        let path_to_inode = inode_to_path
            .clone()
            .into_iter()
            .map(|(inode, path)| (path, inode))
            .collect();

        Self {
            tree,
            next_inode,
            inode_to_path,
            path_to_inode,
        }
    }

    pub fn update_tree(&mut self, tree: Tree) {
        // Nothing to do if the tree hasn't changed.
        if tree == self.tree {
            return;
        }

        for (path, _) in &tree {
            if self.path_to_inode.get(&path).is_none() {
                self.path_to_inode.insert(path.clone(), self.next_inode);
                self.inode_to_path.insert(self.next_inode, path);
                self.next_inode += 1;
            }
        }

        for (path, _) in &self.tree {
            if let Ok(Some(_)) = tree.find(&path) {
                continue;
            }

            self.path_to_inode.remove(&path);
            self.inode_to_path.remove(&self.path_to_inode[&path]);
        }

        self.tree = tree;
    }

    pub fn get_inode_by_path(&self, path: &CanonicalPath) -> Option<u64> {
        self.path_to_inode.get(path).copied()
    }

    pub fn get_tree_node(&self, ino: u64) -> Option<(u64, CanonicalPath, &Tree)> {
        if let Some(path) = self.inode_to_path.get(&ino) {
            if let Ok(Some(tree)) = self.tree.find(path) {
                Some((ino, path.clone(), tree))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_tree_node_by_path(
        &self,
        path: CanonicalPath,
    ) -> Option<(u64, CanonicalPath, &Tree)> {
        if let Ok(r) = self.tree.find(&path) {
            r.map(|tree| (self.path_to_inode[&path], path, tree))
        } else {
            None
        }
    }

    pub fn get_tree_node_by_parent_path(
        &self,
        parent_ino: u64,
        name: &str,
    ) -> Option<(u64, CanonicalPath, &Tree)> {
        if let Some((_, parent_path, _)) = self.get_tree_node(parent_ino) {
            let path = parent_path.append(name);

            self.get_tree_node_by_path(path)
        } else {
            None
        }
    }
}
