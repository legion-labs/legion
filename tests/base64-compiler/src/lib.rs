// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use base64::encode;
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
        binary_resource::BinaryResource::TYPE,
        text_resource::TextResource::TYPE,
    ),
    compiler_creator: || Box::new(Base64Compiler {}),
};

struct Base64Compiler();

#[async_trait]
impl Compiler for Base64Compiler {
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry.add_loader::<binary_resource::BinaryResource>()
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

        let resource =
            resources.load_sync::<binary_resource::BinaryResource>(context.source.resource_id());
        let resource = resource.get(&resources).unwrap();

        let base64string = encode(&resource.content);
        let compiled_asset = base64string.as_bytes();

        let asset = context.store(compiled_asset, context.target_unnamed.clone())?;

        // in this mock build dependency are _not_ runtime references.
        Ok(CompilationOutput {
            compiled_resources: vec![asset],
            resource_references: vec![],
        })
    }
}
