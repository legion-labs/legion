use std::{
    collections::HashMap,
    fmt,
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_recursion::async_recursion;
use lgn_content_store::{
    indexing::{IndexKey, ResourceIdentifier, SharedTreeIdentifier, TreeIdentifier},
    Provider,
};
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, AssetRegistry, AssetRegistryError, HandleUntyped, ResourceId,
    ResourcePathId, ResourceType, ResourceTypeAndId, ResourceTypeAndIdIndexer,
};
use lgn_source_control::{
    CommitMode, LocalRepositoryIndex, RepositoryIndex, RepositoryName, Workspace,
};
use lgn_tracing::error;
use thiserror::Error;

use crate::resource::{metadata::Metadata, ResourcePathName};

use super::deserialize_and_skip_metadata;

/// A source-control-backed state of the project
///
/// This structure captures the state of the project. This includes `remote
/// resources` pulled from `source-control` as well as `local resources`
/// added/removed/edited locally.
///
/// It provides a resource-oriented interface to source-control.
///
/// # Project Index
///
/// The state of the project is read from a file once [`Project`] is opened and
/// kept in memory throughout its lifetime. The changes are written back to the
/// file once [`Project`] is dropped.
///
/// The state of a project consists of two sets of [`ResourceId`]s:
/// - Local [`ResourceId`] list - locally modified resources.
/// - Remote [`ResourceId`] list - synced resources.
///
/// A resource consists of a resource content file and a `.meta` file associated
/// to it. [`ResourceId`] is enough to locate a resource content file and its
/// associated `.meta` file on disk.
///
/// ## Example directory structure
///
/// An example of a project with 2 offline resources on disk looks as follows:
/// ```markdown
///  ./
///  | + offline/
///  | |- .lsc/
///  | |- a81fb4498cd04368
///  | |- a81fb4498cd04368.meta
///  | |- 8063daaf864780d6
///  | |- 8063daaf864780d6.meta
/// ```
///
/// ## Resource `.meta` file
///
/// The information in `.meta` file includes:
/// - List of [`ResourceId`]s of resource's build dependencies.
/// - Resource's name - [`ResourcePathName`].
/// - Identifier of resource's content file.
///
/// Note: Resource's [`ResourcePathName`] is only used for display purposes and
/// can be changed freely.
pub struct Project {
    workspace: Workspace<ResourceTypeAndIdIndexer>,
    deleted_pending: HashMap<ResourceId, (ResourcePathName, ResourceType)>,
}

