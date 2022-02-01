use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, VecDeque},
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::Seek,
    path::{Path, PathBuf},
};

use lgn_content_store::Checksum;
use lgn_data_compiler::CompiledResource;
use lgn_data_offline::{resource::ResourceHash, ResourcePathId};
use lgn_data_runtime::ResourceTypeAndId;
use lgn_utils::DefaultHasher;
use petgraph::{Directed, Graph};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledResourceInfo {
    /// The path the resource was compiled from, i.e.:
    /// "ResourcePathId("anim.fbx").push("anim.offline")
    pub(crate) compile_path: ResourcePathId,
    pub(crate) context_hash: AssetHash,
    pub(crate) source_hash: AssetHash,
    /// The path the resource was compiled into, i.e.:
    /// "ResourcePathId("anim.fbx").push("anim.offline")["idle"]
    pub(crate) compiled_path: ResourcePathId,
    pub(crate) compiled_checksum: Checksum,
    pub(crate) compiled_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledResourceReference {
    pub(crate) compile_path: ResourcePathId,
    pub(crate) context_hash: AssetHash,
    pub(crate) source_hash: AssetHash,
    pub(crate) compiled_path: ResourcePathId,
    pub(crate) compiled_reference: ResourcePathId,
}

impl CompiledResourceReference {
    pub fn is_same_context(&self, resource_info: &CompiledResourceInfo) -> bool {
        self.context_hash == resource_info.context_hash
            && self.source_hash == resource_info.source_hash
    }

    pub fn is_from_same_source(&self, resource_info: &CompiledResourceInfo) -> bool {
        self.is_same_context(resource_info) && self.compile_path == resource_info.compile_path
    }

    pub fn is_reference_of(&self, resource_info: &CompiledResourceInfo) -> bool {
        self.is_from_same_source(resource_info) && self.compiled_path == resource_info.compiled_path
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct SourceContent {
    version: String,
    resources: Vec<ResourceInfo>,
    #[serde_as(as = "Vec<(DisplayFromStr, _)>")]
    pathid_mapping: BTreeMap<ResourceTypeAndId, ResourcePathId>,
}

impl SourceContent {
    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.resources.sort_by(|a, b| a.id.cmp(&b.id));
        for resource in &mut self.resources {
            resource.pre_serialize();
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
struct OutputContent {
    version: String,
    compiled_resources: Vec<CompiledResourceInfo>,
    compiled_resource_references: Vec<CompiledResourceReference>,
}

impl OutputContent {
    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.compiled_resources.sort_by(|a, b| {
            let mut result = a.compile_path.cmp(&b.compile_path);
            if result == Ordering::Equal {
                result = a.compiled_path.cmp(&b.compiled_path);
            }
            result
        });
        self.compiled_resource_references.sort_by(|a, b| {
            let mut result = a.compile_path.cmp(&b.compile_path);
            if result == Ordering::Equal {
                result = a.compiled_path.cmp(&b.compiled_path);
                if result == Ordering::Equal {
                    result = a.compiled_reference.cmp(&b.compiled_reference);
                }
            }
            result
        });
    }
}

#[derive(Debug)]
pub(crate) struct SourceIndex {
    content: SourceContent,
    file: File,
}

impl SourceIndex {
    pub fn record_pathid(&mut self, id: &ResourcePathId) {
        self.content
            .pathid_mapping
            .insert(id.resource_id(), id.clone());
    }

    pub fn lookup_pathid(&self, id: ResourceTypeAndId) -> Option<ResourcePathId> {
        self.content.pathid_mapping.get(&id).cloned()
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
                if let Some(resource_info) =
                    self.content.resources.iter().find(|r| r.id == resource)
                {
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
        if let Some(existing_res) = self.content.resources.iter_mut().find(|r| r.id == id) {
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
            self.content.resources.push(info);
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
        self.content
            .resources
            .iter()
            .find(|r| &r.id == id)
            .map(|resource| resource.dependencies.clone())
    }

    pub(crate) fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let source_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|_e| Error::NotFound)?;

        let source_content: SourceContent =
            serde_json::from_reader(&source_file).map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            content: source_content,
            file: source_file,
        })
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.file, &self.content).map_err(|e| Error::Io(e.into()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct OutputIndex {
    content: OutputContent,
    file: File,
}

impl OutputIndex {
    pub(crate) fn insert_compiled(
        &mut self,
        compile_path: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
        compiled_resources: &[CompiledResource],
        compiled_references: &[(ResourcePathId, ResourcePathId)],
    ) {
        // For now we assume there is not concurrent compilation
        // so there is no way to compile the same resources twice.
        // Once we support it we will have to make sure the result of the compilation
        // is exactly the same for all compiled_assets.
        assert!(self
            .find_compiled(compile_path, context_hash, source_hash)
            .is_none());

        let mut compiled_assets_desc: Vec<_> = compiled_resources
            .iter()
            .map(|asset| CompiledResourceInfo {
                compile_path: compile_path.clone(),
                context_hash: context_hash.into(),
                source_hash: source_hash.into(),
                compiled_path: asset.path.clone(),
                compiled_checksum: asset.checksum,
                compiled_size: asset.size,
            })
            .collect();

        let mut compiled_references_desc: Vec<_> = compiled_references
            .iter()
            .map(
                |(compiled_guid, compiled_reference)| CompiledResourceReference {
                    context_hash: context_hash.into(),
                    compile_path: compile_path.clone(),
                    source_hash: source_hash.into(),
                    compiled_path: compiled_guid.clone(),
                    compiled_reference: compiled_reference.clone(),
                },
            )
            .collect();

        self.content
            .compiled_resources
            .append(&mut compiled_assets_desc);

        self.content
            .compiled_resource_references
            .append(&mut compiled_references_desc);
    }

    pub(crate) fn find_compiled(
        &self,
        compile_path: &ResourcePathId,
        context_hash: u64,
        source_hash: u64,
    ) -> Option<(Vec<CompiledResourceInfo>, Vec<CompiledResourceReference>)> {
        let asset_objects: Vec<CompiledResourceInfo> = self
            .content
            .compiled_resources
            .iter()
            .filter(|asset| {
                &asset.compile_path == compile_path
                    && asset.context_hash.get() == context_hash
                    && asset.source_hash.get() == source_hash
            })
            .cloned()
            .collect();

        if asset_objects.is_empty() {
            None
        } else {
            let asset_references: Vec<CompiledResourceReference> = self
                .content
                .compiled_resource_references
                .iter()
                .filter(|reference| {
                    &reference.compile_path == compile_path
                        && reference.context_hash.get() == context_hash
                        && reference.source_hash.get() == source_hash
                })
                .cloned()
                .collect();

            Some((asset_objects, asset_references))
        }
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.file, &self.content).map_err(|e| Error::Io(e.into()))?;
        Ok(())
    }

    pub(crate) fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())
            .map_err(|_e| Error::NotFound)?;

        let output_content: OutputContent =
            serde_json::from_reader(&output_file).map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            content: output_content,
            file: output_file,
        })
    }
}

