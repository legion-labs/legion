use legion_data_compiler::CompiledResource;
use legion_data_runtime::AssetChecksum;
use petgraph::{Directed, Graph};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cmp::Ordering,
    collections::{hash_map::DefaultHasher, HashMap, VecDeque},
    convert::TryInto,
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::Seek,
    path::{Path, PathBuf},
};

use crate::Error;
use legion_data_offline::{
    asset::AssetPathId,
    resource::{Project, ResourceHash},
};

#[derive(Serialize, Deserialize, Debug)]
struct ResourceInfo {
    id: AssetPathId,
    dependencies: Vec<AssetPathId>,
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
    /// The path the resource was compiled from, i.e.: "AssetPathId("anim.fbx").push("anim.offline")
    pub(crate) compile_path: AssetPathId,
    pub(crate) context_hash: AssetHash,
    pub(crate) source_hash: AssetHash,
    /// The path the resource was compiled into, i.e.: "AssetPathId("anim.fbx").push("anim.offline")["idle"]
    pub(crate) compiled_path: AssetPathId,
    pub(crate) compiled_checksum: AssetChecksum,
    pub(crate) compiled_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledResourceReference {
    pub(crate) compile_path: AssetPathId,
    pub(crate) context_hash: AssetHash,
    pub(crate) source_hash: AssetHash,
    pub(crate) compiled_path: AssetPathId,
    pub(crate) compiled_reference: AssetPathId,
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

#[derive(Serialize, Deserialize, Debug)]
struct BuildIndexContent {
    version: String,
    project_index: PathBuf,
    resources: Vec<ResourceInfo>,
    compiled_resources: Vec<CompiledResourceInfo>,
    compiled_resource_references: Vec<CompiledResourceReference>,
}

impl BuildIndexContent {
    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.resources.sort_by(|a, b| a.id.cmp(&b.id));
        for resource in &mut self.resources {
            resource.pre_serialize();
        }
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
                    result = a.compiled_reference.cmp(&b.compiled_path);
                }
            }
            result
        });
    }
}

#[derive(Debug)]
pub(crate) struct BuildIndex {
    content: BuildIndexContent,
    file: File,
}

impl BuildIndex {
    pub(crate) fn create_new(
        buildindex_path: &Path,
        projectindex_path: &Path,
        version: &str,
    ) -> Result<Self, Error> {
        if !projectindex_path.exists() {
            return Err(Error::InvalidProject);
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(buildindex_path)
            .map_err(|_e| Error::IOError)?;

        let content = BuildIndexContent {
            version: String::from(version),
            project_index: projectindex_path.to_owned(),
            resources: vec![],
            compiled_resources: vec![],
            compiled_resource_references: vec![],
        };

        serde_json::to_writer(&file, &content).map_err(|_e| Error::IOError)?;

        Ok(Self { content, file })
    }

    pub(crate) fn open(buildindex_path: &Path, version: &str) -> Result<Self, Error> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(buildindex_path)
            .map_err(|_e| Error::NotFound)?;

        let content: BuildIndexContent =
            serde_json::from_reader(&file).map_err(|_e| Error::IOError)?;

        if !content.project_index.exists() {
            return Err(Error::InvalidProject);
        }

        if content.version != version {
            return Err(Error::VersionMismatch);
        }

        Ok(Self { content, file })
    }

    pub(crate) fn open_project(&self) -> Result<Project, Error> {
        if !self.validate_project_index() {
            return Err(Error::InvalidProject);
        }
        Project::open(&self.content.project_index).map_err(|_e| Error::InvalidProject)
    }

    pub(crate) fn validate_project_index(&self) -> bool {
        self.content.project_index.exists()
    }

