use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
    sync::Arc,
};

use lgn_content_store2::{Chunker, ContentProvider};
use lgn_data_build::DataBuildOptions;
use lgn_data_compiler::{
    compiler_api::CompilationEnv, compiler_node::CompilerRegistryOptions, Locale, Platform, Target,
};
use lgn_data_offline::{
    resource::{Project, ResourcePathName},
    ResourcePathId,
};
use lgn_data_runtime::Resource;
use lgn_source_control::RepositoryIndex;
use lgn_tracing::info;
use sample_data::offline as offline_data;
use sample_data::runtime as runtime_data;

//use crate::offline_to_runtime::find_derived_path;

pub fn find_derived_path(path: &ResourcePathId) -> ResourcePathId {
    let offline_type = path.content_type();
    match offline_type {
        offline_data::Entity::TYPE => path.push(runtime_data::Entity::TYPE),
        offline_data::Instance::TYPE => path.push(runtime_data::Instance::TYPE),
        lgn_graphics_data::offline::Model::TYPE => {
            path.push(lgn_graphics_data::runtime::Model::TYPE)
        }
        lgn_graphics_data::offline_psd::PsdFile::TYPE => path
            .push(lgn_graphics_data::offline_texture::Texture::TYPE)
            .push(lgn_graphics_data::runtime_texture::Texture::TYPE),
        lgn_graphics_data::offline::Material::TYPE => {
            path.push(lgn_graphics_data::runtime::Material::TYPE)
        }
        _ => {
            panic!("unrecognized offline type {}", offline_type);
        }
    }
}

pub async fn build(
    root_folder: impl AsRef<Path>,
    resource_name: &ResourcePathName,
    repository_index: impl RepositoryIndex,
    source_control_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>>,
    data_content_provider: Arc<Box<dyn ContentProvider + Send + Sync>>,
) {
    let root_folder = root_folder.as_ref();

    let temp_dir = root_folder.join("temp");
    if !temp_dir.exists() {
        fs::create_dir(&temp_dir).expect("unable to create temp sub-folder");
    }

    let build_index_dir = temp_dir.clone();

    let mut exe_path = env::current_exe().expect("cannot access current_exe");
    exe_path.pop();

    let project = Project::open(root_folder, repository_index, Arc::clone(&source_control_content_provider))
        .await
        .unwrap();

    let mut build = DataBuildOptions::new_with_sqlite_output(
        build_index_dir,
        CompilerRegistryOptions::local_compilers(exe_path),
        Arc::clone(&data_content_provider),
    )
    .open_or_create(&project)
    .await
    .expect("new build index");

    build.source_pull(&project).await.expect("successful pull");

    let runtime_dir = root_folder.join("runtime");
    if !runtime_dir.exists() {
        fs::create_dir(&runtime_dir).expect("unable to create runtime sub-folder");
    }

    let platform = Platform::Windows;
    let locale = Locale::new("en");

    if let Ok(resource_id) = project.find_resource(resource_name).await {
        let asset_path = find_derived_path(&ResourcePathId::from(resource_id));
        let source_name = project
            .resource_name(asset_path.source_resource().id)
            .ok()
            .unwrap();

        info!("Compiling: {} from {}...", asset_path, source_name);

        let manifest = build
            .compile(
                asset_path,
                &CompilationEnv {
                    target: Target::Server,
                    platform,
                    locale,
                },
            )
            .await
            .expect("valid manifest");

        //
        // for now, we generate a runtime manifest in this simple way
        // as data build process does not implement *packaging* yet.
        //
        let runtime_manifest_path = runtime_dir.join("game.manifest");

        let filter = |p: &ResourcePathId| {
            matches!(
                p.content_type(),
                runtime_data::Entity::TYPE
                    | runtime_data::Instance::TYPE
                    | lgn_graphics_data::runtime_texture::Texture::TYPE
                    | lgn_graphics_data::runtime::Material::TYPE
                    | lgn_graphics_data::runtime::Model::TYPE
            )
        };

        let rt_manifest = manifest.into_rt_manifest(filter);

        let mut buffer = vec![];
        serde_json::to_writer_pretty(&mut buffer, &rt_manifest).expect("to write manifest");

        let chunker = Chunker::default();
        let chunk_id = chunker
            .write_chunk(content_provider, &buffer)
            .await
            .expect("failed to write manifest to cas");

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(runtime_manifest_path)
            .expect("open file");

        write!(file, "{}", chunk_id).expect("failed to write manifest id to file");
    }
}
