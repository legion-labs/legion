use crate::ron_data;
use legion_data_offline::resource::Project;
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

pub fn load_data(root_folder: impl AsRef<Path>) -> Option<Project> {
    let root_folder = root_folder.as_ref();
    if let Ok(entries) = root_folder.read_dir() {
        let mut raw_dir = entries
            .flatten()
            .filter(|e| e.file_type().unwrap().is_dir() && e.file_name() == "raw");
        if let Some(raw_dir) = raw_dir.next() {
            // create/load project
            let mut project = match Project::open(root_folder) {
                Ok(project) => Ok(project),
                Err(_) => Project::create_new(root_folder),
            }
            .unwrap();
            load_dir(raw_dir.path(), &mut project);
            Some(project)
        } else {
            eprintln!(
                "did not find a 'raw' sub-directory in {}",
                root_folder.display()
            );
            None
        }
    } else {
        eprintln!("unable to open directory {}", root_folder.display());
        None
    }
}

fn load_dir(dir: impl AsRef<Path>, project: &mut Project) {
    let dir = dir.as_ref();
    println!("loading folder {}", dir.display());
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    load_dir(entry.path(), project);
                } else {
                    assert!(!file_type.is_symlink());
                    load_file(entry.path(), project);
                }
            }
        }
    }
}

fn load_file(file: impl AsRef<Path>, _project: &mut Project) {
    let file = file.as_ref();
    if let Some(ext) = file.extension() {
        let ext = ext.to_string_lossy();
        if ext == "meta" {
            // do nothing
        } else {
            let name = file.file_name().unwrap().to_string_lossy();
            //println!("processing file {}", name);

            if let Ok(f) = File::open(file) {
                let reader = BufReader::new(f);

                fn deserialize<T, R>(reader: R) -> T
                where
                    T: DeserializeOwned,
                    R: Read,
                {
                    ron::de::from_reader(reader).unwrap()
                }

                if ext == "ent" {
                    // Entity
                    let _entity: ron_data::Entity = deserialize(reader);
                    //project.add_resource(name, kind, handle, registry);
                } else if ext == "ins" {
                    // Instance
                    let _instance: ron_data::Instance = deserialize(reader);
                } else if ext == "mat" {
                    // Material
                    let _material: ron_data::Material = deserialize(reader);
                } else if ext == "mesh" {
                    // Mesh
                    let _mesh: ron_data::Mesh = deserialize(reader);
                } else {
                    eprintln!("unrecognized file extension '{}', for file {}", ext, name);
                }
            }
        }
    }
}
