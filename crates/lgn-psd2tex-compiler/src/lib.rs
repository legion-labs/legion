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
use lgn_data_runtime::{AssetRegistryOptions, Resource, ResourceProcessor, Transform};
use lgn_graphics_data::{offline_psd::PsdFile, offline_texture::TextureProcessor};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_psd::PsdFile::TYPE,
        lgn_graphics_data::offline_texture::Texture::TYPE,
    ),
    compiler_creator: || Box::new(Psd2TextCompiler {}),
};

struct Psd2TextCompiler();

#[async_trait]
impl Compiler for Psd2TextCompiler {
    async fn init(&self, options: AssetRegistryOptions) -> AssetRegistryOptions {
        options.add_loader::<lgn_graphics_data::offline_psd::PsdFile>()
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
                .load_async::<lgn_graphics_data::offline_psd::PsdFile>(context.source.resource_id())
                .await;

            let resource = resource.get(&resources).unwrap();

            let mut compiled_resources = vec![];
            let texture_proc = TextureProcessor {};

            let compiled_content = {
                let final_image = resource.final_texture().ok_or_else(|| {
                    CompilerError::CompilationError("Failed to generate texture".into())
                })?;
                let mut content = vec![];
                texture_proc
                    .write_resource(&final_image, &mut content)
                    .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
                content
            };
            compiled_resources.push((context.target_unnamed.clone(), compiled_content));

            let compile_layer = |psd: &PsdFile, layer_name| -> Vec<u8> {
                let image = psd.layer_texture(layer_name).unwrap();
                let mut layer_content = vec![];
                texture_proc
                    .write_resource(&image, &mut layer_content)
                    .unwrap_or_else(|_| panic!("writing to file, from layer {}", layer_name));
                layer_content
            };

            for layer_name in resource.layer_list().ok_or_else(|| {
                CompilerError::CompilationError("Failed to extract layer names".into())
            })? {
                let pixels = compile_layer(&resource, layer_name);
                compiled_resources.push((context.target_unnamed.new_named(layer_name), pixels));
            }
            compiled_resources
        };

        let mut compiled_resources = vec![];
        for (id, content) in outputs {
            compiled_resources.push(context.store(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references: vec![],
        })
    }
}
