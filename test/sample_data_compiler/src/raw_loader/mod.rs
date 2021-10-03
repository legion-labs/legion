mod raw_data;
mod raw_to_offline;

use crate::offline_data::{self, CompilableResource};
use legion_data_offline::resource::{
    Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions,
};
use legion_data_runtime::{Resource, ResourceId, ResourceType};
use legion_graphics_offline::psd::PsdFile;
use serde::de::DeserializeOwned;
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    ffi::OsStr,
    fs::{self, File},
    hash::{Hash, Hasher},
    io::BufReader,
    path::{Path, PathBuf},
};

use self::raw_to_offline::FromRaw;

pub fn build_offline(root_folder: impl AsRef<Path>) {
    let root_folder = root_folder.as_ref();
    if let Ok(entries) = root_folder.read_dir() {
        let mut raw_dir = entries
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
        if let Some(raw_dir) = raw_dir.next() {
            let raw_dir = raw_dir.path();
            let (mut project, mut resources) = setup_project(root_folder);

            let file_paths = find_files(&raw_dir, &["ent", "ins", "mat", "mesh", "psd"]);

            let resource_names = file_paths
                .iter()
                .map(|s| path_to_resource_name(s))
                .collect::<Vec<_>>();

            let resource_ids =
                create_or_find_default(&file_paths, &resource_names, &mut project, &mut resources);

            println!("Created resources: {:#?}", project);

            for (i, path) in file_paths.iter().enumerate() {
                let resource_name = &resource_names[i];
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
                        load_ron_resource::<
                            raw_data::Material,
                            legion_graphics_offline::material::Material,
                        >(
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

fn setup_project(root_folder: &Path) -> (Project, ResourceRegistry) {
    // create/load project
    let project = match Project::open(root_folder) {
        Ok(project) => Ok(project),
        Err(_) => Project::create_new(root_folder),
    }
    .unwrap();

    fn add_resource<T>(resources: ResourceRegistryOptions) -> ResourceRegistryOptions
    where
        T: CompilableResource,
    {
        resources.add_type(T::TYPE_ID, Box::new(T::Processor::default()))
    }

    let mut resources = ResourceRegistryOptions::new();
    resources = add_resource::<offline_data::Entity>(resources);
    resources = add_resource::<offline_data::Instance>(resources);
    resources = add_resource::<offline_data::Mesh>(resources);

    let resources = resources
        .add_type(
            legion_graphics_offline::material::TYPE_ID,
            Box::new(legion_graphics_offline::material::MaterialProcessor {}),
        )
        .add_type(
            legion_graphics_offline::psd::TYPE_ID,
            Box::new(legion_graphics_offline::psd::PsdFileProcessor {}),
        )
        .create_registry();

    (project, resources)
}

fn ext_to_resource_kind(ext: &str) -> ResourceType {
    match ext {
        "ent" => offline_data::Entity::TYPE_ID,
        "ins" => offline_data::Instance::TYPE_ID,
        "mat" => legion_graphics_offline::material::TYPE_ID,
        "mesh" => offline_data::Mesh::TYPE_ID,
        "psd" => legion_graphics_offline::psd::TYPE_ID,
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
) -> HashMap<ResourcePathName, ResourceId> {
    let mut ids: HashMap<ResourcePathName, ResourceId> = HashMap::default();

    for (i, path) in file_paths.iter().enumerate() {
        let name = &resource_names[i];
        let kind = ext_to_resource_kind(path.extension().unwrap().to_str().unwrap());

        let id = {
            if let Ok(id) = project.find_resource(name) {
                id
            } else {
                let mut hasher = DefaultHasher::new();
                name.hash(&mut hasher);
                let resource_hash = hasher.finish();
                let id = ResourceId::new(kind, resource_hash);
                project
                    .add_resource_with_id(
                        name.clone(),
                        kind,
                        id,
                        resources.new_resource(kind).unwrap(),
                        resources,
                    )
                    .unwrap()
            }
        };
        ids.insert(name.clone(), id);
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

fn load_ron_resource<RawType, OfflineType>(
    resource_id: ResourceId,
    file: &Path,
    references: &HashMap<ResourcePathName, ResourceId>,
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> Option<ResourceId>
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
    resource_id: ResourceId,
    file: &Path,
    project: &mut Project,
    resources: &mut ResourceRegistry,
) -> Option<ResourceId> {
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
