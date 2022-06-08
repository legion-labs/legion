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
use lgn_graphics_data::{runtime_texture, TextureFormat};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_texture::Texture::TYPE,
        lgn_graphics_data::runtime_texture::Texture::TYPE,
    ),
    compiler_creator: || Box::new(Tex2BinCompiler {}),
};

struct Tex2BinCompiler();

#[async_trait]
impl Compiler for Tex2BinCompiler {
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry.add_loader::<lgn_graphics_data::offline_texture::Texture>()
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

        let output = {
            let resource = resources
                .load_async::<lgn_graphics_data::offline_texture::Texture>(
                    context.source.resource_id(),
                )
                .await;

            let target_unnamed = context.target_unnamed.clone();
            let image = resource.get(&resources).unwrap().clone();

            CompilerContext::execute_workload(move || {
                let mut compiled_resources = vec![];

                let pixel_size = image.rgba.len() as u32 / image.width / image.height;
                if pixel_size == 1 {
                    let mut compiled_asset = vec![];
                    runtime_texture::Texture::compile_from_offline(
                        image.width,
                        image.height,
                        TextureFormat::BC4,
                        false,
                        false,
                        &image.rgba,
                        &mut compiled_asset,
                    );

                    compiled_resources.push((
                        target_unnamed.new_named("Roughness"),
                        compiled_asset.clone(),
                    ));
                    compiled_resources.push((
                        target_unnamed.new_named("Metalness"),
                        compiled_asset.clone(),
                    ));
                } else {
                    let mut compiled_asset_srgb = vec![];
                    runtime_texture::Texture::compile_from_offline(
                        image.width,
                        image.height,
                        TextureFormat::BC7,
                        true,
                        false,
                        &image.rgba,
                        &mut compiled_asset_srgb,
                    );

                    compiled_resources
                        .push((target_unnamed.new_named("Albedo"), compiled_asset_srgb));

                    // todo: normal compression doest require bc7 (verify and modify)
                    let mut compiled_asset_linear = vec![];
                    runtime_texture::Texture::compile_from_offline(
                        image.width,
                        image.height,
                        TextureFormat::BC7,
                        false,
                        false,
                        &image.rgba,
                        &mut compiled_asset_linear,
                    );

                    compiled_resources
                        .push((target_unnamed.new_named("Normal"), compiled_asset_linear));

                    let mut compiled_asset_blended = vec![];
                    runtime_texture::Texture::compile_from_offline(
                        image.width,
                        image.height,
                        TextureFormat::BC7,
                        true,
                        true,
                        &image.rgba,
                        &mut compiled_asset_blended,
                    );

                    compiled_resources.push((
                        target_unnamed.new_named("AlbedoBlend"),
                        compiled_asset_blended,
                    ));
                }
                Ok(compiled_resources)
            })
            .await?
        };

        let mut compiled_resources = vec![];
        for (id, content) in output {
            compiled_resources.push(context.store(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references: vec![],
        })
    }
}
