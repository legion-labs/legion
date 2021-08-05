use legion_assets::AssetId;
use legion_data_compiler::CompiledAsset;
use petgraph::{algo, Directed, Graph};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap, VecDeque},
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io::Seek,
    path::{Path, PathBuf},
};

use crate::Error;
use legion_resources::{Project, ResourceHash, ResourceId};

#[derive(Serialize, Deserialize, Debug)]
struct ResourceInfo {
    id: ResourceId,
    build_deps: Vec<ResourceId>,
    resource_hash: ResourceHash, // hash of this asset
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledAssetInfo {
    pub(crate) context_hash: u64,
    pub(crate) source_guid: ResourceId,
    pub(crate) source_hash: u64,
    pub(crate) compiled_guid: AssetId,
    pub(crate) compiled_checksum: i128,
    pub(crate) compiled_size: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledAssetReference {
    context_hash: u64,
    source_guid: ResourceId,
    source_hash: u64,
    pub(crate) compiled_guid: AssetId,
    pub(crate) compiled_reference: AssetId,
}

#[derive(Serialize, Deserialize, Debug)]
struct BuildIndexContent {
    version: String,
    project_index: PathBuf,
    resources: Vec<ResourceInfo>,
    compiled_assets: Vec<CompiledAssetInfo>,
    compiled_asset_references: Vec<CompiledAssetReference>,
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
            compiled_assets: vec![],
            compiled_asset_references: vec![],
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

    /// Returns ordered list of dependencies starting from leaf-dependencies ending with `resource_id` - the root.
    pub(crate) fn evaluation_order(
        &self,
        resource_id: ResourceId,
    ) -> Result<Vec<ResourceId>, Error> {
        let mut dep_graph = Graph::<ResourceId, (), Directed>::new();

        let mut indices = HashMap::<ResourceId, petgraph::prelude::NodeIndex>::new();
        let mut processed = vec![];
        let mut queue = VecDeque::<ResourceId>::new();

        queue.push_back(resource_id);

        let mut get_or_create_index = |res, dep_graph: &mut Graph<ResourceId, ()>| {
            if let Some(own_index) = indices.get(&res) {
                *own_index
            } else {
                let own_index = dep_graph.add_node(res);
                indices.insert(res, own_index);
                own_index
            }
        };

        while let Some(res) = queue.pop_front() {
            processed.push(res);

            let own_index = get_or_create_index(res, &mut dep_graph);

            let deps = self.find_dependencies(res).ok_or(Error::IntegrityFailure)?;

            for d in &deps {
                let other_index = get_or_create_index(*d, &mut dep_graph);
                dep_graph.add_edge(own_index, other_index, ());
            }

            let unprocessed: VecDeque<ResourceId> = deps
                .into_iter()
                .filter(|r| !processed.contains(r))
                .collect();
            queue.extend(unprocessed);
        }

        let topological_order =
            algo::toposort(&dep_graph, None).map_err(|_e| Error::IntegrityFailure)?;

        let evaluation_order = topological_order
            .iter()
            .map(|i| *dep_graph.node_weight(*i).unwrap())
            .rev()
            .collect();
        Ok(evaluation_order)
    }

    /// Returns a combined hash of:
    /// * `id` resource's content.
    /// * content of all `id`'s dependencies.
    /// todo: at one point dependency filtering here will be useful.
    pub(crate) fn compute_source_hash(&self, id: ResourceId) -> Result<ResourceHash, Error> {
        let sorted_unique_resource_hashes: Vec<ResourceHash> = {
            let mut unique_resources = HashMap::new();
            let mut queue: VecDeque<_> = VecDeque::new();
            queue.push_back(id);

            while let Some(resource) = queue.pop_front() {
                let resource_info = self
                    .content
                    .resources
                    .iter()
                    .find(|r| r.id == resource)
                    .ok_or(Error::NotFound)?;

                unique_resources.insert(resource, resource_info.resource_hash);

                let newly_discovered_deps: Vec<_> = resource_info
                    .build_deps
                    .iter()
                    .filter(|r| !unique_resources.contains_key(*r))
                    .collect();

                queue.extend(newly_discovered_deps);
            }

            let mut hashes: Vec<ResourceHash> = unique_resources.into_iter().map(|t| t.1).collect();
            hashes.sort_unstable();
            hashes
        };

        let mut hasher = DefaultHasher::new();
        for h in sorted_unique_resource_hashes {
            h.hash(&mut hasher);
        }
        Ok(hasher.finish())
    }

    pub(crate) fn update_resource(
        &mut self,
        id: ResourceId,
        resource_hash: ResourceHash,
        mut deps: Vec<ResourceId>,
    ) -> bool {
        if let Some(existing_res) = self.content.resources.iter_mut().find(|r| r.id == id) {
            deps.sort();

            let matching = existing_res
                .build_deps
                .iter()
                .zip(deps.iter())
                .filter(|&(a, b)| a == b)
                .count();
            if deps.len() == matching && existing_res.resource_hash == resource_hash {
                false
            } else {
                existing_res.build_deps = deps;
                existing_res.resource_hash = resource_hash;
                true
            }
        } else {
            let info = ResourceInfo {
                id,
                build_deps: deps,
                resource_hash,
            };
            self.content.resources.push(info);
            true
        }
    }

    pub(crate) fn find_dependencies(&self, id: ResourceId) -> Option<Vec<ResourceId>> {
        self.content
            .resources
            .iter()
            .find(|r| r.id == id)
            .map(|resource| resource.build_deps.clone())
    }

    pub(crate) fn insert_compiled(
        &mut self,
        context_hash: u64,
        source_guid: ResourceId,
        source_hash: u64,
        compiled_assets: &[CompiledAsset],
        compiled_references: &[(AssetId, AssetId)],
    ) {
        let mut compiled_assets_desc: Vec<_> = compiled_assets
            .iter()
            .map(|asset| CompiledAssetInfo {
                context_hash,
                source_guid,
                source_hash,
                compiled_guid: asset.guid,
                compiled_checksum: asset.checksum,
                compiled_size: asset.size,
            })
            .collect();

        let mut compiled_references_desc: Vec<_> = compiled_references
            .iter()
            .map(
                |&(compiled_guid, compiled_reference)| CompiledAssetReference {
                    context_hash,
                    source_guid,
                    source_hash,
                    compiled_guid,
                    compiled_reference,
                },
            )
            .collect();

        // For now we assume there is not concurrent compilation
        // so there is no way to compile the same resources twice.
        // Once we support it we will have to make sure the result of the compilation
        // is exactly the same for all compiled_assets.
        assert_eq!(self.find_compiled(context_hash, source_hash).len(), 0);

        self.content
            .compiled_assets
            .append(&mut compiled_assets_desc);

        self.content
            .compiled_asset_references
            .append(&mut compiled_references_desc);
    }

    pub(crate) fn find_compiled(
        &self,
        context_hash: u64,
        source_hash: u64,
    ) -> Vec<CompiledAssetInfo> {
        self.content
            .compiled_assets
            .iter()
            .filter(|asset| asset.context_hash == context_hash && asset.source_hash == source_hash)
            .cloned()
            .collect::<Vec<CompiledAssetInfo>>()
    }

    pub(crate) fn flush(&mut self) -> Result<(), Error> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.file, &self.content).map_err(|_e| Error::IOError)
    }
}

