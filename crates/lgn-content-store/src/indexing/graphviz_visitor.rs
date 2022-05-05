use std::{collections::HashSet, path::Path};

use async_trait::async_trait;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use super::{
    tree::TreeIdentifier, IndexKey, Result, Tree, TreeLeafNode, TreeVisitor, TreeVisitorAction,
};

pub struct GraphvizVisitor<Output> {
    output: Output,
    visited: HashSet<(TreeIdentifier, IndexKey)>,
}

impl<Output> GraphvizVisitor<Output> {
    pub fn new(output: Output) -> Self {
        Self {
            output,
            visited: HashSet::new(),
        }
    }

    fn alias(s: impl Into<String>) -> String {
        const ALIAS_LENGTH: usize = 8;

        let s = s.into();
        if s.len() <= ALIAS_LENGTH {
            s
        } else {
            format!("{}...", &s[..ALIAS_LENGTH])
        }
    }
}

impl GraphvizVisitor<tokio::fs::File> {
    /// Write a graphviz file to the specified path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened.
    pub async fn create(path: impl AsRef<Path>) -> Result<Self> {
        let file = tokio::fs::File::create(path).await?;
        Ok(Self::new(file))
    }
}

#[async_trait]
impl<Output> TreeVisitor for GraphvizVisitor<Output>
where
    Output: AsyncWrite + Send + Unpin,
{
    async fn visit_root(
        &mut self,
        root_id: &TreeIdentifier,
        _root: &Tree,
    ) -> Result<TreeVisitorAction> {
        self.visited.clear();
        self.output.write_all(b"digraph G {\n").await?;
        self.output
            .write_all(format!("\"{}\" [label=\"root\", shape=\"diamond\"]\n", root_id).as_bytes())
            .await?;
        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_branch(
        &mut self,
        parent_id: &TreeIdentifier,
        _key: &IndexKey,
        local_key: &IndexKey,
        branch_id: &TreeIdentifier,
        _branch: &Tree,
        _depth: usize,
    ) -> Result<TreeVisitorAction> {
        if self.visited.insert((parent_id.clone(), local_key.clone())) {
            self.output
                .write_all(
                    format!(
                        "\"{}\" [label=\"{}\"]\n",
                        branch_id,
                        Self::alias(branch_id.to_string()),
                    )
                    .as_bytes(),
                )
                .await?;
            self.output
                .write_all(
                    format!(
                        "\"{}\" -> \"{}\" [label=\"{}\"]\n",
                        parent_id, branch_id, local_key,
                    )
                    .as_bytes(),
                )
                .await?;
        }

        Ok(TreeVisitorAction::Continue)
    }

    async fn visit_leaf(
        &mut self,
        parent_id: &TreeIdentifier,
        _key: &IndexKey,
        local_key: &IndexKey,
        leaf: &TreeLeafNode,
        _depth: usize,
    ) -> Result<()> {
        if self.visited.insert((parent_id.clone(), local_key.clone())) {
            self.output
                .write_all(
                    format!(
                        "\"{}\" [label=\"{}\", shape=\"rectangle\", color=\"green\"]\n",
                        leaf,
                        Self::alias(leaf.to_string()),
                    )
                    .as_bytes(),
                )
                .await?;
            self.output
                .write_all(
                    format!(
                        "\"{}\" -> \"{}\" [label=\"{}\"]\n",
                        parent_id, leaf, local_key,
                    )
                    .as_bytes(),
                )
                .await?;
        }

        Ok(())
    }

    async fn visit_done(&mut self, _root_id: &TreeIdentifier) -> Result<()> {
        self.output.write_all(b"}\n").await?;
        self.output.shutdown().await?;

        Ok(())
    }
}
