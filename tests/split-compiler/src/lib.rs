// crate-specific lint exceptions:
//#![allow()]

use async_trait::async_trait;
use std::env;

use generic_data::offline::{MultiTextResource, TextResource};
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
    transform: &Transform::new(MultiTextResource::TYPE, TextResource::TYPE),
    compiler_creator: || Box::new(SplitCompiler {}),
};

struct SplitCompiler();

#[async_trait]
impl Compiler for SplitCompiler {
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

        let resource = resources
            .load_async::<MultiTextResource>(context.source.resource_id())
            .await?;

        let content_list = {
            let resource = resource.get().unwrap();

            let source_text_list = resource.text_list.clone();

            let content_list = source_text_list
                .iter()
                .enumerate()
                .map(|(index, content)| {
                    let output_resource = TextResource {
                        meta: Metadata::new_default::<TextResource>(),
                        content: content.clone(),
                    };

                    let mut bytes = vec![];
                    let _nbytes =
                        lgn_data_offline::to_json_writer(&output_resource, &mut bytes).unwrap();
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
            let asset = context.store_volatile(&bytes, id).await?;

            output.compiled_resources.push(asset);
        }

        Ok(output)
    }
}
