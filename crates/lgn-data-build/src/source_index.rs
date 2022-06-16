use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    hash::{Hash, Hasher},
    sync::Arc,
};

use hex::ToHex;
use lgn_content_store::{indexing::TreeIdentifier, Provider};
use lgn_data_offline::resource::Project;
use lgn_data_runtime::{ResourcePathId, ResourceTypeAndId};
use lgn_tracing::span_scope;
use lgn_utils::{DefaultHasher, DefaultHasher256};
use petgraph::{Directed, Graph};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::{output_index::AssetHash, Error};

pub const LGN_DATA_BUILD: &str = "data-build";

#[derive(Serialize, Deserialize, Debug)]
struct ResourceInfo {
    id: ResourcePathId,
    dependencies: Vec<ResourcePathId>,
    // hash of the content of this resource, None for derived resources.
    resource_hash: Option<String>,
}

impl ResourceInfo {
    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.dependencies.sort();
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SourceContent {
    version: String,
    resources: Vec<ResourceInfo>,
    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    pathid_mapping: BTreeMap<ResourceTypeAndId, ResourcePathId>,
}

impl SourceContent {
    fn new(version: &str) -> Self {
        Self {
            version: version.to_owned(),
            resources: vec![],
            pathid_mapping: BTreeMap::<_, _>::new(),
        }
    }

    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.resources.sort_by(|a, b| a.id.cmp(&b.id));
        for resource in &mut self.resources {
            resource.pre_serialize();
        }
    }

    fn write(&mut self) -> Result<Vec<u8>, Error> {
        self.pre_serialize();
        let mut buffer = vec![];
        serde_json::to_writer_pretty(&mut buffer, &self).map_err(|e| Error::Io(e.into()))?;
        Ok(buffer)
    }

    fn read(buffer: &[u8]) -> Result<Self, Error> {
        serde_json::from_reader(buffer).map_err(|e| Error::Io(e.into()))
    }

    pub fn record_pathid(&mut self, id: &ResourcePathId) {
        self.pathid_mapping.insert(id.resource_id(), id.clone());
    }

    pub fn lookup_pathid(&self, id: ResourceTypeAndId) -> Option<ResourcePathId> {
        self.pathid_mapping.get(&id).cloned()
    }

    /// Returns a combined hash of:
    /// * `id` resource's content.
    /// * content of all `id`'s dependencies.
    /// todo: at one point dependency filtering here will be useful.
    pub(crate) fn compute_source_hash(&self, id: ResourcePathId) -> AssetHash {
        let sorted_unique_resource_hashes: Vec<String> = {
            let mut unique_resources = HashMap::new();
            let mut queue: VecDeque<_> = VecDeque::new();

            queue.push_back(id);

            while let Some(resource) = queue.pop_front() {
                if let Some(resource_info) = self.resources.iter().find(|r| r.id == resource) {
                    unique_resources.insert(resource, resource_info.resource_hash.clone());

                    let newly_discovered_deps: Vec<_> = resource_info
                        .dependencies
                        .iter()
                        .filter(|r| !unique_resources.contains_key(*r))
                        .cloned()
                        .collect();

                    queue.extend(newly_discovered_deps);
                } else {
                    // follow the path otherwise.
                    if let Some(dep) = resource.direct_dependency() {
                        if !unique_resources.contains_key(&dep) {
                            queue.push_back(dep);
                        }
                    }
                }
            }

            let mut hashes = unique_resources
                .into_iter()
                .filter_map(|t| t.1)
                .collect::<Vec<String>>();
            hashes.sort_unstable();
            hashes
        };

        let mut hasher = DefaultHasher::new();
        for h in sorted_unique_resource_hashes {
            h.hash(&mut hasher);
        }
        AssetHash::from(hasher.finish())
    }

