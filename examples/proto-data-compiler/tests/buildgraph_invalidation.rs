use std::{collections::HashMap, hash::Hash, hash::Hasher};

use petgraph::{dot::Dot, Graph};
use service::{
    compiler_interface::{ResourceGuid, ENTITY_COMPILER},
    source_control::CommitRoot,
    EdgeDependencyType, ResourcePathId,
};

#[derive(Clone, Debug, Eq, Default)]
struct GraphNode {
    resource_path_id: ResourcePathId,
    verified_at: CommitRoot,
    changed_at: CommitRoot,
}

impl GraphNode {
    fn new(
        resource_path_id: ResourcePathId,
        verified_at: CommitRoot,
        changed_at: CommitRoot,
    ) -> Self {
        Self {
            resource_path_id,
            verified_at,
            changed_at,
        }
    }
}

impl PartialEq for GraphNode {
    fn eq(&self, other: &Self) -> bool {
        self.resource_path_id == other.resource_path_id && self.verified_at == other.verified_at
    }
}

impl Hash for GraphNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.resource_path_id.hash(state);
        self.verified_at.hash(state);
    }
}

impl std::fmt::Display for GraphNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\nverified_at {}\nchanged_at {}",
            self.resource_path_id, self.verified_at, self.changed_at
        )
    }
}

fn diff_files(previous_root: CommitRoot, _current_root: CommitRoot) -> Vec<GraphNode> {
    vec![GraphNode::new(
        ResourcePathId::new(ResourceGuid::ResourceC),
        previous_root,
        previous_root,
    )]
}

#[tokio::test]
async fn invalidate_graph() {
    let source_id =
        ResourcePathId::new(ResourceGuid::ResourceA).transform(ENTITY_COMPILER.to_string());

    let current_root: CommitRoot = 1234;

    let mut graph = Graph::<GraphNode, EdgeDependencyType>::new();

    let mut all_nodes = HashMap::new();

    let a_transform_node = GraphNode::new(source_id.clone(), current_root, current_root);
    let a_transform_index = graph.add_node(a_transform_node.clone());
    all_nodes.insert(a_transform_node, a_transform_index);

    let a_source_node = GraphNode::new(
        ResourcePathId::new(ResourceGuid::ResourceA),
        current_root,
        current_root,
    );
    let a_source_index = graph.add_node(a_source_node.clone());
    all_nodes.insert(a_source_node, a_source_index);

    let b_transform_node = GraphNode::new(
        ResourcePathId::new(ResourceGuid::ResourceB).transform(ENTITY_COMPILER.to_string()),
        current_root,
        current_root,
    );
    let b_transform_index = graph.add_node(b_transform_node.clone());
    all_nodes.insert(b_transform_node, b_transform_index);

    let b_source_node = GraphNode::new(
        ResourcePathId::new(ResourceGuid::ResourceB),
        current_root,
        current_root,
    );
    let b_source_index = graph.add_node(b_source_node.clone());
    all_nodes.insert(b_source_node, b_source_index);

    let c_transform_node = GraphNode::new(
        ResourcePathId::new(ResourceGuid::ResourceC).transform(ENTITY_COMPILER.to_string()),
        current_root,
        current_root,
    );
    let c_transform_index = graph.add_node(c_transform_node.clone());
    all_nodes.insert(c_transform_node, c_transform_index);

    let c_source_node = GraphNode::new(
        ResourcePathId::new(ResourceGuid::ResourceC),
        current_root,
        current_root,
    );
    let c_source_index = graph.add_node(c_source_node.clone());
    all_nodes.insert(c_source_node, c_source_index);

    graph.extend_with_edges(&[
        (a_transform_index, a_source_index, EdgeDependencyType::Build),
        (
            a_transform_index,
            b_transform_index,
            EdgeDependencyType::Runtime,
        ),
        (b_transform_index, b_source_index, EdgeDependencyType::Build),
        (
            a_transform_index,
            c_transform_index,
            EdgeDependencyType::Runtime,
        ),
        (c_transform_index, c_source_index, EdgeDependencyType::Build),
    ]);

    println!("Initial graph build\n{}", Dot::new(&graph));

    let previous_root = current_root;
    let current_root: CommitRoot = 4567;

    let nodes = diff_files(previous_root, current_root);

    for node in nodes {
        all_nodes.get(&node);
    }
}
