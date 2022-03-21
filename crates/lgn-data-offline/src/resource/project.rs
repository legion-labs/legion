use core::fmt;
use std::{
    fs::{self, File, OpenOptions},
    io::Seek,
    path::{Path, PathBuf},
    str::FromStr,
};

use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_source_control::{
    Change, CommitMode, IndexBackend, LocalIndexBackend, Workspace, WorkspaceConfig,
    WorkspaceRegistration,
};
use thiserror::Error;

use crate::resource::{
    metadata::Metadata, ResourceHandleUntyped, ResourcePathName, ResourceRegistry,
};
use crate::ResourcePathId;

const METADATA_EXT: &str = "meta";

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
/// - Checksum of resource's content file.
///
/// Note: Resource's [`ResourcePathName`] is only used for display purposes and
/// can be changed freely.
///
/// For more about loading, saving and managing resources in memory see
/// [`ResourceRegistry`]
pub struct Project {
    project_dir: PathBuf,
    resource_dir: PathBuf,
    local_remote: Option<LocalIndexBackend>,
    workspace: Workspace,
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
    #[error("Source Control Error: '{0}'")]
    SourceControl(#[source] lgn_source_control::Error),
    /// RegistryRegistry Error
    #[error("ResourceRegistry Error: '{1}' on resource '{0}'")]
    ResourceRegistry(
        ResourceTypeAndId,
        #[source] crate::resource::ResourceRegistryError,
    ),
}

impl Project {
    /// Returns directory the project is in, relative to CWD.
    pub fn project_dir(&self) -> &Path {
        &self.project_dir
    }

    /// Creates a local source control index.
    pub async fn create_local_origin(path: impl AsRef<Path>) -> Result<LocalIndexBackend, Error> {
        let remote = LocalIndexBackend::new(&path).map_err(Error::SourceControl)?;
        if !remote.index_exists().await.map_err(Error::SourceControl)? {
            remote.create_index().await.map_err(Error::SourceControl)?;
        }
        Ok(remote)
    }

    /// Same as [`Self::create`] but it creates an origin source control index at ``project_dir/remote``.
    pub async fn create_with_remote_mock(project_dir: impl AsRef<Path>) -> Result<Self, Error> {
        let remote_dir = project_dir.as_ref().join("remote");
        let remote = Self::create_local_origin(&remote_dir).await?;
        let content_store_configuration = lgn_content_store2::Config::local(&remote_dir);

        let mut project = Self::create(
            project_dir,
            "../remote".to_string(),
            content_store_configuration,
        )
        .await?;
        project.local_remote = Some(remote);
        Ok(project)
    }

    /// Creates a new project index file turning the containing directory into a
    /// project.
    pub async fn create(
        project_dir: impl AsRef<Path>,
        remote_path: String,
        content_store_configuration: lgn_content_store2::Config,
    ) -> Result<Self, Error> {
        let resource_dir = project_dir.as_ref().join("offline");
        if !resource_dir.exists() {
            std::fs::create_dir(&resource_dir).map_err(|e| Error::Io(resource_dir.clone(), e))?;
        }

        let content_provider = content_store_configuration
            .instanciate_provider()
            .await
            .map_err(|e| {
                Error::SourceControl(lgn_source_control::Error::Other {
                    source: anyhow::Error::new(e),
                    context: "failed to instanciate content-store provider".to_string(),
                })
            })?;

        let workspace = Workspace::init(
            &resource_dir,
            WorkspaceConfig::new(remote_path, WorkspaceRegistration::new_with_current_user())
                .with_content_store_configuration(content_store_configuration),
            content_provider,
        )
        .await
        .map_err(|e| {
            Error::SourceControl(lgn_source_control::Error::Other {
                source: anyhow::Error::new(e),
                context: "Workspace::init".to_string(),
            })
        })?;

        Ok(Self {
            project_dir: project_dir.as_ref().to_owned(),
            resource_dir,
            local_remote: None,
            workspace,
        })
    }

    /// Opens the project index specified
    pub async fn open(project_dir: impl AsRef<Path>) -> Result<Self, Error> {
        let resource_dir = project_dir.as_ref().join("offline");

        let workspace = Workspace::load(&resource_dir).await.map_err(|e| {
            Error::SourceControl(lgn_source_control::Error::Other {
                source: anyhow::Error::new(e),
                context: "Workspace::load".to_string(),
            })
        })?;

        Ok(Self {
            project_dir: project_dir.as_ref().to_owned(),
            resource_dir,
            local_remote: None,
            workspace,
        })
    }

