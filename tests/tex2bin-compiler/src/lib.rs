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
use lgn_data_runtime::prelude::*;
use lgn_graphics_data::{encode_mip_chain_from_offline_texture, TextureFormat};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::runtime::RawTexture::TYPE,
        lgn_graphics_data::runtime::BinTexture::TYPE,
    ),
    compiler_creator: || Box::new(Tex2BinCompiler {}),
};

fn compile_from_offline(
    width: u32,
    height: u32,
    format: TextureFormat,
    srgb: bool,
    alpha_blended: bool,
    rgba: &[u8],
    writer: &mut dyn std::io::Write,
) {
    let texture = lgn_graphics_data::runtime::BinTexture {
        width,
        height,
        format,
        srgb,
        mips: encode_mip_chain_from_offline_texture(width, height, format, alpha_blended, rgba)
            .into_iter()
            .map(|texel_data| lgn_graphics_data::runtime::Mips {
                texel_data: serde_bytes::ByteBuf::from(texel_data),
            })
            .collect::<Vec<_>>(),
    };
    lgn_data_runtime::to_binary_writer(&texture, writer).unwrap();
}

struct Tex2BinCompiler();

#[async_trait]
impl Compiler for Tex2BinCompiler {
    async fn init(&self, mut registry: AssetRegistryOptions) -> AssetRegistryOptions {
        lgn_graphics_data::register_types(&mut registry);
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

        let output = {
            let image = {
                let image = resources
                    .load_async::<lgn_graphics_data::runtime::RawTexture>(
                        context.source.resource_id(),
                    )
                    .await?;
                let image = image.get().ok_or_else(|| {
                    AssetRegistryError::ResourceNotFound(context.source.resource_id())
                })?;

                image.clone()
            };
            let target_unnamed = context.target_unnamed.clone();
            CompilerContext::execute_workload(move || {
                let mut compiled_resources = Vec::<(ResourcePathId, Vec<u8>)>::new();

                let pixel_size = image.rgba.len() as u32 / image.width / image.height;
                if pixel_size == 1 {
                    let mut compiled_asset = vec![];
                    compile_from_offline(
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
                    compile_from_offline(
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
                    compile_from_offline(
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
                    compile_from_offline(
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
            compiled_resources.push(context.store_volatile(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references: vec![],
        })
    }
}
