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
use lgn_data_offline::{
    resource::{Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions},
    ResourcePathId,
};
use lgn_data_runtime::{Resource, ResourceId, ResourceType, ResourceTypeAndId};
use lgn_graphics_data::{offline_gltf::GltfFile, offline_png::PngFile, offline_psd::PsdFile};
use lgn_utils::DefaultHasher;
use sample_data::offline as offline_data;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

use self::raw_to_offline::FromRaw;

pub async fn build_offline(root_folder: impl AsRef<Path>, incremental: bool) {
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

    if let Some(raw_dir) = raw_dir {
        let mut file_paths = find_files(&raw_dir, &["ent", "ins", "mat", "psd", "png", "gltf"]);

        let raw_checksum = {
            let mut hasher = DefaultHasher::new();
            for file in &file_paths {
                let meta = std::fs::metadata(file).unwrap();
                meta.modified().unwrap().hash(&mut hasher);
            }
            hasher.finish()
        };

        let generated_checksum = {
            std::fs::read_to_string(root_folder.as_ref().join("VERSION"))
                .map_or(None, |version| version.parse::<u64>().ok())
        };

        if let Some(generated_checksum) = generated_checksum {
            if generated_checksum == raw_checksum {
                println!("Skipping Project Generation");
                return;
            }
        }

        if !incremental {
            std::fs::remove_dir_all(root_folder.as_ref().join("remote"))
                .unwrap_or_else(|e| println!("failed to delete remote: {}.", e));

            std::fs::remove_dir_all(root_folder.as_ref().join("offline"))
                .unwrap_or_else(|e| println!("failed to delete offline: {}.", e));

            std::fs::remove_file(root_folder.as_ref().join("VERSION"))
                .unwrap_or_else(|e| println!("failed to delete VERSION: {}.", e));
        }

        //
        let (mut project, resources) = setup_project(root_folder.as_ref()).await;
        let mut resources = resources.lock().await;

        let gltf_folders = file_paths
            .iter()
            .filter_map(|v| {
                let mut v = v.clone();
                if let Some(extension) = v.extension() {
                    if extension.eq("gltf") && v.pop() {
                        Some(v)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<PathBuf>>();

        // hack to only load .png unassociated with .gltf
        file_paths = file_paths
            .iter()
            .filter(|v| {
                let mut f = (*v).clone();
                if let Some(extension) = v.extension() {
                    if f.pop() && !(extension.eq("png") && gltf_folders.contains(&f)) {
                        return true;
                    }
                }
                false
            })
            .cloned()
            .collect();

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
            create_or_find_default(&file_paths, &in_resources, &mut project, &mut resources).await;

        println!("Created resources: {:#?}", project);

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
                        &mut resources,
                    )
                    .await;
                }
                "ins" => {
                    load_ron_resource::<raw_data::Instance, offline_data::Instance>(
                        resource_id,
                        path,
                        &resource_ids,
                        &mut project,
                        &mut resources,
                    )
                    .await;
                }
                "mat" => {
                    load_ron_resource::<raw_data::Material, lgn_graphics_data::offline::Material>(
                        resource_id,
                        path,
                        &resource_ids,
                        &mut project,
                        &mut resources,
                    )
                    .await;
                }
                "psd" => {
                    load_psd_resource(resource_id, path, &mut project, &mut resources).await;
                }
                "png" => {
                    load_png_resource(resource_id, path, &mut project, &mut resources).await;
                }
                "gltf" => {
                    load_gltf_resource(resource_id, path, &mut project, &mut resources).await;
                }
                _ => panic!(),
            }

            println!("Loaded: {}. id: {}", resource_name, resource_id);
        }

        project.commit("sample data generation").await.unwrap();

        let mut version_file = std::fs::File::create(root_folder.as_ref().join("VERSION")).unwrap();
        version_file
            .write_all(raw_checksum.to_string().as_bytes())
            .unwrap();
    } else {
        eprintln!(
            "did not find a 'raw' sub-directory in {}",
            root_folder.as_ref().display()
        );
    }
}

