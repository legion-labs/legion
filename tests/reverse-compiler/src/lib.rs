// crate-specific lint exceptions:
//#![allow()]

use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::{resource::ResourceProcessor, Transform};
use lgn_data_runtime::{AssetRegistryOptions, Resource};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        text_resource::TextResource::TYPE,
        text_resource::TextResource::TYPE,
    ),
    init_func: init,
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn init(registry: AssetRegistryOptions) -> AssetRegistryOptions {
    registry.add_loader::<text_resource::TextResource>()
}

fn compile(mut context: CompilerContext<'_>) -> Result<CompilationOutput, CompilerError> {
    let resources = context.registry();

    let resource = resources.load_sync::<text_resource::TextResource>(context.source.resource_id());
    let resource = resource.get(&resources).unwrap();

    let bytes = {
        let mut bytes = vec![];
        let output = text_resource::TextResource {
            content: resource.content.chars().rev().collect(),
        };

        let processor = text_resource::TextResourceProc {};
        let _nbytes = processor
            .write_resource(&output, &mut bytes)
            .map_err(CompilerError::ResourceWriteFailed)?;
        bytes
    };

    let asset = context.store(&bytes, context.target_unnamed.clone())?;

    // in this mock build dependency are _not_ runtime references.
    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: vec![],
    })
}
