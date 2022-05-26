mod raw_data;
mod raw_to_offline;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{BufReader, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use generic_data::offline::{TestComponent, TestEntity};
use lgn_content_store::Provider;
use lgn_data_offline::resource::{Project, ResourcePathName};
use lgn_data_runtime::{
    AssetRegistry, AssetRegistryOptions, Resource, ResourceDescriptor, ResourceId, ResourceType,
    ResourceTypeAndId,
};
use lgn_graphics_data::{offline_gltf::GltfFile, offline_psd::PsdFile};
use lgn_source_control::{RepositoryIndex, RepositoryName};
use lgn_tracing::{error, info};
use lgn_utils::DefaultHasher;
use sample_data::offline as offline_data;
use serde::de::DeserializeOwned;

use self::raw_to_offline::FromRaw;

pub async fn build_offline(
    root_folder: impl AsRef<Path>,
    repository_index: impl RepositoryIndex,
    repository_name: &RepositoryName,
    branch_name: &str,
    source_control_content_provider: Arc<Provider>,
    incremental: bool,
) -> Project {
    let raw_dir = {
        if let Ok(entries) = root_folder.as_ref().read_dir() {
            let mut raw_dir = entries
                .flatten()
                .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
            raw_dir.next().map(|d| d.path())
        } else {
            None
        }
    };

    let (mut project, resources) = setup_project(
        root_folder.as_ref(),
        repository_index,
        repository_name,
        branch_name,
        source_control_content_provider,
    )
    .await;

    if let Some(raw_dir) = raw_dir {
        let file_paths = find_files(
            &raw_dir,
            &["ent", "ins", "mat", "psd", "png", "gltf", "glb"],
        );

        let raw_checksum = {
            let mut hasher = DefaultHasher::new();
            for file in &file_paths {
                let meta = std::fs::metadata(file).unwrap();
                meta.modified().unwrap().hash(&mut hasher);
            }
            hasher.finish()
        };

        let generated_checksum = {
            if !root_folder.as_ref().join("VERSION").exists() {
                None
            } else {
                std::fs::read_to_string(root_folder.as_ref().join("VERSION"))
                    .map_or(None, |version| version.parse::<u64>().ok())
            }
        };

        if let Some(generated_checksum) = generated_checksum {
            if generated_checksum == raw_checksum {
                info!("Skipping Project Generation");
                return project;
            }
        }

        if !incremental {
            std::fs::remove_file(root_folder.as_ref().join("VERSION")).unwrap_or_default();
        }

        // cleanup data from source control before we generate new data.
        if !incremental {
            let all_resources = project.resource_list().await;
            if !all_resources.is_empty() {
                for type_id in all_resources {
                    project.delete_resource(type_id).await.unwrap();
                }

                project.commit("cleanup resources").await.unwrap();
            }
        }

        let file_paths_guids = file_paths
            .iter()
            .map(|s| {
                let mut p = s.clone();
                p.set_extension(s.extension().unwrap().to_str().unwrap().to_owned() + ".guid");
                ResourceId::from_str(&fs::read_to_string(p).unwrap()).unwrap()
            })
            .collect::<Vec<_>>();

        let in_resources = file_paths
            .iter()
            .map(|s| path_to_resource_name(s))
            .zip(file_paths_guids)
            .collect::<Vec<_>>();

        let resource_ids =
            create_or_find_default(&file_paths, &in_resources, &mut project, &resources).await;

        info!("Created resources: {:#?}", project);

        for (i, path) in file_paths.iter().enumerate() {
            let resource_name = &in_resources[i].0;
            let resource_id = *resource_ids.get(resource_name).unwrap();
            match path.extension().unwrap().to_str().unwrap() {
                "ent" => {
                    load_ron_resource::<raw_data::Entity, offline_data::Entity>(
                        resource_id,
                        path,
                        &resource_ids,
                        &mut project,
                        &resources,
                    )
                    .await;

                    let handle = project
                        .load_resource(resource_id, &resources)
                        .await
                        .unwrap()
                        .typed::<offline_data::Entity>();

                    if let Some(entity) = handle.instantiate(&resources) {
                        if let Some(parent_id) = &entity.parent {
                            let mut raw_name =
                                project.raw_resource_name(resource_id).await.unwrap();
                            raw_name.replace_parent_info(Some(parent_id.source_resource()), None);
                            project
                                .rename_resource(resource_id, &raw_name)
                                .await
                                .unwrap();

                            handle.apply(entity, &resources);
                        }
                    }
                }
                "ins" => {
                    load_ron_resource::<raw_data::Instance, offline_data::Instance>(
                        resource_id,
                        path,
                        &resource_ids,
                        &mut project,
                        &resources,
                    )
                    .await;
                }
                "mat" => {
                    load_ron_resource::<raw_data::Material, lgn_graphics_data::offline::Material>(
                        resource_id,
                        path,
                        &resource_ids,
                        &mut project,
                        &resources,
                    )
                    .await;
                }
                "psd" => {
                    load_psd_resource(resource_id, path, &mut project, &resources).await;
                }
                "png" => {
                    load_png_resource(resource_id, path, &mut project, &resources).await;
                }
                "gltf" | "glb" => {
                    load_gltf_resource(resource_id, path, &mut project, &resources).await;
                }
                _ => panic!(),
            }

            info!("Loaded: {}. id: {}", resource_name, resource_id);
        }

        project.commit("sample data generation").await.unwrap();

        let mut version_file = std::fs::File::create(root_folder.as_ref().join("VERSION")).unwrap();
        version_file
            .write_all(raw_checksum.to_string().as_bytes())
            .unwrap();
    } else {
        error!(
            "did not find a 'raw' sub-directory in {}",
            root_folder.as_ref().display()
        );
    }

    project
}

async fn setup_project(
    root_folder: &Path,
    repository_index: impl RepositoryIndex,
    repository_name: &RepositoryName,
    branch_name: &str,
    source_control_content_provider: Arc<Provider>,
) -> (Project, Arc<AssetRegistry>) {
    // create/load project
    let project = Project::open(
        root_folder,
        &repository_index,
        repository_name,
        branch_name,
        source_control_content_provider,
    )
    .await
    .unwrap();
    // {
    //     Ok(project)
    // } else {
    //     Project::create(
    //         root_folder,
    //         repository_index,
    //         repository_name,
    //         source_control_content_provider,
    //     )
    //     .await
    // }
    // .unwrap();

    let mut registry = AssetRegistryOptions::new()
        .add_processor::<lgn_graphics_data::offline_texture::Texture>()
        .add_processor::<lgn_graphics_data::offline_psd::PsdFile>()
        .add_processor::<lgn_graphics_data::offline_png::PngFile>()
        .add_processor::<lgn_graphics_data::offline_gltf::GltfFile>();

    offline_data::add_loaders(&mut registry);
    lgn_graphics_data::offline::add_loaders(&mut registry);
    generic_data::offline::add_loaders(&mut registry);

    (project, registry.create().await)
}

fn ext_to_resource_kind(ext: &str) -> (&str, ResourceType) {
    match ext {
        "ent" => (offline_data::Entity::TYPENAME, offline_data::Entity::TYPE),
        "ins" => (
            offline_data::Instance::TYPENAME,
            offline_data::Instance::TYPE,
        ),
        "mat" => (
            lgn_graphics_data::offline::Material::TYPENAME,
            lgn_graphics_data::offline::Material::TYPE,
        ),
        "psd" => (
            lgn_graphics_data::offline_psd::PsdFile::TYPENAME,
            lgn_graphics_data::offline_psd::PsdFile::TYPE,
        ),
        "png" => (
            lgn_graphics_data::offline_png::PngFile::TYPENAME,
            lgn_graphics_data::offline_png::PngFile::TYPE,
        ),
        "gltf" => (
            lgn_graphics_data::offline_gltf::GltfFile::TYPENAME,
            lgn_graphics_data::offline_gltf::GltfFile::TYPE,
        ),
        _ => panic!(),
    }
}

/// Creates resources for all `file_paths` containing default values (empty
/// content).
///
/// The content of resources is loaded later.
///
/// This is done because we need to assign `ResourceId` for all resources before
/// we load them in order to resolve references from a `ResourcePathName`
/// (/path/to/resource) to `ResourceId` (125463453).
async fn create_or_find_default(
    file_paths: &[PathBuf],
    in_resources: &[(ResourcePathName, ResourceId)],
    project: &mut Project,
    resources: &AssetRegistry,
) -> HashMap<ResourcePathName, ResourceTypeAndId> {
    let mut ids = HashMap::<ResourcePathName, ResourceTypeAndId>::default();
    build_resource_from_raw(file_paths, in_resources, project, resources, &mut ids).await;
    build_test_entity(project, resources, &mut ids).await;
    ids
}

async fn build_resource_from_raw(
    file_paths: &[PathBuf],
    in_resources: &[(ResourcePathName, ResourceId)],
    project: &mut Project,
    resources: &AssetRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    for (i, path) in file_paths.iter().enumerate() {
        let name = &in_resources[i].0;
        let (_kind_name, kind) = ext_to_resource_kind(path.extension().unwrap().to_str().unwrap());

        let id = {
            if let Ok(id) = project.find_resource(name).await {
                id
            } else {
                let id = ResourceTypeAndId {
                    kind,
                    id: in_resources[i].1,
                };

                if project.exists(id).await {
                    project.delete_resource(id).await.unwrap();
                }

                project
                    .add_resource_with_id(
                        name.clone(),
                        id,
                        resources.new_resource_with_id(id).unwrap(),
                        resources,
                    )
                    .await
                    .unwrap();

                id
            }
        };
        ids.insert(name.clone(), id);
    }
}

async fn build_test_entity(
    project: &mut Project,
    resources: &AssetRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    // Create TestEntity Generic DataContainer
    let name: ResourcePathName = "/entity/TEST_ENTITY_NAME.dc".into();
    let id = {
        if let Ok(id) = project.find_resource(&name).await {
            id
        } else {
            let kind = TestEntity::TYPE;
            let id = ResourceTypeAndId {
                kind,
                id: ResourceId::from_str("D8FE06A0-1317-46F5-902B-266B0EAE6FA8").unwrap(),
            };
            let test_entity_handle = resources.new_resource_with_id(id).unwrap();
            let mut test_entity = test_entity_handle
                .instantiate::<TestEntity>(resources)
                .unwrap();
            test_entity.test_string = "Editable String Value".into();
            test_entity.test_float32 = 1.0;
            test_entity.test_float64 = 2.0;
            test_entity.test_int = 1337;
            test_entity.test_position = lgn_math::Vec3::new(0.0, 100.0, 0.0);

            (0..3).for_each(|i| {
                test_entity
                    .test_sub_type
                    .test_components
                    .push(Box::new(TestComponent { test_i32: i }));
            });
            test_entity.test_option_set = Some(generic_data::offline::TestSubType2::default());
            test_entity.test_option_primitive_set = Some(lgn_math::Vec3::default());

            test_entity_handle.apply(test_entity, resources);

            project
                .add_resource_with_id(name.clone(), id, test_entity_handle, resources)
                .await
                .unwrap();

            id
        }
    };
    ids.insert(name, id);
}

fn path_to_resource_name(path: &Path) -> ResourcePathName {
    let mut found = false;
    let name = path
        .iter()
        .filter_map(|component| {
            let was_found = found;
            if !found && component == OsStr::new("raw") {
                found = true;
            }
            if was_found {
                let mut s = String::from("/");
                s.push_str(&component.to_owned().into_string().unwrap());
                Some(s)
            } else {
                None
            }
        })
        .collect::<String>();
    ResourcePathName::from(name)
}

fn find_files(raw_dir: impl AsRef<Path>, extensions: &[&str]) -> Vec<PathBuf> {
    let dir = raw_dir.as_ref();

    let mut files = vec![];

    for entry in dir.read_dir().unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.append(&mut find_files(&path, extensions));
        } else if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            //
            // include only supported extensions
            //
            if extensions.contains(&ext) {
                files.push(path);
            }
        }
    }

    files
}

