use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};

use legion_data_compiler::{
    compiler_api::{
        compiler_main, CompilationOutput, CompilerContext, CompilerDescriptor, CompilerError,
        DATA_BUILD_VERSION,
    },
    CompiledResource, CompilerHash, Locale, Platform, Target,
};
use legion_data_offline::resource::ResourceRegistry;

static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &(multitext_resource::TYPE_ID, text_resource::TYPE_ID),
    compiler_hash_func: compiler_hash,
    compile_func: compile,
};

fn compiler_hash(
    code: &'static str,
    data: &'static str,
    _target: Target,
    _platform: Platform,
    _locale: Locale,
) -> Vec<CompilerHash> {
    let mut hasher = DefaultHasher::new();
    code.hash(&mut hasher);
    data.hash(&mut hasher);
    vec![CompilerHash(hasher.finish())]
}

fn compile(context: CompilerContext) -> Result<CompilationOutput, CompilerError> {
    let mut resources = ResourceRegistry::default();
    resources.register_type(
        multitext_resource::TYPE_ID,
        Box::new(multitext_resource::MultiTextResourceProc {}),
    );
    resources.register_type(
        text_resource::TYPE_ID,
        Box::new(text_resource::TextResourceProc {}),
    );

    let source_handle = context.load_resource(
        &context.compile_path.direct_dependency().unwrap(),
        &mut resources,
    )?;
    let source_resource = source_handle
        .get::<multitext_resource::MultiTextResource>(&resources)
        .unwrap();
    let source_text_list = source_resource.text_list.clone();

    let output_handle = resources.new_resource(text_resource::TYPE_ID).unwrap();

    let mut output = CompilationOutput {
        compiled_resources: vec![],
        resource_references: vec![],
    };

    for content in source_text_list {
        let output_resource = output_handle
            .get_mut::<text_resource::TextResource>(&mut resources)
            .unwrap();
        output_resource.content = content;

        let mut bytes = vec![];
        let (nbytes, _) = resources
            .serialize_resource(text_resource::TYPE_ID, &output_handle, &mut bytes)
            .map_err(CompilerError::ResourceWriteFailed)?;

        let checksum = context
            .content_store
            .store(&bytes)
            .ok_or(CompilerError::AssetStoreError)?;

        output.compiled_resources.push(CompiledResource {
            path: context.compile_path.clone(), // todo: add stuff here to have id uniqueness.
            checksum,
            size: nbytes,
        });
    }

    Ok(output)
}

fn main() {
    std::process::exit(match compiler_main(env::args(), &COMPILER_INFO) {
        Ok(_) => 0,
        Err(_) => 1,
    });
}
