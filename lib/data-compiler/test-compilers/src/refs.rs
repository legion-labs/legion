use std::env;

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use legion_data_runtime::Resource;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(
        refs_resource::TestResource::TYPE,
        refs_asset::RefsAsset::TYPE,
    ),
    compiler_hash_func: hash_code_and_data,
    compile_func: compile,
};

fn compile(mut context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let resources = context
        .take_registry()
        .add_loader::<refs_resource::TestResource>()
        .create();

    let resource = resources.load_sync::<refs_resource::TestResource>(context.source.content_id());
    assert!(!resource.is_err(&resources));
    assert!(resource.is_loaded(&resources));
    let resource = resource.get(&resources).unwrap();

    let compiled_asset = {
        let mut text = resource.content.as_bytes().to_owned();
        text.reverse();
        let mut content = text.len().to_le_bytes().to_vec();
        content.append(&mut text);

        // the compiled asset has no reference.
        let reference_id = 0u128;
        content.append(&mut reference_id.to_ne_bytes().to_vec());
        content
    };

    let asset = context.store(&compiled_asset, context.target_unnamed.clone())?;

    // in this test example every build dependency becomes a reference/load-time dependency.
    let source = context.target_unnamed.clone();
    let references: Vec<_> = context
        .dependencies
        .iter()
        .map(|destination| (source.clone(), destination.clone()))
        .collect();

    Ok(CompilationOutput {
        compiled_resources: vec![asset],
        resource_references: references,
    })
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
