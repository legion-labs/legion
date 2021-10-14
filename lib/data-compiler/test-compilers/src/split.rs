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
        multitext_resource::MultiTextResource::TYPE,
        text_resource::TextResource::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<multitext_resource::MultiTextResource>()
        .add_loader::<text_resource::TextResource>()
        .create();
    let mut resources = resources.lock().unwrap();

    let resource =
        resources.load_sync::<multitext_resource::MultiTextResource>(context.source.content_id());
    let resource = resource.get(&resources).unwrap();

    let source_text_list = resource.text_list.clone();

    let mut output = CompilationOutput {
        compiled_resources: vec![],
        resource_references: vec![],
    };

    let mut proc = text_resource::TextResourceProc {};

    for (index, content) in source_text_list.iter().enumerate() {
        let output_resource = text_resource::TextResource {
            content: content.clone(),
        };

        let mut bytes = vec![];

        let _nbytes = proc
            .write_resource(&output_resource, &mut bytes)
            .map_err(CompilerError::ResourceWriteFailed)?;

        let asset = context.store(
            &bytes,
            context.target_unnamed.new_named(&format!("text_{}", index)),
        )?;

        output.compiled_resources.push(asset);
    }

    Ok(output)
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
