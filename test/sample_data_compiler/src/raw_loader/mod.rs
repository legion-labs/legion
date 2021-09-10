mod raw_data;
mod raw_to_offline;

use crate::offline_data::{self};
use legion_data_offline::resource::{
    Project, Resource, ResourceId, ResourcePathName, ResourceRegistry, ResourceRegistryOptions,
    ResourceType,
};
use serde::de::DeserializeOwned;
use std::{
    ffi::OsStr,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

pub fn build_offline(root_folder: impl AsRef<Path>) {
    let root_folder = root_folder.as_ref();
    if let Ok(entries) = root_folder.read_dir() {
        let mut raw_dir = entries
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
        if let Some(raw_dir) = raw_dir.next() {
            let raw_dir = raw_dir.path();
            let (mut project, mut resources) = setup_project(root_folder);

            let file_paths = find_files(&raw_dir, &["mat", "mesh"]);

            let resource_names = file_paths
                .iter()
                .map(|s| path_to_resource_name(s))
                .collect::<Vec<_>>();

            let resource_ids =
                create_or_find_default(&file_paths, &resource_names, &mut project, &mut resources);

            for (i, path) in file_paths.iter().enumerate() {
                let resource_id = resource_ids[i];
                match path.extension().unwrap().to_str().unwrap() {
                    "mat" => {
                        load_resource::<raw_data::Material, offline_data::Material>(
                            resource_id,
                            path,
                            &mut project,
                            &mut resources,
                        );
                    }
                    "mesh" => {
                        load_resource::<raw_data::Mesh, offline_data::Mesh>(
                            resource_id,
                            path,
                            &mut project,
                            &mut resources,
                        );
                    }
                    _ => panic!(),
                }
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

fn setup_project(root_folder: &Path) -> (Project, ResourceRegistry) {
    // create/load project
    let project = match Project::open(root_folder) {
        Ok(project) => Ok(project),
        Err(_) => Project::create_new(root_folder),
    }
    .unwrap();

    let resources = ResourceRegistryOptions::new()
        .add_type(
            offline_data::MATERIAL_TYPE_ID,
            Box::new(offline_data::MaterialProcessor {}),
        )
        .add_type(
            offline_data::MESH_TYPE_ID,
            Box::new(offline_data::MeshProcessor {}),
        )
        .create_registry();

    (project, resources)
}

fn ext_to_resource_kind(ext: &str) -> ResourceType {
    match ext {
        "mat" => offline_data::MATERIAL_TYPE_ID,
        "mesh" => offline_data::MESH_TYPE_ID,
        _ => panic!(),
    }
}

/// Creates resources for all `file_paths` containing default values (empty content).
///
/// The content of resources is loaded later.
///
/// This is done because we need to assign `ResourceId` for all resources before we load them
/// in order to resolve references from a `ResourcePathName` (/path/to/resource) to `ResourceId` (125463453).
fn create_or_find_default(
    file_paths: &[PathBuf],
    resource_names: &[ResourcePathName],
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> Vec<ResourceId> {
    let mut ids = vec![];

    for (i, path) in file_paths.iter().enumerate() {
        let name = &resource_names[i];
        let kind = ext_to_resource_kind(path.extension().unwrap().to_str().unwrap());

        let id = {
            if let Ok(id) = project.find_resource(name) {
                id
            } else {
                project
                    .add_resource(
                        name.clone(),
                        kind,
                        resources.new_resource(kind).unwrap(),
                        resources,
                    )
                    .unwrap()
            }
        };
        ids.push(id);
    }
    ids
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
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
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

fn load_resource<RawType, OfflineType>(
    resource_id: ResourceId,
    file: &Path,
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> Option<ResourceId>
where
    RawType: DeserializeOwned,
    OfflineType: Resource + From<RawType>,
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
        *offline_data = raw_data.into();

        project
            .save_resource(resource_id, resource, resources)
            .unwrap();
        Some(resource_id)
    } else {
        None
    }
}