async fn setup_project(root_folder: &Path) -> (Project, Arc<Mutex<ResourceRegistry>>) {
    // create/load project
    let project = if let Ok(project) = Project::open(root_folder).await {
        Ok(project)
    } else {
        Project::create_with_remote_mock(root_folder).await
    }
    .unwrap();

    let mut registry = ResourceRegistryOptions::new();
    offline_data::register_resource_types(&mut registry);
    lgn_graphics_data::offline::register_resource_types(&mut registry)
        .add_type_mut::<lgn_graphics_data::offline_texture::Texture>()
        .add_type_mut::<lgn_graphics_data::offline_psd::PsdFile>()
        .add_type_mut::<lgn_graphics_data::offline_png::PngFile>()
        .add_type_mut::<lgn_graphics_data::offline_gltf::GltfFile>();
    generic_data::offline::register_resource_types(&mut registry);
    let registry = registry.create_async_registry();

    (project, registry)
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
    resources: &mut ResourceRegistry,
) -> HashMap<ResourcePathName, ResourceTypeAndId> {
    let mut ids = HashMap::<ResourcePathName, ResourceTypeAndId>::default();
    build_resource_from_raw(file_paths, in_resources, project, resources, &mut ids).await;
    build_test_entity(project, resources, &mut ids).await;
    build_debug_cubes(project, resources, &mut ids).await;
    ids
}

async fn build_resource_from_raw(
    file_paths: &[PathBuf],
    in_resources: &[(ResourcePathName, ResourceId)],
    project: &mut Project,
    resources: &mut ResourceRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    for (i, path) in file_paths.iter().enumerate() {
        let name = &in_resources[i].0;
        let kind = ext_to_resource_kind(path.extension().unwrap().to_str().unwrap());

        let id = {
            if let Ok(id) = project.find_resource(name).await {
                id
            } else {
                let id = ResourceTypeAndId {
                    kind: kind.1,
                    id: in_resources[i].1,
                };
                project
                    .add_resource_with_id(
                        name.clone(),
                        kind.0,
                        kind.1,
                        id.id,
                        resources.new_resource(kind.1).unwrap(),
                        resources,
                    )
                    .await
                    .unwrap()
            }
        };
        ids.insert(name.clone(), id);
    }
}

async fn build_test_entity(
    project: &mut Project,
    resources: &mut ResourceRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    // Create TestEntity Generic DataContainer
    let name: ResourcePathName = "/entity/TEST_ENTITY_NAME.dc".into();
    let id = {
        if let Ok(id) = project.find_resource(&name).await {
            id
        } else {
            let kind_name = TestEntity::TYPENAME;
            let kind = TestEntity::TYPE;
            let id = ResourceTypeAndId {
                kind,
                id: ResourceId::from_str("D8FE06A0-1317-46F5-902B-266B0EAE6FA8").unwrap(),
            };
            let test_entity_handle = resources.new_resource(kind).unwrap();
            let test_entity = test_entity_handle.get_mut::<TestEntity>(resources).unwrap();
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

            project
                .add_resource_with_id(
                    name.clone(),
                    kind_name,
                    kind,
                    id.id,
                    test_entity_handle,
                    resources,
                )
                .await
                .unwrap()
        }
    };
    ids.insert(name, id);
}