#[cfg(test)]
mod tests {

    use super::BuildIndex;
    use legion_resources::{Project, ResourceId, ResourceType};

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

        const RESOURCE_ACTOR: ResourceType = ResourceType::new(b"actor");

        // dummy ids - the actual project structure is irrelevant in this test.
        let child = ResourceId::generate_new(RESOURCE_ACTOR);
        let parent = ResourceId::generate_new(RESOURCE_ACTOR);

        let buildindex_path = work_dir.path().join(TEST_BUILDINDEX_FILENAME);
        let projectindex_path = project.indexfile_path();

        let mut db = BuildIndex::create_new(&buildindex_path, &projectindex_path, "0.0.1").unwrap();

        let parent_deps = vec![child];

        let resource_hash = 0; // this is irrelevant to the test

        db.update_resource(parent, resource_hash, parent_deps.clone());
        assert_eq!(db.content.resources.len(), 1);
        assert_eq!(db.content.resources[0].build_deps.len(), 1);

        db.update_resource(child, resource_hash, vec![]);
        assert_eq!(db.content.resources.len(), 2);
        assert_eq!(db.content.resources[1].build_deps.len(), 0);

        db.update_resource(parent, resource_hash, parent_deps);
        assert_eq!(db.content.resources.len(), 2);
        assert_eq!(db.content.resources[0].build_deps.len(), 1);

        db.flush().unwrap();
    }
}