    /// Deletes the project by deleting the index file.
    pub async fn delete(self) {
        std::fs::remove_dir_all(self.resource_dir()).unwrap_or(());

        if let Some(remote) = &self.local_remote {
            remote.destroy_index().await.unwrap();
        }
    }

    /// Return the list of stages resources
    pub async fn get_staged_changes(&self) -> Result<Vec<(ResourceId, Change)>, Error> {
        let local_changes = self
            .workspace
            .get_staged_changes()
            .await
            .map_err(Error::SourceControl)?;

        let changes = local_changes
            .iter()
            .map(|(path, change)| (PathBuf::from(path.to_string()), change))
            .filter(|(path, _)| path.extension().is_none())
            .map(|(path, change)| {
                (
                    ResourceId::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap(),
                    change.clone(),
                )
            })
            .collect::<Vec<_>>();

        Ok(changes)
    }

    /// Return the list of local resources
    pub async fn local_resource_list(&self) -> Result<Vec<ResourceId>, Error> {
        let local_changes = self
            .workspace
            .get_staged_changes()
            .await
            .map_err(Error::SourceControl)?;

        let changes = local_changes
            .iter()
            .map(|(path, _)| PathBuf::from(path.to_string()))
            .filter(|path| path.extension().is_none())
            .map(|path| ResourceId::from_str(path.file_name().unwrap().to_str().unwrap()).unwrap())
            .collect::<Vec<_>>();

        Ok(changes)
    }

    async fn remote_resource_list(&self) -> Result<Vec<ResourceId>, Error> {
        let tree = self
            .workspace
            .get_current_tree()
            .await
            .map_err(Error::SourceControl)?;

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
        let res = self
            .resource_list()
            .await
            .into_iter()
            .find_map(|id| match self.read_meta(id) {
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
                Err(e) => Some(Err(e)),
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
        kind_name: &str,
        kind: ResourceType,
        handle: impl AsRef<ResourceHandleUntyped>,
        registry: &mut ResourceRegistry,
    ) -> Result<ResourceTypeAndId, Error> {
        self.add_resource_with_id(name, kind_name, kind, ResourceId::new(), handle, registry)
            .await
    }

    /// Add a given resource of a given type and id with an associated `.meta`.
    ///
    /// The created `.meta` file contains a checksum of the resource content.
    /// `TODO`: the checksum of content needs to be updated when file is
    /// modified.
    ///
    /// Both resource file and its corresponding `.meta` file are `staged`.
    /// Use [`Self::commit()`] to push changes to remote.
    pub async fn add_resource_with_id(
        &mut self,
        name: ResourcePathName,
        kind_name: &str,
        kind: ResourceType,
        id: ResourceId,
        handle: impl AsRef<ResourceHandleUntyped>,
        registry: &mut ResourceRegistry,
    ) -> Result<ResourceTypeAndId, Error> {
        let meta_path = self.metadata_path(id);
        let resource_path = self.resource_path(id);

        let directory = {
            let mut directory = resource_path.clone();
            directory.pop();
            directory
        };

        std::fs::create_dir_all(&directory).map_err(|e| Error::Io(directory.clone(), e))?;

        let build_dependencies = {
            let mut resource_file =
                File::create(&resource_path).map_err(|e| Error::Io(resource_path.clone(), e))?;

            let (_written, build_deps) = registry
                .serialize_resource(kind, handle, &mut resource_file)
                .map_err(|e| Error::ResourceRegistry(ResourceTypeAndId { kind, id }, e))?;
            build_deps
        };

        let meta_file = File::create(&meta_path).map_err(|e| {
            fs::remove_file(&resource_path).unwrap();
            Error::Io(meta_path.clone(), e)
        })?;

        let metadata = Metadata::new_with_dependencies(name, kind_name, kind, &build_dependencies);
        serde_json::to_writer_pretty(meta_file, &metadata).unwrap();

        let content_provider = self
            .workspace
            .instanciate_content_store_provider()
            .await
            .map_err(|e| {
                Error::SourceControl(lgn_source_control::Error::Other {
                    source: anyhow::Error::new(e),
                    context: "failed to instanciate content-store provider".to_string(),
                })
            })?;

        self.workspace
            .add_files(
                content_provider,
                [meta_path.as_path(), resource_path.as_path()],
            )
            .await
            .map_err(Error::SourceControl)?;

        let type_id = ResourceTypeAndId { kind, id };

        Ok(type_id)
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub async fn delete_resource(&mut self, id: ResourceId) -> Result<(), Error> {
        let resource_path = self.resource_path(id);
        let metadata_path = self.metadata_path(id);

        {
            let files = [metadata_path.as_path(), resource_path.as_path()];

            self.workspace
                .delete_files(files)
                .await
                .map_err(Error::SourceControl)?;
        }

        Ok(())
    }

    /// Writes the resource behind `handle` from memory to disk and updates the
    /// corresponding .meta file.
    pub async fn save_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<ResourceHandleUntyped>,
        resources: &mut ResourceRegistry,
    ) -> Result<(), Error> {
        let resource_path = self.resource_path(type_id.id);
        let metadata_path = self.metadata_path(type_id.id);

        self.checkout(type_id).await?;

        let mut meta_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&metadata_path)
            .map_err(|e| Error::Io(metadata_path.clone(), e))?;
        let mut metadata: Metadata = serde_json::from_reader(&meta_file)
            .map_err(|e| Error::Parse(metadata_path.clone(), e))?;

        let build_dependencies = {
            let mut resource_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&resource_path)
                .map_err(|e| Error::Io(resource_path.clone(), e))?;

            let (_written, build_deps) = resources
                .serialize_resource(type_id.kind, handle, &mut resource_file)
                .map_err(|e| Error::ResourceRegistry(type_id, e))?;
            build_deps
        };

        metadata.dependencies = build_dependencies;

        meta_file.set_len(0).unwrap();
        meta_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&meta_file, &metadata).unwrap(); // todo(kstasik): same as above.

