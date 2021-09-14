use std::{fs, path::Path};

use legion_content_store::ContentStoreAddr;
use legion_data_build::DataBuildOptions;
//use legion_data_compiler::{Locale, Platform, Target};
//use legion_data_offline::asset::AssetPathId;

pub fn build(root_folder: impl AsRef<Path>) {
    let root_folder = root_folder.as_ref();

    let temp_dir = root_folder.join("temp");
    if !temp_dir.exists() {
        fs::create_dir(&temp_dir).expect("unable to create temp sub-folder");
    }

    let build_index_path = temp_dir.join("build.index");
    let asset_store_path = ContentStoreAddr::from(temp_dir);
    let project_dir = root_folder.to_owned();

    let mut build = DataBuildOptions::new(build_index_path)
        .content_store(&asset_store_path)
        //.compiler_dir(target_dir())
        .open_or_create(project_dir)
        .expect("new build index");

    build.source_pull().expect("successful pull");

    let runtime_dir = root_folder.join("runtime");
    if !runtime_dir.exists() {
        fs::create_dir(&runtime_dir).expect("unable to create runtime sub-folder");
    }

    //let root: AssetPathId = AssetPathId::default();

    // let manifest_path = runtime_dir.join("game.manifest");

    // let platform = Platform::Windows;
    // let locale = Locale::new("en");

    //let output = build.compile(root, &manifest_path, Target::Server, platform, &locale);
}
