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
use lgn_data_runtime::{AssetRegistryOptions, Resource, ResourceProcessor, Transform};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        multitext_resource::MultiTextResource::TYPE,
        text_resource::TextResource::TYPE,
    ),
    compiler_creator: || Box::new(SplitCompiler {}),
};

struct SplitCompiler();

#[async_trait]
impl Compiler for SplitCompiler {
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry
            .add_loader::<multitext_resource::MultiTextResource>()
            .add_loader::<text_resource::TextResource>()
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

        let resource = resources
            .load_async::<multitext_resource::MultiTextResource>(context.source.resource_id())
            .await;

        let content_list = {
            let resource = resource.get(&resources).unwrap();

            let source_text_list = resource.text_list.clone();

            let proc = text_resource::TextResourceProc {};

            let content_list = source_text_list
                .iter()
                .enumerate()
                .map(|(index, content)| {
                    let output_resource = text_resource::TextResource {
                        content: content.clone(),
                    };

                    let mut bytes = vec![];
                    let _nbytes = proc.write_resource(&output_resource, &mut bytes);
                    // todo: handle error

                    (
                        bytes,
                        context.target_unnamed.new_named(&format!("text_{}", index)),
                    )
                })
                .collect::<Vec<_>>();
            content_list
        };

        let mut output = CompilationOutput {
            compiled_resources: vec![],
            resource_references: vec![],
        };

        for (bytes, id) in content_list {
            let asset = context.store(&bytes, id).await?;

            output.compiled_resources.push(asset);
        }

        Ok(output)
    }
}
