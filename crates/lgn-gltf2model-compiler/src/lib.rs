use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::resource::ResourceProcessor;
use lgn_data_offline::Transform;
use lgn_data_runtime::{AssetRegistryOptions, Resource};
use lgn_graphics_data::offline::ModelProcessor;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_gltf::GltfFile::TYPE,
        lgn_graphics_data::offline::Model::TYPE,
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
    let model_proc = ModelProcessor {};

    let mut resource_references = Vec::new();
    let models = resource.gather_models(context.source.resource_id());
    for (model, name) in models {
        let mut compiled_asset = vec![];
        model_proc
            .write_resource(&model, &mut compiled_asset)
            .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
        let model_rpid = context.target_unnamed.new_named(&name);
        let asset = context.store(&compiled_asset, model_rpid.clone())?;
        compiled_resources.push(asset);
        for mesh in model.meshes {
            if let Some(material_rpid) = mesh.material {
                resource_references.push((model_rpid.clone(), material_rpid));
            }
        }
    }

    Ok(CompilationOutput {
        compiled_resources,
        resource_references,
    })
}
