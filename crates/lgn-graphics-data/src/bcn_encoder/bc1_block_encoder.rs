use crate::{
    bcn_encoder::{interpolate_color, variate_565, PcaVectors},
    Color,
};

use super::{ColorRgb565, CompressionQuality, RawBlock4X4Rgba32, RgbBoundingBox};

#[derive(Default)]
pub(crate) struct Bc1Block {
    pub(crate) color_0: ColorRgb565,
    pub(crate) color_1: ColorRgb565,
    pub(crate) color_indices: u32,
}

impl Bc1Block {
    pub fn write_block(&self, encoded_data: &mut Vec<u8>) {
        encoded_data.push((self.color_1.data) as u8);
        encoded_data.push((self.color_1.data >> 8) as u8);
        encoded_data.push((self.color_0.data) as u8);
        encoded_data.push((self.color_0.data >> 8) as u8);

        encoded_data.push((self.color_indices) as u8);
        encoded_data.push((self.color_indices >> 8) as u8);
        encoded_data.push((self.color_indices >> 16) as u8);
        encoded_data.push((self.color_indices >> 24) as u8);
    }
}

impl Bc1Block {
    pub fn set_color_index(&mut self, index: u32, value: u32) {
        self.color_indices &= !(0b11 << (index * 2));
        let val = value & 0b11;
        self.color_indices |= val << (index * 2);
    }

    pub fn has_alpha_or_blackk(&self) -> bool {
        self.color_0.data <= self.color_1.data
    }
}
pub(super) struct Bc1BlockEncoder {}

const MAX_TRIES: usize = 24 * 2;
const ERROR_THRESHOLD: f32 = 0.05;

impl Bc1BlockEncoder {
    pub fn encode_block(raw_block: &RawBlock4X4Rgba32, quality: CompressionQuality) -> Bc1Block {
        match quality {
            CompressionQuality::Fast => Self::encode_block_fast(raw_block),
            CompressionQuality::Balanaced => Self::encode_block_balanced(raw_block),
            CompressionQuality::BestQuality => Self::encode_block_best_quality(raw_block),
        }
    }

    fn encode_block_fast(raw_block: &RawBlock4X4Rgba32) -> Bc1Block {
        let pixels = raw_block.as_array();

        let (min, max) = RgbBoundingBox::create_565(&pixels);

        let (output, _error) = try_colors(&pixels, max, min);

        output
    }

    fn encode_block_balanced(raw_block: &RawBlock4X4Rgba32) -> Bc1Block {
        let pixels = raw_block.as_array();

        let pca_vectors = PcaVectors::create(&pixels);
        let (min, max) = pca_vectors.get_min_max_color_565(&pixels);

        let mut c0 = max;
        let mut c1 = min;

        if c0.data < c1.data {
            std::mem::swap(&mut c0, &mut c1);
        }

        let (mut best, mut best_error) = try_colors(&pixels, c0, c1);

        for i in 0..MAX_TRIES {
            let (mut new_c0, mut new_c1) = variate_565(c0, c1, i);

            if new_c0.data < new_c1.data {
                std::mem::swap(&mut new_c0, &mut new_c1);
            }

            let (block, error) = try_colors(&pixels, new_c0, new_c1);

            if error < best_error {
                best = block;
                best_error = error;
                c0 = new_c0;
                c1 = new_c1;
            }

            if best_error < ERROR_THRESHOLD {
                break;
            }
        }

        best
    }

    fn encode_block_best_quality(raw_block: &RawBlock4X4Rgba32) -> Bc1Block {
        let pixels = raw_block.as_array();

        let has_alpha = raw_block.has_transparent_pixels();

        let pca_vectors = PcaVectors::create(&pixels);
        let (min, max) = pca_vectors.get_min_max_color_565(&pixels);

        let mut c0 = max;
        let mut c1 = min;

        if (!has_alpha && c0.data < c1.data) || (has_alpha && c1.data < c0.data) {
            std::mem::swap(&mut c0, &mut c1);
        }

        let (mut best, mut best_error) = try_colors(&pixels, c0, c1);

        for i in 0..MAX_TRIES {
            let (mut new_c0, mut new_c1) = variate_565(c0, c1, i);

            if (!has_alpha && new_c0.data < new_c1.data) || (has_alpha && new_c1.data < new_c0.data)
            {
                std::mem::swap(&mut new_c0, &mut new_c1);
            }

            let (block, error) = try_colors(&pixels, new_c0, new_c1);

            if error < best_error {
                best = block;
                best_error = error;
                c0 = new_c0;
                c1 = new_c1;
            }

            if best_error < ERROR_THRESHOLD {
                break;
            }
        }

        best
    }
}

fn abs_diff(color_0: u8, color_1: u8) -> f32 {
    (f32::from(color_0) - f32::from(color_1)).abs()
}

fn choose_closest_color_4(
    colors: &[Color],
    color: Color,
    r_weight: f32,
    g_weight: f32,
    b_weight: f32,
) -> (u32, f32) {
    let d = [
        abs_diff(colors[0].r, color.r) * r_weight
            + abs_diff(colors[0].g, color.g) * g_weight
            + abs_diff(colors[0].b, color.b) * b_weight,
        abs_diff(colors[1].r, color.r) * r_weight
            + abs_diff(colors[1].g, color.g) * g_weight
            + abs_diff(colors[1].b, color.b) * b_weight,
        abs_diff(colors[2].r, color.r) * r_weight
            + abs_diff(colors[2].g, color.g) * g_weight
            + abs_diff(colors[2].b, color.b) * b_weight,
        abs_diff(colors[3].r, color.r) * r_weight
            + abs_diff(colors[3].g, color.g) * g_weight
            + abs_diff(colors[3].b, color.b) * b_weight,
    ];

    let b0 = if d[0] > d[3] { 1 } else { 0 };
    let b1 = if d[1] > d[2] { 1 } else { 0 };
    let b2 = if d[0] > d[2] { 1 } else { 0 };
    let b3 = if d[1] > d[3] { 1 } else { 0 };
    let b4 = if d[2] > d[3] { 1 } else { 0 };

    let x0 = b1 & b2;
    let x1 = b0 & b3;
    let x2 = b0 & b4;

    let idx = x2 | ((x0 | x1) << 1);
    let error = d[idx];

    (idx as u32, error)
}

fn try_colors(pixels: &[Color], color_0: ColorRgb565, color_1: ColorRgb565) -> (Bc1Block, f32) {
    const R_WEIGHT: f32 = 0.3;
    const G_WEIGHT: f32 = 0.6;
    const B_WEIGHT: f32 = 0.1;

    let mut output = Bc1Block {
        color_0,
        color_1,
        ..Bc1Block::default()
    };

    let c0 = color_0.to_color_rgba_32();
    let c1 = color_1.to_color_rgba_32();

    let colors = if output.has_alpha_or_blackk() {
        [
            c0,
            c1,
            interpolate_color(c1, c0, 0.5),
            Color::from((0, 0, 0)),
        ]
    } else {
        [
            c0,
            c1,
            interpolate_color(c0, c1, 0.33),
            interpolate_color(c0, c1, 0.67),
        ]
    };

    let mut error = 0.0;
    for (i, color) in pixels.iter().enumerate().take(16) {
        let (outpuut, e) = choose_closest_color_4(&colors, *color, R_WEIGHT, G_WEIGHT, B_WEIGHT);

        output.set_color_index(i as u32, outpuut);
        error += e;
    }

    (output, error)
}
