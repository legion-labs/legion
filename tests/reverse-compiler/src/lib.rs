// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_runtime::{
    AssetRegistryError, AssetRegistryOptions, ResourceDescriptor, ResourceProcessor, Transform,
};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        text_resource::TextResource::TYPE,
        text_resource::TextResource::TYPE,
    ),
    compiler_creator: || Box::new(ReverseCompiler {}),
};

struct ReverseCompiler();

#[async_trait]
impl Compiler for ReverseCompiler {
    async fn init(&self, mut registry: AssetRegistryOptions) -> AssetRegistryOptions {
        text_resource::TextResource::register_type(&mut registry);
        registry
    }

    async fn hash(
        &self,
        code: &'static str,
        data: &'static str,
        env: &CompilationEnv,
    ) -> CompilerHash {
        hash_code_and_data(code, data, env)
    }

    async fn compile(
        &self,
        mut context: CompilerContext<'_>,
    ) -> Result<CompilationOutput, CompilerError> {
        let resources = context.registry();

        let bytes = {
            let resource = resources
                .load_async::<text_resource::TextResource>(context.source.resource_id())
                .await?;
            let resource = resource.get().ok_or_else(|| {
                AssetRegistryError::ResourceNotFound(context.source.resource_id())
            })?;

            let mut bytes = vec![];
            let output = text_resource::TextResource {
                content: resource.content.chars().rev().collect(),
            };

            let processor = text_resource::TextResourceProc {};
            let _nbytes = processor.write_resource(&output, &mut bytes)?;
            bytes
        };

        let asset = context
            .store(&bytes, context.target_unnamed.clone())
            .await?;

        // in this mock build dependency are _not_ runtime references.
        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
