mod raw_data;
mod raw_to_offline;

use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
};

use generic_data::offline::{DebugCube, TestComponent, TestEntity};
use lgn_data_offline::resource::{
    Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions,
};
use lgn_data_runtime::{Resource, ResourceId, ResourceType, ResourceTypeAndId};
use lgn_graphics_offline::PsdFile;
use sample_data_offline as offline_data;
use serde::de::DeserializeOwned;

use self::raw_to_offline::FromRaw;

pub fn build_offline(root_folder: impl AsRef<Path>) {
    let root_folder = root_folder.as_ref();
    if let Ok(entries) = root_folder.read_dir() {
        let mut raw_dir = entries
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
        if let Some(raw_dir) = raw_dir.next() {
            let raw_dir = raw_dir.path();
            let (mut project, resources) = setup_project(root_folder);
            let mut resources = resources.lock().unwrap();

            let file_paths = find_files(&raw_dir, &["ent", "ins", "mat", "mesh", "psd"]);

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
                create_or_find_default(&file_paths, &in_resources, &mut project, &mut resources);

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
                        );
                    }
                    "ins" => {
                        load_ron_resource::<raw_data::Instance, offline_data::Instance>(
                            resource_id,
                            path,
                            &resource_ids,
                            &mut project,
                            &mut resources,
                        );
                    }
                    "mat" => {
                        load_ron_resource::<raw_data::Material, lgn_graphics_offline::Material>(
                            resource_id,
                            path,
                            &resource_ids,
                            &mut project,
                            &mut resources,
                        );
                    }
                    "mesh" => {
                        load_ron_resource::<raw_data::Mesh, offline_data::Mesh>(
                            resource_id,
                            path,
                            &resource_ids,
                            &mut project,
                            &mut resources,
                        );
                    }
                    "psd" => {
                        load_psd_resource(resource_id, path, &mut project, &mut resources);
                    }
                    _ => panic!(),
                }

                println!("Loaded: {}. id: {}", resource_name, resource_id);
            }
        } else {
            eprintln!(
                "did not find a 'raw' sub-directory in {}",
                root_folder.display()
            );
        }
    } else {
        eprintln!("unable to open directory {}", root_folder.display());
    }
}

fn setup_project(root_folder: &Path) -> (Project, Arc<Mutex<ResourceRegistry>>) {
    // create/load project
    let project = match Project::open(root_folder) {
        Ok(project) => Ok(project),
        Err(_) => Project::create_new(root_folder),
    }
    .unwrap();

    let mut registry = ResourceRegistryOptions::new();
    registry = offline_data::register_resource_types(registry);
    registry = lgn_graphics_offline::register_resource_types(registry);
    registry = generic_data::offline::register_resource_types(registry);
    let registry = registry.create_registry();

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
            lgn_graphics_offline::Material::TYPENAME,
            lgn_graphics_offline::Material::TYPE,
        ),
        "mesh" => (offline_data::Mesh::TYPENAME, offline_data::Mesh::TYPE),
        "psd" => (
            lgn_graphics_offline::PsdFile::TYPENAME,
            lgn_graphics_offline::PsdFile::TYPE,
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
fn create_or_find_default(
    file_paths: &[PathBuf],
    in_resources: &[(ResourcePathName, ResourceId)],
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> HashMap<ResourcePathName, ResourceTypeAndId> {
    let mut ids = HashMap::<ResourcePathName, ResourceTypeAndId>::default();
    build_resource_from_raw(file_paths, in_resources, project, resources, &mut ids);
    build_test_entity(project, resources, &mut ids);
    build_debug_cubes(project, resources, &mut ids);
    ids
}

fn build_resource_from_raw(
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
            if let Ok(id) = project.find_resource(name) {
                id
            } else {
                let id = ResourceTypeAndId {
                    t: kind.1,
                    id: in_resources[i].1,
                };
                project
                    .add_resource_with_id(
                        name.clone(),
                        kind.0,
                        kind.1,
                        id,
                        resources.new_resource(kind.1).unwrap(),
                        resources,
                    )
                    .unwrap()
            }
        };
        ids.insert(name.clone(), id);
    }
}

