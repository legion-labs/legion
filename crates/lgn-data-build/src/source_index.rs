use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::Seek,
    path::{Path, PathBuf},
    str::FromStr,
};

use lgn_content_store::{Checksum, ContentStore};
use lgn_data_offline::{
    resource::{Project, ResourceHash, Tree},
    ResourcePathId,
};
use lgn_data_runtime::{ResourceId, ResourceTypeAndId};
use lgn_utils::DefaultHasher;
use petgraph::{Directed, Graph};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

use crate::Error;

#[derive(Serialize, Deserialize, Debug)]
struct ResourceInfo {
    id: ResourcePathId,
    dependencies: Vec<ResourcePathId>,
    // hash of the content of this resource, None for derived resources.
    resource_hash: Option<ResourceHash>,
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
    pub(crate) fn compute_source_hash(&self, id: ResourcePathId) -> ResourceHash {
        let sorted_unique_resource_hashes: Vec<ResourceHash> = {
            let mut unique_resources = HashMap::new();
            let mut queue: VecDeque<_> = VecDeque::new();

            queue.push_back(id);

            while let Some(resource) = queue.pop_front() {
                if let Some(resource_info) = self.resources.iter().find(|r| r.id == resource) {
                    unique_resources.insert(resource, resource_info.resource_hash);

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

            let mut hashes: Vec<ResourceHash> =
                unique_resources.into_iter().filter_map(|t| t.1).collect();
            hashes.sort_unstable();
            hashes
        };

        let mut hasher = DefaultHasher::new();
        for h in sorted_unique_resource_hashes {
            h.hash(&mut hasher);
        }
        hasher.finish().into()
    }

    pub(crate) fn update_resource(
        &mut self,
        id: ResourcePathId,
        resource_hash: Option<ResourceHash>,
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

impl Extend<SourceContent> for SourceContent {
    fn extend<T: IntoIterator<Item = SourceContent>>(&mut self, iter: T) {
        for e in iter {
            assert_eq!(self.version, e.version);
            self.resources.extend(e.resources);
            self.pathid_mapping.extend(e.pathid_mapping);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct IndexKeys {
    keys: BTreeMap<String, Checksum>,
    version: String,
}

pub(crate) struct SourceIndex {
    index_keys: IndexKeys,
    current: Option<(String, SourceContent)>,
    file: File,
    content_store: Box<dyn ContentStore + Send + Sync>,
}

impl std::fmt::Debug for SourceIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SourceIndex")
            .field("index_keys", &self.index_keys)
            .field("file", &self.file)
            .finish()
    }
}

impl SourceIndex {
    pub(crate) fn create_new(
        source_index: &Path,
        content_store: Box<dyn ContentStore + Send + Sync>,
        version: &str,
    ) -> Result<Self, Error> {
        let source_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(source_index)
            .map_err(|e| Error::Io(e.into()))?;

        let index_keys = IndexKeys {
            version: String::from(version),
            keys: BTreeMap::<String, Checksum>::new(),
        };

        serde_json::to_writer_pretty(&source_file, &index_keys).map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            index_keys,
            content_store,
            current: None,
            file: source_file,
        })
    }

    fn load(
        path: impl AsRef<Path>,
        content_store: Box<dyn ContentStore + Send + Sync>,
    ) -> Result<Self, Error> {
        let source_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|_e| Error::NotFound)?;

        let index_keys: IndexKeys =
            serde_json::from_reader(&source_file).map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            index_keys,
            content_store,
            current: None,
            file: source_file,
        })
    }

