use std::env;

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    compiler_utils::{hash_code_and_data, path_id_to_binary},
};
use legion_data_offline::ResourcePathId;
use legion_data_runtime::Resource;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(
        legion_graphics_offline::Material::TYPE,
        legion_graphics_runtime::Material::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<legion_graphics_offline::Material>()
        .create();

    let resource =
        resources.load_sync::<legion_graphics_offline::Material>(context.source.content_id());

    let resource = resource.get(&resources).unwrap();

    let compiled_asset = {
        let mut c: Vec<u8> = vec![];
        c.append(&mut path_id_to_binary(&resource.albedo).to_le_bytes().to_vec());
        c.append(&mut path_id_to_binary(&resource.normal).to_le_bytes().to_vec());
        c.append(
            &mut path_id_to_binary(&resource.roughness)
                .to_le_bytes()
                .to_vec(),
        );
        c.append(
            &mut path_id_to_binary(&resource.metalness)
                .to_le_bytes()
                .to_vec(),
        );
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

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
