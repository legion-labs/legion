use std::collections::hash_map::DefaultHasher;
use std::env;
use std::hash::{Hash, Hasher};
use std::path::Path;

use legion_assets::AssetId;
use legion_resources::{Project, ResourceId, ResourcePathRef, ResourceType};

use crate::buildindex::BuildIndex;
use crate::compiledassetstore::LocalCompiledAssetStore;
use crate::compilers::CompilerId;
use crate::compilers::CompilerRegistry;
use crate::{Error, Locale, Platform, Target};

const DATABUILD_VERSION: &str = env!("CARGO_PKG_VERSION");

// everything outside of the resource itself that goes into compiling a resource.
#[derive(Hash)]
struct CompilerDesc {
    resource_type: ResourceType,
    compiler_id: CompilerId,
    databuild_version: &'static str,
    // todo(kstasik): localization_id
}

impl CompilerDesc {
    fn to_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

/// Description of a compiled asset.
///
/// The contained information can be used to retrieve and validate the asset from a [`CompiledAssetStore`](`super::compiledassetstore::CompiledAssetStore`).
#[derive(Debug, PartialEq, Eq)]
pub struct CompiledAsset {
    /// The id of the asset.
    pub guid: AssetId,
    /// The md5 checksum of the asset.
    pub md5: i128,
    /// The size of the asset.
    pub size: usize,
}

impl CompiledAsset {
    fn compute_md5(data: &[u8]) -> i128 {
        // this is not md5 ;)
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish() as i128
    }

    /// Returns
    pub fn new(id: AssetId, data: &[u8]) -> Self {
        Self {
            guid: id,
            md5: Self::compute_md5(data),
            size: data.len(),
        }
    }
}

/// The output of data compilation.
///
/// `Manifest` contains the list of compiled assets.
#[derive(Debug)]
pub struct Manifest {
    /// The description of all compiled assets.
    pub compiled_assets: Vec<CompiledAsset>,
}

/// Data build interface.
///
/// `DataBuild` provides methods to compile offline resources into runtime format.
///
/// Data build uses file-based storage to persist the state of data builds and data compilation.
/// It requires access to offline resources to retrieve resource metadata - throught  [`legion_resources::Project`].
pub struct DataBuild {
    build_db: BuildIndex,
    project: Project,
    asset_store: LocalCompiledAssetStore,
}

impl DataBuild {
    fn new(
        buildindex_path: &Path,
        projectindex_path: &Path,
        assetstore_path: &Path,
    ) -> Result<Self, Error> {
        let project = Self::open_project(projectindex_path)?;

        let build_db = BuildIndex::create_new(buildindex_path, projectindex_path, Self::version())
            .map_err(|_e| Error::IOError)?;

        let asset_store = LocalCompiledAssetStore::new(assetstore_path).ok_or(Error::NotFound)?;

        Ok(Self {
            build_db,
            project,
            asset_store,
        })
    }

    /// Opens the existing build index.
    ///
    /// If the build index does not exist it creates one if a project present in the directory.
    pub fn open(buildindex_path: &Path, assetstore_path: &Path) -> Result<Self, Error> {
        // todo(kstasik): better error
        let asset_store = LocalCompiledAssetStore::new(assetstore_path).ok_or(Error::NotFound)?;
        match BuildIndex::open(buildindex_path, Self::version()) {
            Ok(build_db) => {
                let project = build_db.open_project()?;
                Ok(Self {
                    build_db,
                    project,
                    asset_store,
                })
            }
            Err(Error::NotFound) => {
                let projectindex_path = buildindex_path; // we are going to try to locate the project index in the same directory
                Self::new(buildindex_path, projectindex_path, assetstore_path)
            }
            Err(e) => Err(e),
        }
    }

    fn map_resource_reference(
        id: ResourceId,
        references: &[ResourceId],
    ) -> Result<ResourceId, Error> {
        if let Some(p) = references.iter().find(|&e| *e == id) {
            return Ok(*p);
        }
        Err(Error::IntegrityFailure)
    }

    fn open_project(projectindex_path: &Path) -> Result<Project, Error> {
        Project::open(projectindex_path).map_err(|e| match e {
            legion_resources::Error::ParseError => Error::IntegrityFailure,
            legion_resources::Error::NotFound | legion_resources::Error::InvalidPath => {
                Error::NotFound
            }
            legion_resources::Error::IOError(_) => Error::IOError,
        })
    }

    /// Updates the build database with information about resources from provided resource database.
    pub fn source_pull(&mut self) -> Result<i32, Error> {
        let mut updated_resources = 0;

        let all_resources = self.project.resource_list();

        for res in &all_resources {
            let (resource_hash, deps) = self.project.collect_resource_info(*res)?;
            let dependencies = deps
                .into_iter()
                .map(|d| Self::map_resource_reference(d, &all_resources))
                .collect::<Result<Vec<ResourceId>, Error>>()?;

            if self
                .build_db
                .update_resource(*res, resource_hash, dependencies)
            {
                updated_resources += 1;
            }
        }

        Ok(updated_resources)
    }

    // compile_input:
    // - compiler_hash: (asset_type, databuild_ver, compiler_id, loc_id)
    // - source_guid: guid of source resource
    // - source_hash: asset_hash (md5 of meta, md5 of content, flags) + asset_hash(es) of deps
    // compile_output:
    // - compiled_guid
    // - compiled_type
    // - compiled_md5
    // - compiled_size
    // - compiled_flags