    pub(crate) fn open(
        source_index: &Path,
        content_store: Box<dyn ContentStore + Send + Sync>,
        version: &str,
    ) -> Result<Self, Error> {
        if !source_index.exists() {
            return Err(Error::NotFound);
        }

        let source_index = Self::load(source_index, content_store)?;

        if source_index.index_keys.version != version {
            return Err(Error::VersionMismatch {
                value: source_index.index_keys.version,
                expected: version.to_owned(),
            });
        }

        Ok(source_index)
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.file, &self.index_keys)
            .map_err(|e| Error::Io(e.into()))?;
        Ok(())
    }

    pub(crate) fn source_index_file(buildindex_dir: impl AsRef<Path>) -> PathBuf {
        buildindex_dir.as_ref().join("source.index")
    }

    pub fn current(&self) -> Option<&SourceContent> {
        self.current.as_ref().map(|(_, index)| index)
    }

    #[async_recursion::async_recursion]
    async fn source_pull_tree(
        &self,
        directory: Tree,
        project: &Project,
        version: &str,
        mut uploads: Vec<(String, Vec<u8>)>,
    ) -> Result<(SourceContent, Vec<(String, Vec<u8>)>), Error> {
        let dir_checksum = directory.id();

        if let Some(cached_data) = self
            .index_keys
            .keys
            .get(&dir_checksum)
            .and_then(|entry| self.content_store.read(*entry))
        {
            let source_index = SourceContent::read(&cached_data)?;
            Ok((source_index, uploads))
        } else {
            let (content, uploads) = match directory {
                Tree::Directory { name: _, children } => {
                    let is_leaf = !children
                        .iter()
                        .any(|(_, tree)| matches!(tree, Tree::Directory { .. }));

                    if is_leaf {
                        // calculate SourceContent and return.
                        let mut content = SourceContent::new(version);

                        let resource_paths = children
                            .into_iter()
                            .filter_map(|(_, tree)| match tree {
                                Tree::Directory { .. } => None,
                                Tree::File { name, .. } => Some(PathBuf::from(&name)),
                            })
                            .filter(|path| path.extension().is_none());

                        let resource_list = resource_paths.map(|p| {
                            ResourceId::from_str(p.file_stem().unwrap().to_str().unwrap()).unwrap()
                        });

                        for resource_id in resource_list {
                            let (kind, resource_hash, resource_deps) =
                                project.resource_info(resource_id)?;

                            content.update_resource(
                                ResourcePathId::from(ResourceTypeAndId {
                                    id: resource_id,
                                    kind,
                                }),
                                Some(resource_hash),
                                resource_deps.clone(),
                            );

                            // add each derived dependency with it's direct dependency listed in deps.
                            for dependency in resource_deps {
                                if let Some(direct_dependency) = dependency.direct_dependency() {
                                    content.update_resource(
                                        dependency,
                                        None,
                                        vec![direct_dependency],
                                    );
                                }
                            }
                        }

                        uploads.push((dir_checksum, content.write()?));
                        (content, uploads)
                    } else {
                        let mut content = SourceContent::new(version);
                        for (_, subtree) in children {
                            if matches!(subtree, Tree::Directory { .. }) {
                                let (sub_content, upl) = self
                                    .source_pull_tree(subtree, project, version, uploads)
                                    .await?;
                                uploads = upl;
                                content.extend(Some(sub_content));
                            }
                        }

                        uploads.push((dir_checksum, content.write()?));
                        (content, uploads)
                    }
                }
                _ => panic!(),
            };

            Ok((content, uploads))
        }
    }

    pub async fn source_pull(&mut self, project: &Project, version: &str) -> Result<(), Error> {
        let tree = project.tree().await?;

        let root_checksum = tree.id();

        let (current_checksum, mut source_index) = self
            .current
            .take()
            .unwrap_or(("".to_string(), SourceContent::new(version)));

        if current_checksum != root_checksum {
            let (final_content, uploads) = self
                .source_pull_tree(tree, project, version, vec![])
                .await?;

            for (dir_checksum, buffer) in uploads {
                let checksum = self
                    .content_store
                    .store(&buffer)
                    .ok_or(Error::InvalidContentStore)?;

                self.index_keys.keys.insert(dir_checksum.clone(), checksum);
            }

            source_index = final_content;
        }

        self.current = Some((root_checksum, source_index));
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use lgn_content_store::{ContentStoreAddr, HddContentStore, RamContentStore};
    use lgn_data_offline::{
        resource::{Project, ResourcePathName, ResourceRegistryOptions},
        ResourcePathId,
    };
    use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};

    use crate::source_index::{SourceContent, SourceIndex};

    #[tokio::test]
    async fn version_check() {
        let work_dir = tempfile::tempdir().unwrap();

        let buildindex_dir = work_dir.path();
        {
            let _source_index = SourceIndex::create_new(
                &SourceIndex::source_index_file(&buildindex_dir),
                Box::new(RamContentStore::default()),
                "0.0.1",
            )
            .unwrap();
        }

        assert!(SourceIndex::open(
            &SourceIndex::source_index_file(&buildindex_dir),
            Box::new(RamContentStore::default()),
            "0.0.2"
        )
        .is_err());
    }

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

            let resource_hash = Some(0.into()); // this is irrelevant to the test

            source_index.update_resource(
                intermediate_resource.clone(),
                resource_hash,
                intermediate_deps.clone(),
            );
            source_index.update_resource(source_resource.clone(), resource_hash, vec![]);
            source_index.update_resource(
                intermediate_resource.clone(),
                resource_hash,
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

        let resource_hash = Some(0.into()); // this is irrelevant to the test

        source_index.update_resource(
            intermediate_resource.clone(),
            resource_hash,
            intermediate_deps.clone(),
        );
        assert_eq!(source_index.resources.len(), 1);
        assert_eq!(source_index.resources[0].dependencies.len(), 1);

        source_index.update_resource(source_resource, resource_hash, vec![]);
        assert_eq!(source_index.resources.len(), 2);
        assert_eq!(source_index.resources[1].dependencies.len(), 0);

        source_index.update_resource(intermediate_resource, resource_hash, intermediate_deps);
        assert_eq!(source_index.resources.len(), 2);
        assert_eq!(source_index.resources[0].dependencies.len(), 1);

        source_index.update_resource(output_resources, resource_hash, output_deps);
        assert_eq!(source_index.resources.len(), 3);
        assert_eq!(source_index.resources[2].dependencies.len(), 1);
    }

    fn current_checksum(index: &SourceIndex) -> String {
        index.current.as_ref().map(|(id, _)| id.clone()).unwrap()
    }

    #[tokio::test]
    async fn source_index_cache() {
        let work_dir = tempfile::tempdir().unwrap();

        let mut project = Project::create_new(&work_dir.path())
            .await
            .expect("failed to create a project");

        let temp_dir = work_dir.path().join("temp");
        std::fs::create_dir(&temp_dir).expect("new directory");

        let content_store =
            HddContentStore::open(ContentStoreAddr::from(temp_dir.clone())).unwrap();

        let version = "0.0.1";

        let mut source_index = SourceIndex::create_new(
            &SourceIndex::source_index_file(&temp_dir),
            Box::new(content_store),
            version,
        )
        .unwrap();

        // repeated source_pull fetches results from cache
        let first_entry_checksum = {
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1);

            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1);
            current_checksum(&source_index)
        };

        let resource_registry = ResourceRegistryOptions::new()
            .add_type::<refs_resource::TestResource>()
            .create_async_registry();
        let mut resources = resource_registry.lock().await;

        let (resource_id, resource_handle) = {
            let resource_handle = resources
                .new_resource(refs_resource::TestResource::TYPE)
                .expect("new resource")
                .typed::<refs_resource::TestResource>();

            resource_handle.get_mut(&mut resources).unwrap().content = "hello".to_string();

            let id = ResourceTypeAndId {
                kind: refs_resource::TestResource::TYPE,
                id: ResourceId::new_explicit(0xaabbccddeeff0011),
            };
            let resource_id = project
                .add_resource_with_id(
                    ResourcePathName::new("test_source"),
                    refs_resource::TestResource::TYPENAME,
                    refs_resource::TestResource::TYPE,
                    id,
                    &resource_handle,
                    &mut resources,
                )
                .await
                .expect("adding the resource");
            (resource_id, resource_handle)
        };

        // initially we have 0 subfolders
        //number of indeces: 1

        // one resource creates 3-levels deep folder hierarchy.
        // including the root index refresh that creates 4 new cached entries.
        // number of indices: 1 + 4

        // new resource creates a new cached entry
        let second_entry_checksum = {
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1 + 4);
            current_checksum(&source_index)
        };

        // committing changes does not create a new entry
        {
            project.commit().expect("sucessful commit");
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1 + 4);
            assert_eq!(current_checksum(&source_index), second_entry_checksum);
        }

        // modifying a resource changes the whole hierarchy.
        // number of indices: 1 + 4 + 4

        // modify a resource
        let third_checksum = {
            let res = resource_handle
                .get_mut(&mut resources)
                .expect("loaded resource");
            res.content = "hello world!".to_string();

            project
                .save_resource(resource_id, resource_handle, &mut resources)
                .await
                .expect("successful save");

            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1 + 4 + 4);
            current_checksum(&source_index)
        };

        // committing changes does not create a new entry
        {
            project.commit().expect("sucessful commit");
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1 + 4 + 4);
            assert_eq!(current_checksum(&source_index), third_checksum);
        }

        // deleting the resource takes us back to the previous cache entry.
        {
            project
                .delete_resource(resource_id.id)
                .await
                .expect("removed resource");
            source_index.source_pull(&project, version).await.unwrap();
            assert_eq!(source_index.index_keys.keys.len(), 1 + 4 + 4);
            assert_eq!(current_checksum(&source_index), first_entry_checksum);
        }
        source_index.flush().unwrap();
    }
}
