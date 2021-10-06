use std::{
    env,
    fs::{self, OpenOptions},
    path::Path,
};

use legion_content_store::ContentStoreAddr;
use legion_data_build::{generate_rt_manifest, DataBuildOptions};
use legion_data_compiler::{Locale, Platform, Target};
use legion_data_offline::{resource::ResourcePathName, ResourcePathId};
use legion_data_runtime::Resource;

use crate::{offline_to_runtime::find_derived_path, runtime_data};

pub fn build(root_folder: impl AsRef<Path>, resource_name: &ResourcePathName) {
    let root_folder = root_folder.as_ref();

    let temp_dir = root_folder.join("temp");
    if !temp_dir.exists() {
        fs::create_dir(&temp_dir).expect("unable to create temp sub-folder");
    }

    let build_index_path = temp_dir.join("build.index");
    let asset_store_path = ContentStoreAddr::from(temp_dir.clone());
    let mut exe_path = env::current_exe().expect("cannot access current_exe");
    exe_path.pop();
    let project_dir = root_folder.to_owned();

    let mut build = DataBuildOptions::new(build_index_path)
        .content_store(&asset_store_path)
        .compiler_dir(exe_path)
        .open_or_create(project_dir)
        .expect("new build index");

    build.source_pull().expect("successful pull");

    let runtime_dir = root_folder.join("runtime");
    if !runtime_dir.exists() {
        fs::create_dir(&runtime_dir).expect("unable to create runtime sub-folder");
    }

    let offline_manifest_path = temp_dir.join("editor.manifest");

    let platform = Platform::Windows;
    let locale = Locale::new("en");

    if let Ok(resource_id) = build.project().find_resource(resource_name) {
        let asset_path = find_derived_path(&ResourcePathId::from(resource_id));
        let source_name = build
            .project()
            .resource_name(asset_path.source_resource())
            .ok()
            .unwrap();

        println!("Compiling: {} from {}...", asset_path, source_name);

        let manifest = build
            .compile(
                asset_path,
                &offline_manifest_path,
                Target::Server,
                platform,
                &locale,
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
                    | legion_graphics_runtime::texture::Texture::TYPE
                    | legion_graphics_runtime::Material::TYPE
            )
        };

        let rt_manifest = generate_rt_manifest(manifest, filter);
        serde_json::to_writer_pretty(file, &rt_manifest).expect("to write manifest");
    }
}
