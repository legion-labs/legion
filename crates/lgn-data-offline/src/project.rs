use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::offline::Metadata;
use crate::ResourcePathName;
use async_recursion::async_recursion;
use lgn_content_store::{
    indexing::{IndexKey, ResourceIdentifier, SharedTreeIdentifier, TreeIdentifier},
    Provider,
};
use lgn_data_runtime::{
    new_resource_type_and_id_indexer, AssetRegistryError, AssetRegistryReader, Resource,
    ResourceId, ResourcePathId, ResourceType, ResourceTypeAndId, ResourceTypeAndIdIndexer,
};
use lgn_source_control::{
    CommitMode, LocalRepositoryIndex, RepositoryIndex, RepositoryName, Workspace,
};
use lgn_tracing::error;
use thiserror::Error;

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
/// A resource consists of a resource content file and metadata associated
/// to it. [`ResourceId`] is enough to locate a resource content file. Some resources
/// might not have metadata.
///
/// ## Example directory structure
///
/// An example of a project with 2 offline resources on disk looks as follows:
/// ```markdown
///  ./
///  | + offline/
///  | |- .lsc/
///  | |- a8/1f/b4/a81fb4498cd04368
///  | |- 80/63/da/8063daaf864780d6
/// ```
///
/// ## Resource's `meta` portion
///
/// Each resource might have metadata associated to it, stored internally in the resource itself.
/// The metadata contains the following:
/// - List of [`ResourceId`]s of resource's build dependencies.
/// - Resource's name - [`ResourcePathName`].
/// - Identifier of resource's content file.
///
/// Note: The resource's [`ResourcePathName`] is only used for display purposes and
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
    /// Serialization Error
    #[error("Serialization error: '{0}'")]
    Serialization(#[from] serde_json::Error),
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

impl<'a> From<&'a lgn_source_control::ChangeType> for ChangeType {
    fn from(change_type: &lgn_source_control::ChangeType) -> Self {
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

    /*
    /// Return the list of stages resources
    pub async fn get_staged_changes(&self) -> Result<Vec<(ResourceId, ChangeType)>, Error> {
        let local_changes = self.workspace.get_staged_changes().await?;

        let changes = local_changes
            .into_iter()
            .map(|(path, change)| (PathBuf::from(path.to_string()), change))
            .filter(|(path, _)| path.extension().is_none())
            .map(|(path, change)| {
                (
                    ResourceId::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap(),
                    change.change_type().into(),
                )
            })
            .collect::<Vec<_>>();

        Ok(changes)
    }
    */

    /// Returns an iterator on the list of resources.
    ///
    /// This method flattens the `remote` and `local` resources into one list.
    pub async fn resource_list(&self) -> Vec<ResourceTypeAndId> {
        self.get_resources()
            .await
            .unwrap()
            .iter()
            .map(|(index_key, _resource_id)| index_key.into())
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
        resource: &dyn Resource,
    ) -> Result<ResourceTypeAndId, Error> {
        let type_id = crate::get_meta(resource).type_id;
        self.add_resource_with_id(type_id, resource).await?;
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
        type_id: ResourceTypeAndId,
        resource: &dyn Resource,
    ) -> Result<(), Error> {
        let mut contents = Vec::new();
        crate::to_json_writer(resource, &mut contents)
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;

        let meta = crate::get_meta(resource);
        assert_eq!(meta.type_id, type_id);
        self.workspace
            .add_resource(&type_id.into(), meta.name.as_str(), &contents)
            .await?;

        Ok(())
    }

    /*
    fn get_resource_contents(
        name: &ResourcePathName,
        type_id: ResourceTypeAndId,
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
    }*/

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn delete_resource(&mut self, type_id: ResourceTypeAndId) -> Result<(), Error> {
        let name = self.raw_resource_name(type_id).await?;
        self.workspace
            .delete_resource(&type_id.into(), name.as_str())
            .await?;

        Ok(())
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn revert_resource(&mut self, _type_id: ResourceTypeAndId) -> Result<(), Error> {
        // let resource_path = self.resource_path(id);
        // let metadata_path = self.metadata_path(id);

        // {
        //     let files = [metadata_path.as_path(), resource_path.as_path()];
        //     self.workspace
        //         .revert_files(files, Staging::StagedAndUnstaged)
        //         .await?;
        // }

        Ok(())
    }

    /// Writes the resource behind `handle` from memory to disk and updates the
    /// corresponding .meta file.
    pub async fn save_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        resource: &dyn Resource,
    ) -> Result<(), Error> {
        let mut contents = Vec::new();
        crate::to_json_writer(resource, &mut contents)
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;

        if let Some(old_resource_id) = self
            .workspace
            .get_resource_identifier(&type_id.into())
            .await?
        {
            let meta = crate::get_meta(resource);
            assert_eq!(meta.type_id, type_id);
            self.workspace
                .update_resource(
                    &type_id.into(),
                    meta.name.as_str(),
                    &contents,
                    &old_resource_id,
                )
                .await?;
        }

        Ok(())
    }

    /// Loads a resource of a given id.
    pub async fn load_resource_untyped(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<Box<dyn Resource>, Error> {
        let (reader, _resource_ident) = self.workspace.get_reader(&type_id.into()).await?;
        let mut reader = Box::pin(reader) as AssetRegistryReader;

        let resource = crate::from_json_reader_untyped(&mut reader)
            .await
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;
        let meta = crate::get_meta(resource.as_ref());
        assert_eq!(meta.type_id, type_id);
        Ok(resource)
    }

    /// Loads a resource of a given type and id.
    pub async fn load_resource<T: Resource + Default>(
        &self,
        type_id: ResourceTypeAndId,
    ) -> Result<Box<T>, Error> {
        let resource = self.load_resource_untyped(type_id).await?;
        if resource.is::<T>() {
            let raw: *mut dyn lgn_data_runtime::Resource = Box::into_raw(resource);
            #[allow(unsafe_code, clippy::cast_ptr_alignment)]
            let boxed_asset = unsafe { Box::from_raw(raw.cast::<T>()) };
            return Ok(boxed_asset);
        }
        Err(Error::ResourceRegistry(
            type_id,
            lgn_data_runtime::AssetRegistryError::Generic("invalid type".into()),
        ))
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
            assert_eq!(metadata.type_id, type_id);
            Ok((metadata, resource_id))
        } else {
            Err(Error::SourceControl(
                lgn_source_control::Error::ResourceNotFoundById { id: type_id.into() },
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
                lgn_source_control::Error::ResourceNotFoundByPath {
                    path: name.as_str().to_owned(),
                },
            ))
        }
    }

    async fn read_meta_by_resource_id(
        &self,
        resource_id: &ResourceIdentifier,
    ) -> Result<Metadata, Error> {
        let resource_bytes = self.workspace.load_resource_by_id(resource_id).await?;
        let mut stream = serde_json::Deserializer::from_slice(resource_bytes.as_slice())
            .into_iter::<serde_json::Value>();
        let meta_json = stream
            .next()
            .ok_or_else(|| {
                Error::ContentStore(lgn_content_store::Error::IdentifierNotFound(
                    resource_id.as_identifier().clone(),
                ))
            })?
            .map_err(Error::from)?;

        let metadata = serde_json::from_value(meta_json)?;
        //let mut metadata = Metadata::default();
        //reflection_apply_json_edit(&mut metadata, &meta_json);
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
        let (reader, resource_ident) = self.workspace.get_reader(&type_id.into()).await?;
        let mut reader = Box::pin(reader) as AssetRegistryReader;

        let mut resource = crate::from_json_reader_untyped(&mut reader)
            .await
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;

        let old_name = {
            let metadata = crate::get_meta_mut(resource.as_mut());
            metadata.rename(new_name)
        };
        if new_name != &old_name {
            // already used?
            if let Ok(existing_type_id) = self.find_resource(new_name).await {
                return Err(Error::NameAlreadyUsed(new_name.clone(), existing_type_id));
            }

            let mut contents = Vec::new();
            crate::to_json_writer(resource.as_ref(), &mut contents)
                .map_err(|e| Error::ResourceRegistry(type_id, e))?;

            self.workspace
                .update_resource_and_path(
                    &type_id.into(),
                    old_name.as_str(),
                    new_name.as_str(),
                    &contents,
                    &resource_ident,
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
    pub async fn get_resources(&self) -> Result<Vec<(IndexKey, ResourceIdentifier)>, Error> {
        self.workspace
            .get_resources()
            .await
            .map_err(Error::SourceControl)
    }

    /// Return the list of resources that have pending (uncommitted) changes
    pub async fn get_pending_changes(&self) -> Result<Vec<ResourceTypeAndId>, Error> {
        Ok(self
            .workspace
            .get_pending_changes()
            .await?
            .into_iter()
            .map(Into::into)
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

    /// Returns whether or not the workspace contains any changes that have not yet been committed to the content-store.
    pub async fn has_pending_changes(&self) -> bool {
        self.workspace.has_pending_changes().await
    }
}

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<ResourceTypeAndId> = vec![]; // todo self.resource_list().map(|r| self.resource_name(r).unwrap());
        f.debug_list().entries(names).finish()
    }
}
