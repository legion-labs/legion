use core::fmt;
use std::{
    collections::{hash_map::Entry, HashMap},
    fs::{File, OpenOptions},
    io::Seek,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use lgn_content_store::Provider;
use lgn_data_runtime::{
    AssetRegistry, AssetRegistryError, HandleUntyped, Metadata, ResourceId, ResourcePathId,
    ResourcePathName, ResourceType, ResourceTypeAndId,
};
use lgn_source_control::{
    CanonicalPath, CommitMode, LocalRepositoryIndex, RepositoryIndex, RepositoryName, Staging,
    Workspace, WorkspaceConfig, WorkspaceRegistration,
};
use lgn_tracing::error;
use serde_json::Value;
use thiserror::Error;

pub use lgn_source_control::data_types::Tree;

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
    project_dir: PathBuf,
    resource_dir: PathBuf,
    workspace: Workspace,
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
    /// Returns directory the project is in, relative to CWD.
    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    /// Same as [`Self::create`] but it creates an origin source control index at ``project_dir/remote``.
    pub async fn create_with_remote_mock(
        project_dir: impl AsRef<Path>,
        source_control_content_provider: Arc<Provider>,
    ) -> Result<Self, Error> {
        let remote_dir = project_dir.as_ref().join("remote");
        let repository_index = LocalRepositoryIndex::new(remote_dir).await?;
        let repository_name: RepositoryName = "default".parse().unwrap();

        repository_index
            .create_repository(repository_name.clone())
            .await?;

        Self::create(
            project_dir,
            repository_index,
            repository_name,
            source_control_content_provider,
        )
        .await
    }

    /// Creates a new project index file turning the containing directory into a
    /// project.
    pub async fn create(
        project_dir: impl AsRef<Path>,
        repository_index: impl RepositoryIndex,
        repository_name: RepositoryName,
        source_control_content_provider: Arc<Provider>,
    ) -> Result<Self, Error> {
        let resource_dir = project_dir.as_ref().join("offline");
        if !resource_dir.exists() {
            std::fs::create_dir_all(&resource_dir)
                .map_err(|e| Error::Io(resource_dir.clone(), e))?;
        }

        let workspace = Workspace::init(
            &resource_dir,
            repository_index,
            WorkspaceConfig::new(
                repository_name,
                WorkspaceRegistration::new_with_current_user(),
            ),
            source_control_content_provider,
        )
        .await?;

        Ok(Self {
            project_dir: project_dir.as_ref().to_owned(),
            resource_dir,
            workspace,
            deleted_pending: HashMap::new(),
        })
    }

    /// Opens the project index specified
    pub async fn open(
        project_dir: impl AsRef<Path>,
        repository_index: impl RepositoryIndex,
        source_control_content_provider: Arc<Provider>,
    ) -> Result<Self, Error> {
        let resource_dir = project_dir.as_ref().join("offline");

        let workspace = Workspace::load(
            &resource_dir,
            repository_index,
            source_control_content_provider,
        )
        .await?;

        Ok(Self {
            project_dir: project_dir.as_ref().to_owned(),
            resource_dir,
            workspace,
            deleted_pending: HashMap::new(),
        })
    }

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

    /// Return the list of local resources
    pub async fn local_resource_list(&self) -> Result<Vec<ResourceId>, Error> {
        let local_changes = self.workspace.get_staged_changes().await?;

        let changes = local_changes
            .iter()
            .map(|(path, _)| PathBuf::from(path.to_string()))
            .filter(|path| path.extension().is_none())
            .map(|path| ResourceId::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap())
            .collect::<Vec<_>>();

        Ok(changes)
    }

    async fn remote_resource_list(&self) -> Result<Vec<ResourceId>, Error> {
        let tree = self.workspace.get_current_tree().await?;

        let files = tree
            .files()
            .map(|(path, _)| PathBuf::from(path.to_string()))
            .filter(|path| path.extension().is_none())
            .map(|path| ResourceId::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap())
            .collect::<Vec<_>>();

        Ok(files)
    }

    /// Returns an iterator on the list of resources.
    ///
    /// This method flattens the `remote` and `local` resources into one list.
    pub async fn resource_list(&self) -> Vec<ResourceId> {
        let mut all = self.local_resource_list().await.unwrap();
        match self.remote_resource_list().await {
            Ok(remote) => all.extend(remote),
            Err(err) => lgn_tracing::error!("Error fetching remote resources: {}", err),
        }
        all
    }

    /// Finds resource by its name and returns its `ResourceTypeAndId`.
    pub async fn find_resource(&self, name: &ResourcePathName) -> Result<ResourceTypeAndId, Error> {
        // this below would be better expressed as try_map (still experimental).
        let res = self.resource_list().await.into_iter().find_map(|id| {
            match self.read_meta(id) {
                Ok(meta) => {
                    if &meta.name == name {
                        Some(Ok(ResourceTypeAndId {
                            id,
                            kind: meta.type_id,
                        }))
                    } else {
                        None
                    }
                }
                Err(_err) => None, // TODO: Ignore for now to fix deleted files
            }
        });

        match res {
            None => Err(Error::FileNotFound(name.to_string())),
            Some(e) => e,
        }
    }

    /// Checks if a resource with a given name is part of the project.
    pub async fn exists_named(&self, name: &ResourcePathName) -> bool {
        self.find_resource(name).await.is_ok()
    }

    /// Checks if a resource is part of the project.
    pub async fn exists(&self, id: ResourceId) -> bool {
        self.resource_list().await.iter().any(|v| v == &id)
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

    /// Add a given resource of a given type with an associated metadata.
    ///
    /// The resource file is `staged`. Use [`Self::commit()`] to push changes to remote.
    pub async fn add_resource(
        &mut self,
        name: ResourcePathName,
        kind_name: &str,
        kind: ResourceType,
        handle: impl AsRef<HandleUntyped>,
        registry: &AssetRegistry,
    ) -> Result<ResourceTypeAndId, Error> {
        self.add_resource_with_id(name, kind_name, kind, ResourceId::new(), handle, registry)
            .await
    }

    /// Add a given resource of a given type and id with an associated metadata.
    ///
    /// The  resource file is `staged`. Use [`Self::commit()`] to push changes to remote.
    #[allow(clippy::too_many_arguments)]
    pub async fn add_resource_with_id(
        &mut self,
        name: ResourcePathName,
        _kind_name: &str,
        kind: ResourceType,
        id: ResourceId,
        handle: impl AsRef<HandleUntyped>,
        registry: &AssetRegistry,
    ) -> Result<ResourceTypeAndId, Error> {
        let resource_path = self.resource_path(id);

        let directory = {
            let mut directory = resource_path.clone();
            directory.pop();
            directory
        };

        std::fs::create_dir_all(&directory).map_err(|e| Error::Io(directory.clone(), e))?;

        let mut resource_file =
            File::create(&resource_path).map_err(|e| Error::Io(resource_path.clone(), e))?;

        if let Some(meta) = registry
            .get_resource_reflection_mut(handle.as_ref())
            .unwrap()
            .get_meta_mut()
        {
            meta.rename(&name);
        }

        registry
            .serialize_resource(kind, handle, &mut resource_file)
            .map_err(|e| Error::ResourceRegistry(ResourceTypeAndId { kind, id }, e))?;

        self.workspace.add_files([resource_path.as_path()]).await?;

        let type_id = ResourceTypeAndId { kind, id };

        Ok(type_id)
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn delete_resource(&mut self, id: ResourceId) -> Result<(), Error> {
        let resource_path = self.resource_path(id);

        self.workspace
            .delete_files([resource_path.as_path()])
            .await?;

        Ok(())
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn revert_resource(&mut self, id: ResourceId) -> Result<(), Error> {
        let resource_path = self.resource_path(id);

        self.workspace
            .revert_files([resource_path.as_path()], Staging::StagedAndUnstaged)
            .await?;

        Ok(())
    }

    /// Writes the resource behind `handle` from memory to disk and updates the
    /// corresponding .meta file.
    pub async fn save_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<HandleUntyped>,
        resources: &AssetRegistry,
    ) -> Result<(), Error> {
        self.checkout(type_id).await?;

        let resource_path = self.resource_path(type_id.id);
        let mut resource_file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&resource_path)
            .map_err(|e| Error::Io(resource_path.clone(), e))?;

        resources
            .serialize_resource(type_id.kind, handle, &mut resource_file)
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;

        self.workspace
            .add_files([resource_path.as_path()]) // add
            .await?;

        Ok(())
    }

    /// Loads a resource of a given id.
    pub fn load_resource(
        &self,
        type_id: ResourceTypeAndId,
        resources: &AssetRegistry,
    ) -> Result<HandleUntyped, Error> {
        let resource_path = self.resource_path(type_id.id);

        let mut resource_file =
            File::open(&resource_path).map_err(|e| Error::Io(resource_path.clone(), e))?;
        let handle = resources
            .deserialize_resource(type_id, &mut resource_file)
            .map_err(|e| Error::ResourceRegistry(type_id, e))?;
        Ok(handle)
    }

    /// Returns information about a given resource from its `.meta` file.
    pub fn resource_info(
        &self,
        id: ResourceId,
    ) -> Result<(ResourceType, Vec<ResourcePathId>), Error> {
        let meta = self.read_meta(id)?;
        let dependencies = meta.dependencies;

        Ok((meta.type_id, dependencies))
    }

    /// Returns type of the resource.
    pub fn resource_type(&self, id: ResourceId) -> Result<ResourceType, Error> {
        let meta = self.read_meta(id)?;
        Ok(meta.type_id)
    }

    /// Returns the name of the resource from its `.meta` file.
    pub fn resource_name(&self, id: ResourceId) -> Result<ResourcePathName, Error> {
        let meta = self.read_meta(id)?;
        if let Some((resource_id, suffix)) = meta
            .name
            .as_str()
            .strip_prefix("/!")
            .and_then(|v| v.split_once('/'))
        {
            if let Ok(type_id) = <ResourceTypeAndId as std::str::FromStr>::from_str(resource_id) {
                if let Ok(mut parent_path) = self.resource_name(type_id.id) {
                    parent_path.push(suffix);
                    return Ok(parent_path);
                }
            }
        }
        Ok(meta.name)
    }

    /// Returns the name of the resource from its `.meta` file.
    pub async fn deleted_resource_info(
        &mut self,
        id: ResourceId,
    ) -> Result<(ResourcePathName, ResourceType), Error> {
        let resource_path = self.resource_path(id);
        match self.deleted_pending.entry(id) {
            Entry::Vacant(entry) => {
                let tree = self.workspace.get_staged_changes().await?;

                let resource_lsc_path =
                    CanonicalPath::new_from_canonical_paths(self.workspace.root(), &resource_path)
                        .map_err(|err| {
                            error!(
                                "Failed to retrieve delete info for Resource {}: {}",
                                id, err
                            );
                            Error::SourceControl(err)
                        })?;

                if let Some(lgn_source_control::ChangeType::Delete { old_id }) = tree
                    .get(&resource_lsc_path)
                    .map(lgn_source_control::Change::change_type)
                {
                    match self.workspace.provider().read(old_id).await {
                        Ok(data) => {
                            let resource: Value = serde_json::from_slice(&data).unwrap();
                            let meta: Metadata =
                                serde_json::from_value(resource["meta"].clone()).unwrap();
                            let value = (meta.name, meta.type_id);
                            entry.insert(value.clone());
                            return Ok(value);
                        }
                        Err(err) => {
                            error!(
                                "Failed to retrieve delete info for Resource {}: {}",
                                id, err
                            );
                        }
                    }
                }

                Err(Error::FileNotFound(resource_lsc_path.to_string()))
            }
            Entry::Occupied(entry) => Ok(entry.get().clone()),
        }
    }

    /// Returns the raw name of the resource from its `.meta` file.
    pub fn raw_resource_name(&self, id: ResourceId) -> Result<ResourcePathName, Error> {
        let meta = self.read_meta(id)?;
        Ok(meta.name)
    }

    /// Returns the type name of the resource from its `.meta` file.
    pub fn resource_type_name(&self, id: ResourceId) -> Result<String, Error> {
        let meta = self.read_meta(id)?;
        Ok(meta.type_name)
    }

    /// Returns the root directory where resources are located.
    pub fn resource_dir(&self) -> PathBuf {
        self.resource_dir.clone()
    }

    fn resource_path(&self, id: ResourceId) -> PathBuf {
        self.resource_dir().join(id.resource_path())
    }

    /// Moves a `remote` resources to the list of `local` resources.
    pub async fn checkout(&mut self, id: ResourceTypeAndId) -> Result<(), Error> {
        let resource_path = self.resource_path(id.id);
        self.workspace
            .checkout_files([resource_path.as_path()])
            .await
            .map_err(Error::SourceControl)
            .map(|_e| ())
    }

    fn read_meta(&self, id: ResourceId) -> Result<Metadata, Error> {
        let resource_path = self.resource_path(id);

        let file = OpenOptions::new()
            .read(true)
            .open(&resource_path)
            .map_err(|e| Error::Io(resource_path.clone(), e))?;

        let resource: Value = serde_json::from_reader(&file)?;
        let meta: Metadata = serde_json::from_value(resource["meta"].clone())?;

        Ok(meta)
    }

    async fn update_meta<F>(&self, id: ResourceId, mut func: F)
    where
        F: FnMut(&mut Metadata),
    {
        let resource_path = self.resource_path(id);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&resource_path)
            .unwrap(); // todo(kstasik): return a result and propagate an error

        let mut resource: Value = serde_json::from_reader(&file).unwrap();
        let mut meta: Metadata = serde_json::from_value(resource["meta"].clone()).unwrap();

        func(&mut meta);

        resource["meta"] = serde_json::to_value(meta).unwrap();

        file.set_len(0).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&file, &resource).unwrap();

        self.workspace
            .checkout_files([resource_path.as_path()])
            .await
            .map_err(Error::SourceControl)
            .unwrap();
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
        self.checkout(type_id).await?;

        let mut old_name: Option<ResourcePathName> = None;
        self.update_meta(type_id.id, |data| {
            old_name = Some(data.rename(new_name));
        })
        .await;

        let resource_path = self.resource_path(type_id.id);
        self.workspace
            .add_files([resource_path.as_path()]) // add
            .await
            .map_err(Error::SourceControl)?;

        Ok(old_name.unwrap())
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
    pub async fn sync_latest(&mut self) -> Result<Vec<(ResourceId, ChangeType)>, Error> {
        let (_, changed) = self.workspace.sync().await.map_err(Error::SourceControl)?;

        let resources = changed
            .iter()
            .filter_map(|change| {
                let id = change
                    .canonical_path()
                    .name()
                    .map(|filename| (ResourceId::from_str(filename).unwrap()));

                id.map(|id| (id, change.change_type().into()))
            })
            .collect::<Vec<_>>();
        Ok(resources)
    }

    /// Returns the current state of the workspace that includes staged changes.
    pub async fn tree(&self) -> Result<Tree, Error> {
        let remote = self
            .workspace
            .get_current_tree()
            .await
            .map_err(Error::SourceControl)?;

        let staged_changes = self
            .workspace
            .get_staged_changes()
            .await
            .map_err(Error::SourceControl)?;

        let local = remote
            .with_changes(staged_changes.values())
            .map_err(Error::SourceControl)?;
        Ok(local)
    }

    /// Returns the checksum of the root project directory at the current state.
    pub async fn root_checksum(&self) -> Result<String, Error> {
        let tree = self.tree().await?;
        Ok(tree.id())
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
    use std::sync::Arc;

    use lgn_content_store::Provider;
    use lgn_data_runtime::{
        resource, AssetRegistry, AssetRegistryOptions, Metadata, Resource, ResourceDescriptor,
        ResourcePathId, ResourcePathName, ResourceProcessor, ResourceProcessorError, ResourceType,
    };
    use serde::{Deserialize, Serialize};

    use crate::resource::project::Project;

    const RESOURCE_TEXTURE: &str = "texture";
    const RESOURCE_MATERIAL: &str = "material";
    const RESOURCE_GEOMETRY: &str = "geometry";
    const RESOURCE_SKELETON: &str = "skeleton";
    const RESOURCE_ACTOR: &str = "actor";

    #[resource("null")]
    #[derive(Clone, Serialize, Deserialize)]
    struct NullResource {
        meta: Metadata,
        content: isize,
        dependencies: Vec<ResourcePathId>,
    }

    struct NullResourceProc {}
    impl ResourceProcessor for NullResourceProc {
        fn new_resource(&mut self) -> Box<dyn Resource> {
            Box::new(NullResource {
                meta: Metadata::new(
                    ResourcePathName::default(),
                    NullResource::TYPENAME,
                    NullResource::TYPE,
                ),
                content: 0,
                dependencies: vec![],
            })
        }

        fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<ResourcePathId> {
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
            serde_json::to_writer_pretty(writer, resource).unwrap();
            Ok(1) // no bytes written exposed by serde.
        }

        fn read_resource(
            &mut self,
            reader: &mut dyn std::io::Read,
        ) -> Result<Box<dyn Resource>, ResourceProcessorError> {
            let resource: NullResource = serde_json::from_reader(reader).unwrap();
            Ok(Box::new(resource))
        }
    }

    #[allow(clippy::too_many_lines)]
    async fn create_actor(project: &mut Project) -> Arc<AssetRegistry> {
        let resources = AssetRegistryOptions::new()
            .add_processor_ext(
                ResourceType::new(RESOURCE_TEXTURE.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_processor_ext(
                ResourceType::new(RESOURCE_MATERIAL.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_processor_ext(
                ResourceType::new(RESOURCE_GEOMETRY.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_processor_ext(
                ResourceType::new(RESOURCE_SKELETON.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_processor_ext(
                ResourceType::new(RESOURCE_ACTOR.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .create()
            .await;

        let texture_type = ResourceType::new(RESOURCE_TEXTURE.as_bytes());
        let texture = project
            .add_resource(
                ResourcePathName::new("albedo.texture"),
                RESOURCE_TEXTURE,
                texture_type,
                resources.new_resource(texture_type).unwrap(),
                &resources,
            )
            .await
            .unwrap();

        let material_type = ResourceType::new(RESOURCE_MATERIAL.as_bytes());
        let material = resources
            .new_resource(material_type)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = material.instantiate(&resources).unwrap();
        edit.dependencies.push(ResourcePathId::from(texture));
        material.apply(edit, &resources);

        let material = project
            .add_resource(
                ResourcePathName::new("body.material"),
                RESOURCE_MATERIAL,
                material_type,
                &material,
                &resources,
            )
            .await
            .unwrap();

        let geometry_type = ResourceType::new(RESOURCE_GEOMETRY.as_bytes());
        let geometry = resources
            .new_resource(geometry_type)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = geometry.instantiate(&resources).unwrap();
        edit.dependencies.push(ResourcePathId::from(material));
        geometry.apply(edit, &resources);
        let geometry = project
            .add_resource(
                ResourcePathName::new("hero.geometry"),
                RESOURCE_GEOMETRY,
                geometry_type,
                &geometry,
                &resources,
            )
            .await
            .unwrap();

        let skeleton_type = ResourceType::new(RESOURCE_SKELETON.as_bytes());
        let skeleton = project
            .add_resource(
                ResourcePathName::new("hero.skeleton"),
                RESOURCE_SKELETON,
                skeleton_type,
                &resources.new_resource(skeleton_type).unwrap(),
                &resources,
            )
            .await
            .unwrap();

        let actor_type = ResourceType::new(RESOURCE_ACTOR.as_bytes());
        let actor = resources
            .new_resource(actor_type)
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
                actor_type,
                &actor,
                &resources,
            )
            .await
            .unwrap();

        resources
    }

    async fn create_sky_material(project: &mut Project, resources: &AssetRegistry) {
        let texture_type = ResourceType::new(RESOURCE_TEXTURE.as_bytes());
        let texture = project
            .add_resource(
                ResourcePathName::new("sky.texture"),
                RESOURCE_TEXTURE,
                texture_type,
                &resources.new_resource(texture_type).unwrap(),
                resources,
            )
            .await
            .unwrap();

        let material_type = ResourceType::new(RESOURCE_MATERIAL.as_bytes());
        let material = resources
            .new_resource(material_type)
            .unwrap()
            .typed::<NullResource>();
        let mut edit = material.instantiate(resources).unwrap();
        edit.dependencies.push(ResourcePathId::from(texture));
        material.apply(edit, resources);

        let _material = project
            .add_resource(
                ResourcePathName::new("sky.material"),
                RESOURCE_MATERIAL,
                material_type,
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

        let project = Project::create_with_remote_mock(root.path())
            .await
            .expect("failed to create project");
        let same_project = Project::create_with_remote_mock(root.path()).await;
        assert!(same_project.is_err());

        project.delete().await;

        let _project = Project::create_with_remote_mock(root.path())
            .await
            .expect("failed to re-create project");
        let same_project = Project::create_with_remote_mock(root.path()).await;
        assert!(same_project.is_err());
    }*/

    #[tokio::test]
    async fn local_changes() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let _resources = create_actor(&mut project).await;

        assert_eq!(project.local_resource_list().await.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn commit() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 5);
        assert_eq!(project.remote_resource_list().await.unwrap().len(), 0);

        // modify before commit
        {
            let handle = project.load_resource(actor_id, &resources).unwrap();
            let mut content = handle.instantiate::<NullResource>(&resources).unwrap();
            content.content = 8;
            handle.apply(content, &resources);

            project
                .save_resource(actor_id, &handle, &resources)
                .await
                .unwrap();
        }

        project.commit("add resources").await.unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 0);
        assert_eq!(project.remote_resource_list().await.unwrap().len(), 5);

        // modify resource
        {
            let handle = project.load_resource(actor_id, &resources).unwrap();
            let mut content = handle.instantiate::<NullResource>(&resources).unwrap();
            assert_eq!(content.content, 8);
            content.content = 9;
            handle.apply(content, &resources);
            project
                .save_resource(actor_id, &handle, &resources)
                .await
                .unwrap();

            assert_eq!(project.local_resource_list().await.unwrap().len(), 1);
        }

        project.commit("update actor").await.unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn change_to_previous() {
        let root = tempfile::tempdir().unwrap();
        let provider = Arc::new(Provider::new_in_memory());
        let mut project = Project::create_with_remote_mock(root.path(), provider)
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
            let handle = project.load_resource(actor_id, &resources).unwrap();
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
            let handle = project.load_resource(actor_id, &resources).unwrap();
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
        let mut project = Project::create_with_remote_mock(root.path(), provider)
            .await
            .expect("new project");
        let _resources = create_actor(&mut project).await;

        let top_level_resource = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        let (_, dependencies) = project.resource_info(top_level_resource.id).unwrap();

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
        let mut project = Project::create_with_remote_mock(root.path(), provider)
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
