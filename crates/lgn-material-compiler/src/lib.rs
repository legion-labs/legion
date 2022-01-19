// crate-specific lint exceptions:
//#![allow()]

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::{hash_code_and_data, path_id_to_binary},
};
use lgn_data_offline::{ResourcePathId, Transform};
use lgn_data_runtime::{AssetRegistryOptions, Resource};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_offline::Material::TYPE,
        lgn_graphics_runtime::Material::TYPE,
    ),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry.add_loader::<lgn_graphics_offline::Material>()
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource =
        resources.load_sync::<lgn_graphics_offline::Material>(context.source.resource_id());

    let resource = resource.get(&resources).unwrap();

    let compiled_asset = {
        let mut c: Vec<u8> = vec![];
        c.append(&mut path_id_to_binary(&resource.albedo));
        c.append(&mut path_id_to_binary(&resource.normal));
        c.append(&mut path_id_to_binary(&resource.roughness));
        c.append(&mut path_id_to_binary(&resource.metalness));
        c
    };

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    let mut resource_references: Vec<(ResourcePathId, ResourcePathId)> = Vec::new();

    let mut add_reference = |reference: &Option<ResourcePathId>| {
        if let Some(reference) = reference {
            resource_references.push((context.target_unnamed.clone(), reference.clone()));
        }
    };

    add_reference(&resource.albedo);
    add_reference(&resource.normal);
    add_reference(&resource.roughness);
    add_reference(&resource.metalness);

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references,
    })
}
