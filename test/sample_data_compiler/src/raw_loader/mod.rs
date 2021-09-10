mod raw_data;
mod raw_to_offline;

use crate::offline_data;
use legion_data_offline::resource::{
    Project, Resource, ResourceId, ResourcePathName, ResourceRegistry, ResourceRegistryOptions,
    ResourceType,
};
use serde::de::DeserializeOwned;
use std::{fs::File, io::BufReader, path::Path};

pub fn build_offline(root_folder: impl AsRef<Path>) {
    let root_folder = root_folder.as_ref();
    if let Ok(entries) = root_folder.read_dir() {
        let mut raw_dir = entries
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
        if let Some(raw_dir) = raw_dir.next() {
            let raw_dir = raw_dir.path();
            load_raw_dir(root_folder, &raw_dir);
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

fn load_raw_dir(root_folder: &Path, raw_dir: &Path) {
    // create/load project
    let mut project = match Project::open(root_folder) {
        Ok(project) => Ok(project),
        Err(_) => Project::create_new(root_folder),
    }
    .unwrap();

    let mut resources = ResourceRegistryOptions::new()
        .add_type(
            offline_data::MATERIAL_TYPE_ID,
            Box::new(offline_data::MaterialProcessor {}),
        )
        .add_type(
            offline_data::MESH_TYPE_ID,
            Box::new(offline_data::MeshProcessor {}),
        )
        .create_registry();

    load_dir(raw_dir, raw_dir, &mut project, &mut resources);
}

fn load_dir(
    raw_dir: &Path,
    dir: impl AsRef<Path>,
    project: &mut Project,
    resources: &mut ResourceRegistry,
) {
    let dir = dir.as_ref();
    println!("loading folder {}", dir.display());
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    load_dir(raw_dir, entry.path(), project, resources);
                } else {
                    assert!(!file_type.is_symlink());
                    load_file(raw_dir, entry.path(), project, resources);
                }
            }
        }
    }
}

fn load_file(
    raw_dir: &Path,
    file: impl AsRef<Path>,
    project: &mut Project,
    resources: &mut ResourceRegistry,
) {
    let file = file.as_ref();
    if let Some(ext) = file.extension() {
        let ext = ext.to_string_lossy();
        if ext == "meta" {
            // do nothing
        } else if ext == "ent" {
            // Entity
            println!("todo Entity");
            //let _entity: raw_data::Entity = deserialize(reader);
            //project.add_resource(name, kind, handle, registry);
        } else if ext == "ins" {
            // Instance
            println!("todo Instance");
            //let _instance: raw_data::Instance = deserialize(reader);
        } else if ext == "mat" {
            create_resource::<raw_data::Material, offline_data::Material>(
                raw_dir,
                file,
                project,
                resources,
                offline_data::MATERIAL_TYPE_ID,
                ".material",
            );
        } else if ext == "mesh" {
            create_resource::<raw_data::Mesh, offline_data::Mesh>(
                raw_dir,
                file,
                project,
                resources,
                offline_data::MESH_TYPE_ID,
                ".mesh",
            );
        } else {
            eprintln!(
                "unrecognized file extension '{}', for file {}",
                ext,
                file.file_name().unwrap().to_string_lossy()
            );
        }
    }
}

fn create_resource<RawType, OfflineType>(
    raw_dir: &Path,
    file: &Path,
    project: &mut Project,
    resources: &mut ResourceRegistry,
    resource_kind: ResourceType,
    extension: &str,
) -> Option<ResourceId>
where
    RawType: DeserializeOwned,
    OfflineType: Resource + From<RawType>,
{
    if let Ok(f) = File::open(file) {
        let file_name = file.file_stem().unwrap().to_string_lossy();

        let reader = BufReader::new(f);

        // get path relative to root of raw data
        let relative_path = file.strip_prefix(raw_dir).unwrap();

        // split up into folder components
        let mut path_components: Vec<String> = relative_path
            .components()
            .filter_map(|c| match c {
                std::path::Component::Normal(s) => Some(s.to_string_lossy().to_string()),
                _ => None,
            })
            .collect();
        // remove file itself
        path_components.truncate(path_components.len() - 1);
        let mut component_iter = path_components.iter();

        // build resource path from folder components
        let mut resource_path = ResourcePathName::new(component_iter.next().unwrap());
        for component in component_iter {
            resource_path.push(component);
        }

        let raw_data: RawType = ron::de::from_reader(reader).unwrap();

        let resource = resources
            .new_resource(resource_kind)
            .unwrap()
            .typed::<OfflineType>();

        // remap extension
        resource_path.push(file_name + extension);

        // convert raw to offline
        let offline_data = resource.get_mut(resources).unwrap();
        *offline_data = raw_data.into();

        let resource_id = project
            .add_resource(resource_path, resource_kind, resource, resources)
            .expect("failed to add resource to project");
        Some(resource_id)
    } else {
        None
    }
}
