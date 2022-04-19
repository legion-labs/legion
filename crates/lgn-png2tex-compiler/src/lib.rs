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
use lgn_data_runtime::{
    AssetRegistryError, AssetRegistryOptions, ResourceDescriptor, ResourceProcessor, Transform,
};
use lgn_graphics_data::{offline_texture::TextureProcessor, rgba_from_source, ColorChannels};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_png::PngFile::TYPE,
        lgn_graphics_data::offline_texture::Texture::TYPE,
    ),
    compiler_creator: || Box::new(Png2TexCompiler {}),
};

struct Png2TexCompiler();

#[async_trait]
impl Compiler for Png2TexCompiler {
    async fn init(&self, mut options: AssetRegistryOptions) -> AssetRegistryOptions {
        lgn_graphics_data::offline_png::PngFile::register_type(&mut options);
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
                .load_async::<lgn_graphics_data::offline_png::PngFile>(context.source.resource_id())
                .await?;

            let png_file = png_file.get().ok_or_else(|| {
                AssetRegistryError::ResourceNotFound(context.source.resource_id())
            })?;

            let decoder = png::Decoder::new(png_file.content.as_slice());
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
                Ok(lgn_graphics_data::offline_texture::Texture {
                    kind: lgn_graphics_data::offline_texture::TextureType::_2D,
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
            let texture_proc = TextureProcessor {};
            texture_proc
                .write_resource(&texture, &mut content)
                .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
            content
        };

        let output = context
            .store(&content, context.target_unnamed.clone())
            .await?;

        Ok(CompilationOutput {
            compiled_resources: vec![output],
            resource_references: vec![],
        })
    }
}
