use crate::Color;

use super::ColorRgb565;

/// <summary>
/// Calculate the bounding box of rgb values as described in
/// "Real-Time DXT Compression by J.M.P. van Waveren, 2006, Id Software, Inc." and
/// "Real-Time YCoCg-DXT Compression J.M.P. van Waveren,  Ignacio Castaño id Software, Inc. NVIDIA Corp."
/// </summary>
pub(crate) struct RgbBoundingBox {}

impl RgbBoundingBox {
    pub fn create_565(colors: &[Color]) -> (ColorRgb565, ColorRgb565) {
        const COLOR_INSET_SHIFT: i32 = 4;
        const C5655_MASK: i32 = 0xF8;
        const C5656_MASK: i32 = 0xFC;

        let mut min_r: i32 = 255;
        let mut min_g: i32 = 255;
        let mut min_b: i32 = 255;

        let mut max_r: i32 = 0;
        let mut max_g: i32 = 0;
        let mut max_b: i32 = 0;

        for c in colors {
            if c.r < min_r as u8 {
                min_r = i32::from(c.r);
            }
            if c.g < min_g as u8 {
                min_g = i32::from(c.g);
            }
            if c.b < min_b as u8 {
                min_b = i32::from(c.b);
            }

            if c.r > max_r as u8 {
                max_r = i32::from(c.r);
            }
            if c.g > max_g as u8 {
                max_g = i32::from(c.g);
            }
            if c.b > max_b as u8 {
                max_b = i32::from(c.b);
            }
        }

        let inset_r = (max_r - min_r) >> COLOR_INSET_SHIFT;
        let inset_g = (max_g - min_g) >> COLOR_INSET_SHIFT;
        let inset_b = (max_b - min_b) >> COLOR_INSET_SHIFT;

        // Inset by 1/16th
        min_r = ((min_r << COLOR_INSET_SHIFT) + inset_r) >> COLOR_INSET_SHIFT;
        min_g = ((min_g << COLOR_INSET_SHIFT) + inset_g) >> COLOR_INSET_SHIFT;
        min_b = ((min_b << COLOR_INSET_SHIFT) + inset_b) >> COLOR_INSET_SHIFT;

        max_r = ((max_r << COLOR_INSET_SHIFT) - inset_r) >> COLOR_INSET_SHIFT;
        max_g = ((max_g << COLOR_INSET_SHIFT) - inset_g) >> COLOR_INSET_SHIFT;
        max_b = ((max_b << COLOR_INSET_SHIFT) - inset_b) >> COLOR_INSET_SHIFT;

        min_r = if min_r >= 0 { min_r } else { 0 };
        min_g = if min_g >= 0 { min_g } else { 0 };
        min_b = if min_b >= 0 { min_b } else { 0 };

        max_r = if max_r <= 255 { max_r } else { 255 };
        max_g = if max_g <= 255 { max_g } else { 255 };
        max_b = if max_b <= 255 { max_b } else { 255 };

        // Optimal rounding
        min_r = (min_r & C5655_MASK) | (min_r >> 5);
        min_g = (min_g & C5656_MASK) | (min_g >> 6);
        min_b = (min_b & C5655_MASK) | (min_b >> 5);

        max_r = (max_r & C5655_MASK) | (max_r >> 5);
        max_g = (max_g & C5656_MASK) | (max_g >> 6);
        max_b = (max_b & C5655_MASK) | (max_b >> 5);

        (
            ColorRgb565::new(min_r as u8, min_g as u8, min_b as u8),
            ColorRgb565::new(max_r as u8, max_g as u8, max_b as u8),
        )
    }
}
