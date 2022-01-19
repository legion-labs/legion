// crate-specific lint exceptions:
//#![allow()]

use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistryOptions, Resource};
use sample_data_compiler::offline_to_runtime::FromOffline;
use sample_data_offline as offline_data;
use sample_data_runtime as runtime_data;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(offline_data::Mesh::TYPE, runtime_data::Mesh::TYPE),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry.add_loader::<offline_data::Mesh>()
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let mesh = resources.load_sync::<offline_data::Mesh>(context.source.resource_id());
    let mesh = mesh.get(&resources).unwrap();

    let runtime_mesh = runtime_data::Mesh::from_offline(&mesh);
    let compiled_asset = bincode::serialize(&runtime_mesh).unwrap();

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    let mut resource_references: Vec<(ResourcePathId, ResourcePathId)> = Vec::new();
    for sub_mesh in &mesh.sub_meshes {
        if let Some(material) = &sub_mesh.material {
            resource_references.push((context.target_unnamed.clone(), material.clone()));
        }
    }

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references,
    })
}
