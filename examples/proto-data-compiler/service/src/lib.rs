#![allow(dead_code)]
use std::{
    cmp::Ordering,
    collections::{hash_map::DefaultHasher, HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
};

use build_db::{BuildDb, VersionHash};
use compiler_interface::{BuildParams, CompilerType, ResourceGuid};
use petgraph::{graph::NodeIndex, Graph};
use source_control::{CommitRoot, SourceControl};
use strum_macros::Display;

pub mod build_db;
pub mod compiler_interface;
pub mod content_store;
pub mod data_execution_provider;
pub mod resource_manager;
pub mod source_control;
pub mod worker;

#[derive(Clone, Display, Debug, PartialEq)]
pub enum EdgeDependencyType {
    Runtime,
    Build,
}

enum Reference {
    Runtime(ResourcePathId),
    Build(ResourcePathId),
}

pub struct CompilationInputs {
    pub data_input: String,
    pub output_id: ResourcePathId,
}

#[derive(Clone, Eq, Debug)]
pub struct ResourcePathId {
    pub source_resource: ResourceGuid,
    pub transformations: Vec<CompilerType>,
}

impl Default for ResourcePathId {
    fn default() -> Self {
        Self {
            source_resource: ResourceGuid::Invalid,
            transformations: vec![],
        }
    }
}

impl From<&ResourcePathId> for ResourcePathId {
    fn from(id: &ResourcePathId) -> Self {
        id.clone()
    }
}

impl PartialEq for ResourcePathId {
    fn eq(&self, other: &Self) -> bool {
        self.source_resource == other.source_resource
    }
}

impl Hash for ResourcePathId {
    fn hash<H: Hasher>(&self, resource: &mut H) {
        self.source_resource.hash(resource);
    }
}

impl PartialOrd for ResourcePathId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.source_resource.partial_cmp(&other.source_resource)
    }
}

impl std::fmt::Display for ResourcePathId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formated_transformations = self
            .transformations
            .iter()
            .fold(String::new(), |result, compiler_type| {
                result + " | " + compiler_type
            });

        write!(f, "{}{}", self.source_resource, formated_transformations,)
    }
}

impl ResourcePathId {
    pub fn new_transformation(source: ResourceGuid, transformations: Vec<CompilerType>) -> Self {
        Self {
            source_resource: source,
            transformations,
        }
    }

    pub fn new(source: ResourceGuid) -> Self {
        Self {
            source_resource: source,
            transformations: vec![],
        }
    }

    pub fn transform(mut self, t: CompilerType) -> Self {
        self.transformations.push(t);
        self
    }

    pub fn path_dependency(&self) -> Option<Self> {
        if self.is_source_resource() {
            None
        } else {
            let transformations: Vec<String> = Vec::from_iter(
                self.transformations[0..(self.transformations.len() - 1).clone()]
                    .iter()
                    .cloned(),
            );
            Some(Self {
                source_resource: self.source_resource.clone(),
                transformations,
            })
        }
    }

    pub fn is_source_resource(&self) -> bool {
        self.transformations.is_empty()
    }

    pub fn last_transformation(&self) -> Option<CompilerType> {
        if self.transformations.is_empty() {
            None
        } else {
            Some(self.transformations[self.transformations.len() - 1].clone())
        }
    }
}

// returns none if not built

pub async fn minimal_hash_internal(
    id: ResourcePathId,
    dependencies: Vec<ResourcePathId>,
    commit_root: CommitRoot,
    _build_params: &BuildParams,
    source_control: &SourceControl,
) -> Option<VersionHash> {
    print!("hash of '{}': ", id);

    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);

    // this depends on all sources.
    // instead it should depend on the content of the resource.
    //commit_root.hash(&mut hasher);

    let source_addr = source_control
        .find_address(id.source_resource, commit_root)
        .unwrap();
    source_addr.hash(&mut hasher);

    print!("{} ", source_addr);

    // todo: recursively find_dependencies...
    for dependency_addr in dependencies.iter().map(|dep| {
        source_control
            .find_address(dep.source_resource, commit_root)
            .unwrap()
    }) {
        print!("|{} ", dependency_addr);
        dependency_addr.hash(&mut hasher);
    }

    let output = hasher.finish();

    println!("= {}", output);
    Some(output as u128)
}

