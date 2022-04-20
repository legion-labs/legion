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
use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor, Transform};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        refs_resource::TestResource::TYPE,
        refs_asset::RefsAsset::TYPE,
    ),
    compiler_creator: || Box::new(RefCompiler {}),
};

pub struct RefCompiler();

#[async_trait]
impl Compiler for RefCompiler {
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry.add_loader::<refs_resource::TestResource>()
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
            let resource = resources
                .load_async::<refs_resource::TestResource>(context.source.resource_id())
                .await;
            assert!(!resource.is_err(&resources));
            assert!(resource.is_loaded(&resources));
            let resource = resource.get(&resources).unwrap();

            let mut text = resource.content.as_bytes().to_owned();
            text.reverse();
            let mut content = text.len().to_le_bytes().to_vec();
            content.append(&mut text);

            // the compiled asset has no reference.
            let reference_id = 0u128;
            content.append(&mut reference_id.to_ne_bytes().to_vec());
            content
        };

        let asset = context
            .store(&compiled_asset, context.target_unnamed.clone())
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
