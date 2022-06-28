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
use lgn_graphics_data::{rgba_from_source, ColorChannels};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline::Png::TYPE,
        lgn_graphics_data::runtime::RawTexture::TYPE,
    ),
    compiler_creator: || Box::new(Png2TexCompiler {}),
};

struct Png2TexCompiler();

#[async_trait]
impl Compiler for Png2TexCompiler {
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
        let asset_registry = context.registry();

        let content = {
            let png_file = asset_registry
                .load_async::<lgn_graphics_data::offline::Png>(context.source.resource_id())
                .await?;

            // minimize lock
            let content_id = {
                let png_file = png_file.get().unwrap();
                png_file.content_id.clone()
            };

            let identifier = lgn_content_store::Identifier::from_str(&content_id)
                .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

            // TODO: aganea - should we read from a Device directly?
            let raw_data = context.persistent_provider.read(&identifier).await?;

            let decoder = png::Decoder::new(raw_data.as_slice());
            let mut reader = decoder.read_info().map_err(|err| {
                CompilerError::CompilationError(format!(
                    "Failed to read png info for resource {} ({})",
                    context.source.resource_id(),
                    err
                ))
            })?;
            let mut img_data = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut img_data).map_err(|err| {
                CompilerError::CompilationError(format!(
                    "Failed to read png next frame for resource '{}' ({})",
                    context.source.resource_id(),
                    err
                ))
            })?;

            let texture = if info.color_type != png::ColorType::Indexed {
                let color_channels = match info.color_type {
                    png::ColorType::Grayscale => ColorChannels::R,
                    png::ColorType::Rgb | png::ColorType::Indexed => ColorChannels::Rgb,
                    png::ColorType::GrayscaleAlpha => ColorChannels::Ra,
                    png::ColorType::Rgba => ColorChannels::Rgba,
                };
                Ok(lgn_graphics_data::runtime::RawTexture {
                    kind: lgn_graphics_data::TextureType::_2D,
                    width: info.width,
                    height: info.height,
                    rgba: rgba_from_source(info.width, info.height, color_channels, &img_data),
                })
            } else {
                Err(CompilerError::CompilationError(format!(
                    "Unsupported indexed png format for resource '{}'",
                    context.source.resource_id(),
                )))
            }?;

            let mut content = vec![];
            lgn_data_runtime::to_binary_writer(&texture, &mut content)?;
            content
        };

        let output = context
            .store_volatile(&content, context.target_unnamed.clone())
            .await?;

        Ok(CompilationOutput {
            compiled_resources: vec![output],
            resource_references: vec![],
        })
    }
}
