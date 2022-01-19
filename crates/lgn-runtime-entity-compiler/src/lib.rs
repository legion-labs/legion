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
    transform: &Transform::new(offline_data::Entity::TYPE, runtime_data::Entity::TYPE),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(options: AssetRegistryOptions) -> AssetRegistryOptions {
    options.add_loader::<offline_data::Entity>()
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let entity = resources.load_sync::<offline_data::Entity>(context.source.resource_id());
    let entity = entity.get(&resources).unwrap();

    let runtime_entity = runtime_data::Entity::from_offline(&entity);
    let compiled_asset = bincode::serialize(&runtime_entity).unwrap();

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    let mut resource_references: Vec<(ResourcePathId, ResourcePathId)> = Vec::new();
    for child in &entity.children {
        resource_references.push((context.target_unnamed.clone(), child.clone()));
    }
    for component in &entity.components {
        if let Some(visual) = component.downcast_ref::<offline_data::Visual>() {
            if let Some(mesh_ref) = &visual.renderable_geometry {
                resource_references.push((context.target_unnamed.clone(), mesh_ref.clone()));
            }
        } else if let Some(physics) = component.downcast_ref::<offline_data::Physics>() {
            if let Some(mesh_ref) = &physics.collision_geometry {
                resource_references.push((context.target_unnamed.clone(), mesh_ref.clone()));
            }
        }
    }

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references,
    })
}
