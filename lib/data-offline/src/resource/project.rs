use core::fmt;
use std::{
    fs::{self, File, OpenOptions},
    io::Seek,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use lgn_content_store::content_checksum_from_read;
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use serde::{Deserialize, Serialize};

use crate::resource::{
    metadata::{Metadata, ResourceHash},
    ResourceHandleUntyped, ResourcePathName, ResourceRegistry,
};
use crate::ResourcePathId;

const METADATA_EXT: &str = "meta";

/// A project exists always within a given directory and this file
/// will be created directly in that directory.
const PROJECT_INDEX_FILENAME: &str = "project.index";

#[derive(Serialize, Deserialize, Default)]
struct ResourceDb {
    remote_resources: Vec<ResourceTypeAndId>,
    local_resources: Vec<ResourceTypeAndId>,
}

impl ResourceDb {
    // sort contents so serialization is deterministic
    fn pre_serialize(&mut self) {
        self.remote_resources.sort();
        self.local_resources.sort();
    }
}

/// A file-backed state of the project
///
/// This structure captures the state of the project. This includes `remote resources`
/// pulled from `source-control` as well as `local resources` added/removed/edited locally.
///
/// It provides a resource-oriented interface to source-control.
///
/// # Project Index
///
/// The state of the project is read from a file once [`Project`] is opened and kept in memory throughout its lifetime.
/// The changes are written back to the file once [`Project`] is dropped.
///
/// The state of a project consists of two sets of [`ResourceId`]s:
/// - Local [`ResourceId`] list - locally modified resources.
/// - Remote [`ResourceId`] list - synced resources.
///
/// A resource consists of a resource content file and a `.meta` file associated to it.
/// [`ResourceId`] is enough to locate a resource content file and its associated `.meta` file on disk.
///
/// ## Example directory structure
///
/// An example of a project with 2 offline resources on disk looks as follows:
/// ```markdown
///  ./
///  | + offline/
///  | |- a81fb4498cd04368
///  | |- a81fb4498cd04368.meta
///  | |- 8063daaf864780d6
///  | |- 8063daaf864780d6.meta
///  |- project.index
/// ```
///
/// ## Resource `.meta` file
///
/// The information in `.meta` file includes:
/// - List of [`ResourceId`]s of resource's build dependencies.
/// - Resource's name - [`ResourcePathName`].
/// - Checksum of resource's content file.
///
/// Note: Resource's [`ResourcePathName`] is only used for display purposes and can be changed freely.
///
/// For more about loading, saving and managing resources in memory see [`ResourceRegistry`]
pub struct Project {
    file: std::fs::File,
    db: ResourceDb,
    project_dir: PathBuf,
    resource_dir: PathBuf,
}

impl Project {
    /// Returns the default location of the index file in a given directory.
    ///
    /// This method replaces the filename in `work_dir` (if one exists) with
    /// the file name of the project index.
    pub fn root_to_index_path(project_dir: impl AsRef<Path>) -> PathBuf {
        let mut path = project_dir.as_ref().to_owned();
        if path.is_dir() {
            path.push(PROJECT_INDEX_FILENAME);
        } else {
            path.set_file_name(PROJECT_INDEX_FILENAME);
        }
        path
    }

    /// Returns the path to project's index file.
    pub fn indexfile_path(&self) -> PathBuf {
        Self::root_to_index_path(&self.project_dir)
    }

    /// Creates a new project index file turning the containing directory into a project.
    pub fn create_new(project_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let index_path = Self::root_to_index_path(project_dir.as_ref());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&index_path)
            .with_context(|| format!("Creating file '{:?}'", &index_path))?;

        let db = ResourceDb::default();
        serde_json::to_writer(&file, &db)
            .with_context(|| format!("Parsing file '{:?}'", &index_path))?;

        let project_dir = index_path.parent().unwrap().to_owned();
        let resource_dir = project_dir.join("offline");
        if !resource_dir.exists() {
            std::fs::create_dir(&resource_dir)
                .with_context(|| format!("Creating dir '{:?}'", &resource_dir))?;
        }

        Ok(Self {
            file,
            db,
            project_dir,
            resource_dir,
        })
    }

    /// Opens the project index specified
    pub fn open(project_dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let index_path = Self::root_to_index_path(project_dir.as_ref());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .open(&index_path)
            .with_context(|| format!("Opening '{:?}'", &index_path))?;

        let db = serde_json::from_reader(&file)
            .with_context(|| format!("Parsing file '{:?}'", &index_path))?;

        let project_dir = index_path.parent().unwrap().to_owned();
        let resource_dir = project_dir.join("offline");
        Ok(Self {
            file,
            db,
            project_dir,
            resource_dir,
        })
    }

    /// Deletes the project by deleting the index file.
    pub fn delete(self) {
        std::fs::remove_dir_all(self.resource_dir()).unwrap_or(());
        let index_path = self.indexfile_path();
        let _res = fs::remove_file(index_path);
    }

    /// Returns an iterator on the list of resources.
    ///
    /// This method flattens the `remote` and `local` resources into one list.
    pub fn resource_list(&self) -> impl Iterator<Item = ResourceTypeAndId> + '_ {
        self.db
            .remote_resources
            .iter()
            .chain(self.db.local_resources.iter())
            .copied()
    }

    /// Finds resource by its name and returns its `ResourceTypeAndId`.
    pub fn find_resource(&self, name: &ResourcePathName) -> anyhow::Result<ResourceTypeAndId> {
        // this below would be better expressed as try_map (still experimental).
        let res = self
            .resource_list()
            .find_map(|id| match self.read_meta(id) {
                Ok(meta) => {
                    if &meta.name == name {
                        Some(Ok(id))
                    } else {
                        None
                    }
                }
                Err(e) => Some(Err(
                    e.context(format!("Reading meta '{}' of resource: '{}'", id, name))
                )),
            });

        match res {
            None => Err(anyhow!("Failed to find resource '{}'", name)),
            Some(e) => e,
        }
    }

    /// Checks if a resource with a given name is part of the project.
    pub fn exists_named(&self, name: &ResourcePathName) -> bool {
        self.find_resource(name).is_ok()
    }

    /// Checks if a resource is part of the project.
    pub fn exists(&self, id: ResourceTypeAndId) -> bool {
        self.resource_list().any(|v| v == id)
    }

    /// Add a given resource of a given type with an associated `.meta`.
    ///
    /// The created `.meta` file contains a checksum of the resource content.
    /// `TODO`: the checksum of content needs to be updated when file is modified.
    ///
    /// Both resource file and its corresponding `.meta` file are `staged`.
    /// Use [`Self::commit()`] to push changes to remote.
    pub fn add_resource(
        &mut self,
        name: ResourcePathName,
        kind: ResourceType,
        handle: impl AsRef<ResourceHandleUntyped>,
        registry: &mut ResourceRegistry,
    ) -> anyhow::Result<ResourceTypeAndId> {
        let type_id = ResourceTypeAndId {
            t: kind,
            id: ResourceId::new(),
        };
        self.add_resource_with_id(name, kind, type_id, handle, registry)
    }

    /// Add a given resource of a given type and id with an associated `.meta`.
    ///
    /// The created `.meta` file contains a checksum of the resource content.
    /// `TODO`: the checksum of content needs to be updated when file is modified.
    ///
    /// Both resource file and its corresponding `.meta` file are `staged`.
    /// Use [`Self::commit()`] to push changes to remote.
    pub fn add_resource_with_id(
        &mut self,
        name: ResourcePathName,
        kind: ResourceType,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<ResourceHandleUntyped>,
        registry: &mut ResourceRegistry,
    ) -> anyhow::Result<ResourceTypeAndId> {
        let meta_path = self.metadata_path(type_id);
        let resource_path = self.resource_path(type_id);

        let resource_err_context = || format!("Adding resource at '{:?}'", &resource_path);
        let meta_err_context = || format!("Adding resource meta at '{:?}'", &meta_path);

        let build_dependencies = {
            let mut resource_file =
                File::create(&resource_path).with_context(resource_err_context)?;

            let (_written, build_deps) = registry
                .serialize_resource(kind, handle, &mut resource_file)
                .with_context(resource_err_context)?;
            build_deps
        };

        let content_checksum = {
            let mut resource_file =
                File::open(&resource_path).with_context(resource_err_context)?;
            content_checksum_from_read(&mut resource_file).with_context(resource_err_context)?
        };

        let meta_file = File::create(&meta_path)
            .map_err(|e| {
                fs::remove_file(&resource_path).unwrap();
                e
            })
            .with_context(meta_err_context)?;

        let metadata = Metadata::new_with_dependencies(name, content_checksum, &build_dependencies);
        serde_json::to_writer_pretty(meta_file, &metadata)
            .map_err(|e| {
                fs::remove_file(&meta_path).unwrap();
                fs::remove_file(&resource_path).unwrap();
                e
            })
            .with_context(meta_err_context)?;

        self.db.local_resources.push(type_id);
        Ok(type_id)
    }

    /// Delete the resource+meta files, remove from Registry and Flush index
    pub fn delete_resource(&mut self, type_id: ResourceTypeAndId) -> anyhow::Result<()> {
        let resource_path = self.resource_path(type_id);
        let metadata_path = self.metadata_path(type_id);

        std::fs::remove_file(&resource_path)
            .with_context(|| format!("Deleting '{:?}' of '{}'", resource_path, type_id))?;
        std::fs::remove_file(&metadata_path)
            .with_context(|| format!("Deleting '{:?}' of '{}'", metadata_path, type_id))?;

        self.db.local_resources.retain(|x| *x != type_id);
        self.db.remote_resources.retain(|x| *x != type_id);
        Ok(())
    }

    /// Writes the resource behind `handle` from memory to disk and updates the corresponding .meta file.
    pub fn save_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        handle: impl AsRef<ResourceHandleUntyped>,
        resources: &mut ResourceRegistry,
    ) -> anyhow::Result<()> {
        let resource_path = self.resource_path(type_id);
        let metadata_path = self.metadata_path(type_id);

        let mut meta_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&metadata_path)
            .with_context(|| format!("Opening '{:?}'", &metadata_path))?;
        let mut metadata: Metadata = serde_json::from_reader(&meta_file)
            .with_context(|| format!("Writing '{:?}'", &metadata_path))?;

        let build_dependencies = {
            let mut resource_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&resource_path)
                .with_context(|| format!("Opening '{:?}'", &resource_path))?;

            let (_written, build_deps) = resources
                .serialize_resource(type_id.t, handle, &mut resource_file)
                .with_context(|| format!("Writing '{:?}'", &resource_path))?;
            build_deps
        };

        let content_checksum = {
            let mut resource_file = File::open(&resource_path)
                .with_context(|| format!("Opening '{:?}' to calculate checksum", &resource_path))?;
            content_checksum_from_read(&mut resource_file)
                .with_context(|| format!("Reading '{:?}' to calculate checksum", &resource_path))?
        };

        metadata.content_checksum = content_checksum;
        metadata.dependencies = build_dependencies;

        meta_file.set_len(0).unwrap();
        meta_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&meta_file, &metadata)
            .with_context(|| format!("Writing '{:?}'", &metadata_path))?;
        Ok(())
    }

    /// Loads a resource of a given id.
    ///
    /// In-memory representation of that resource is managed by `ResourceRegistry`.
    /// In order to update the resource on disk see [`Self::save_resource()`].
    pub fn load_resource(
        &self,
        type_id: ResourceTypeAndId,
        resources: &mut ResourceRegistry,
    ) -> anyhow::Result<ResourceHandleUntyped> {
        let resource_path = self.resource_path(type_id);

        let mut resource_file = File::open(&resource_path)
            .with_context(|| format!("Opening '{:?}'", &resource_path))?;
        let handle = resources
            .deserialize_resource(type_id.t, &mut resource_file)
            .with_context(|| format!("Loading '{}' at '{:?}'", type_id, &resource_path))?;
        Ok(handle)
    }

    /// Returns information about a given resource from its `.meta` file.
    pub fn resource_info(
        &self,
        type_id: ResourceTypeAndId,
    ) -> anyhow::Result<(ResourceHash, Vec<ResourcePathId>)> {
        let meta = self.read_meta(type_id)?;
        let resource_hash = meta.resource_hash();
        let dependencies = meta.dependencies;

        Ok((resource_hash, dependencies))
    }

    /// Returns the name of the resource from its `.meta` file.
    pub fn resource_name(&self, type_id: ResourceTypeAndId) -> anyhow::Result<ResourcePathName> {
        let meta = self.read_meta(type_id)?;
        Ok(meta.name)
    }

    /// Returns the root directory where resources are located.
    pub fn resource_dir(&self) -> PathBuf {
        self.resource_dir.clone()
    }

    fn metadata_path(&self, type_id: ResourceTypeAndId) -> PathBuf {
        let mut path = self.resource_dir();
        path.push(type_id.to_string());
        path.set_extension(METADATA_EXT);
        path
    }

    fn resource_path(&self, type_id: ResourceTypeAndId) -> PathBuf {
        self.resource_dir().join(type_id.to_string())
    }

    /// Moves a `remote` resources to the list of `local` resources.
    pub fn checkout(&mut self, type_id: ResourceTypeAndId) -> anyhow::Result<()> {
        if let Some(_resource) = self.db.local_resources.iter().find(|&res| *res == type_id) {
            return Ok(()); // already checked out
        }

        if let Some(index) = self
            .db
            .remote_resources
            .iter()
            .position(|res| *res == type_id)
        {
            let resource = self.db.remote_resources.remove(index);
            self.db.local_resources.push(resource);
            return Ok(());
        }

        Err(anyhow!("Resource '{}' not found", type_id))
    }

    fn read_meta(&self, type_id: ResourceTypeAndId) -> anyhow::Result<Metadata> {
        let path = self.metadata_path(type_id);

        let file =
            File::open(&path).with_context(|| format!("Opening '{}' at '{:?}'", type_id, &path))?;

        let result = serde_json::from_reader(file)
            .with_context(|| format!("Reading '{}' at '{:?}'", type_id, &path))?;
        Ok(result)
    }

    fn update_meta<F>(&self, type_id: ResourceTypeAndId, mut func: F)
    where
        F: FnMut(&mut Metadata),
    {
        let path = self.metadata_path(type_id);

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap(); // todo(kstasik): return a result and propagate an error

        let mut meta = serde_json::from_reader(&file).unwrap();

        func(&mut meta);

        file.set_len(0).unwrap();
        file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&file, &meta).unwrap();
    }

    /// Change the name of the resource.
    ///
    /// Changing the name of the resource if `free`. It does not change its `ResourceId`
    /// nor it invalidates any build using that asset.
    pub fn rename_resource(
        &mut self,
        type_id: ResourceTypeAndId,
        new_name: &ResourcePathName,
    ) -> anyhow::Result<ResourcePathName> {
        self.checkout(type_id)?;

        let mut old_name: Option<ResourcePathName> = None;
        self.update_meta(type_id, |data| {
            old_name = Some(data.rename(new_name));
        });
        Ok(old_name.unwrap())
    }

    /// Moves `local` resources to `remote` resource list.
    pub fn commit(&mut self) -> anyhow::Result<()> {
        self.db
            .remote_resources
            .append(&mut self.db.local_resources);
        self.flush()
    }

    fn pre_serialize(&mut self) {
        self.db.pre_serialize();
    }

    fn flush(&mut self) -> anyhow::Result<()> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        self.pre_serialize();
        serde_json::to_writer_pretty(&self.file, &self.db).with_context(|| "Flushing project index")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        // todo(kstasik): writing to a file on drop can be problematic
        self.flush().unwrap();
    }
}