fn build_test_entity(
    project: &mut Project,
    resources: &mut ResourceRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    // Create TestEntity Generic DataContainer
    let name: ResourcePathName = "/entity/TEST_ENTITY_NAME.dc".into();
    let id = {
        if let Ok(id) = project.find_resource(&name) {
            id
        } else {
            let kind_name = TestEntity::TYPENAME;
            let kind = TestEntity::TYPE;
            let id = ResourceTypeAndId {
                t: kind,
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

            project
                .add_resource_with_id(
                    name.clone(),
                    kind_name,
                    kind,
                    id,
                    test_entity_handle,
                    resources,
                )
                .unwrap()
        }
    };
    ids.insert(name, id);
}

fn build_debug_cubes(
    project: &mut Project,
    resources: &mut ResourceRegistry,
    ids: &mut HashMap<ResourcePathName, ResourceTypeAndId>,
) {
    let cube_ids = [
        "DB051B98-6FF5-4BAC-BEA8-50B5A13C3F1B",
        "202E3AA6-F158-4C77-890B-3F59B183B6BD",
        "7483C534-FE2A-4F16-B655-E9AFE39A93BA",
    ];

    // Create DebugCube DataContainer
    (0..3).for_each(|index| {
        let name: ResourcePathName = format!("/entity/DebugCube{}", index).into();
        let id = project.find_resource(&name).unwrap_or_else(|_err| {
            let kind = DebugCube::TYPE;
            let id = ResourceTypeAndId {
                t: kind,
                id: ResourceId::from_str(cube_ids[index]).unwrap(),
            };
            let cube_entity_handle = resources.new_resource(kind).unwrap();
            let cube_entity = cube_entity_handle.get_mut::<DebugCube>(resources).unwrap();

            cube_entity.color = match index {
                0 => (255, 0, 0).into(),
                1 => (255, 255, 0).into(),
                2 => (255, 0, 255).into(),
                3 => (0, 0, 255).into(),
                _ => (192, 192, 192).into(),
            };

            cube_entity.mesh_id = 1;
            cube_entity.rotation_speed = match index {
                0 => (0.4f32, 0.0f32, 0.0f32).into(),
                1 => (0.0f32, 0.4f32, 0.0f32).into(),
                2 => (0.0f32, 0.0f32, 0.4f32).into(),
                3 => (0.0f32, 0.3f32, 0.0f32).into(),
                _ => (0.0f32, 0.0f32, 0.0f32).into(),
            };

            cube_entity.position = match index {
                0 => (0.0f32, 0.0f32, 1.0f32).into(),
                1 => (1.0f32, 0.0f32, 0.0f32).into(),
                2 => (-1.0f32, 0.0f32, 0.0f32).into(),
                3 => (0.0f32, 1.0f32, 0.0f32).into(),
                _ => (0.0f32, 0.0f32, 0.0f32).into(),
            };

            project
                .add_resource_with_id(
                    name.clone(),
                    DebugCube::TYPENAME,
                    DebugCube::TYPE,
                    id,
                    cube_entity_handle,
                    resources,
                )
                .unwrap()
        });

        ids.insert(name, id);
    });
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

fn load_ron_resource<RawType, OfflineType>(
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

        let resource = project
            .load_resource(resource_id, resources)
            .unwrap()
            .typed::<OfflineType>();

        // convert raw to offline
        let offline_data = resource.get_mut(resources).unwrap();
        *offline_data = OfflineType::from_raw(raw_data, references);

        project
            .save_resource(resource_id, resource, resources)
            .unwrap();
        Some(resource_id)
    } else {
        None
    }
}

fn load_psd_resource(
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
        .unwrap();
    Some(resource_id)
}