#[derive(Debug)]
pub(crate) struct BuildIndex {
    pub(crate) source_index: SourceIndex,
    pub(crate) output_index: OutputIndex,
}

impl BuildIndex {
    fn source_index_file(buildindex_dir: impl AsRef<Path>) -> PathBuf {
        buildindex_dir.as_ref().join("source.index")
    }

    fn output_index_file(buildindex_dir: impl AsRef<Path>) -> PathBuf {
        buildindex_dir.as_ref().join("output.index")
    }

    pub(crate) fn create_new(buildindex_dir: &Path, version: &str) -> Result<Self, Error> {
        let source_index = Self::source_index_file(buildindex_dir);
        let output_index = Self::output_index_file(buildindex_dir);

        let source_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(source_index)
            .map_err(|e| Error::Io(e.into()))?;

        let output_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(output_index)
            .map_err(|e| Error::Io(e.into()))?;

        let source_content = SourceContent {
            version: String::from(version),
            resources: vec![],
            pathid_mapping: BTreeMap::new(),
        };

        let output_content = OutputContent {
            version: String::from(version),
            compiled_resources: vec![],
            compiled_resource_references: vec![],
        };

        // todo: write the output file

        serde_json::to_writer_pretty(&source_file, &source_content)
            .map_err(|e| Error::Io(e.into()))?;
        serde_json::to_writer_pretty(&output_file, &output_content)
            .map_err(|e| Error::Io(e.into()))?;

        Ok(Self {
            source_index: SourceIndex {
                content: source_content,
                file: source_file,
            },
            output_index: OutputIndex {
                content: output_content,
                file: output_file,
            },
        })
    }

    pub(crate) fn open(buildindex_dir: &Path, version: &str) -> Result<Self, Error> {
        let source_index = Self::source_index_file(buildindex_dir);
        let output_index = Self::output_index_file(buildindex_dir);

        if !source_index.exists() || !output_index.exists() {
            return Err(Error::NotFound);
        }

        let source_index = SourceIndex::load(source_index)?;
        let output_index = OutputIndex::load(output_index)?;

        if source_index.content.version != version {
            return Err(Error::VersionMismatch {
                value: source_index.content.version,
                expected: version.to_owned(),
            });
        }

        if output_index.content.version != version {
            return Err(Error::VersionMismatch {
                value: output_index.content.version,
                expected: version.to_owned(),
            });
        }

        Ok(Self {
            source_index,
            output_index,
        })
    }

