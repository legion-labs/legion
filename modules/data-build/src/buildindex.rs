use legion_assets::AssetId;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::Seek,
    path::{Path, PathBuf},
};

use crate::{CompiledAsset, Error};
use legion_resources::{Project, ResourceHash, ResourceId};

#[derive(Serialize, Deserialize, Debug)]
struct ResourceInfo {
    id: ResourceId,
    build_deps: Vec<ResourceId>,
    resource_hash: ResourceHash, // hash of this asset
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CompiledAssetInfo {
    context_hash: u64,
    source_guid: ResourceId,
    source_hash: u64,
    pub(crate) compiled_guid: AssetId,
    pub(crate) compiled_checksum: i128,
    pub(crate) compiled_size: usize,
}

#[derive(Serialize, Deserialize, Debug)]
struct BuildIndexContent {
    version: String,
    project_index: PathBuf,
    resources: Vec<ResourceInfo>, // resource_references
    compiled_assets: Vec<CompiledAssetInfo>,
    // todo(kstasik): compiled_asset_references
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

    pub(crate) fn compute_source_hash(&self, id: ResourceId) -> Result<ResourceHash, Error> {
        let resource = self
            .content
            .resources
            .iter()
            .find(|r| r.id == id)
            .ok_or(Error::NotFound)?;

        // TODO: this should include hashes of (filtered) dependencies
        Ok(resource.resource_hash)
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

    // todo: remove this api? rename?
    pub(crate) fn find(&self, id: ResourceId) -> Option<(ResourceId, &Vec<ResourceId>)> {
        self.content
            .resources
            .iter()
            .find(|r| r.id == id)
            .map(|resource| (resource.id, &resource.build_deps))
    }

    pub(crate) fn insert_compiled(
        &mut self,
        context_hash: u64,
        source_guid: ResourceId,
        source_hash: u64,
        compiled_assets: &[CompiledAsset],
    ) {
        let mut compiled_desc = compiled_assets
            .iter()
            .map(|asset| CompiledAssetInfo {
                context_hash,
                source_guid,
                source_hash,
                compiled_guid: asset.guid,
                compiled_checksum: asset.checksum,
                compiled_size: asset.size,
            })
            .collect::<Vec<CompiledAssetInfo>>();

        // For now we assume there is not concurrent compilation
        // so there is no way to compile the same resources twice.
        // Once we support it we will have to make sure the result of the compilation
        // is exactly the same for all compiled_assets.
        assert_eq!(self.find_compiled(context_hash, source_hash).len(), 0);

        self.content.compiled_assets.append(&mut compiled_desc);
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
    use crate::Error;
    use legion_resources::{Project, ResourcePath, ResourceType};
    use std::slice;

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

        assert_eq!(
            BuildIndex::open(&buildindex_path, "0.0.2").unwrap_err(),
            Error::VersionMismatch
        );
    }

    #[test]
    fn dependency_update() {
        let work_dir = tempfile::tempdir().unwrap();
        let mut project = Project::create_new(work_dir.path()).expect("failed to create project");

        let child = project
            .create_resource(ResourcePath::from("child"), ResourceType::Actor, b"test")
            .unwrap();
        let parent = project
            .create_resource_with_deps(
                ResourcePath::from("parent"),
                ResourceType::Actor,
                slice::from_ref(&child),
                b"test",
            )
            .unwrap();

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

/*

- asset's definitions (offline and runtime formats)
- adding factories (offline and runtime)
- editor appearance
- asset-spcific management
- pluginizing it???
- own editor, import, compiler, etc


class MyResource : OfflineResource {
    string text_content;
}

class MyAsset : RuntimeAsset {
    int integer_value;
}

- content browser uses OfflineResource
- content browser defaults to property grid
- 1 offline resource can generate many runtime resoureces

- create and register MyResourceFactory.


*/
