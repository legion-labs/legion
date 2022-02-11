use serde::{Deserialize, Serialize};

use crate::{bcn_encoder::Bc1BlockEncoder, Color};

/// High level texture format to let artist control how it is encoded
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum TextureFormat {
    /// encode RGB or RGBA channels into BC1 (4 bits per pixel).
    BC1 = 0,
    /// encode RGB or RGBA channels into BC3 (8 bits per pixel).
    BC3,
    /// Encode R channel into BC4 (4 bits per pixel)
    BC4,
    /// Encode R and G channels
    BC5,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum CompressionQuality {
    Fast = 0,
    Balanaced,
    BestQuality,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
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

use super::ColorRgb565;

pub(super) struct RawBlock4X4Rgba32 {
    block_data: [Color; 16],
}

impl RawBlock4X4Rgba32 {
    pub fn from_rgba_array(
        rgba: &[u8],
        height_start: usize,
        height_stride: usize,
        width_start: usize,
        width_stride: usize,
        color_channels: ColorChannels,
    ) -> Self {
        let mut block_data = [Color::from((0, 0, 0)); 16];
        for height_index in height_start..height_start + 4 {
            for width_index in width_start..width_start + 4 {
                let r = rgba[height_index * height_stride + width_index * width_stride];
                let g = match color_channels {
                    ColorChannels::R | ColorChannels::Ra => r,
                    ColorChannels::Rgb | ColorChannels::Rgba => {
                        rgba[height_index * height_stride + width_index * width_stride + 1]
                    }
                };

                let b = match color_channels {
                    ColorChannels::R | ColorChannels::Ra => r,
                    ColorChannels::Rgb | ColorChannels::Rgba => {
                        rgba[height_index * height_stride + width_index * width_stride + 2]
                    }
                };

                let a = match color_channels {
                    ColorChannels::R | ColorChannels::Rgb => 1,
                    ColorChannels::Ra | ColorChannels::Rgba => {
                        rgba[height_index * height_stride + width_index * width_stride + 3]
                    }
                };

                let block_index = (height_index - height_start) * 4 + width_index - width_start;
                block_data[block_index] = Color::from((r, g, b, a));
            }
        }
        Self { block_data }
    }

    pub fn as_array(&self) -> [Color; 16] {
        self.block_data
    }

    pub fn has_transparent_pixels(&self) -> bool {
        for pixel in self.block_data {
            if pixel.a < 255 {
                return true;
            }
        }
        false
    }
}

const VAR_PATTERN_COUNT: usize = 24;
const VARIATE_PATTERN_EP0_R: [i32; VAR_PATTERN_COUNT] = [
    1, 1, 0, 0, -1, 0, 0, -1, 1, -1, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const VARIATE_PATTERN_EP0_G: [i32; VAR_PATTERN_COUNT] = [
    1, 0, 1, 0, 0, -1, 0, -1, 1, -1, 0, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const VARIATE_PATTERN_EP0_B: [i32; VAR_PATTERN_COUNT] = [
    1, 0, 0, 1, 0, 0, -1, -1, 1, -1, 0, 0, 1, 0, 0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
];
const VARIATE_PATTERN_EP1_R: [i32; VAR_PATTERN_COUNT] = [
    -1, -1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, -1, 1, 0, 0, -1, 0, 0,
];
const VARIATE_PATTERN_EP1_G: [i32; VAR_PATTERN_COUNT] = [
    -1, 0, -1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, -1, 0, 1, 0, 0, -1, 0,
];
const VARIATE_PATTERN_EP1_B: [i32; VAR_PATTERN_COUNT] = [
    -1, 0, 0, -1, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, -1, 0, 0, 1, 0, 0, -1,
];

fn clamp_to_byte(value: i32) -> i32 {
    if value < 0 {
        0
    } else if value > 255 {
        255
    } else {
        value
    }
}

#[allow(clippy::cast_precision_loss)]
pub(crate) fn variate_565(
    c0: ColorRgb565,
    c1: ColorRgb565,
    i: usize,
) -> (ColorRgb565, ColorRgb565) {
    let idx = i % VAR_PATTERN_COUNT;
    let mut new_ep0 = ColorRgb565::default();
    let mut new_ep1 = ColorRgb565::default();

    new_ep0.set_raw_r(clamp_to_byte(
        i32::from(c0.get_raw_r()) + VARIATE_PATTERN_EP0_R[idx],
    ));
    new_ep0.set_raw_g(clamp_to_byte(
        i32::from(c0.get_raw_g()) + VARIATE_PATTERN_EP0_G[idx],
    ));
    new_ep0.set_raw_b(clamp_to_byte(
        i32::from(c0.get_raw_b()) + VARIATE_PATTERN_EP0_B[idx],
    ));

    new_ep1.set_raw_r(clamp_to_byte(
        i32::from(c1.get_raw_r()) + VARIATE_PATTERN_EP1_R[idx],
    ));

    new_ep1.set_raw_g(clamp_to_byte(
        i32::from(c1.get_raw_g()) + VARIATE_PATTERN_EP1_G[idx],
    ));
    new_ep1.set_raw_b(clamp_to_byte(
        i32::from(c1.get_raw_b()) + VARIATE_PATTERN_EP1_B[idx],
    ));

    (new_ep0, new_ep1)
}

pub(crate) fn encode_mip_chain_from_offline_texture(
    width: usize,
    height: usize,
    format: TextureFormat,
    quality: CompressionQuality,
    color_channels: ColorChannels,
    rgba: &[u8],
) -> Vec<Vec<u8>> {
    let block_byte_size = match format {
        TextureFormat::BC1 | TextureFormat::BC4 => 8,
        TextureFormat::BC3 | TextureFormat::BC5 => 16,
    };
    let mip_width_in_blocks = width / 4;
    let mip_height_in_blocks = height / 4;

    let mut mip_data =
        Vec::with_capacity(mip_height_in_blocks * mip_width_in_blocks * block_byte_size);

    #[allow(unsafe_code)]
    let mip_data_comp = unsafe {
        tbc::encode_image_bc1_conv_u8(
            std::slice::from_raw_parts(
                rgba.as_ptr().cast::<tbc::color::Rgb8>(),
                rgba.len() / std::mem::size_of::<tbc::color::Rgb8>(),
            ),
            width,
            height,
        )
    };

    let pixel_width = match color_channels {
        ColorChannels::R => 1usize,
        ColorChannels::Ra => 2usize,
        ColorChannels::Rgb => 3usize,
        ColorChannels::Rgba => 4usize,
    };

    #[allow(clippy::todo)]
    for block_height_index in 0..mip_height_in_blocks {
        for block_width_index in 0..mip_width_in_blocks {
            let height_stride = (width * pixel_width) as usize;
            let width_stride = pixel_width;

            let raw_block = RawBlock4X4Rgba32::from_rgba_array(
                rgba,
                block_height_index * 4,
                height_stride,
                block_width_index * 4,
                width_stride,
                color_channels,
            );

            match format {
                TextureFormat::BC1 => {
                    Bc1BlockEncoder::encode_block(&raw_block, quality).write_block(&mut mip_data);
                }
                TextureFormat::BC3 | TextureFormat::BC4 | TextureFormat::BC5 => todo!(),
            };
        }
    }

    vec![mip_data_comp]
}
