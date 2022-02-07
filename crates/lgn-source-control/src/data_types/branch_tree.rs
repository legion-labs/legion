use std::collections::{BTreeMap, BTreeSet};

use crate::Branch;

/// A tree of branches.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BranchTree {
    pub branch: Branch,
    pub children: BTreeSet<Self>,
}

impl BranchTree {
    #[allow(clippy::needless_collect)]
    pub fn from_branches(branches: impl IntoIterator<Item = Branch>) -> BTreeSet<Self> {
        let mut branches = branches
            .into_iter()
            .map(|branch| (branch.name.clone(), branch))
            .collect::<BTreeMap<_, _>>();

        let roots = branches
            .values()
            .filter_map(|branch| {
                if branch.is_root() {
                    Some(Self {
                        branch: branch.clone(),
                        children: BTreeSet::new(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        roots
            .into_iter()
            .map(|child| child.feed_from_branches(&mut branches))
            .collect()
    }

    #[allow(clippy::needless_collect)]
    fn feed_from_branches(mut self, branches: &mut BTreeMap<String, Branch>) -> Self {
        branches.remove(&self.branch.name);

        let children = branches
            .values()
            .filter_map(|branch| {
                if branch.parent == Some(self.branch.name.clone()) {
                    Some(Self {
                        branch: branch.clone(),
                        children: BTreeSet::new(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.children = children
            .into_iter()
            .map(|child| child.feed_from_branches(branches))
            .collect();

        self
    }
}

impl From<BranchTree> for termtree::Tree<String> {
    fn from(branch_tree: BranchTree) -> Self {
        Self::new(
            branch_tree.branch.name,
            branch_tree.children.into_iter().map(Into::into).collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(name: &str, parent: Option<&str>) -> Branch {
        Branch {
            name: name.to_string(),
            head: "abc".to_string(),
            parent: parent.map(Into::into),
            lock_domain_id: "".to_string(),
        }
    }

    fn bt<'t>(
        name: &str,
        parent: Option<&str>,
        children: impl IntoIterator<Item = &'t BranchTree>,
    ) -> BranchTree {
        BranchTree {
            branch: b(name, parent),
            children: children.into_iter().cloned().collect(),
        }
    }

    #[test]
    fn test_branch_tree_from_branches() {
        let branches = vec![
            b("main", None),
            b("a", Some("main")),
            b("b", Some("a")),
            b("c", Some("main")),
            b("old", None),
            b("d", Some("old")),
        ];

        let branch_tree = BranchTree::from_branches(branches);

        assert_eq!(
            branch_tree,
            [
                bt(
                    "main",
                    None,
                    &[
                        bt("a", Some("main"), &[bt("b", Some("a"), &[])]),
                        bt("c", Some("main"), &[])
                    ]
                ),
                bt("old", None, &[bt("d", Some("old"), &[])]),
            ]
            .into()
        );
    }
}
