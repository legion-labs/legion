use lgn_tracing::{span_fn, span_scope};
use serde::{Deserialize, Serialize};

/// High level texture format to let artist control how it is encoded
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TextureFormat {
    /// encode RGB or RGBA channels into BC1 (4 bits per pixel).
    BC1 = 0,
    /// encode RGB or RGBA channels into BC3 (8 bits per pixel).
    BC3,
    /// Encode R channel into BC4 (4 bits per pixel)
    BC4,
    /// encode RGB or RGBA channels into BC3 (8 bits per pixel).
    BC7,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum CompressionQuality {
    Fast = 0,
    Balanaced,
    BestQuality,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ColorChannels {
    /// Single channel (greyscale)
    R = 0,
    // Single color channel pluus alpha
    Ra,
    /// Full color samples, no alpha
    Rgb,
    /// Full color samples with alpha
    Rgba,
}

fn average_color_values(data: &[u8], v0: u32, v1: u32, u0: u32, u1: u32, offset: usize) -> u8 {
    ((u32::from(data[(v0 + u0) as usize + offset])
        + u32::from(data[(v0 + u1) as usize + offset])
        + u32::from(data[(v1 + u0) as usize + offset])
        + u32::from(data[(v1 + u1) as usize + offset]))
        / 4) as u8
}

#[span_fn]
fn calc_mip_chain(
    width: u32,
    height: u32,
    format: TextureFormat,
    alpha_blended: bool,
    rgba: &[u8],
    mip_chain: &mut Vec<Vec<u8>>,
) {
    let pixel_size = if format == TextureFormat::BC4 { 1 } else { 4 };
    let stride = width * pixel_size;

    let surface = ispc_tex::RgbaSurface {
        data: rgba,
        width,
        height,
        stride,
    };

    mip_chain.push(match format {
        TextureFormat::BC1 => ispc_tex::bc1::compress_blocks(&surface),
        TextureFormat::BC3 => ispc_tex::bc3::compress_blocks(&surface),
        #[allow(unsafe_code)]
        TextureFormat::BC4 => unsafe {
            span_scope!("tbc::encode_image_bc4_r8_conv_u8");
            tbc::encode_image_bc4_r8_conv_u8(
                std::slice::from_raw_parts(
                    rgba.as_ptr().cast::<tbc::color::Red8>(),
                    rgba.len() / std::mem::size_of::<tbc::color::Red8>(),
                ),
                width as usize,
                height as usize,
            )
        },
        TextureFormat::BC7 => {
            if alpha_blended {
                span_scope!("ispc_tex::bc7::compress_blocks_alpha");
                ispc_tex::bc7::compress_blocks(&ispc_tex::bc7::opaque_basic_settings(), &surface)
            } else {
                span_scope!(" ispc_tex::bc7::compress_blocks");
                ispc_tex::bc7::compress_blocks(&ispc_tex::bc7::alpha_basic_settings(), &surface)
            }
        }
    });

    let mut new_mip = Vec::with_capacity(rgba.len() / 4);
    for v in 0..(height / 2) {
        for u in 0..(width / 2) {
            let v0 = (v * 2) * stride;
            let v1 = ((v * 2) + 1) * stride;
            let u0 = (u * 2) * pixel_size;
            let u1 = ((u * 2) + 1) * pixel_size;

            new_mip.push(average_color_values(rgba, v0, v1, u0, u1, 0usize));
            if pixel_size > 1 {
                new_mip.push(average_color_values(rgba, v0, v1, u0, u1, 1));
            }
            if pixel_size > 2 {
                new_mip.push(average_color_values(rgba, v0, v1, u0, u1, 2));
            }
            if pixel_size > 3 {
                new_mip.push(average_color_values(rgba, v0, v1, u0, u1, 3));
            }
        }
    }

    if width > 4 && height > 4 {
        let width = width / 2;
        let height = height / 2;

        calc_mip_chain(width, height, format, alpha_blended, &new_mip, mip_chain);
    }
}

#[span_fn]
pub fn encode_mip_chain_from_offline_texture(
    width: u32,
    height: u32,
    format: TextureFormat,
    alpha_blended: bool,
    rgba: &[u8],
) -> Vec<Vec<u8>> {
    let mut mip_chain = Vec::new();

    calc_mip_chain(width, height, format, alpha_blended, rgba, &mut mip_chain);

    mip_chain
}

// Vec<u8> as input is needed because it may be returned as output
#[allow(clippy::ptr_arg)]
#[span_fn]
pub fn rgba_from_source(
    width: u32,
    height: u32,
    color_channels: ColorChannels,
    data: &Vec<u8>,
) -> Vec<u8> {
    // TODO - Replace when switching BC4 encoder to ispc_tex
    if color_channels == ColorChannels::R {
        data.clone()
    } else {
        let source_w_stride = match color_channels {
            ColorChannels::R => 1usize,
            ColorChannels::Ra => 2usize,
            ColorChannels::Rgb => 3usize,
            ColorChannels::Rgba => 4usize,
        };
        let source_h_stride = width as usize * source_w_stride;

        let mut rgba = Vec::with_capacity(data.len());
        for h in 0..height as usize {
            for w in 0..width as usize {
                let base_source_offset = h * source_h_stride + w * source_w_stride;

                rgba.push(data[base_source_offset]);
                rgba.push(if source_w_stride > 1 {
                    data[base_source_offset + 1]
                } else {
                    0
                });
                rgba.push(if source_w_stride > 2 {
                    data[base_source_offset + 2]
                } else {
                    0
                });
                rgba.push(if source_w_stride > 3 {
                    data[base_source_offset + 3]
                } else {
                    255
                });
            }
        }
        rgba
    }
}