    pub(crate) fn update_resource(
        &mut self,
        id: ResourcePathId,
        resource_hash: Option<String>,
        mut deps: Vec<ResourcePathId>,
    ) -> bool {
        self.record_pathid(&id);
        for id in &deps {
            self.record_pathid(id);
        }
        if let Some(existing_res) = self.resources.iter_mut().find(|r| r.id == id) {
            deps.sort();

            let matching = existing_res
                .dependencies
                .iter()
                .zip(deps.iter())
                .filter(|&(a, b)| a == b)
                .count();
            if deps.len() == matching && existing_res.resource_hash == resource_hash {
                false
            } else {
                existing_res.dependencies = deps;
                existing_res.resource_hash = resource_hash;
                true
            }
        } else {
            let info = ResourceInfo {
                id,
                dependencies: deps,
                resource_hash,
            };
            self.resources.push(info);
            true
        }
    }

    /// Create an ordered build graph with edges directed towards
    /// `compile_path`.
    pub(crate) fn generate_build_graph(
        &self,
        compile_path: ResourcePathId,
    ) -> Graph<ResourcePathId, ()> {
        span_scope!("generate_build_graph");

        let mut dep_graph = Graph::<ResourcePathId, (), Directed>::new();
        let mut indices = HashMap::<ResourcePathId, petgraph::prelude::NodeIndex>::new();
        let mut processed = vec![];
        let mut queue = VecDeque::<ResourcePathId>::new();

        // we process the whole path as derived resources might not exist in
        // the build index as those are never referred to as dependencies.
        let mut resource_path = Some(compile_path);
        while let Some(path) = resource_path {
            let direct_dependency = path.direct_dependency();
            queue.push_back(path);
            resource_path = direct_dependency;
        }

        let mut get_or_create_index = |res, dep_graph: &mut Graph<_, _>| {
            if let Some(own_index) = indices.get(&res) {
                *own_index
            } else {
                let own_index = dep_graph.add_node(res.clone());
                indices.insert(res, own_index);
                own_index
            }
        };

        while let Some(res) = queue.pop_front() {
            processed.push(res.clone());

            let own_index = get_or_create_index(res.clone(), &mut dep_graph);

            //
            // todo: this does not include transitive dependencies now.
            // this means that all the derived resources only depend on their
            // direct dependency
            //
            if let Some(deps) = self.find_dependencies(&res) {
                assert!(
                    res.direct_dependency().is_none()
                        || deps.contains(&res.direct_dependency().unwrap())
                );
                for d in &deps {
                    let other_index = get_or_create_index(d.clone(), &mut dep_graph);
                    dep_graph.update_edge(other_index, own_index, ());
                }

                let unprocessed: VecDeque<ResourcePathId> = deps
                    .into_iter()
                    .filter(|r| !processed.contains(r))
                    .collect();
                queue.extend(unprocessed);
            } else if let Some(direct_dependency) = res.direct_dependency() {
                let other_index = get_or_create_index(direct_dependency, &mut dep_graph);
                dep_graph.update_edge(other_index, own_index, ());
            }
        }
        dep_graph
    }

    pub(crate) fn find_dependencies(&self, id: &ResourcePathId) -> Option<Vec<ResourcePathId>> {
        self.resources
            .iter()
            .find(|r| &r.id == id)
            .map(|resource| resource.dependencies.clone())
    }
}

impl Extend<Self> for SourceContent {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        for e in iter {
            assert_eq!(self.version, e.version);
            self.resources.extend(e.resources);
            self.pathid_mapping.extend(e.pathid_mapping);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceChecksum((TreeIdentifier, TreeIdentifier));

pub(crate) struct SourceIndex {
    current: Option<(SourceChecksum, SourceContent)>,
    pub(super) content_store: Arc<Provider>,
}

impl<'a> std::fmt::Debug for SourceIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceIndex")
            .field("current", &self.current)
            .finish()
    }
}

impl SourceIndex {
    pub(crate) fn new(content_store: Arc<Provider>) -> Self {
        Self {
            content_store,
            current: None,
        }
    }
    pub fn current(&self) -> Option<&SourceContent> {
        self.current.as_ref().map(|(_, index)| index)
    }

