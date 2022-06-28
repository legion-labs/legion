// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use base64::encode;
use generic_data::offline::{BinaryResource, TextResource};
use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::SourceResource;
use lgn_data_runtime::prelude::*;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(BinaryResource::TYPE, TextResource::TYPE),
    compiler_creator: || Box::new(Base64Compiler {}),
};

struct Base64Compiler();

#[async_trait]
impl Compiler for Base64Compiler {
    async fn init(&self, mut registry: AssetRegistryOptions) -> AssetRegistryOptions {
        generic_data::register_types(&mut registry);
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

        let output = {
            let resource = resources
                .load_async::<BinaryResource>(context.source.resource_id())
                .await?;
            let resource = resource.get().ok_or_else(|| {
                AssetRegistryError::ResourceNotFound(context.source.resource_id())
            })?;

            let mut text_resource = TextResource::new_named("test");
            text_resource.content = encode(&resource.content);
            let mut output = Vec::<u8>::new();
            lgn_data_offline::to_json_writer(&text_resource, &mut output)?;
            output
        };

        let asset = context
            .store_volatile(&output, context.target_unnamed.clone())
            .await?;

        // in this mock build dependency are _not_ runtime references.
        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
