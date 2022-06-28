// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use generic_data::offline::{IntegerAsset, TextResource};
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
    transform: &Transform::new(TextResource::TYPE, IntegerAsset::TYPE),
    compiler_creator: || Box::new(AtoiCompiler {}),
};

struct AtoiCompiler();

#[async_trait]
impl Compiler for AtoiCompiler {
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

        let compiled_output = {
            let resource = resources
                .load_async::<TextResource>(context.source.resource_id())
                .await?;
            let resource = resource.get().ok_or_else(|| {
                AssetRegistryError::ResourceNotFound(context.source.resource_id())
            })?;

            let mut integer_asset = IntegerAsset::new_named("test");
            integer_asset.magic_value = resource.content.parse::<i32>().unwrap_or(0);

            let mut compiled_output = Vec::new();
            lgn_data_offline::to_json_writer(&integer_asset, &mut compiled_output)?;
            compiled_output
        };

        let asset = context
            .store_volatile(&compiled_output, context.target_unnamed.clone())
            .await?;

        // in this mock build dependency are _not_ runtime references.
        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