    /// Compiles a named resource and all its dependencies. The compilation results are stored in `compilation database`.
    ///
    /// The data compilation results in a `manifest` that describes the resulting runtime resources.
    pub fn compile(
        &mut self,
        root_resource_name: &ResourcePathRef,
        target: Target,
        platform: Platform,
        locale: Locale,
    ) -> Result<Manifest, Error> {
        let resource_id = self.project.find_resource(root_resource_name)?;

        // todo(kstasik): for now dependencies are not compiled - only the root resource is.
        let (resource, dependencies) = self.build_db.find(resource_id).ok_or(Error::NotFound)?;

        let compilers = CompilerRegistry::new();
        let compiler_info = compilers
            .find(resource.resource_type())
            .ok_or(Error::CompilerNotFound)?;

        // todo(kstasik): support triggering compilation for multiple platforms
        let compiler_id = compiler_info.compiler_id(target, platform, locale);

        let compiler_desc = CompilerDesc {
            resource_type: resource.resource_type(),
            compiler_id,
            databuild_version: Self::version(),
        }; // compiler_hash

        let source_guid = resource;

        //
        // todo(kstasik): source_hash computation can include filtering of resource types in the future.
        // the same resource can have a different source_hash depending on the compiler
        // used as compilers can filter dependencies out.
        //
        let source_hash = self.build_db.compute_source_hash(resource)?;

        let compilerdesc_hash = compiler_desc.to_hash();

        let compiled_assets = {
            let cached = self.build_db.find_compiled(compilerdesc_hash, source_hash);
            if !cached.is_empty() {
                cached
                    .iter()
                    .map(|asset| CompiledAsset {
                        guid: asset.compiled_guid,
                        md5: asset.compiled_md5,
                        size: asset.compiled_size,
                    })
                    .collect()
            } else {
                // for now we only focus on top level asset
                // todo(kstasik): how do we know that GI needs to be run? taking many assets as arguments?

                let compiled_assets = compiler_info.compile(
                    resource,
                    dependencies,
                    &mut self.asset_store,
                    &self.project,
                )?;

                self.build_db.insert_compiled(
                    compilerdesc_hash,
                    source_guid,
                    source_hash,
                    &compiled_assets,
                );
                compiled_assets
            }
        };

        let manifest = Manifest { compiled_assets };
        Ok(manifest)
    }

    /// Returns the global version of the databuild module.
    pub fn version() -> &'static str {
        DATABUILD_VERSION
    }
}

// todo(kstasik): file IO on descructor - is it ok?
impl Drop for DataBuild {
    fn drop(&mut self) {
        self.build_db.flush().unwrap();
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        compiledassetstore::{CompiledAssetStore, LocalCompiledAssetStore},
        databuild::DataBuild,
        Platform, Target,
    };
    use legion_resources::{Project, ResourcePath, ResourceType};

    pub const TEST_BUILDINDEX_FILENAME: &str = "build.index";

    #[test]
    fn source_pull() {
        let work_dir = tempfile::tempdir().unwrap();
        {
            let mut project =
                Project::create_new(work_dir.path()).expect("failed to create a project");
            let texture = project
                .create_resource(ResourcePath::from("child"), ResourceType::Texture)
                .unwrap();
            let _material = project
                .create_resource_with_deps(
                    ResourcePath::from("parent"),
                    ResourceType::Material,
                    &[texture],
                )
                .unwrap();
        }

        let buildindex_path = work_dir.path().join(TEST_BUILDINDEX_FILENAME);
        let assetstore_root = work_dir.path();

        {
            let mut build = DataBuild::open(&buildindex_path, assetstore_root).unwrap();

            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 2);

            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 0);
        }

        {
            let mut project = Project::open(work_dir.path()).unwrap();
            project
                .create_resource(ResourcePath::from("orphan"), ResourceType::Texture)
                .unwrap();
        }

        {
            let mut build = DataBuild::open(&buildindex_path, assetstore_root).unwrap();
            let updated_count = build.source_pull().unwrap();
            assert_eq!(updated_count, 1);
        }
    }

    #[test]
    fn compile() {
        let work_dir = tempfile::tempdir().unwrap();
        {
            let mut project =
                Project::create_new(work_dir.path()).expect("failed to create a project");
            let texture = project
                .create_resource(ResourcePath::from("child"), ResourceType::Texture)
                .unwrap();
            let _material = project
                .create_resource_with_deps(
                    ResourcePath::from("parent"),
                    ResourceType::Material,
                    &[texture],
                )
                .unwrap();
        }

        let buildindex_path = work_dir.path().join(TEST_BUILDINDEX_FILENAME);
        let assetstore_root = work_dir.path();
        let mut build = DataBuild::open(&buildindex_path, assetstore_root).unwrap();

        build.source_pull().unwrap();

        let manifest = build
            .compile(
                &ResourcePath::from("child"),
                Target::Game,
                Platform::Windows,
                ['e', 'n'],
            )
            .unwrap();

        assert_eq!(manifest.compiled_assets.len(), 1); // for now only the root asset is compiled

        let compiled_guid = manifest.compiled_assets[0].guid;
        let asset_store = LocalCompiledAssetStore::new(assetstore_root).unwrap();

        assert!(asset_store.find(compiled_guid).is_some());

        println!("{:?}", manifest);

        build
            .compile(
                &ResourcePath::from("child"),
                Target::Game,
                Platform::Windows,
                ['e', 'n'],
            )
            .unwrap();
    }
}
