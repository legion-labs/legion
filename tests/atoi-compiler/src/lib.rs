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
use lgn_data_offline::Transform;
use lgn_data_runtime::{AssetRegistryOptions, Resource};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        text_resource::TextResource::TYPE,
        integer_asset::IntegerAsset::TYPE,
    ),
    compiler_creator: || Box::new(AtoiCompiler {}),
};

struct AtoiCompiler();

#[async_trait]
impl Compiler for AtoiCompiler {
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry.add_loader::<text_resource::TextResource>()
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
            let resource =
                resources.load_sync::<text_resource::TextResource>(context.source.resource_id());
            let resource = resource.get(&resources).unwrap();

            let parsed_value = resource.content.parse::<usize>().unwrap_or(0);
            parsed_value.to_ne_bytes()
        };

        let asset = context
            .store(&compiled_output, context.target_unnamed.clone())
            .await?;

        // in this mock build dependency are _not_ runtime references.
        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