    fn pre_serialize(&mut self) {
        self.source_index.content.pre_serialize();
        self.output_index.content.pre_serialize();
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.pre_serialize();

        self.source_index.flush()?;
        self.output_index.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use lgn_data_offline::ResourcePathId;
    use lgn_data_runtime::{Resource, ResourceId, ResourceTypeAndId};

    use super::BuildIndex;

    #[tokio::test]
    async fn version_check() {
        let work_dir = tempfile::tempdir().unwrap();

        let buildindex_dir = work_dir.path();
        {
            let _buildindex = BuildIndex::create_new(buildindex_dir, "0.0.1").unwrap();
        }

        assert!(BuildIndex::open(buildindex_dir, "0.0.2").is_err());
    }

    #[tokio::test]
    async fn pathid_records() {
        let work_dir = tempfile::tempdir().unwrap();

        // dummy ids - the actual project structure is irrelevant in this test.
        let source_id = ResourceTypeAndId {
            kind: refs_resource::TestResource::TYPE,
            id: ResourceId::new(),
        };
        let source_resource = ResourcePathId::from(source_id);
        let intermediate_resource = source_resource.push(refs_resource::TestResource::TYPE);
        let output_resource = intermediate_resource.push(refs_resource::TestResource::TYPE);

        let buildindex_dir = work_dir.path();

        {
            let mut db = BuildIndex::create_new(buildindex_dir, "0.0.1").unwrap();

            // all dependencies need to be explicitly specified
            let intermediate_deps = vec![source_resource.clone()];
            let output_deps = vec![intermediate_resource.clone()];

            let resource_hash = Some(0.into()); // this is irrelevant to the test

            db.source_index.update_resource(
                intermediate_resource.clone(),
                resource_hash,
                intermediate_deps.clone(),
            );
            db.source_index
                .update_resource(source_resource.clone(), resource_hash, vec![]);
            db.source_index.update_resource(
                intermediate_resource.clone(),
                resource_hash,
                intermediate_deps,
            );
            db.source_index
                .update_resource(output_resource.clone(), resource_hash, output_deps);

            db.flush().unwrap();
        }

        let db = BuildIndex::open(buildindex_dir, "0.0.1").unwrap();
        assert_eq!(
            db.source_index.lookup_pathid(source_id).unwrap(),
            source_resource
        );
        assert_eq!(
            db.source_index
                .lookup_pathid(intermediate_resource.resource_id())
                .unwrap(),
            intermediate_resource
        );
        assert_eq!(
            db.source_index
                .lookup_pathid(output_resource.resource_id())
                .unwrap(),
            output_resource
        );
    }

    #[tokio::test]
    async fn dependency_update() {
        let work_dir = tempfile::tempdir().unwrap();

        // dummy ids - the actual project structure is irrelevant in this test.
        let source_id = ResourceTypeAndId {
            kind: refs_resource::TestResource::TYPE,
            id: ResourceId::new(),
        };
        let source_resource = ResourcePathId::from(source_id);
        let intermediate_resource = source_resource.push(refs_resource::TestResource::TYPE);
        let output_resources = intermediate_resource.push(refs_resource::TestResource::TYPE);

        let buildindex_dir = work_dir.path();

        let mut db = BuildIndex::create_new(buildindex_dir, "0.0.1").unwrap();

        // all dependencies need to be explicitly specified
        let intermediate_deps = vec![source_resource.clone()];
        let output_deps = vec![intermediate_resource.clone()];

        let resource_hash = Some(0.into()); // this is irrelevant to the test

        db.source_index.update_resource(
            intermediate_resource.clone(),
            resource_hash,
            intermediate_deps.clone(),
        );
        assert_eq!(db.source_index.content.resources.len(), 1);
        assert_eq!(db.source_index.content.resources[0].dependencies.len(), 1);

        db.source_index
            .update_resource(source_resource, resource_hash, vec![]);
        assert_eq!(db.source_index.content.resources.len(), 2);
        assert_eq!(db.source_index.content.resources[1].dependencies.len(), 0);

        db.source_index
            .update_resource(intermediate_resource, resource_hash, intermediate_deps);
        assert_eq!(db.source_index.content.resources.len(), 2);
        assert_eq!(db.source_index.content.resources[0].dependencies.len(), 1);

        db.source_index
            .update_resource(output_resources, resource_hash, output_deps);
        assert_eq!(db.source_index.content.resources.len(), 3);
        assert_eq!(db.source_index.content.resources[2].dependencies.len(), 1);

        db.flush().unwrap();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssetHash(u64);

impl AssetHash {
    pub(crate) fn get(&self) -> u64 {
        self.0
    }
}

impl From<u64> for AssetHash {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl Serialize for AssetHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let bytes = self.0.to_be_bytes();
            let hex = hex::encode(bytes);
            serializer.serialize_str(&hex)
        } else {
            serializer.serialize_u64(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for AssetHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = {
            if deserializer.is_human_readable() {
                let hex = String::deserialize(deserializer)?;
                let digits = hex::decode(hex).map_err(D::Error::custom)?;
                u64::from_be_bytes(digits.try_into().unwrap())
            } else {
                u64::deserialize(deserializer)?
            }
        };
        Ok(value.into())
    }
}
