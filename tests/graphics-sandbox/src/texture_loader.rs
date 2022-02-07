use lgn_graphics_api::Format;
use lgn_renderer::{components::TextureComponent, resources::GpuUniformDataContext};
use png::{ColorType, OutputInfo};

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
    let mut mip_chain = Vec::new();

    let width = raw_data.info.width;
    let height = raw_data.info.height;

    let format = match raw_data.info.color_type {
        ColorType::Rgb => Format::BC1_RGBA_UNORM_BLOCK,
        ColorType::Rgba => Format::BC1_RGBA_UNORM_BLOCK,
        ColorType::Grayscale => Format::BC4_UNORM_BLOCK,
        _ => unreachable!(),
    };

    calc_mip_chain(&raw_data.data, raw_data.info, &mut mip_chain);

    fn average_color_values(
        data: &[u8],
        v0: usize,
        v1: usize,
        u0: usize,
        u1: usize,
        offset: usize,
    ) -> u8 {
        ((u32::from(data[v0 + u0 + offset])
            + u32::from(data[v0 + u1 + offset])
            + u32::from(data[v1 + u0 + offset])
            + u32::from(data[v1 + u1 + offset]))
            / 4) as u8
    }

    fn calc_mip_chain(data: &[u8], mut info: OutputInfo, mip_chain: &mut Vec<Vec<u8>>) {
        #[allow(unsafe_code)]
        mip_chain.push(unsafe {
            match info.color_type {
                ColorType::Rgb => tbc::encode_image_bc1_conv_u8(
                    std::slice::from_raw_parts(
                        data.as_ptr().cast::<tbc::color::Rgb8>(),
                        data.len() / std::mem::size_of::<tbc::color::Rgb8>(),
                    ),
                    info.width as usize,
                    info.height as usize,
                ),
                ColorType::Rgba => tbc::encode_image_bc1_conv_u8(
                    std::slice::from_raw_parts(
                        data.as_ptr().cast::<tbc::color::Rgba8>(),
                        data.len() / std::mem::size_of::<tbc::color::Rgba8>(),
                    ),
                    info.width as usize,
                    info.height as usize,
                ),
                ColorType::Grayscale => tbc::encode_image_bc4_r8_conv_u8(
                    std::slice::from_raw_parts(
                        data.as_ptr().cast::<tbc::color::Red8>(),
                        data.len() / std::mem::size_of::<tbc::color::Red8>(),
                    ),
                    info.width as usize,
                    info.height as usize,
                ),
                _ => unreachable!(),
            }
        });

        let mut new_mip = Vec::with_capacity(data.len() / 4);
        for v in 0..(info.height / 2) as usize {
            for u in 0..(info.width / 2) as usize {
                let component_size = match info.color_type {
                    ColorType::Rgb => 3usize,
                    ColorType::Rgba => 4usize,
                    ColorType::Grayscale => 1usize,
                    _ => unreachable!(),
                };
                let v0 = (v * 2) * info.line_size;
                let v1 = ((v * 2) + 1) * info.line_size;
                let u0 = (u * 2) * component_size;
                let u1 = ((u * 2) + 1) * component_size;

                new_mip.push(average_color_values(data, v0, v1, u0, u1, 0usize));
                if component_size > 1 {
                    new_mip.push(average_color_values(data, v0, v1, u0, u1, 1));
                }
                if component_size > 2 {
                    new_mip.push(average_color_values(data, v0, v1, u0, u1, 2));
                }
                if component_size > 3 {
                    new_mip.push(average_color_values(data, v0, v1, u0, u1, 3));
                }
            }
        }

        if info.width > 4 && info.height > 4 {
            info.width /= 2;
            info.height /= 2;
            info.line_size /= 2;

            calc_mip_chain(&new_mip, info, mip_chain);
        }
    }

    let mip_count = mip_chain.len() as u32;
    TextureComponent::new(mip_chain, format, width, height, mip_count, data_context)
}
