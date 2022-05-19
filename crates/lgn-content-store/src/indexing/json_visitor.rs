use std::collections::HashSet;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{
    IndexKeyDisplayFormat, Result, Tree, TreeBranchInfo, TreeIdentifier, TreeLeafInfo, TreeVisitor,
    TreeVisitorAction,
};

/// A visitor that generates a JSON representation of the tree.
///
/// Suitable for using with d3.js, `d3.stratify`.
///
/// # Output format
///
/// The output format is a JSON object with the following structure:
///
/// ```json
/// {
///     "nodes": [
///        {"id": "<tree-node-0>", "alias": "root", "isLeaf": false},
///        {"id": "<tree-node-1>", "alias": "node-1", "isLeaf": true},
///        {"id": "<tree-node-2>", "alias": "node-2", "isLeaf": true},
///     ],
///     "links": [
///        {"source": "<tree-node-0>", "target": "<tree-node-1>", "alias": "00"},
///        {"source": "<tree-node-0>", "target": "<tree-node-2>", "alias": "01"},
///     ],
/// }
/// ```
#[derive(Debug)]
pub struct JsonVisitor {
    display_format: IndexKeyDisplayFormat,
    result: JsonResult,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonResult {
    nodes: HashSet<JsonNode>,
    links: Vec<JsonLink>,
}
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonNode {
    id: String,
    alias: String,
    is_root: bool,
    is_leaf: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsonLink {
    source: String,
    target: String,
    alias: String,
}

impl JsonVisitor {
    pub fn new(display_format: IndexKeyDisplayFormat) -> Self {
        Self {
            display_format,
            result: JsonResult::default(),
        }
    }

    pub fn into_result(self) -> JsonResult {
        self.result
    }

    fn alias(s: impl Into<String>) -> String {
        const ALIAS_LENGTH: usize = 8;

        let s = s.into();
        if s.len() < ALIAS_LENGTH {
            s
        } else {
            format!("{}...", &s[..ALIAS_LENGTH])
        }
    }
}

#[async_trait]
impl TreeVisitor for JsonVisitor {
    async fn visit_root(
        &mut self,
        root_id: &TreeIdentifier,
        _root: &Tree,
    ) -> Result<TreeVisitorAction> {
        self.result.nodes.insert(JsonNode {
            id: root_id.to_string(),
            alias: Self::alias(root_id.to_string()),
            is_root: true,
            is_leaf: false,
        });

        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_branch(&mut self, info: TreeBranchInfo<'_>) -> Result<TreeVisitorAction> {
        self.result.nodes.insert(JsonNode {
            id: info.branch_id.to_string(),
            alias: Self::alias(info.branch_id.to_string()),
            is_root: false,
            is_leaf: false,
        });
        self.result.links.push(JsonLink {
            source: info.parent_id.to_string(),
            target: info.branch_id.to_string(),
            alias: info.local_key.format(self.display_format),
        });

        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_leaf(&mut self, info: TreeLeafInfo<'_>) -> Result<()> {
        self.result.nodes.insert(JsonNode {
            id: info.leaf_node.to_string(),
            alias: Self::alias(info.leaf_node.to_string()),
            is_root: false,
            is_leaf: true,
        });
        self.result.links.push(JsonLink {
            source: info.parent_id.to_string(),
            target: info.leaf_node.to_string(),
            alias: info.local_key.format(self.display_format),
        });

        Ok(())
    }
}