    async fn source_pull_tree(
        &self,
        project: &Project,
        version: &str,
        mut uploads: Vec<(Vec<u8>, Vec<u8>)>,
    ) -> Result<(SourceContent, Vec<(Vec<u8>, Vec<u8>)>), Error> {
        let dir_checksum = {
            let mut hasher = DefaultHasher256::new();
            LGN_DATA_BUILD.hash(&mut hasher);
            project.root_checksum().hash(&mut hasher);
            version.hash(&mut hasher);
            hasher.finish_256()[..].to_vec()
        };

        let content = self.content_store.read_alias(dir_checksum.clone()).await;

        if let Ok(cached_data) = content {
            let source_index = SourceContent::read(&cached_data)?;
            Ok((source_index, uploads))
        } else {
            let resources = project.get_resources().await?;

            let mut content = SourceContent::new(version);

            for (resource_type_id, content_store_resource_id) in resources {
                let resource_hash = {
                    let mut hasher = DefaultHasher256::new();
                    content_store_resource_id.hash(&mut hasher);
                    hasher.finish_256().encode_hex::<String>()
                };

                let resource_deps = project.resource_dependencies(resource_type_id).await?;

                content.update_resource(
                    ResourcePathId::from(resource_type_id),
                    Some(resource_hash),
                    resource_deps.clone(),
                );

                // add each derived dependency with it's direct dependency listed in deps.
                for dependency in resource_deps {
                    if let Some(direct_dependency) = dependency.direct_dependency() {
                        content.update_resource(dependency, None, vec![direct_dependency]);
                    }
                }
            }

            uploads.push((dir_checksum, content.write()?));
            Ok((content, uploads))
        }
    }

    pub async fn source_pull(&mut self, project: &Project, version: &str) -> Result<(), Error> {
        let root_checksum = SourceChecksum(project.root_checksum());

        if let Some((current_checksum, _source_index)) = &self.current {
            if current_checksum == &root_checksum {
                return Ok(());
            }
        }

        let (content, uploads) = self.source_pull_tree(project, version, vec![]).await?;

        for (dir_checksum, buffer) in uploads {
            self.content_store
                .write_alias(dir_checksum, &buffer)
                .await?;
        }

        self.current = Some((root_checksum, content));
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use lgn_data_offline::resource::ResourcePathName;
    use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor, ResourceId};

    use super::*;

    #[tokio::test]
    async fn pathid_records() {
        // dummy ids - the actual project structure is irrelevant in this test.
        let source_id = ResourceTypeAndId {
            kind: refs_resource::TestResource::TYPE,
            id: ResourceId::new(),
        };
        let source_resource = ResourcePathId::from(source_id);
        let intermediate_resource = source_resource.push(refs_resource::TestResource::TYPE);
        let output_resource = intermediate_resource.push(refs_resource::TestResource::TYPE);

        let source_index = {
            let mut source_index = SourceContent::new("0.0.1");

            // all dependencies need to be explicitly specified
            let intermediate_deps = vec![source_resource.clone()];
            let output_deps = vec![intermediate_resource.clone()];

            let resource_hash = Some("Blah".to_string()); // this is irrelevant to the test

            source_index.update_resource(
                intermediate_resource.clone(),
                resource_hash.clone(),
                intermediate_deps.clone(),
            );
            source_index.update_resource(source_resource.clone(), resource_hash.clone(), vec![]);
            source_index.update_resource(
                intermediate_resource.clone(),
                resource_hash.clone(),
                intermediate_deps,
            );
            source_index.update_resource(output_resource.clone(), resource_hash, output_deps);
            source_index
        };

        assert_eq!(
            source_index.lookup_pathid(source_id).unwrap(),
            source_resource
        );
        assert_eq!(
            source_index
                .lookup_pathid(intermediate_resource.resource_id())
                .unwrap(),
            intermediate_resource
        );
        assert_eq!(
            source_index
                .lookup_pathid(output_resource.resource_id())
                .unwrap(),
            output_resource
        );
    }

