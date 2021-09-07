use crate::asset::AssetPathId;

use crate::resource::{
    metadata::{Metadata, ResourceHash},
    types::{ResourceId, ResourceType},
    ResourceHandleUntyped, ResourcePathName, ResourceRegistry,
};

use std::collections::hash_map::DefaultHasher;
use std::{
    fs::{self, File, OpenOptions},
    hash::Hasher,
};
use std::{
    io::{Read, Seek},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const METADATA_EXT: &str = "meta";

/// A project exists always within a given directory and this file
/// will be created directly in that directory.
const PROJECT_INDEX_FILENAME: &str = ".project.index";

#[derive(Serialize, Deserialize, Default)]
struct ResourceDb {
    remote_resources: Vec<ResourceId>,
    local_resources: Vec<ResourceId>,
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

#[derive(Debug)]
/// Error returned by the project.
pub enum Error {
    /// Project index parsing error.
    ParseError,
    /// Not found.
    NotFound,
    /// Specified path is invalid.
    InvalidPath,
    /// IO error on the project index file.
    IOError(std::io::Error), // todo(kstasik): have clearer Open/Read/Write errors that will be easier to handle layer above
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::ParseError => write!(f, "Error Parsing Content"),
            Error::NotFound => write!(f, "Resource Not Found"),
            Error::InvalidPath => write!(f, "Path Not Found"),
            Error::IOError(ref err) => err.fmt(f),
        }
    }
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

    /// Creates a new project index file turining the containing directory into a project.
    pub fn create_new(project_dir: impl AsRef<Path>) -> Result<Self, Error> {
        let index_path = Self::root_to_index_path(project_dir.as_ref());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&index_path)
            .map_err(Error::IOError)?;

        let db = ResourceDb::default();
        serde_json::to_writer(&file, &db).map_err(|_e| Error::ParseError)?;

        let project_dir = index_path.parent().unwrap().to_owned();
        let resource_dir = project_dir.join("offline");
        std::fs::create_dir(&resource_dir).map_err(Error::IOError)?;

        Ok(Self {
            file,
            db,
            project_dir,
            resource_dir,
        })
    }

    /// Opens the project index specified
    pub fn open(project_dir: impl AsRef<Path>) -> Result<Self, Error> {
        let index_path = Self::root_to_index_path(project_dir.as_ref());
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .open(&index_path)
            .map_err(|_e| Error::NotFound)?;

        let db = serde_json::from_reader(&file).map_err(|_e| Error::ParseError)?;

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

    /// Returns the list resources.
    ///
    /// This method flattens the `remote` and `local` resources into one list.
    pub fn resource_list(&self) -> Vec<ResourceId> {
        let all_resources = [&self.db.remote_resources, &self.db.local_resources];
        let references = all_resources.iter().flat_map(|v| v.iter());
        references.cloned().collect()
    }

    /// Finds resource by its name and returns its `ResourceId`.
    pub fn find_resource(&self, name: &ResourcePathName) -> Result<ResourceId, Error> {
        let all_resources = [&self.db.remote_resources, &self.db.local_resources];
        let mut references = all_resources.iter().flat_map(|v| v.iter());

        // this below would be better expressed as try_map (still experimental).
        let res = references.find_map(|id| match self.read_meta(*id) {
            Ok(meta) => {
                if &meta.name == name {
                    Some(Ok(*id))
                } else {
                    None
                }
            }
            Err(e) => Some(Err(e)),
        });

        match res {
            None => Err(Error::NotFound),
            Some(e) => e,
        }
    }

    /// Checks if a resource with a given name is part of the project.
    pub fn exists_named(&self, name: &ResourcePathName) -> bool {
        self.find_resource(name).is_ok()
    }

    /// Checks if a resource is part of the project.
    pub fn exists(&self, id: ResourceId) -> bool {
        let all_resources = [&self.db.remote_resources, &self.db.local_resources];
        all_resources
            .iter()
            .flat_map(|v| v.iter())
            .any(|v| v == &id)
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
    ) -> Result<ResourceId, Error> {
        let id = ResourceId::generate_new(kind);
        let meta_path = self.metadata_path(id);
        let resource_path = self.resource_path(id);

        let build_dependencies = {
            let mut resource_file = File::create(&resource_path).map_err(Error::IOError)?;

            let (_written, build_deps) = registry
                .serialize_resource(kind, handle, &mut resource_file)
                .map_err(Error::IOError)?;
            build_deps
        };

        let content_checksum = {
            let mut resource_file = File::open(&resource_path).map_err(Error::IOError)?;

            let mut hasher = DefaultHasher::new();
            let mut buffer = [0; 1024];
            loop {
                let count = resource_file.read(&mut buffer).map_err(Error::IOError)?;
                if count == 0 {
                    break;
                }

                hasher.write(&buffer[..count]);
            }

            hasher.finish() as i128
        };

        let meta_file = File::create(&meta_path).map_err(|e| {
            fs::remove_file(&resource_path).unwrap();
            Error::IOError(e)
        })?;

        let metadata = Metadata::new_with_dependencies(name, content_checksum, &build_dependencies);
        serde_json::to_writer_pretty(meta_file, &metadata).unwrap();

        self.db.local_resources.push(id);
        Ok(id)
    }

    /// Writes the resource behind `handle` from memory to disk and updates the corresponding .meta file.
    pub fn save_resource(
        &mut self,
        id: ResourceId,
        handle: impl AsRef<ResourceHandleUntyped>,
        resources: &mut ResourceRegistry,
    ) -> Result<(), Error> {
        let resource_path = self.resource_path(id);
        let metadata_path = self.metadata_path(id);

        let mut meta_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(metadata_path)
            .map_err(Error::IOError)?;
        let mut metadata: Metadata =
            serde_json::from_reader(&meta_file).map_err(|_e| Error::ParseError)?;

        let build_dependencies = {
            let mut resource_file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&resource_path)
                .map_err(Error::IOError)?;

            let (_written, build_deps) = resources
                .serialize_resource(id.resource_type(), handle, &mut resource_file)
                .map_err(Error::IOError)?;
            build_deps
        };

        let content_checksum = {
            let mut resource_file = File::open(&resource_path).map_err(Error::IOError)?;

            let mut hasher = DefaultHasher::new();
            let mut buffer = [0; 1024];
            loop {
                let count = resource_file.read(&mut buffer).map_err(Error::IOError)?;
                if count == 0 {
                    break;
                }

                hasher.write(&buffer[..count]);
            }

            hasher.finish() as i128
        };

        metadata.content_checksum = content_checksum;
        metadata.dependencies = build_dependencies;

        meta_file.set_len(0).unwrap();
        meta_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&meta_file, &metadata).unwrap(); // todo(kstasik): same as above.
        Ok(())
    }

    /// Loads a resource of a given id.
    ///
    /// In-memory representation of that resource is managed by `ResourceRegistry`.
    /// In order to update the resource on disk see [`Self::save_resource()`].
    pub fn load_resource(
        &self,
        id: ResourceId,
        resources: &mut ResourceRegistry,
    ) -> Result<ResourceHandleUntyped, Error> {
        let resource_path = self.resource_path(id);

        let mut resource_file = File::open(resource_path).map_err(Error::IOError)?;
        let handle = resources
            .deserialize_resource(id.resource_type(), &mut resource_file)
            .map_err(Error::IOError)?;
        Ok(handle)
    }

    /// Returns information about a given resource from its `.meta` file.
    pub fn resource_info(&self, id: ResourceId) -> Result<(ResourceHash, Vec<AssetPathId>), Error> {
        let meta = self.read_meta(id)?;
        let resource_hash = meta.resource_hash();
        let dependencies = meta.dependencies;

        Ok((resource_hash, dependencies))
    }

    /// Returns the root directory where resources are located.
    pub fn resource_dir(&self) -> PathBuf {
        self.resource_dir.clone()
    }

    fn metadata_path(&self, id: ResourceId) -> PathBuf {
        let mut path = self.resource_dir();
        path.push(format!("{:x}", id));
        path.set_extension(METADATA_EXT);
        path
    }

    fn resource_path(&self, id: ResourceId) -> PathBuf {
        self.resource_dir().join(format!("{:x}", id))
    }

    /// Moves a `remote` resources to the list of `local` resources.
    pub fn checkout(&mut self, id: ResourceId) -> Result<(), Error> {
        if let Some(_resource) = self.db.local_resources.iter().find(|&res| *res == id) {
            return Ok(()); // already checked out
        }

        if let Some(index) = self.db.remote_resources.iter().position(|res| *res == id) {
            let resource = self.db.remote_resources.remove(index);
            self.db.local_resources.push(resource);
            return Ok(());
        }

        Err(Error::NotFound)
    }

    fn read_meta(&self, id: ResourceId) -> Result<Metadata, Error> {
        let path = self.metadata_path(id);

        let file = File::open(path).map_err(Error::IOError)?;

        let result = serde_json::from_reader(file).map_err(|_e| Error::ParseError)?;
        Ok(result)
    }

    fn update_meta<F>(&self, id: ResourceId, mut func: F)
    where
        F: FnMut(&mut Metadata),
    {
        let path = self.metadata_path(id);

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
        id: ResourceId,
        new_name: &ResourcePathName,
    ) -> Result<ResourcePathName, Error> {
        self.checkout(id)?;

        let mut old_name: Option<ResourcePathName> = None;
        self.update_meta(id, |data| {
            old_name = Some(data.rename(new_name));
        });
        Ok(old_name.unwrap())
    }

    /// Moves `local` resources to `remote` resource list.
    pub fn commit(&mut self) -> Result<(), Error> {
        self.db
            .remote_resources
            .append(&mut self.db.local_resources);
        self.flush()
    }

    fn flush(&mut self) -> Result<(), Error> {
        self.file.set_len(0).unwrap();
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap();
        serde_json::to_writer_pretty(&self.file, &self.db).map_err(|_e| Error::ParseError)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        // todo(kstasik): writing to a file on drop can be problematic
        self.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path, str::FromStr};

    use tempfile::TempDir;

    use crate::resource::project::Project;
    use crate::{
        asset::AssetPathId,
        resource::{
            Resource, ResourcePathName, ResourceProcessor, ResourceRegistry,
            ResourceRegistryOptions, ResourceType,
        },
    };

    use super::ResourceDb;

    fn setup_test() -> TempDir {
        let root = tempfile::tempdir().unwrap();

        let projectindex_path = Project::root_to_index_path(root.path());
        let projectindex_file = File::create(projectindex_path).unwrap();
        std::fs::create_dir(root.path().join("offline")).unwrap();

        serde_json::to_writer(projectindex_file, &ResourceDb::default()).unwrap();
        root
    }

    const RESOURCE_TEXTURE: ResourceType = ResourceType::new(b"texture");
    const RESOURCE_MATERIAL: ResourceType = ResourceType::new(b"material");
    const RESOURCE_GEOMETRY: ResourceType = ResourceType::new(b"geometry");
    const RESOURCE_SKELETON: ResourceType = ResourceType::new(b"skeleton");
    const RESOURCE_ACTOR: ResourceType = ResourceType::new(b"actor");

    #[derive(Resource)]
    struct NullResource {
        content: isize,
        dependencies: Vec<AssetPathId>,
    }

    struct NullResourceProc {}
    impl ResourceProcessor for NullResourceProc {
        fn new_resource(&mut self) -> Box<dyn Resource> {
            Box::new(NullResource {
                content: 0,
                dependencies: vec![],
            })
        }

        fn extract_build_dependencies(&mut self, resource: &dyn Resource) -> Vec<AssetPathId> {
            resource
                .downcast_ref::<NullResource>()
                .unwrap()
                .dependencies
                .clone()
        }

        fn write_resource(
            &mut self,
            resource: &dyn Resource,
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
                let str = format!("{}", dep);
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
        ) -> std::io::Result<Box<dyn Resource>> {
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
                    .push(AssetPathId::from_str(std::str::from_utf8(&buf).unwrap()).unwrap());
            }

            Ok(resource)
        }
    }

    fn create_actor(project_dir: &Path) -> (Project, ResourceRegistry) {
        let index_path = Project::root_to_index_path(project_dir);
        let mut project = Project::open(&index_path).unwrap();
        let mut resources = ResourceRegistryOptions::new()
            .add_type(RESOURCE_TEXTURE, Box::new(NullResourceProc {}))
            .add_type(RESOURCE_MATERIAL, Box::new(NullResourceProc {}))
            .add_type(RESOURCE_GEOMETRY, Box::new(NullResourceProc {}))
            .add_type(RESOURCE_SKELETON, Box::new(NullResourceProc {}))
            .add_type(RESOURCE_ACTOR, Box::new(NullResourceProc {}))
            .create_registry();

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
            .push(AssetPathId::from(texture));
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
            .push(AssetPathId::from(material));
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
        actor.get_mut(&mut resources).unwrap().dependencies =
            vec![AssetPathId::from(geometry), AssetPathId::from(skeleton)];
        let _actor = project
            .add_resource(
                ResourcePathName::new("hero.actor"),
                RESOURCE_ACTOR,
                &actor,
                &mut resources,
            )
            .unwrap();

        (project, resources)
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
            .push(AssetPathId::from(texture));

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
    //  - body.metarial // texture ref
    //  - hero.geometry // material ref
    //  - hero.actor // geometry ref, skeleton ref
    //  - hero.skeleton // no refs
     */

    #[test]
    fn proj_create_delete() {
        let root = tempfile::tempdir().unwrap();

        let project = Project::create_new(root.path()).expect("faild to create project");
        let same_project = Project::create_new(root.path());
        assert!(same_project.is_err());

        project.delete();

        let _project = Project::create_new(root.path()).expect("faild to re-create project");
        let same_project = Project::create_new(root.path());
        assert!(same_project.is_err());
    }

    #[test]
    fn local_changes() {
        let projroot_path = setup_test();
        let (project, _) = create_actor(projroot_path.path());

        assert_eq!(project.db.local_resources.len(), 5);
        assert_eq!(project.db.remote_resources.len(), 0);
    }

    #[test]
    fn commit() {
        let projroot_path = setup_test();
        let (mut project, _) = create_actor(projroot_path.path());

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
        let (mut project, mut resources) = create_actor(project_dir.path());
        assert!(project.commit().is_ok());
        create_sky_material(&mut project, &mut resources);

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
