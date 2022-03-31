//! A module providing Color type definition

use std::ops::{Shl, Shr};

use lgn_math::Vec4;
use serde::{Deserialize, Deserializer, Serialize};

/// Structure definining a RGBA colors
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct Color {
    /// Red color
    pub r: u8,
    /// Green color
    pub g: u8,
    /// Blue color
    pub b: u8,
    /// Alpha color
    pub a: u8,
}

#[allow(dead_code)]
impl Color {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const RED: Self = Self::new(255, 0, 0, 255);
    pub const GREEN: Self = Self::new(0, 255, 0, 255);
    pub const BLUE: Self = Self::new(0, 0, 255, 255);

    pub const YELLOW: Self = Self::new(255, 255, 0, 255);
    pub const MAGENTA: Self = Self::new(255, 0, 255, 255);
    pub const CYAN: Self = Self::new(0, 255, 255, 255);

    pub const WHITE: Self = Self::new(255, 255, 255, 255);
    pub const BLACK: Self = Self::new(0, 0, 0, 255);
    pub const ORANGE: Self = Self::new(255, 165, 0, 255);

    pub fn as_linear(self) -> Vec4 {
        Vec4::new(
            linear_from_srgb(f32::from(self.r)),
            linear_from_srgb(f32::from(self.g)),
            linear_from_srgb(f32::from(self.b)),
            f32::from(self.a) / 255.0,
        )
    }
}

fn linear_from_srgb(srgb: f32) -> f32 {
    if srgb < 10.31475 {
        srgb / 3294.6
    } else {
        ((srgb + 14.025) / 269.025).powf(2.4)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        }
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8)) -> Self {
        Self {
            r: val.0,
            g: val.1,
            b: val.2,
            a: 255,
        }
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8, u8)) -> Self {
        Self {
            r: val.0,
            g: val.1,
            b: val.2,
            a: val.3,
        }
    }
}

impl From<Color> for [u8; 4] {
    fn from(val: Color) -> Self {
        [val.r, val.g, val.b, val.a]
    }
}

impl From<[u8; 4]> for Color {
    fn from(val: [u8; 4]) -> Self {
        Self {
            r: val[0],
            g: val[1],
            b: val[2],
            a: val[3],
        }
    }
}

impl From<Color> for u32 {
    fn from(val: Color) -> Self {
        Self::from(val.r)
            | Self::from(val.g) << 8
            | Self::from(val.b) << 16
            | Self::from(val.a) << 24
    }
}

impl From<u32> for Color {
    fn from(val: u32) -> Self {
        Self {
            r: (val & 0xFF) as u8,
            g: (val >> 8 & 0xFF) as u8,
            b: (val >> 16 & 0xFF) as u8,
            a: (val >> 24 & 0xFF) as u8,
        }
    }
}

#[allow(clippy::cast_lossless)]
impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let color = (self.r as u32).shl(24)
            | (self.g as u32).shl(16)
            | (self.b as u32).shl(8)
            | (self.a as u32);
        serializer.serialize_u32(color)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let color = u32::deserialize(deserializer)?;
        Ok(Self {
            r: color.shr(24) as u8,
            g: color.shr(16) as u8,
            b: color.shr(8) as u8,
            a: color as u8,
        })
    }
}

lgn_data_model::implement_primitive_type_def!(Color);