impl fmt::Debug for Project {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.resource_list().map(|r| self.resource_name(r).unwrap());
        f.debug_list().entries(names).finish()
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::sync::{Arc, Mutex};
    use std::{fs::File, path::Path, str::FromStr};

    use lgn_data_runtime::{resource, Resource, ResourceType};
    use tempfile::TempDir;

    use super::ResourceDb;
    use crate::resource::project::Project;
    use crate::{
        resource::{
            ResourcePathName, ResourceProcessor, ResourceRegistry, ResourceRegistryOptions,
        },
        ResourcePathId,
    };

    fn setup_test() -> TempDir {
        let root = tempfile::tempdir().unwrap();

        let project_index_path = Project::root_to_index_path(root.path());
        let project_index_file = File::create(project_index_path).unwrap();
        std::fs::create_dir(root.path().join("offline")).unwrap();

        serde_json::to_writer(project_index_file, &ResourceDb::default()).unwrap();
        root
    }

    const RESOURCE_TEXTURE: ResourceType = ResourceType::new(b"texture");
    const RESOURCE_MATERIAL: ResourceType = ResourceType::new(b"material");
    const RESOURCE_GEOMETRY: ResourceType = ResourceType::new(b"geometry");
    const RESOURCE_SKELETON: ResourceType = ResourceType::new(b"skeleton");
    const RESOURCE_ACTOR: ResourceType = ResourceType::new(b"actor");

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
            &mut self,
            resource: &dyn Any,
            writer: &mut dyn std::io::Write,
        ) -> std::io::Result<usize> {
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
        ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
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

    fn create_actor(project_dir: &Path) -> (Project, Arc<Mutex<ResourceRegistry>>) {
        let index_path = Project::root_to_index_path(project_dir);
        let mut project = Project::open(&index_path).unwrap();
        let resources_arc = ResourceRegistryOptions::new()
            .add_type_processor(RESOURCE_TEXTURE, Box::new(NullResourceProc {}))
            .add_type_processor(RESOURCE_MATERIAL, Box::new(NullResourceProc {}))
            .add_type_processor(RESOURCE_GEOMETRY, Box::new(NullResourceProc {}))
            .add_type_processor(RESOURCE_SKELETON, Box::new(NullResourceProc {}))
            .add_type_processor(RESOURCE_ACTOR, Box::new(NullResourceProc {}))
            .create_registry();

        let mut resources = resources_arc.lock().unwrap();
        let texture = project
            .add_resource(
                ResourcePathName::new("albedo.texture"),
                RESOURCE_TEXTURE,
                &resources.new_resource(RESOURCE_TEXTURE).unwrap(),
                &mut resources,
            )
            .unwrap();

        let material = resources
            .new_resource(RESOURCE_MATERIAL)
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
                &material,
                &mut resources,
            )
            .unwrap();

        let geometry = resources
            .new_resource(RESOURCE_GEOMETRY)
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
                &geometry,
                &mut resources,
            )
            .unwrap();

        let skeleton = project
            .add_resource(
                ResourcePathName::new("hero.skeleton"),
                RESOURCE_SKELETON,
                &resources.new_resource(RESOURCE_SKELETON).unwrap(),
                &mut resources,
            )
            .unwrap();

        let actor = resources
            .new_resource(RESOURCE_ACTOR)
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
                &actor,
                &mut resources,
            )
            .unwrap();

        drop(resources);
        (project, resources_arc)
    }

    fn create_sky_material(project: &mut Project, resources: &mut ResourceRegistry) {
        let texture = project
            .add_resource(
                ResourcePathName::new("sky.texture"),
                RESOURCE_TEXTURE,
                &resources.new_resource(RESOURCE_TEXTURE).unwrap(),
                resources,
            )
            .unwrap();

        let material = resources
            .new_resource(RESOURCE_MATERIAL)
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
                &material,
                resources,
            )
            .unwrap();
    }

    /*
    // + data-offline/
    //  - albedo.texture
    //  - body.material // texture ref
    //  - hero.geometry // material ref
    //  - hero.actor // geometry ref, skeleton ref
    //  - hero.skeleton // no refs
     */

    #[test]
    fn proj_create_delete() {
        let root = tempfile::tempdir().unwrap();

        let project = Project::create_new(root.path()).expect("failed to create project");
        let same_project = Project::create_new(root.path());
        assert!(same_project.is_err());

        project.delete();

        let _project = Project::create_new(root.path()).expect("failed to re-create project");
        let same_project = Project::create_new(root.path());
        assert!(same_project.is_err());
    }

    #[test]
    fn proj_open() {
        let root = tempfile::tempdir().unwrap();
        let proj_dir = root.path();

        assert!(Project::open(proj_dir).is_err());

        {
            let _proj = Project::create_new(proj_dir).expect("failed to create project");
        }

        let project = Project::open(proj_dir).expect("to open project");
        project.delete();

        assert!(Project::open(proj_dir).is_err());
    }

    #[test]
    fn local_changes() {
        let proj_root_path = setup_test();
        let (project, _) = create_actor(proj_root_path.path());

        assert_eq!(project.db.local_resources.len(), 5);
        assert_eq!(project.db.remote_resources.len(), 0);
    }

    #[test]
    fn commit() {
        let proj_root_path = setup_test();
        let (mut project, _) = create_actor(proj_root_path.path());

        project.commit().unwrap();

        assert_eq!(project.db.local_resources.len(), 0);
        assert_eq!(project.db.remote_resources.len(), 5);
    }

    #[test]
    fn immediate_dependencies() {
        let project_dir = setup_test();
        let (project, _) = create_actor(project_dir.path());

        let top_level_resource = project
            .find_resource(&ResourcePathName::new("hero.actor"))
            .unwrap();

        let (_, dependencies) = project.resource_info(top_level_resource).unwrap();

        assert_eq!(dependencies.len(), 2);
    }

    #[test]
    fn rename() {
        let rename_assert =
            |proj: &mut Project, old_name: ResourcePathName, new_name: ResourcePathName| {
                let skeleton_id = proj.find_resource(&old_name);
                assert!(skeleton_id.is_ok());
                let skeleton_id = skeleton_id.unwrap();

                let prev_name = proj.rename_resource(skeleton_id, &new_name);
                assert!(prev_name.is_ok());
                let prev_name = prev_name.unwrap();
                assert_eq!(&prev_name, &old_name);

                assert!(proj.find_resource(&old_name).is_err());
                assert_eq!(proj.find_resource(&new_name).unwrap(), skeleton_id);
            };

        let project_dir = setup_test();
        let (mut project, resources) = create_actor(project_dir.path());
        assert!(project.commit().is_ok());
        create_sky_material(&mut project, &mut resources.lock().unwrap());

        rename_assert(
            &mut project,
            ResourcePathName::new("hero.skeleton"),
            ResourcePathName::new("boss.skeleton"),
        );
        rename_assert(
            &mut project,
            ResourcePathName::new("sky.material"),
            ResourcePathName::new("clouds.material"),
        );
    }
}