async fn build_debug_cubes(
    project: &mut Project,
    resources: &mut ResourceRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    let cube_ids = [
        "DB051B98-6FF5-4BAC-BEA8-50B5A13C3F1B",
        "202E3AA6-F158-4C77-890B-3F59B183B6BD",
        "7483C534-FE2A-4F16-B655-E9AFE39A93BA",
    ];

    let scene: ResourcePathName = "/world/sample_1.ent".into();
    if let Ok(parent_id) = project.find_resource(&scene).await {
        // Create DebugCube DataContainer
        for (index, _) in cube_ids.iter().enumerate() {
            let name: ResourcePathName = format!("/world/sample_1/DebugCube{}", index).into();
            let id = if let Ok(id) = project.find_resource(&name).await {
                id
            } else {
                let kind = offline_data::Entity::TYPE;
                let id = ResourceTypeAndId {
                    kind,
                    id: ResourceId::from_str(cube_ids[index]).unwrap(),
                };
                let cube_entity_handle = resources.new_resource(kind).unwrap();
                let cube_entity = cube_entity_handle
                    .get_mut::<offline_data::Entity>(resources)
                    .unwrap();

                let mut parent_path: ResourcePathId = parent_id.into();
                parent_path = parent_path.push(sample_data::runtime::Entity::TYPE);
                cube_entity.parent = Some(parent_path);

                cube_entity.components.push(Box::new(offline_data::Name {
                    name: format!("DebugCube{}", index),
                }));

                cube_entity
                    .components
                    .push(Box::new(offline_data::Transform {
                        position: match index {
                            0 => (0.0f32, 0.0f32, 1.0f32).into(),
                            1 => (1.0f32, 0.0f32, 0.0f32).into(),
                            2 => (-1.0f32, 0.0f32, 0.0f32).into(),
                            3 => (0.0f32, 1.0f32, 0.0f32).into(),
                            _ => (0.0f32, 0.0f32, 0.0f32).into(),
                        },
                        ..sample_data::offline::Transform::default()
                    }));

                cube_entity.components.push(Box::new(offline_data::Visual {
                    color: match index {
                        0 => (255, 0, 0).into(),
                        1 => (255, 255, 0).into(),
                        2 => (255, 0, 255).into(),
                        3 => (0, 0, 255).into(),
                        _ => (192, 192, 192).into(),
                    },
                    ..sample_data::offline::Visual::default()
                }));

                project
                    .add_resource_with_id(
                        name.clone(),
                        offline_data::Entity::TYPENAME,
                        offline_data::Entity::TYPE,
                        id.id,
                        cube_entity_handle,
                        resources,
                    )
                    .await
                    .unwrap()
            };
            ids.insert(name, id);
        }
    }
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
    resources: &mut ResourceRegistry,
) -> Option<ResourceTypeAndId>
where
    RawType: DeserializeOwned,
    OfflineType: Resource + FromRaw<RawType> + 'static,
{
    if let Ok(f) = File::open(file) {
        let reader = BufReader::new(f);
        let raw_data: RawType = ron::de::from_reader(reader).unwrap();

        let resource = resources.new_resource(OfflineType::TYPE).unwrap();

        // convert raw to offline
        let offline_data = resource.get_mut(resources).unwrap();
        *offline_data = OfflineType::from_raw(raw_data, references);

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
    resources: &mut ResourceRegistry,
) -> Option<ResourceTypeAndId> {
    let raw_data = fs::read(file).ok()?;
    let loaded_psd = PsdFile::from_bytes(&raw_data)?;

    let resource = project
        .load_resource(resource_id, resources)
        .unwrap()
        .typed::<PsdFile>();

    let initial_resource = resource.get_mut(resources).unwrap();
    *initial_resource = loaded_psd;

    project
        .save_resource(resource_id, resource, resources)
        .await
        .unwrap();
    Some(resource_id)
}

async fn load_png_resource(
    resource_id: ResourceTypeAndId,
    file: &Path,
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> Option<ResourceTypeAndId> {
    let reader = fs::read(file).ok()?;
    let handle = resources
        .deserialize_resource(PngFile::TYPE, &mut reader.as_slice())
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
    resources: &mut ResourceRegistry,
) -> Option<ResourceTypeAndId> {
    let handle = resources.new_resource(GltfFile::TYPE).unwrap();
    let gltf_file = handle.get_mut::<GltfFile>(resources).unwrap();
    *gltf_file = GltfFile::from_path(file);

    project
        .save_resource(resource_id, handle, resources)
        .await
        .unwrap();
    Some(resource_id)
}