    #[tokio::test]
    async fn dependency_update() {
        // dummy ids - the actual project structure is irrelevant in this test.
        let source_id = ResourceTypeAndId {
            kind: refs_resource::TestResource::TYPE,
            id: ResourceId::new(),
        };
        let source_resource = ResourcePathId::from(source_id);
        let intermediate_resource = source_resource.push(refs_resource::TestResource::TYPE);
        let output_resources = intermediate_resource.push(refs_resource::TestResource::TYPE);

        let mut source_index = SourceContent::new("0.0.1");

        // all dependencies need to be explicitly specified
        let intermediate_deps = vec![source_resource.clone()];
        let output_deps = vec![intermediate_resource.clone()];

        let resource_hash = Some("Blah".to_string()); // this is irrelevant to the test

        source_index.update_resource(
            intermediate_resource.clone(),
            resource_hash.clone(),
            intermediate_deps.clone(),
        );
        assert_eq!(source_index.resources.len(), 1);
        assert_eq!(source_index.resources[0].dependencies.len(), 1);

        source_index.update_resource(source_resource, resource_hash.clone(), vec![]);
        assert_eq!(source_index.resources.len(), 2);
        assert_eq!(source_index.resources[1].dependencies.len(), 0);

        source_index.update_resource(
            intermediate_resource,
            resource_hash.clone(),
            intermediate_deps,
        );
        assert_eq!(source_index.resources.len(), 2);
        assert_eq!(source_index.resources[0].dependencies.len(), 1);

        source_index.update_resource(output_resources, resource_hash, output_deps);
        assert_eq!(source_index.resources.len(), 3);
        assert_eq!(source_index.resources[2].dependencies.len(), 1);
    }

    fn current_checksum(index: &SourceIndex) -> SourceChecksum {
        index
            .current
            .as_ref()
            .map(|(checksum, _)| checksum.clone())
            .unwrap()
    }

    #[tokio::test]
    async fn source_index_cache() {
        let work_dir = tempfile::tempdir().unwrap();
        let data_provider = Arc::new(Provider::new_in_memory());

        let mut project =
            Project::new_with_remote_mock(&work_dir.path(), Arc::new(Provider::new_in_memory()))
                .await
                .expect("failed to create a project");

        let version = "0.0.1";

        let mut source_index = SourceIndex::new(data_provider);

        let _first_entry_checksum = {
            source_index.source_pull(&project, version).await.unwrap();
            current_checksum(&source_index)
        };

        let resources = AssetRegistryOptions::new()
            .add_processor::<refs_resource::TestResource>()
            .create()
            .await;

        let (resource_id, resource_handle) = {
            let resource_handle = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .expect("new resource")
                .typed::<refs_resource::TestResource>();

            let mut edit = resource_handle.instantiate(&resources).unwrap();
            edit.content = "hello".to_string();
            resource_handle.apply(edit, &resources);

            let id = ResourceId::from_raw(0xaabbccddeeff00000000000000000000);

            let resource_id = ResourceTypeAndId {
                kind: refs_resource::TestResource::TYPE,
                id,
            };

            project
                .add_resource_with_id(
                    ResourcePathName::new("test_source"),
                    resource_id,
                    &resource_handle,
                    &resources,
                )
                .await
                .expect("adding the resource");

            (resource_id, resource_handle)
        };

        // initially we have 0 subfolders
        // number of indices: 1

        // one resource creates 3-levels deep folder hierarchy.
        // including the root index refresh that creates 4 new cached entries.
        // number of indices: 1 + 4

        // new resource creates a new cached entry
        let second_entry_checksum = {
            source_index.source_pull(&project, version).await.unwrap();
            current_checksum(&source_index)
        };

        // committing changes does not create a new entry
        {
            project.commit("test").await.expect("successful commit");
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(current_checksum(&source_index), second_entry_checksum);
        }

        // modifying a resource changes the whole hierarchy.
        // number of indices: 1 + 4 + 4

        // modify a resource
        let third_checksum = {
            let mut edit = resource_handle
                .instantiate(&resources)
                .expect("loaded resource");
            edit.content = "hello world!".to_string();
            resource_handle.apply(edit, &resources);

            project
                .save_resource(resource_id, resource_handle, &resources)
                .await
                .expect("successful save");

            source_index.source_pull(&project, version).await.unwrap();
            current_checksum(&source_index)
        };

        // committing changes does not create a new entry
        {
            project.commit("test").await.expect("successful commit");
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(current_checksum(&source_index), third_checksum);
        }

        // deleting the resource takes us back to the previous cache entry.
        {
            project
                .delete_resource(resource_id)
                .await
                .expect("removed resource");
            source_index.source_pull(&project, version).await.unwrap();
            // TODO: fix test, main index id changes event though content returns to "empty"
            // assert_eq!(current_checksum(&source_index), first_entry_checksum);
        }
    }
}
