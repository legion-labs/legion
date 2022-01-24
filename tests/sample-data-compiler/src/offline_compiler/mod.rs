use std::{
    env,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};

use lgn_content_store::ContentStoreAddr;
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target,
};
use lgn_data_offline::{resource::ResourcePathName, ResourcePathId};
use lgn_data_runtime::Resource;
use sample_data::offline as offline_data;
use sample_data::runtime as runtime_data;

//use crate::offline_to_runtime::find_derived_path;

pub fn find_derived_path(path: &ResourcePathId) -> ResourcePathId {
    let offline_type = path.content_type();
    match offline_type {
        offline_data::Entity::TYPE => path.push(runtime_data::Entity::TYPE),
        offline_data::Instance::TYPE => path.push(runtime_data::Instance::TYPE),
        offline_data::Mesh::TYPE => path.push(runtime_data::Mesh::TYPE),
        lgn_graphics_data::offline_psd::PsdFile::TYPE => path
            .push(lgn_graphics_data::offline_texture::Texture::TYPE)
            .push(lgn_graphics_data::runtime_texture::Texture::TYPE),
        lgn_graphics_data::offline::Material::TYPE => {
            path.push(lgn_graphics_data::runtime::Material::TYPE)
        }
        generic_data::offline::DebugCube::TYPE => path.push(generic_data::runtime::DebugCube::TYPE),
        _ => {
            panic!("unrecognized offline type {}", offline_type);
        }
    }
}

pub async fn build(root_folder: impl AsRef<Path>, resource_name: &ResourcePathName) {
    let root_folder = root_folder.as_ref();

    let temp_dir = root_folder.join("temp");
    if !temp_dir.exists() {
        fs::create_dir(&temp_dir).expect("unable to create temp sub-folder");
    }

    let build_index_dir = temp_dir.clone();
    let asset_store_path = ContentStoreAddr::from(temp_dir.clone());
    let mut exe_path = env::current_exe().expect("cannot access current_exe");
    exe_path.pop();
    let project_dir = PathBuf::from("..\\");

    let (mut build, mut project) =
        DataBuildOptions::new(build_index_dir, CompilerRegistryOptions::from_dir(exe_path))
            .content_store(&asset_store_path)
            .open_or_create(project_dir)
            .await
            .expect("new build index");

    build.source_pull(&mut project).expect("successful pull");

    let runtime_dir = root_folder.join("runtime");
    if !runtime_dir.exists() {
        fs::create_dir(&runtime_dir).expect("unable to create runtime sub-folder");
    }

    let offline_manifest_path = temp_dir.join("editor.manifest");

    let platform = Platform::Windows;
    let locale = Locale::new("en");

    if let Ok(resource_id) = project.find_resource(resource_name) {
        let asset_path = find_derived_path(&ResourcePathId::from(resource_id));
        let source_name = project
            .resource_name(asset_path.source_resource().id)
            .ok()
            .unwrap();

        println!("Compiling: {} from {}...", asset_path, source_name);

        let manifest = build
            .compile(
                asset_path,
                Some(offline_manifest_path),
                &CompilationEnv {
                    target: Target::Server,
                    platform,
                    locale,
                },
            )
            .expect("valid manifest");

        //
        // for now, we generate a runtime manifest in this simple way
        // as data build process does not implement *packaging* yet.
        //
        let runtime_manifest_path = runtime_dir.join("game.manifest");
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(runtime_manifest_path)
            .expect("open file");

        let filter = |p: &ResourcePathId| {
            matches!(
                p.content_type(),
                runtime_data::Entity::TYPE
                    | runtime_data::Instance::TYPE
                    | runtime_data::Mesh::TYPE
                    | lgn_graphics_data::runtime_texture::Texture::TYPE
                    | lgn_graphics_data::runtime::Material::TYPE
                    | generic_data::runtime::DebugCube::TYPE
            )
        };

        let rt_manifest = manifest.into_rt_manifest(filter);
        serde_json::to_writer_pretty(file, &rt_manifest).expect("to write manifest");
    }
}
