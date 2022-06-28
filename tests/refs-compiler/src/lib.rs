// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use generic_data::{offline::RefsAsset, offline::TestResource};
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
    transform: &Transform::new(TestResource::TYPE, RefsAsset::TYPE),
    compiler_creator: || Box::new(RefCompiler {}),
};

pub struct RefCompiler();

#[async_trait]
impl Compiler for RefCompiler {
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

        let compiled_asset = {
            let resource_handle = resources
                .load_async::<TestResource>(context.source.resource_id())
                .await?;
            let resource = resource_handle.get().unwrap();

            // Transform
            let mut text = resource.content.as_bytes().to_owned();
            text.reverse();

            // Create the output resource.
            let mut output_resource = RefsAsset::new_named("test");
            output_resource.content = std::str::from_utf8(&text)
                .map_err(|e| CompilerError::CompilationError(e.to_string()))?
                .to_string();

            let mut output = Vec::new();
            lgn_data_offline::to_json_writer(&output_resource, &mut output)?;
            output
        };

        let asset = context
            .store_volatile(&compiled_asset, context.target_unnamed.clone())
            .await?;

        // in this test example every build dependency becomes a reference/load-time
        // dependency.
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
}