#[derive(Error, Debug)]
/// Error returned by the project.
pub enum Error {
    /// Project index parsing error.
    #[error("Parsing '{0}' failed with {1}")]
    Parse(PathBuf, #[source] serde_json::error::Error),
    /// Not found.
    #[error("File {0} not found")]
    FileNotFound(String),
    /// IO error on the project index file.
    #[error("IO on '{0}' failed with {1}")]
    Io(PathBuf, #[source] std::io::Error),
    /// Source Control related error.
    #[error("source-control: '{0}'")]
    SourceControl(#[from] lgn_source_control::Error),
    /// Content-store related error.
    #[error("content store: '{0}'")]
    ContentStore(#[from] lgn_content_store::Error),
    /// RegistryRegistry Error
    #[error("ResourceRegistry Error: '{1}' on resource '{0}'")]
    ResourceRegistry(ResourceTypeAndId, #[source] AssetRegistryError),
    /// Name already used
    #[error("name '{0}' already used by resource '{1}'")]
    NameAlreadyUsed(ResourcePathName, ResourceTypeAndId),
    /// Resource not found in content-store
    #[error("resource '{0}' not found")]
    ResourceNotFound(ResourceTypeAndId),
}

/// The type of change done to a resource.
#[derive(Debug)]
pub enum ChangeType {
    /// Resource has been added.
    Add,
    /// Resource has been removed.
    Delete,
    /// Resource has been modified.
    Edit,
}

impl From<lgn_source_control::ChangeType> for ChangeType {
    fn from(change_type: lgn_source_control::ChangeType) -> Self {
        match change_type {
            lgn_source_control::ChangeType::Add { new_id: _ } => Self::Add,
            lgn_source_control::ChangeType::Edit {
                old_id: _,
                new_id: _,
            } => Self::Edit,
            lgn_source_control::ChangeType::Delete { old_id: _ } => Self::Delete,
        }
    }
}

impl Project {
    /// Returns current manifest (main index) of the workspace associated with the project
    pub fn source_manifest_id(&self) -> SharedTreeIdentifier {
        self.workspace.clone_main_index_id()
    }

    /// Creates a new project index file turning the containing directory into a
    /// project.
    pub async fn new(
        repository_index: impl RepositoryIndex,
        repository_name: &RepositoryName,
        branch_name: &str,
        source_control_content_provider: Arc<Provider>,
    ) -> Result<Self, Error> {
        let workspace = Workspace::new(
            repository_index,
            repository_name,
            branch_name,
            source_control_content_provider,
            new_resource_type_and_id_indexer(),
        )
        .await?;

        Ok(Self {
            workspace,
            deleted_pending: HashMap::new(),
        })
    }

    /// Same as [`Self::new`] but it creates an origin source control index at ``project_dir/remote``.
    pub async fn new_with_remote_mock(
        project_dir: impl AsRef<Path>,
        source_control_content_provider: Arc<Provider>,
    ) -> Result<Self, Error> {
        let remote_dir = project_dir.as_ref().join("remote");
        let repository_index = LocalRepositoryIndex::new(remote_dir).await?;
        let repository_name: RepositoryName = "default".parse().unwrap();

        repository_index.create_repository(&repository_name).await?;

        Self::new(
            repository_index,
            &repository_name,
            "main",
            source_control_content_provider,
        )
        .await
    }

    /// Return the list of stages resources
    pub async fn get_pending_changes(&self) -> Result<Vec<(ResourceTypeAndId, ChangeType)>, Error> {
        let pending_changes = self.workspace.get_pending_changes().await?;

        let changes = pending_changes
            .into_iter()
            .map(|(index_key, change)| (index_key.into(), change.into()))
            .collect::<Vec<_>>();

        Ok(changes)
    }

    /// Returns an iterator on the list of resources.
    ///
    /// This method flattens the `remote` and `local` resources into one list.
    pub async fn resource_list(&self) -> Vec<ResourceTypeAndId> {
        self.get_resources()
            .await
            .unwrap()
            .into_iter()
            .map(|(type_id, _resource_id)| type_id)
            .collect()
    }

    /// Finds resource by its name and returns its `ResourceTypeAndId`.
    pub async fn find_resource(&self, name: &ResourcePathName) -> Result<ResourceTypeAndId, Error> {
        let (meta, _resource_id) = self.read_meta_by_name(name).await?;
        Ok(meta.type_id)
    }

    /// Checks if a resource with a given name is part of the project.
    pub async fn exists_named(&self, name: &ResourcePathName) -> bool {
        self.workspace
            .resource_exists_by_path(name.as_str())
            .await
            .unwrap()
    }

    /// Checks if a resource is part of the project.
    pub async fn exists(&self, type_id: ResourceTypeAndId) -> bool {
        self.workspace
            .resource_exists(&type_id.into())
            .await
            .unwrap()
    }

    /// From a specific `ResourcePathName`, validate that the resource doesn't already exists
    /// or increment the suffix number until resource name is not used
    /// Ex: /world/sample => /world/sample1
    /// Ex: /world/instance1099 => /world/instance1100
    /// Ex: /world/thingy.psd => /world/thingy1.psd
    pub async fn get_incremental_name(&self, resource_path: &ResourcePathName) -> ResourcePathName {
        let path = Path::new(resource_path.as_ref());

        let ext = path
            .extension()
            .map(|ext| ext.to_string_lossy().into_owned());

        let mut name = if ext.is_some() {
            // We may want to drop non utf-8 character anyways?
            path.with_extension("").to_string_lossy().into_owned()
        } else {
            resource_path.to_string()
        };

        // extract the current suffix number if available
        let mut suffix = String::new();
        name.chars()
            .rev()
            .take_while(|c| c.is_digit(10))
            .for_each(|c| suffix.insert(0, c));

        name = name.trim_end_matches(suffix.as_str()).into();
        let mut index = suffix.parse::<u32>().unwrap_or(1);
        loop {
            // Check if the resource_name exists, if not increment index
            let mut new_path = format!("{}{}", name, index).into();

            if let Some(ref ext) = ext {
                new_path = new_path + "." + ext.as_str();
            }

            if !self.exists_named(&new_path).await {
                return new_path;
            }
            index += 1;
        }
    }

    /// Add a given resource of a given type with an associated `.meta`.
    ///
    /// The created `.meta` file contains a checksum of the resource content.
    /// `TODO`: the checksum of content needs to be updated when file is
    /// modified.
    ///
    /// Both resource file and its corresponding `.meta` file are `staged`.
    /// Use [`Self::commit()`] to push changes to remote.
    pub async fn add_resource(
        &mut self,
        name: ResourcePathName,
        kind: ResourceType,
        handle: impl AsRef<HandleUntyped>,
        registry: &AssetRegistry,
    ) -> Result<ResourceTypeAndId, Error> {
        let type_id = ResourceTypeAndId {
            kind,
            id: ResourceId::new(),
        };
        self.add_resource_with_id(name, type_id, handle, registry)
            .await?;
        Ok(type_id)
    }

    /// Add a given resource of a given type and id with an associated `.meta`.
    ///
    /// The created `.meta` file contains a checksum of the resource content.
    /// `TODO`: the checksum of content needs to be updated when file is
    /// modified.
    ///
    /// Both resource file and its corresponding `.meta` file are `staged`.
    /// Use [`Self::commit()`] to push changes to remote.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_resource_with_id(
        &mut self,
        name: ResourcePathName,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<HandleUntyped>,
        registry: &AssetRegistry,
    ) -> Result<(), Error> {
        let contents = Self::get_resource_contents(&name, type_id, handle, registry)?;

        self.workspace
            .add_resource(&type_id.into(), name.as_str(), &contents)
            .await?;

        Ok(())
    }

    fn get_resource_contents(
        name: &ResourcePathName,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<HandleUntyped>,
        registry: &AssetRegistry,
    ) -> Result<Vec<u8>, Error> {
        let mut contents = std::io::Cursor::new(Vec::new());

        let dependencies = registry
            .get_build_dependencies(type_id.kind, &handle)
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;

        // pre-pend metadata before serialized resource
        let metadata = Metadata {
            name: name.clone(),
            type_id,
            dependencies,
        };
        metadata.serialize(&mut contents);

        let _written = registry
            .serialize_resource_without_dependencies(type_id.kind, &handle, &mut contents)
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;

        Ok(contents.into_inner())
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn delete_resource(&mut self, type_id: ResourceTypeAndId) -> Result<(), Error> {
        let name = self.raw_resource_name(type_id).await?;
        self.workspace
            .delete_resource(&type_id.into(), name.as_str())
            .await?;

        Ok(())
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn revert_resource(&mut self, type_id: ResourceTypeAndId) -> Result<(), Error> {
        let name = self.raw_resource_name(type_id).await?;
        self.workspace
            .revert_resource(&type_id.into(), name.as_str())
            .await?;

        Ok(())
    }

    /// Writes the resource behind `handle` from memory to disk and updates the
    /// corresponding .meta file.
    pub async fn save_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<HandleUntyped>,
        registry: &AssetRegistry,
    ) -> Result<(), Error> {
        let (meta, resource_id) = self.read_meta(type_id).await?;
        let contents = Self::get_resource_contents(&meta.name, type_id, handle, registry)?;

        self.workspace
            .update_resource(&type_id.into(), meta.name.as_str(), &contents, &resource_id)
            .await?;

        Ok(())
    }

    /// Loads a resource of a given id.
    pub async fn load_resource(
        &self,
        type_id: ResourceTypeAndId,
        resources: &AssetRegistry,
    ) -> Result<HandleUntyped, Error> {
        let (resource_bytes, _resource_id) = self.workspace.load_resource(&type_id.into()).await?;

        let mut reader = std::io::Cursor::new(resource_bytes);

        // skip over the pre-pended metadata
        deserialize_and_skip_metadata(&mut reader);

        resources
            .deserialize_resource(type_id, &mut reader)
            .map_err(|e| Error::ResourceRegistry(type_id, e))
    }

    /// Returns information about a given resource from its `.meta` file.
    pub async fn resource_dependencies(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<Vec<ResourcePathId>, Error> {
        let (meta, _resource_id) = self.read_meta(type_id).await?;
        Ok(meta.dependencies)
    }

    /// Returns the name of the resource from its `.meta` file.
    #[async_recursion]
    pub async fn resource_name(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<ResourcePathName, Error> {
        let name = self.raw_resource_name(type_id).await?;
        if let Some((resource_id, suffix)) = name
            .as_str()
            .strip_prefix("/!")
            .and_then(|v| v.split_once('/'))
        {
            if let Ok(type_id) = <ResourceTypeAndId as std::str::FromStr>::from_str(resource_id) {
                if let Ok(mut parent_path) = self.resource_name(type_id).await {
                    parent_path.push(suffix);
                    return Ok(parent_path);
                }
            }
        }
        Ok(name)
    }

    /// Returns the name of the resource from its `.meta` file.
    pub async fn deleted_resource_info(
        &mut self,
        _type_id: ResourceTypeAndId,
    ) -> Result<ResourcePathName, Error> {
        // let metadata_path = self.metadata_path(id);

        // match self.deleted_pending.entry(id) {
        //     Entry::Vacant(entry) => {
        //         let tree = self.workspace.get_staged_changes().await?;

        //         let meta_lsc_path =
        //             CanonicalPath::new_from_canonical_paths(self.workspace.root(), &metadata_path)
        //                 .map_err(|err| {
        //                     error!(
        //                         "Failed to retrieve delete info for Resource {}: {}",
        //                         id, err
        //                     );
        //                     Error::SourceControl(err)
        //                 })?;

        //         if let Some(lgn_source_control::ChangeType::Delete { old_id }) = tree
        //             .get(&meta_lsc_path)
        //             .map(lgn_source_control::Change::change_type)
        //         {
        //             match self.workspace.provider().read(old_id).await {
        //                 Ok(data) => {
        //                     if let Ok(meta) = serde_json::from_slice::<Metadata>(&data) {
        //                         let value = (meta.name, meta.type_id);
        //                         entry.insert(value.clone());
        //                         return Ok(value);
        //                     }
        //                 }
        //                 Err(err) => {
        //                     error!(
        //                         "Failed to retrieve delete info for Resource {}: {}",
        //                         id, err
        //                     );
        //                 }
        //             }
        //         }

        //         Err(Error::FileNotFound(meta_lsc_path.to_string()))
        //     }
        //     Entry::Occupied(entry) => Ok(entry.get().clone()),
        // }

        Err(Error::FileNotFound("not implemented".to_owned()))
    }

    /// Returns the raw name of the resource from its `.meta` file.
    pub async fn raw_resource_name(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<ResourcePathName, Error> {
        let (meta, _resource_id) = self.read_meta(type_id).await?;
        Ok(meta.name)
    }

    async fn read_meta(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<(Metadata, ResourceIdentifier), Error> {
        if let Some(resource_id) = self
            .workspace
            .get_resource_identifier(&type_id.into())
            .await?
        {
            let metadata = self.read_meta_by_resource_id(&resource_id).await?;
            Ok((metadata, resource_id))
        } else {
            Err(Error::SourceControl(
                lgn_source_control::Error::resource_not_found_by_id(type_id),
            ))
        }
    }

    async fn read_meta_by_name(
        &self,
        name: &ResourcePathName,
    ) -> Result<(Metadata, ResourceIdentifier), Error> {
        if let Some(resource_id) = self
            .workspace
            .get_resource_identifier_by_path(name.as_str())
            .await?
        {
            let metadata = self.read_meta_by_resource_id(&resource_id).await?;
            Ok((metadata, resource_id))
        } else {
            Err(Error::SourceControl(
                lgn_source_control::Error::resource_not_found_by_path(name.as_str()),
            ))
        }
    }

    async fn read_meta_by_resource_id(
        &self,
        resource_id: &ResourceIdentifier,
    ) -> Result<Metadata, Error> {
        let resource_bytes = self.workspace.load_resource_by_id(resource_id).await?;

        let mut reader = std::io::Cursor::new(resource_bytes);

        // just read the pre-pended metadata
        let metadata = Metadata::deserialize(&mut reader);

        Ok(metadata)
    }

    /// Change the name of the resource.
    ///
    /// Changing the name of the resource if `free`. It does not change its
    /// `ResourceId` nor it invalidates any build using that asset.
    pub async fn rename_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        new_name: &ResourcePathName,
    ) -> Result<ResourcePathName, Error> {
        let (resource_bytes, resource_id) = self.workspace.load_resource(&type_id.into()).await?;
        let mut reader = std::io::Cursor::new(resource_bytes);

        // read existing pre-pended metadata
        let mut metadata = Metadata::deserialize(&mut reader);
        let old_name = metadata.rename(new_name);

        if new_name != &old_name {
            // already used?
            if let Ok(existing_type_id) = self.find_resource(new_name).await {
                return Err(Error::NameAlreadyUsed(new_name.clone(), existing_type_id));
            }

            // update resource contents since embedded metadata has changed
            let mut contents = std::io::Cursor::new(Vec::new());
            metadata.serialize(&mut contents);
            let resource_bytes = {
                let pos = reader.position() as usize;
                let resource_bytes = reader.into_inner();
                resource_bytes[pos..].to_vec()
            };
            contents
                .write_all(&resource_bytes)
                .expect("failed to transfert buffer contents");
            let contents = contents.into_inner();

            self.workspace
                .update_resource_and_path(
                    &type_id.into(),
                    old_name.as_str(),
                    new_name.as_str(),
                    &contents,
                    &resource_id,
                )
                .await?;
        }

        Ok(old_name)
    }

    /// Moves `local` resources to `remote` resource list.
    pub async fn commit(&mut self, message: &str) -> Result<(), Error> {
        self.deleted_pending.clear();
        self.workspace
            .commit(message, CommitMode::Lenient)
            .await
            .map_err(Error::SourceControl)
            .map(|_| ())
    }

    /// Pulls all changes from the origin.
    pub async fn sync_latest(&mut self) -> Result<Vec<(ResourceTypeAndId, ChangeType)>, Error> {
        // let (_, changed) = self.workspace.sync().await.map_err(Error::SourceControl)?;

        // let resources = changed
        //     .iter()
        //     .filter_map(|change| {
        //         let id = change
        //             .canonical_path()
        //             .name()
        //             .filter(|filename| !filename.contains('.')) // skip .meta files
        //             .map(|filename| (ResourceId::from_str(filename).unwrap()));

        //         id.map(|id| (id, change.change_type().into()))
        //     })
        //     .collect::<Vec<_>>();
        // Ok(resources)

        Err(Error::FileNotFound("not implemented".to_owned()))
    }

    /// Returns list of resources stored in the content store
    pub async fn get_resources(
        &self,
    ) -> Result<Vec<(ResourceTypeAndId, ResourceIdentifier)>, Error> {
        Ok(self
            .workspace
            .get_resources()
            .await?
            .into_iter()
            .map(|(index_key, resource_id)| (index_key.into(), resource_id))
            .collect())
    }

    /// Return the list of all resources that were previously committed
    pub async fn get_committed_resources(
        &self,
    ) -> Result<Vec<(IndexKey, ResourceIdentifier)>, Error> {
        self.workspace
            .get_committed_resources()
            .await
            .map_err(Error::SourceControl)
    }

    /// Returns the checksum of the root project directory at the current state.
    pub fn root_checksum(&self) -> (TreeIdentifier, TreeIdentifier) {
        self.workspace.indices()
    }
}

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<ResourceTypeAndId> = vec![]; // todo self.resource_list().map(|r| self.resource_name(r).unwrap());
        f.debug_list().entries(names).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use lgn_content_store::Provider;
    use lgn_data_runtime::{
        resource, AssetRegistry, AssetRegistryOptions, Resource, ResourcePathId, ResourceProcessor,
        ResourceProcessorError, ResourceType,
    };

    use crate::resource::project::Project;
    use crate::resource::ResourcePathName;

    const RESOURCE_TEXTURE: ResourceType = ResourceType::new(b"texture");
    const RESOURCE_MATERIAL: ResourceType = ResourceType::new(b"material");
    const RESOURCE_GEOMETRY: ResourceType = ResourceType::new(b"geometry");
    const RESOURCE_SKELETON: ResourceType = ResourceType::new(b"skeleton");
    const RESOURCE_ACTOR: ResourceType = ResourceType::new(b"actor");

    #[resource("null")]
    #[derive(Clone)]
    struct NullResource {
        content: isize,
        dependencies: Vec<ResourcePathId>,
    }

    struct NullResourceProc {}
    impl ResourceProcessor for NullResourceProc {
        fn new_resource(&mut self) -> Box<dyn Resource> {
            Box::new(NullResource {
                content: 0,
                dependencies: vec![],
            })
        }

        fn extract_build_dependencies(&self, resource: &dyn Resource) -> Vec<ResourcePathId> {
            resource
                .downcast_ref::<NullResource>()
                .unwrap()
                .dependencies
                .clone()
        }

        fn write_resource(
            &self,
            resource: &dyn Resource,
            writer: &mut dyn std::io::Write,
        ) -> Result<usize, ResourceProcessorError> {
            let resource = resource.downcast_ref::<NullResource>().unwrap();
            let mut nbytes = 0;

            let bytes = resource.content.to_ne_bytes();
            nbytes += bytes.len();
            writer.write_all(&bytes)?;

            let bytes = resource.dependencies.len().to_ne_bytes();
            nbytes += bytes.len();
            writer.write_all(&bytes)?;

            for dep in &resource.dependencies {
                let str = dep.to_string();
                let str = str.as_bytes();
                let bytes = str.len().to_ne_bytes();
                writer.write_all(&bytes)?;
                nbytes += bytes.len();
                writer.write_all(str)?;
                nbytes += str.len();
            }

            Ok(nbytes)
        }

        fn read_resource(
            &mut self,
            reader: &mut dyn std::io::Read,
        ) -> Result<Box<dyn Resource>, ResourceProcessorError> {
            let mut resource = self.new_resource();
            let mut res = resource.downcast_mut::<NullResource>().unwrap();

            let mut buf = res.content.to_ne_bytes();
            reader.read_exact(&mut buf[..])?;
            res.content = isize::from_ne_bytes(buf);

            let mut buf = res.dependencies.len().to_ne_bytes();
            reader.read_exact(&mut buf[..])?;

            for _ in 0..usize::from_ne_bytes(buf) {
                let mut nbytes = 0u64.to_ne_bytes();
                reader.read_exact(&mut nbytes[..])?;
                let mut buf = vec![0u8; usize::from_ne_bytes(nbytes)];
                reader.read_exact(&mut buf)?;
                res.dependencies
                    .push(ResourcePathId::from_str(std::str::from_utf8(&buf).unwrap()).unwrap());
            }

            Ok(resource)
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn create_actor(project: &mut Project) -> Arc<AssetRegistry> {
        let resources = AssetRegistryOptions::new()
            .add_processor_ext(RESOURCE_TEXTURE, Box::new(NullResourceProc {}))
            .add_processor_ext(RESOURCE_MATERIAL, Box::new(NullResourceProc {}))
            .add_processor_ext(RESOURCE_GEOMETRY, Box::new(NullResourceProc {}))
            .add_processor_ext(RESOURCE_SKELETON, Box::new(NullResourceProc {}))
            .add_processor_ext(RESOURCE_ACTOR, Box::new(NullResourceProc {}))
            .create()
            .await;

        let texture = project
            .add_resource(
                ResourcePathName::new("albedo.texture"),
                RESOURCE_TEXTURE,
                resources.new_resource(RESOURCE_TEXTURE).unwrap(),
                &resources,
            )
            .await
            .unwrap();

        let material = resources
            .new_resource(RESOURCE_MATERIAL)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = material.instantiate(&resources).unwrap();
        edit.dependencies.push(ResourcePathId::from(texture));
        material.apply(edit, &resources);

        let material = project
            .add_resource(
                ResourcePathName::new("body.material"),
                RESOURCE_MATERIAL,
                &material,
                &resources,
            )
            .await
            .unwrap();

        let geometry = resources
            .new_resource(RESOURCE_GEOMETRY)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = geometry.instantiate(&resources).unwrap();
        edit.dependencies.push(ResourcePathId::from(material));
        geometry.apply(edit, &resources);
        let geometry = project
            .add_resource(
                ResourcePathName::new("hero.geometry"),
                RESOURCE_GEOMETRY,
                &geometry,
                &resources,
            )
            .await
            .unwrap();

        let skeleton = project
            .add_resource(
                ResourcePathName::new("hero.skeleton"),
                RESOURCE_SKELETON,
                &resources.new_resource(RESOURCE_SKELETON).unwrap(),
                &resources,
            )
            .await
            .unwrap();

        let actor = resources
            .new_resource(RESOURCE_ACTOR)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = actor.instantiate(&resources).unwrap();
        edit.dependencies = vec![
            ResourcePathId::from(geometry),
            ResourcePathId::from(skeleton),
        ];
        actor.apply(edit, &resources);
        let _actor = project
            .add_resource(
                ResourcePathName::new("hero.actor"),
                RESOURCE_ACTOR,
                &actor,
                &resources,
            )
            .await
            .unwrap();

        resources
    }

    async fn create_sky_material(project: &mut Project, resources: &AssetRegistry) {
        let texture = project
            .add_resource(
                ResourcePathName::new("sky.texture"),
                RESOURCE_TEXTURE,
                &resources.new_resource(RESOURCE_TEXTURE).unwrap(),
                resources,
            )
            .await
            .unwrap();

        let material = resources
            .new_resource(RESOURCE_MATERIAL)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = material.instantiate(resources).unwrap();
        edit.dependencies.push(ResourcePathId::from(texture));
        material.apply(edit, resources);

        let _material = project
            .add_resource(
                ResourcePathName::new("sky.material"),
                RESOURCE_MATERIAL,
                &material,
                resources,
            )
            .await
            .unwrap();
    }

    /* test disabled due to problems with project deletion.
    sqlx doesn't release .db file for some reason.

    #[tokio::test]
    async fn proj_create_delete() {
        let root = tempfile::tempdir().unwrap();

        let project = Project::new_with_remote_mock(root.path())
            .await
            .expect("failed to create project");
        let same_project = Project::new_with_remote_mock(root.path()).await;
        assert!(same_project.is_err());

        project.delete().await;

        let _project = Project::new_with_remote_mock(root.path())
            .await
            .expect("failed to re-create project");
        let same_project = Project::new_with_remote_mock(root.path()).await;
        assert!(same_project.is_err());
    }*/

    #[tokio::test]
    async fn local_changes() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::new_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let _resources = create_actor(&mut project).await;

        assert_eq!(project.get_pending_changes().await.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn commit() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::new_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        assert_eq!(project.get_pending_changes().await.unwrap().len(), 5);
        assert_eq!(project.get_committed_resources().await.unwrap().len(), 0);

        // modify before commit
        {
            let handle = project.load_resource(actor_id, &resources).await.unwrap();
            let mut content = handle.instantiate::<NullResource>(&resources).unwrap();
            content.content = 8;
            handle.apply(content, &resources);

            project
                .save_resource(actor_id, &handle, &resources)
                .await
                .unwrap();
        }

        project.commit("add resources").await.unwrap();

        assert_eq!(project.get_pending_changes().await.unwrap().len(), 0);
        assert_eq!(project.get_committed_resources().await.unwrap().len(), 5);

        // modify resource
        {
            let handle = project.load_resource(actor_id, &resources).await.unwrap();
            let mut content = handle.instantiate::<NullResource>(&resources).unwrap();
            assert_eq!(content.content, 8);
            content.content = 9;
            handle.apply(content, &resources);
            project
                .save_resource(actor_id, &handle, &resources)
                .await
                .unwrap();

            assert_eq!(project.get_pending_changes().await.unwrap().len(), 1);
        }

        project.commit("update actor").await.unwrap();

        assert_eq!(project.get_pending_changes().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn change_to_previous() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::new_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        project.commit("initial actor").await.unwrap();

        // modify resource
        let original_content = {
            let handle = project.load_resource(actor_id, &resources).await.unwrap();
            let mut content = handle.instantiate::<NullResource>(&resources).unwrap();
            let previous_value = content.content;
            content.content = 9;
            handle.apply(content, &resources);
            project
                .save_resource(actor_id, &handle, &resources)
                .await
                .unwrap();

            previous_value
        };

        {
            let handle = project.load_resource(actor_id, &resources).await.unwrap();
            let mut content = handle.instantiate::<NullResource>(&resources).unwrap();
            content.content = original_content;
            handle.apply(content, &resources);
            project
                .save_resource(actor_id, &handle, &resources)
                .await
                .unwrap();
        }

        project.commit("no changes").await.unwrap();
    }

    #[tokio::test]
    async fn immediate_dependencies() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::new_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let _resources = create_actor(&mut project).await;

        let top_level_resource = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        let dependencies = project
            .resource_dependencies(top_level_resource)
            .await
            .unwrap();

        assert_eq!(dependencies.len(), 2);
    }

    async fn rename_assert(
        proj: &mut Project,
        old_name: ResourcePathName,
        new_name: ResourcePathName,
    ) {
        let skeleton_id = proj.find_resource(&old_name).await;
        assert!(skeleton_id.is_ok());
        let skeleton_id = skeleton_id.unwrap();

        let prev_name = proj.rename_resource(skeleton_id, &new_name).await;
        assert!(prev_name.is_ok());
        let prev_name = prev_name.unwrap();
        assert_eq!(&prev_name, &old_name);

        assert!(proj.find_resource(&old_name).await.is_err());
        assert_eq!(proj.find_resource(&new_name).await.unwrap(), skeleton_id);
    }

    #[tokio::test]
    async fn rename() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::new_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;
        assert!(project.commit("rename test").await.is_ok());
        create_sky_material(&mut project, &resources).await;

        rename_assert(
            &mut project,
            ResourcePathName::new("hero.skeleton"),
            ResourcePathName::new("boss.skeleton"),
        )
        .await;
        rename_assert(
            &mut project,
            ResourcePathName::new("sky.material"),
            ResourcePathName::new("clouds.material"),
        )
        .await;
    }
}
