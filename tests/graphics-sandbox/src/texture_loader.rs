use lgn_graphics_data::{
    encode_mip_chain_from_offline_texture, rgba_from_source, ColorChannels, TextureFormat,
};
use lgn_graphics_renderer::{components::TextureComponent, resources::GpuUniformDataContext};
use png::ColorType;

use std::path::Path;

pub fn load_texture(
    file_name: &Path,
    data_context: &mut GpuUniformDataContext<'_>,
) -> TextureComponent {
    let ref_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("refs")
        .join("test-materials")
        .join(file_name)
        .with_extension("png");

    let raw_data = crate::meta_cube_test::load_image(&ref_path).unwrap();

    let width = raw_data.info.width;
    let height = raw_data.info.height;

    let (format, color_channels) = match raw_data.info.color_type {
        ColorType::Rgb => (TextureFormat::BC7, ColorChannels::Rgb),
        ColorType::Rgba => (TextureFormat::BC7, ColorChannels::Rgba),
        ColorType::Grayscale => (TextureFormat::BC4, ColorChannels::R),
        _ => unreachable!(),
    };

    let mip_chain = encode_mip_chain_from_offline_texture(
        width,
        height,
        format,
        false,
        &rgba_from_source(width, height, color_channels, &raw_data.data),
    );

    TextureComponent::new(mip_chain, format, width, height, data_context)
}
