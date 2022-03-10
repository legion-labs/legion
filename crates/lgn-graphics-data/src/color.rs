//! A module providing Color type definition

use std::ops::{Shl, Shr};

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

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
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