async fn load_ron_resource<RawType, OfflineType>(
    resource_id: ResourceTypeAndId,
    file: &Path,
    references: &HashMap<ResourcePathName, ResourceTypeAndId>,
    project: &mut Project,
    resources: &AssetRegistry,
) -> Option<ResourceTypeAndId>
where
    RawType: DeserializeOwned,
    OfflineType: Resource + ResourceDescriptor + FromRaw<RawType> + Send + 'static,
{
    if let Ok(f) = File::open(file) {
        let reader = BufReader::new(f);
        let raw_data: RawType = ron::de::from_reader(reader).unwrap();

        let resource = resources.new_resource_with_id(resource_id).unwrap();

        // convert raw to offline
        let mut offline_data = resource.instantiate(resources).unwrap();
        *offline_data = OfflineType::from_raw(raw_data, references);
        resource.apply(offline_data, resources);

        project
            .save_resource(resource_id, resource, resources)
            .await
            .unwrap();
        Some(resource_id)
    } else {
        None
    }
}

async fn load_psd_resource(
    resource_id: ResourceTypeAndId,
    file: &Path,
    project: &mut Project,
    resources: &AssetRegistry,
) -> Option<ResourceTypeAndId> {
    let raw_data = fs::read(file).ok()?;
    let loaded_psd = PsdFile::from_bytes(&raw_data)?;

    let resource = project
        .load_resource(resource_id, resources)
        .await
        .unwrap()
        .typed::<PsdFile>();

    let mut initial_resource = resource.instantiate(resources).unwrap();
    *initial_resource = loaded_psd;
    resource.apply(initial_resource, resources);

    project
        .save_resource(resource_id, &resource, resources)
        .await
        .unwrap();
    Some(resource_id)
}

async fn load_png_resource(
    resource_id: ResourceTypeAndId,
    file: &Path,
    project: &mut Project,
    resources: &AssetRegistry,
) -> Option<ResourceTypeAndId> {
    let reader = fs::read(file).ok()?;
    let handle = resources
        .deserialize_resource(resource_id, &mut reader.as_slice())
        .ok()?;
    project
        .save_resource(resource_id, handle, resources)
        .await
        .unwrap();
    Some(resource_id)
}

async fn load_gltf_resource(
    resource_id: ResourceTypeAndId,
    file: &Path,
    project: &mut Project,
    resources: &AssetRegistry,
) -> Option<ResourceTypeAndId> {
    let handle = resources.new_resource_with_id(resource_id).unwrap();
    let mut gltf_file = handle.instantiate::<GltfFile>(resources).unwrap();
    let raw_data = fs::read(file).ok()?;
    *gltf_file = GltfFile::from_bytes(raw_data);
    handle.apply(gltf_file, resources);

    project
        .save_resource(resource_id, handle, resources)
        .await
        .unwrap();
    Some(resource_id)
}
