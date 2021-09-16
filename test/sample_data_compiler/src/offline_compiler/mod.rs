use std::{
    env,
    fs::{self, OpenOptions},
    path::Path,
};

use legion_content_store::ContentStoreAddr;
use legion_data_build::{generate_rt_manifest, DataBuildOptions};
use legion_data_compiler::{Locale, Platform, Target};
use legion_data_offline::asset::AssetPathId;

use crate::{
    offline_data::{self, CompilableResource},
    runtime_data::{self, CompilableAsset},
};

pub fn build(root_folder: impl AsRef<Path>) {
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

    let resource_list = build.project().resource_list();
    for resource_id in resource_list {
        let mut asset_path = AssetPathId::from(resource_id);

        let source_type = asset_path.source_resource().resource_type();
        if source_type == offline_data::Entity::TYPE_ID {
            asset_path = asset_path.push(runtime_data::Entity::TYPE_ID);
        } else if source_type == offline_data::Instance::TYPE_ID {
            asset_path = asset_path.push(runtime_data::Instance::TYPE_ID);
        } else if source_type == offline_data::Mesh::TYPE_ID {
            asset_path = asset_path.push(runtime_data::Mesh::TYPE_ID);
        } else if source_type == offline_data::Material::TYPE_ID {
            asset_path = asset_path.push(runtime_data::Material::TYPE_ID);
        }

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
            .open(runtime_manifest_path)
            .expect("open file");

        let rt_manifest = generate_rt_manifest(manifest);
        serde_json::to_writer_pretty(file, &rt_manifest).expect("to write manifest");
    }
}