pub async fn minimal_hash(
    id: ResourcePathId,
    commit_root: CommitRoot,
    _build_params: &BuildParams,
    source_control: &SourceControl,
    build_db: &BuildDb,
) -> Option<VersionHash> {
    // source resource's hash is it's ContentAddr on a given CommitId.
    assert!(!id.is_source_resource());
    if let Some(dependencies) = build_db.find_dependencies(id.clone(), commit_root).await {
        minimal_hash_internal(id, dependencies, commit_root, _build_params, source_control).await
    } else {
        None
    }
}

/// The graph is built on the caller's side using multiple database calls or the build database is a service executing code.
pub async fn dependency_graph(
    source: ResourcePathId,
    commit_root: CommitRoot,
    build_params: &BuildParams,
    build_db: &BuildDb,
    source_control: &SourceControl,
) -> Graph<ResourcePathId, EdgeDependencyType> {
    let mut edges = Vec::<(ResourcePathId, ResourcePathId, EdgeDependencyType)>::new();

    let push_unique =
        |edges: &mut Vec<(ResourcePathId, ResourcePathId, EdgeDependencyType)>,
         value: (ResourcePathId, ResourcePathId, EdgeDependencyType)| {
            if !edges.contains(&value) {
                edges.push(value);
            }
        };

    let mut pending: VecDeque<_> = [source.clone()].iter().cloned().collect();
    let mut processed = HashSet::new();

    while let Some(node) = pending.pop_front() {
        if let Some(path_dep) = node.path_dependency() {
            push_unique(
                &mut edges,
                (node.clone(), path_dep, EdgeDependencyType::Build),
            );
        }

        if !node.is_source_resource() {
            let version_hash = minimal_hash(
                node.clone(),
                commit_root,
                build_params,
                source_control,
                build_db,
            )
            .await
            .unwrap();

            if let Some((output, build_deps)) = build_db.find(node.clone(), version_hash).await {
                for content in &output.content {
                    if node != content.id {
                        push_unique(
                            &mut edges,
                            (node.clone(), content.id.clone(), EdgeDependencyType::Build),
                        );
                    }
                    for reference in &content.references {
                        push_unique(
                            &mut edges,
                            (node.clone(), reference.clone(), EdgeDependencyType::Runtime),
                        );

                        if !processed.contains(reference) && !pending.contains(reference) {
                            pending.push_back(reference.clone());
                        }
                    }
                }

                for dep in build_deps {
                    push_unique(
                        &mut edges,
                        (node.clone(), dep.clone(), EdgeDependencyType::Build),
                    );

                    if !processed.contains(&dep) && !pending.contains(&dep) {
                        pending.push_back(dep);
                    }
                }
            }
        }

        processed.insert(node.clone());
    }

    let mut indices = HashMap::<ResourcePathId, NodeIndex>::new();

    let mut graph = Graph::<ResourcePathId, EdgeDependencyType>::new();

    // build nodes, return edges.
    let edges = edges
        .iter()
        .map(|(from, to, ty)| {
            let from = indices.get(from).cloned().unwrap_or_else(|| {
                let idx = graph.add_node(from.clone());
                indices.insert(from.clone(), idx);
                idx
            });

            let to = indices.get(to).cloned().unwrap_or_else(|| {
                let idx = graph.add_node(to.clone());
                indices.insert(to.clone(), idx);
                idx
            });

            (from, to, ty.clone())
        })
        .collect::<Vec<(NodeIndex, NodeIndex, EdgeDependencyType)>>();

    for (from, to, ty) in edges {
        graph.add_edge(from, to, ty);
    }

    graph
}
