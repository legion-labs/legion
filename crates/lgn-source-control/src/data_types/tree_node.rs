#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub name: String,
    pub hash: String,
}

impl From<TreeNode> for lgn_source_control_proto::TreeNode {
    fn from(tree_node: TreeNode) -> Self {
        Self {
            name: tree_node.name,
            hash: tree_node.hash,
        }
    }
}

impl From<lgn_source_control_proto::TreeNode> for TreeNode {
    fn from(tree_node: lgn_source_control_proto::TreeNode) -> Self {
        Self {
            name: tree_node.name,
            hash: tree_node.hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_node_from_proto() {
        let proto_tree_node = lgn_source_control_proto::TreeNode {
            name: "name".to_string(),
            hash: "hash".to_string(),
        };
        let tree_node = TreeNode::from(proto_tree_node);

        assert_eq!(
            TreeNode {
                name: "name".to_string(),
                hash: "hash".to_string(),
            },
            tree_node
        );
    }

    #[test]
    fn test_tree_node_to_proto() {
        let tree_node = TreeNode {
            name: "name".to_string(),
            hash: "hash".to_string(),
        };
        let proto_tree_node = lgn_source_control_proto::TreeNode::from(tree_node);

        assert_eq!(
            lgn_source_control_proto::TreeNode {
                name: "name".to_string(),
                hash: "hash".to_string(),
            },
            proto_tree_node
        );
    }
}
