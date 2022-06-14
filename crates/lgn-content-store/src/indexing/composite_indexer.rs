use std::pin::Pin;

use async_stream::stream;
use async_trait::async_trait;
use futures::Stream;
use tokio_stream::StreamExt;

use crate::Provider;

use super::{
    empty_tree_id, BasicIndexer, Error, IndexKey, Result, TreeDiffSide, TreeIdentifier,
    TreeLeafNode,
};

/// A composite indexer that combines multiple indexers to create composite
/// indexes.
///
/// Index keys for composite indexers can be built using the `IndexKey::compose`
/// method.
#[derive(Clone)]
pub struct CompositeIndexer<First, Second> {
    first: First,
    second: Second,
}

impl<First, Second> CompositeIndexer<First, Second> {
    /// Instantiates a new composite indexer.
    pub fn new(first: First, second: Second) -> Self {
        Self { first, second }
    }
}

#[async_trait]
impl<First, Second> BasicIndexer for CompositeIndexer<First, Second>
where
    First: BasicIndexer + Send + Sync,
    Second: BasicIndexer + Send + Sync,
{
    async fn get_leaf(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &IndexKey,
    ) -> Result<Option<TreeLeafNode>> {
        let (first, second) = index_key.decompose()?;

        Ok(
            match self.first.get_leaf(provider, root_id, &first).await? {
                Some(leaf) => match leaf {
                    TreeLeafNode::Resource(_) => {
                        return Err(Error::CorruptedTree(
                            "expected a sub-tree leaf but got a resource key".to_string(),
                        ))
                    }
                    TreeLeafNode::TreeRoot(tree_id) => {
                        self.second.get_leaf(provider, &tree_id, &second).await?
                    }
                },
                None => None,
            },
        )
    }

    async fn add_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<TreeIdentifier> {
        let (first, second) = index_key.decompose()?;

        Ok(
            match self.first.get_leaf(provider, root_id, &first).await? {
                Some(leaf) => match leaf {
                    TreeLeafNode::Resource(_) => {
                        return Err(Error::CorruptedTree(
                            "expected a sub-tree leaf but got a resource key".to_string(),
                        ))
                    }
                    TreeLeafNode::TreeRoot(tree_id) => {
                        let tree_id = self
                            .second
                            .add_leaf(provider, &tree_id, &second, leaf_node)
                            .await?;

                        self.first
                            .replace_leaf(
                                provider,
                                root_id,
                                &first,
                                TreeLeafNode::TreeRoot(tree_id),
                            )
                            .await?
                            .0
                    }
                },
                None => {
                    let tree_id = empty_tree_id(provider).await?;
                    let tree_id = self
                        .second
                        .add_leaf(provider, &tree_id, &second, leaf_node)
                        .await?;

                    self.first
                        .add_leaf(provider, root_id, &first, TreeLeafNode::TreeRoot(tree_id))
                        .await?
                }
            },
        )
    }

    async fn replace_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
        leaf_node: TreeLeafNode,
    ) -> Result<(TreeIdentifier, TreeLeafNode)> {
        let (first, second) = index_key.decompose()?;

        Ok(
            match self.first.get_leaf(provider, root_id, &first).await? {
                Some(leaf) => match leaf {
                    TreeLeafNode::Resource(_) => {
                        return Err(Error::CorruptedTree(
                            "expected a sub-tree leaf but got a resource key".to_string(),
                        ))
                    }
                    TreeLeafNode::TreeRoot(tree_id) => {
                        let (tree_id, old_leaf) = self
                            .second
                            .replace_leaf(provider, &tree_id, &second, leaf_node)
                            .await?;

                        (
                            self.first
                                .replace_leaf(
                                    provider,
                                    root_id,
                                    &first,
                                    TreeLeafNode::TreeRoot(tree_id),
                                )
                                .await?
                                .0,
                            old_leaf,
                        )
                    }
                },
                None => return Err(Error::IndexTreeLeafNodeNotFound(index_key.clone())),
            },
        )
    }

    async fn remove_leaf<'call>(
        &self,
        provider: &Provider,
        root_id: &TreeIdentifier,
        index_key: &'call IndexKey,
    ) -> Result<(TreeIdentifier, TreeLeafNode)> {
        let (first, second) = index_key.decompose()?;

        Ok(
            match self.first.get_leaf(provider, root_id, &first).await? {
                Some(leaf) => match leaf {
                    TreeLeafNode::Resource(_) => {
                        return Err(Error::CorruptedTree(
                            "expected a sub-tree leaf but got a resource key".to_string(),
                        ))
                    }
                    TreeLeafNode::TreeRoot(tree_id) => {
                        let (tree_id, old_leaf) =
                            self.second.remove_leaf(provider, &tree_id, &second).await?;

                        (
                            self.first
                                .replace_leaf(
                                    provider,
                                    root_id,
                                    &first,
                                    TreeLeafNode::TreeRoot(tree_id),
                                )
                                .await?
                                .0,
                            old_leaf,
                        )
                    }
                },
                None => return Err(Error::IndexTreeLeafNodeNotFound(index_key.clone())),
            },
        )
    }

    async fn enumerate_leaves<'s>(
        &'s self,
        provider: &'s Provider,
        root_id: &'s TreeIdentifier,
    ) -> Result<Pin<Box<dyn Stream<Item = (IndexKey, Result<TreeLeafNode>)> + Send + 's>>> {
        let leaves = self.first.enumerate_leaves(provider, root_id).await?;

        Ok(Box::pin(stream! {
            tokio::pin!(leaves);

            while let Some((key, leaf)) = leaves.next().await {
                match leaf {
                    Ok(TreeLeafNode::Resource(_)) => {
                        yield (key, Err(Error::CorruptedTree(format!(
                            "expected a sub-tree leaf but got a resource key"
                        ))));
                    }
                    Ok(TreeLeafNode::TreeRoot(tree_id)) => {
                        match self.second.enumerate_leaves(provider, &tree_id).await {
                            Ok(leaves) => {
                                tokio::pin!(leaves);

                                while let Some((subkey, leaf)) = leaves.next().await {
                                    yield (key.clone().compose_with(subkey), leaf);
                                }
                            }
                            Err(err) => {
                                yield (key, Err(err));
                            }
                        }
                    }
                    Err(err) => {
                        yield (key, Err(err));
                    }
                }
            }
        }))
    }

    async fn diff_leaves<'s>(
        &'s self,
        provider: &'s Provider,
        base_key: &'s IndexKey,
        left_id: &'s TreeIdentifier,
        right_id: &'s TreeIdentifier,
    ) -> Result<
        Pin<Box<dyn Stream<Item = (TreeDiffSide, IndexKey, Result<TreeLeafNode>)> + Send + 's>>,
    > {
        let diff = self
            .first
            .diff_leaves(provider, base_key, left_id, right_id)
            .await?;

        Ok(Box::pin(stream! {
            tokio::pin!(diff);

            while let Some((side, index_key, leaf_node_result)) = diff.next().await {
                yield (side, index_key, leaf_node_result);
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;

    use crate::{
        indexing::{
            empty_tree_id, BasicIndexer, CompositeIndexer, IndexKey, ResourceIdentifier,
            StaticIndexer, StringPathIndexer, TreeLeafNode,
        },
        Identifier, Provider,
    };

    #[tokio::test]
    async fn test_composite_indexer() {
        let provider = Provider::new_in_memory();
        let idx = CompositeIndexer::new(StaticIndexer::new(4), StringPathIndexer::default());

        // This is our starting point: we write an empty tree.
        //
        // In all likelyhood, the generated identifier will benefit from
        // small-content optimization and not actually be written anywhere.
        let tree_id = empty_tree_id(&provider).await.unwrap();

        assert!(idx
            .get_leaf(&provider, &tree_id, &IndexKey::compose(4_u32, "/foo/bar"))
            .await
            .unwrap()
            .is_none());

        let tree_id = idx
            .add_leaf(
                &provider,
                &tree_id,
                &IndexKey::compose(4_u32, "/foo/bar"),
                TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello"))),
            )
            .await
            .unwrap();

        let tree_id = idx
            .add_leaf(
                &provider,
                &tree_id,
                &IndexKey::compose(3_u32, "/foo/bar"),
                TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello2"))),
            )
            .await
            .unwrap();

        let tree_id = idx
            .add_leaf(
                &provider,
                &tree_id,
                &IndexKey::compose(4_u32, "/foo/baz"),
                TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello3"))),
            )
            .await
            .unwrap();

        assert_eq!(
            idx.get_leaf(&provider, &tree_id, &IndexKey::compose(4_u32, "/foo/bar"))
                .await
                .unwrap(),
            Some(TreeLeafNode::Resource(ResourceIdentifier(
                Identifier::new_data(b"hello")
            )))
        );

        let (tree_id, old_leaf_node) = idx
            .replace_leaf(
                &provider,
                &tree_id,
                &IndexKey::compose(4_u32, "/foo/bar"),
                TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello-updated"))),
            )
            .await
            .unwrap();

        assert_eq!(
            old_leaf_node,
            TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello")))
        );
        assert_eq!(
            idx.get_leaf(&provider, &tree_id, &IndexKey::compose(4_u32, "/foo/bar"))
                .await
                .unwrap(),
            Some(TreeLeafNode::Resource(ResourceIdentifier(
                Identifier::new_data(b"hello-updated")
            )))
        );

        let (tree_id, old_leaf_node) = idx
            .remove_leaf(&provider, &tree_id, &IndexKey::compose(4_u32, "/foo/bar"))
            .await
            .unwrap();
        assert_eq!(
            old_leaf_node,
            TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello-updated")))
        );

        assert!(idx
            .get_leaf(&provider, &tree_id, &IndexKey::compose(4_u32, "/foo/bar"))
            .await
            .unwrap()
            .is_none());

        let leaves = idx
            .enumerate_leaves(&provider, &tree_id)
            .await
            .unwrap()
            .map(|(key, leaf)| (key, leaf.unwrap()))
            .collect::<Vec<_>>()
            .await;

        assert_eq!(
            leaves,
            vec![
                (
                    IndexKey::compose(3_u32, "/foo/bar"),
                    TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello2")))
                ),
                (
                    IndexKey::compose(4_u32, "/foo/baz"),
                    TreeLeafNode::Resource(ResourceIdentifier(Identifier::new_data(b"hello3")))
                ),
            ],
        );

        //crate::indexing::GraphvizVisitor::create("tree.dot", IndexKeyDisplayFormat::Hex)
        //    .await
        //    .unwrap()
        //    .visit(&provider, &tree_id)
        //    .await
        //    .unwrap();
    }
}
