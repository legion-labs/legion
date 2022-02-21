use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::{resource::ResourceProcessor, Transform};
use lgn_data_runtime::{AssetRegistryOptions, Resource};
use lgn_graphics_data::offline::MeshProcessor;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_gltf::GltfFile::TYPE,
        lgn_graphics_data::runtime::Mesh::TYPE,
    ),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry.add_loader::<lgn_graphics_data::offline_gltf::GltfFile>()
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource = resources
        .load_sync::<lgn_graphics_data::offline_gltf::GltfFile>(context.source.resource_id());
    let resource = resource.get(&resources).unwrap();

    let mut compiled_resources = vec![];
    let mesh_proc = MeshProcessor {};

    let meshes = resource.new_mesh();
    for mesh in meshes {
        let mut compiled_asset = vec![];
        mesh_proc
            .write_resource(&mesh.0, &mut compiled_asset)
            .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
        let asset = context.store(&compiled_asset, context.target_unnamed.new_named(&mesh.1))?;
        compiled_resources.push(asset);
    }

    Ok(CompilationOutput {
        compiled_resources,
        resource_references: vec![],
    })
}