    /// Create an ordered build graph with edges directed towards `compile_path`.
    pub(crate) fn generate_build_graph(&self, compile_path: AssetPathId) -> Graph<AssetPathId, ()> {
        let mut dep_graph = Graph::<AssetPathId, (), Directed>::new();
        let mut indices = HashMap::<AssetPathId, petgraph::prelude::NodeIndex>::new();
        let mut processed = vec![];
        let mut queue = VecDeque::<AssetPathId>::new();

        // we process the whole path as derived resources might not exist in
        // the build index as those are never refered to as dependencies.
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

                let unprocessed: VecDeque<AssetPathId> = deps
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

    /// Returns a combined hash of:
    /// * `id` resource's content.
    /// * content of all `id`'s dependencies.
    /// todo: at one point dependency filtering here will be useful.
    pub(crate) fn compute_source_hash(&self, id: AssetPathId) -> Result<ResourceHash, Error> {
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
        Ok(hasher.finish().into())
    }

    pub(crate) fn update_resource(
        &mut self,
        id: AssetPathId,
        resource_hash: Option<ResourceHash>,
        mut deps: Vec<AssetPathId>,
    ) -> bool {
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

    pub(crate) fn find_dependencies(&self, id: &AssetPathId) -> Option<Vec<AssetPathId>> {
        self.content
            .resources
            .iter()
            .find(|r| &r.id == id)
            .map(|resource| resource.dependencies.clone())
    }

    pub(crate) fn insert_compiled(
        &mut self,
        compile_path: &AssetPathId,
        context_hash: u64,
        source_hash: u64,
        compiled_resources: &[CompiledResource],
        compiled_references: &[(AssetPathId, AssetPathId)],
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
        compile_path: &AssetPathId,
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

    fn pre_serialize(&mut self) {
        self.content.pre_serialize();
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.pre_serialize();
        serde_json::to_writer_pretty(&self.file, &self.content).map_err(|_e| Error::IOError)
    }
}

#[cfg(test)]
mod tests {

    use super::BuildIndex;
    use legion_data_offline::{
        asset::AssetPathId,
        resource::{Project, ResourceId},
    };

    pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

    #[test]
    fn version_check() {
        let work_dir = tempfile::tempdir().unwrap();

        let project = Project::create_new(work_dir.path()).expect("failed to create project");
        let projectindex_path = project.indexfile_path();

        let buildindex_path = work_dir.path().join(TEST_BUILDINDEX_FILENAME);
        {
            let _buildindex =
                BuildIndex::create_new(&buildindex_path, &projectindex_path, "0.0.1").unwrap();
        }

        assert!(BuildIndex::open(&buildindex_path, "0.0.2").is_err());
    }

    #[test]
    fn dependency_update() {
        let work_dir = tempfile::tempdir().unwrap();
        let project = Project::create_new(work_dir.path()).expect("failed to create project");

        // dummy ids - the actual project structure is irrelevant in this test.
        let source_id = ResourceId::generate_new(refs_resource::TYPE_ID);
        let source_resource = AssetPathId::from(source_id);
        let intermediate_resource = source_resource.push(refs_resource::TYPE_ID);
        let output_resources = intermediate_resource.push(refs_resource::TYPE_ID);

        let buildindex_path = work_dir.path().join(TEST_BUILDINDEX_FILENAME);
        let projectindex_path = project.indexfile_path();

        let mut db = BuildIndex::create_new(&buildindex_path, &projectindex_path, "0.0.1").unwrap();

        // all dependencies need to be explicitly specified
        let intermediate_deps = vec![source_resource.clone()];
        let output_deps = vec![intermediate_resource.clone()];

        let resource_hash = Some(0.into()); // this is irrelevant to the test

        db.update_resource(
            intermediate_resource.clone(),
            resource_hash,
            intermediate_deps.clone(),
        );
        assert_eq!(db.content.resources.len(), 1);
        assert_eq!(db.content.resources[0].dependencies.len(), 1);

        db.update_resource(source_resource, resource_hash, vec![]);
        assert_eq!(db.content.resources.len(), 2);
        assert_eq!(db.content.resources[1].dependencies.len(), 0);

        db.update_resource(intermediate_resource, resource_hash, intermediate_deps);
        assert_eq!(db.content.resources.len(), 2);
        assert_eq!(db.content.resources[0].dependencies.len(), 1);

        db.update_resource(output_resources, resource_hash, output_deps);
        assert_eq!(db.content.resources.len(), 3);
        assert_eq!(db.content.resources[2].dependencies.len(), 1);

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
