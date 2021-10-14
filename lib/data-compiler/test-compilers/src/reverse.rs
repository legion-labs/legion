use std::env;

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use legion_data_offline::resource::ResourceProcessor;
use legion_data_runtime::Resource;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(
        text_resource::TextResource::TYPE,
        text_resource::TextResource::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<text_resource::TextResource>()
        .create();
    let mut resources = resources.lock().unwrap();

    let resource = resources.load_sync::<text_resource::TextResource>(context.source.content_id());
    let resource = resource.get(&resources).unwrap();

    let bytes = {
        let mut bytes = vec![];
        let output = text_resource::TextResource {
            content: resource.content.chars().rev().collect(),
        };

        let mut processor = text_resource::TextResourceProc {};
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

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