        let content_provider = self
            .workspace
            .instanciate_content_store_provider()
            .await
            .map_err(|e| {
                Error::SourceControl(lgn_source_control::Error::Other {
                    source: anyhow::Error::new(e),
                    context: "failed to instanciate content-store provider".to_string(),
                })
            })?;

        self.workspace
            .add_files(
                content_provider,
                [metadata_path.as_path(), resource_path.as_path()],
            ) // add
            .await
            .map_err(Error::SourceControl)?;

        Ok(())
    }

    /// Loads a resource of a given id.
    ///
    /// In-memory representation of that resource is managed by
    /// `ResourceRegistry`. In order to update the resource on disk see
    /// [`Self::save_resource()`].
    pub fn load_resource(
        &self,
        type_id: ResourceTypeAndId,
        resources: &mut ResourceRegistry,
    ) -> Result<ResourceHandleUntyped, Error> {
        let resource_path = self.resource_path(type_id.id);

        let mut resource_file =
            File::open(&resource_path).map_err(|e| Error::Io(resource_path.clone(), e))?;
        let handle = resources
            .deserialize_resource(type_id.kind, &mut resource_file)
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

    /// Returns the raw name of the resource from its `.meta` file.
    pub fn raw_resource_name(&self, type_id: ResourceId) -> Result<ResourcePathName, Error> {
        let meta = self.read_meta(type_id)?;
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

    fn metadata_path(&self, id: ResourceId) -> PathBuf {
        let mut path = self.resource_dir();
        path.push(id.resource_path());
        path.set_extension(METADATA_EXT);
        path
    }

    fn resource_path(&self, id: ResourceId) -> PathBuf {
        self.resource_dir().join(id.resource_path())
    }

    /// Moves a `remote` resources to the list of `local` resources.
    pub async fn checkout(&mut self, id: ResourceTypeAndId) -> Result<(), Error> {
        let metadata_path = self.metadata_path(id.id);
        let resource_path = self.resource_path(id.id);
        self.workspace
            .checkout_files([metadata_path.as_path(), resource_path.as_path()])
            .await
            .map_err(Error::SourceControl)
            .map(|_e| ())
    }

    fn read_meta(&self, id: ResourceId) -> Result<Metadata, Error> {
        let path = self.metadata_path(id);

        let file = File::open(&path).map_err(|e| Error::Io(path.clone(), e))?;

        let result = serde_json::from_reader(file).map_err(|e| Error::Parse(path, e))?;
        Ok(result)
    }

    async fn update_meta<F>(&self, id: ResourceId, mut func: F)
    where
        F: FnMut(&mut Metadata),
    {
        let path = self.metadata_path(id);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap(); // todo(kstasik): return a result and propagate an error

        let mut meta = serde_json::from_reader(&file).unwrap();

        func(&mut meta);

        file.set_len(0).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&file, &meta).unwrap();

        {
            self.workspace
                .checkout_files([path.as_path()])
                .await
                .map_err(Error::SourceControl)
                .unwrap();
        }
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
        Ok(old_name.unwrap())
    }

    /// Moves `local` resources to `remote` resource list.
    pub async fn commit(&mut self, message: &str) -> Result<(), Error> {
        self.workspace
            .commit(message, CommitMode::Lenient)
            .await
            .map_err(Error::SourceControl)
            .map(|_| ())
    }

    /// Pulls all changes from the origin.
    pub async fn sync_latest(&mut self) -> Result<(), Error> {
        let content_provider = self
            .workspace
            .instanciate_content_store_provider()
            .await
            .map_err(|e| {
                Error::SourceControl(lgn_source_control::Error::Other {
                    source: anyhow::Error::new(e),
                    context: "failed to instanciate content-store provider".to_string(),
                })
            })?;

        self.workspace
            .sync(content_provider)
            .await
            .map_err(Error::SourceControl)
            .map(|_| ())
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
    use std::any::Any;
    use std::str::FromStr;
    use std::sync::Arc;

    use lgn_data_runtime::{resource, Resource, ResourceType};
    use tokio::sync::Mutex;

    use crate::resource::project::Project;
    use crate::{
        resource::{
            ResourcePathName, ResourceProcessor, ResourceProcessorError, ResourceRegistry,
            ResourceRegistryOptions,
        },
        ResourcePathId,
    };

    const RESOURCE_TEXTURE: &str = "texture";
    const RESOURCE_MATERIAL: &str = "material";
    const RESOURCE_GEOMETRY: &str = "geometry";
    const RESOURCE_SKELETON: &str = "skeleton";
    const RESOURCE_ACTOR: &str = "actor";

    #[resource("null")]
    struct NullResource {
        content: isize,
        dependencies: Vec<ResourcePathId>,
    }

    struct NullResourceProc {}
    impl ResourceProcessor for NullResourceProc {
        fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
            Box::new(NullResource {
                content: 0,
                dependencies: vec![],
            })
        }

        fn extract_build_dependencies(&mut self, resource: &dyn Any) -> Vec<ResourcePathId> {
            resource
                .downcast_ref::<NullResource>()
                .unwrap()
                .dependencies
                .clone()
        }

        fn write_resource(
            &self,
            resource: &dyn Any,
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
        ) -> Result<Box<dyn Any + Send + Sync>, ResourceProcessorError> {
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
    async fn create_actor(project: &mut Project) -> Arc<Mutex<ResourceRegistry>> {
        let resources_arc = ResourceRegistryOptions::new()
            .add_type_processor(
                ResourceType::new(RESOURCE_TEXTURE.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_type_processor(
                ResourceType::new(RESOURCE_MATERIAL.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_type_processor(
                ResourceType::new(RESOURCE_GEOMETRY.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_type_processor(
                ResourceType::new(RESOURCE_SKELETON.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .add_type_processor(
                ResourceType::new(RESOURCE_ACTOR.as_bytes()),
                Box::new(NullResourceProc {}),
            )
            .create_async_registry();

        let mut resources = resources_arc.lock().await;
        let texture_type = ResourceType::new(RESOURCE_TEXTURE.as_bytes());
        let texture = project
            .add_resource(
                ResourcePathName::new("albedo.texture"),
                RESOURCE_TEXTURE,
                texture_type,
                &resources.new_resource(texture_type).unwrap(),
                &mut resources,
            )
            .await
            .unwrap();

        let material_type = ResourceType::new(RESOURCE_MATERIAL.as_bytes());
        let material = resources
            .new_resource(material_type)
            .unwrap()
            .typed::<NullResource>();
        material
            .get_mut(&mut resources)
            .unwrap()
            .dependencies
            .push(ResourcePathId::from(texture));
        let material = project
            .add_resource(
                ResourcePathName::new("body.material"),
                RESOURCE_MATERIAL,
                material_type,
                &material,
                &mut resources,
            )
            .await
            .unwrap();

        let geometry_type = ResourceType::new(RESOURCE_GEOMETRY.as_bytes());
        let geometry = resources
            .new_resource(geometry_type)
            .unwrap()
            .typed::<NullResource>();
        geometry
            .get_mut(&mut resources)
            .unwrap()
            .dependencies
            .push(ResourcePathId::from(material));
        let geometry = project
            .add_resource(
                ResourcePathName::new("hero.geometry"),
                RESOURCE_GEOMETRY,
                geometry_type,
                &geometry,
                &mut resources,
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
                &mut resources,
            )
            .await
            .unwrap();

        let actor_type = ResourceType::new(RESOURCE_ACTOR.as_bytes());
        let actor = resources
            .new_resource(actor_type)
            .unwrap()
            .typed::<NullResource>();
        actor.get_mut(&mut resources).unwrap().dependencies = vec![
            ResourcePathId::from(geometry),
            ResourcePathId::from(skeleton),
        ];
        let _actor = project
            .add_resource(
                ResourcePathName::new("hero.actor"),
                RESOURCE_ACTOR,
                actor_type,
                &actor,
                &mut resources,
            )
            .await
            .unwrap();

        drop(resources);
        resources_arc
    }

    async fn create_sky_material(project: &mut Project, resources: &mut ResourceRegistry) {
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
        material
            .get_mut(resources)
            .unwrap()
            .dependencies
            .push(ResourcePathId::from(texture));

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
        let mut project = Project::create_with_remote_mock(root.path())
            .await
            .expect("new project");
        let _resources = create_actor(&mut project).await;

        assert_eq!(project.local_resource_list().await.unwrap().len(), 5);
    }

    #[tokio::test]
    async fn commit() {
        let root = tempfile::tempdir().unwrap();
        let mut project = Project::create_with_remote_mock(root.path())
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;
        let mut resources = resources.lock().await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 5);
        assert_eq!(project.remote_resource_list().await.unwrap().len(), 0);

        // modify before commit
        {
            let handle = project.load_resource(actor_id, &mut resources).unwrap();
            let content = handle.get_mut::<NullResource>(&mut resources).unwrap();
            content.content = 8;
            project
                .save_resource(actor_id, &handle, &mut resources)
                .await
                .unwrap();
        }

        project.commit("add resources").await.unwrap();

        assert_eq!(project.local_resource_list().await.unwrap().len(), 0);
        assert_eq!(project.remote_resource_list().await.unwrap().len(), 5);

        // modify resource
        {
            let handle = project.load_resource(actor_id, &mut resources).unwrap();
            let content = handle.get_mut::<NullResource>(&mut resources).unwrap();
            assert_eq!(content.content, 8);
            content.content = 9;
            project
                .save_resource(actor_id, &handle, &mut resources)
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
        let mut project = Project::create_with_remote_mock(root.path())
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;
        let mut resources = resources.lock().await;

        let actor_id = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .await
            .unwrap();

        project.commit("initial actor").await.unwrap();

        // modify resource
        let original_content = {
            let handle = project.load_resource(actor_id, &mut resources).unwrap();
            let content = handle.get_mut::<NullResource>(&mut resources).unwrap();
            let previous_value = content.content;
            content.content = 9;
            project
                .save_resource(actor_id, &handle, &mut resources)
                .await
                .unwrap();

            previous_value
        };

        {
            let handle = project.load_resource(actor_id, &mut resources).unwrap();
            let content = handle.get_mut::<NullResource>(&mut resources).unwrap();
            content.content = original_content;
            project
                .save_resource(actor_id, &handle, &mut resources)
                .await
                .unwrap();
        }

        project.commit("no changes").await.unwrap();
    }

    #[tokio::test]
    async fn immediate_dependencies() {
        let root = tempfile::tempdir().unwrap();
        let mut project = Project::create_with_remote_mock(root.path())
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
        let mut project = Project::create_with_remote_mock(root.path())
            .await
            .expect("new project");
        let resources = create_actor(&mut project).await;
        assert!(project.commit("rename test").await.is_ok());
        create_sky_material(&mut project, &mut *resources.lock().await).await;

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
