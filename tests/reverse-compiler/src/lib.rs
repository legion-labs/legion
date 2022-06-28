// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use generic_data::offline::TextResource;
use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::offline::Metadata;
use lgn_data_runtime::prelude::*;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(TextResource::TYPE, TextResource::TYPE),
    compiler_creator: || Box::new(ReverseCompiler {}),
};

struct ReverseCompiler();

#[async_trait]
impl Compiler for ReverseCompiler {
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

        let bytes = {
            let resource = resources
                .load_async::<TextResource>(context.source.resource_id())
                .await?;
            let resource = resource.get().ok_or_else(|| {
                AssetRegistryError::ResourceNotFound(context.source.resource_id())
            })?;

            let mut bytes = vec![];
            let output = TextResource {
                meta: Metadata::new_default::<TextResource>(),
                content: resource.content.chars().rev().collect(),
            };

            lgn_data_offline::to_json_writer(&output, &mut bytes)?;
            bytes
        };

        let asset = context
            .store_volatile(&bytes, context.target_unnamed.clone())
            .await?;

        // in this mock build dependency are _not_ runtime references.
        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
