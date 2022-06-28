use std::str::FromStr;

// crate-specific lint exceptions:
//#![allow()]
use async_trait::async_trait;
use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_runtime::prelude::*;
use lgn_graphics_data::psd_utils::PsdFile;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline::Psd::TYPE,
        lgn_graphics_data::runtime::RawTexture::TYPE,
    ),
    compiler_creator: || Box::new(Psd2TextCompiler {}),
};

struct Psd2TextCompiler();

#[async_trait]
impl Compiler for Psd2TextCompiler {
    async fn init(&self, mut options: AssetRegistryOptions) -> AssetRegistryOptions {
        lgn_graphics_data::register_types(&mut options);
        options
    }

    async fn hash(
        &self,
        code: &'static str,
        data: &'static str,
        env: &CompilationEnv,
    ) -> CompilerHash {
        hash_code_and_data(code, data, env)
    }

    #[lgn_tracing::span_fn]
    async fn compile(
        &self,
        mut context: CompilerContext<'_>,
    ) -> Result<CompilationOutput, CompilerError> {
        let resources = context.registry();

        let outputs = {
            let resource = resources
                .load_async::<lgn_graphics_data::offline::Psd>(context.source.resource_id())
                .await?;

            // minimize lock
            let content_id = {
                let resource = resource.get().unwrap();
                resource.content_id.clone()
            };

            let identifier = lgn_content_store::Identifier::from_str(&content_id)
                .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

            // TODO: aganea - should we read from a Device directly?
            let raw_psd = context.persistent_provider.read(&identifier).await?;
            let psd_file = PsdFile::from_bytes(&raw_psd)?;

            let mut compiled_resources = vec![];

            let compiled_content = {
                let final_image = psd_file.final_texture();
                let mut content = vec![];
                lgn_data_runtime::to_binary_writer(&final_image, &mut content)
                    .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
                content
            };
            compiled_resources.push((context.target_unnamed.clone(), compiled_content));

            let compile_layer = |psd: &PsdFile, layer_name| -> Vec<u8> {
                let image = psd.layer_texture(layer_name).unwrap();
                let mut layer_content = vec![];
                lgn_data_runtime::to_binary_writer(&image, &mut layer_content)
                    .unwrap_or_else(|_| panic!("writing to file, from layer {}", layer_name));
                layer_content
            };

            for layer_name in psd_file.layer_list() {
                let pixels = compile_layer(&psd_file, layer_name);
                compiled_resources.push((context.target_unnamed.new_named(layer_name), pixels));
            }
            compiled_resources
        };

        let mut compiled_resources = vec![];
        for (id, content) in outputs {
            compiled_resources.push(context.store_volatile(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references: vec![],
        })
    }
}
