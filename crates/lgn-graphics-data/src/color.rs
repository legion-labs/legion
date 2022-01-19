use lgn_math::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Mul, MulAssign};

use crate::{HslRepresentation, SrgbColorSpace};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Color {
    /// sRGBA color
    Rgba {
        /// Red component. [0.0, 1.0]
        red: f32,
        /// Green component. [0.0, 1.0]
        green: f32,
        /// Blue component. [0.0, 1.0]
        blue: f32,
        /// Alpha component. [0.0, 1.0]
        alpha: f32,
    },
    /// RGBA color in the Linear sRGB colorspace (often colloquially referred to as "linear", "RGB", or "linear RGB").
    RgbaLinear {
        /// Red component. [0.0, 1.0]
        red: f32,
        /// Green component. [0.0, 1.0]
        green: f32,
        /// Blue component. [0.0, 1.0]
        blue: f32,
        /// Alpha component. [0.0, 1.0]
        alpha: f32,
    },
    /// HSL (hue, saturation, lightness) color with an alpha channel
    Hsla {
        /// Hue component. [0.0, 360.0]
        hue: f32,
        /// Saturation component. [0.0, 1.0]
        saturation: f32,
        /// Lightness component. [0.0, 1.0]
        lightness: f32,
        /// Alpha component. [0.0, 1.0]
        alpha: f32,
    },
}

impl Color {
    pub const ALICE_BLUE: Self = Self::rgb(0.94, 0.97, 1.0);
    pub const ANTIQUE_WHITE: Self = Self::rgb(0.98, 0.92, 0.84);
    pub const AQUAMARINE: Self = Self::rgb(0.49, 1.0, 0.83);
    pub const AZURE: Self = Self::rgb(0.94, 1.0, 1.0);
    pub const BEIGE: Self = Self::rgb(0.96, 0.96, 0.86);
    pub const BISQUE: Self = Self::rgb(1.0, 0.89, 0.77);
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const CRIMSON: Self = Self::rgb(0.86, 0.08, 0.24);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const DARK_GRAY: Self = Self::rgb(0.25, 0.25, 0.25);
    pub const DARK_GREEN: Self = Self::rgb(0.0, 0.5, 0.0);
    pub const FUCHSIA: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const GOLD: Self = Self::rgb(1.0, 0.84, 0.0);
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const INDIGO: Self = Self::rgb(0.29, 0.0, 0.51);
    pub const LIME_GREEN: Self = Self::rgb(0.2, 0.8, 0.2);
    pub const MAROON: Self = Self::rgb(0.5, 0.0, 0.0);
    pub const MIDNIGHT_BLUE: Self = Self::rgb(0.1, 0.1, 0.44);
    pub const NAVY: Self = Self::rgb(0.0, 0.0, 0.5);
    pub const NONE: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const OLIVE: Self = Self::rgb(0.5, 0.5, 0.0);
    pub const ORANGE: Self = Self::rgb(1.0, 0.65, 0.0);
    pub const ORANGE_RED: Self = Self::rgb(1.0, 0.27, 0.0);
    pub const PINK: Self = Self::rgb(1.0, 0.08, 0.58);
    pub const PURPLE: Self = Self::rgb(0.5, 0.0, 0.5);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const SALMON: Self = Self::rgb(0.98, 0.5, 0.45);
    pub const SEA_GREEN: Self = Self::rgb(0.18, 0.55, 0.34);
    pub const SILVER: Self = Self::rgb(0.75, 0.75, 0.75);
    pub const TEAL: Self = Self::rgb(0.0, 0.5, 0.5);
    pub const TOMATO: Self = Self::rgb(1.0, 0.39, 0.28);
    pub const TURQUOISE: Self = Self::rgb(0.25, 0.88, 0.82);
    pub const VIOLET: Self = Self::rgb(0.93, 0.51, 0.93);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const YELLOW_GREEN: Self = Self::rgb(0.6, 0.8, 0.2);

    /// New `Color` from sRGB colorspace.
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::Rgba {
            red: r,
            green: g,
            blue: b,
            alpha: 1.0,
        }
    }

    /// New `Color` from sRGB colorspace.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::Rgba {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }

    /// New `Color` from linear RGB colorspace.
    pub const fn rgb_linear(r: f32, g: f32, b: f32) -> Self {
        Self::RgbaLinear {
            red: r,
            green: g,
            blue: b,
            alpha: 1.0,
        }
    }

    /// New `Color` from linear RGB colorspace.
    pub const fn rgba_linear(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::RgbaLinear {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        }
    }

    /// New `Color` with HSL representation in sRGB colorspace.
    pub const fn hsl(hue: f32, saturation: f32, lightness: f32) -> Self {
        Self::Hsla {
            hue,
            saturation,
            lightness,
            alpha: 1.0,
        }
    }

    /// New `Color` with HSL representation in sRGB colorspace.
    pub const fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
        Self::Hsla {
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    /// New `Color` from sRGB colorspace.
    #[allow(clippy::missing_errors_doc)]
    pub fn hex<T: AsRef<str>>(hex: T) -> Result<Self, HexColorError> {
        let hex = hex.as_ref();

        // RGB
        if hex.len() == 3 {
            let mut data = [0; 6];
            for (i, ch) in hex.chars().enumerate() {
                data[i * 2] = ch as u8;
                data[i * 2 + 1] = ch as u8;
            }
            return decode_rgb(&data);
        }

        // RGBA
        if hex.len() == 4 {
            let mut data = [0; 8];
            for (i, ch) in hex.chars().enumerate() {
                data[i * 2] = ch as u8;
                data[i * 2 + 1] = ch as u8;
            }
            return decode_rgba(&data);
        }

        // RRGGBB
        if hex.len() == 6 {
            return decode_rgb(hex.as_bytes());
        }

        // RRGGBBAA
        if hex.len() == 8 {
            return decode_rgba(hex.as_bytes());
        }

        Err(HexColorError::Length)
    }

    /// New `Color` from sRGB colorspace.
    pub fn rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba_u8(r, g, b, u8::MAX)
    }

    // Float operations in const fn are not stable yet
    // see https://github.com/rust-lang/rust/issues/57241
    /// New `Color` from sRGB colorspace.
    pub fn rgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(
            f32::from(r) / f32::from(u8::MAX),
            f32::from(g) / f32::from(u8::MAX),
            f32::from(b) / f32::from(u8::MAX),
            f32::from(a) / f32::from(u8::MAX),
        )
    }

    /// Get red in sRGB colorspace.
    pub fn r(&self) -> f32 {
        match self.as_rgba() {
            Self::Rgba { red, .. } => red,
            _ => unreachable!(),
        }
    }

    /// Get green in sRGB colorspace.
    pub fn g(&self) -> f32 {
        match self.as_rgba() {
            Self::Rgba { green, .. } => green,
            _ => unreachable!(),
        }
    }

    /// Get blue in sRGB colorspace.
    pub fn b(&self) -> f32 {
        match self.as_rgba() {
            Self::Rgba { blue, .. } => blue,
            _ => unreachable!(),
        }
    }

    /// Set red in sRGB colorspace.
    pub fn set_r(&mut self, r: f32) -> &mut Self {
        *self = self.as_rgba();
        match self {
            Self::Rgba { red, .. } => *red = r,
            _ => unreachable!(),
        }
        self
    }

    /// Set green in sRGB colorspace.
    pub fn set_g(&mut self, g: f32) -> &mut Self {
        *self = self.as_rgba();
        match self {
            Self::Rgba { green, .. } => *green = g,
            _ => unreachable!(),
        }
        self
    }

    /// Set blue in sRGB colorspace.
    pub fn set_b(&mut self, b: f32) -> &mut Self {
        *self = self.as_rgba();
        match self {
            Self::Rgba { blue, .. } => *blue = b,
            _ => unreachable!(),
        }
        self
    }

    /// Get alpha.
    pub fn a(&self) -> f32 {
        match self {
            Self::Rgba { alpha, .. }
            | Self::RgbaLinear { alpha, .. }
            | Self::Hsla { alpha, .. } => *alpha,
        }
    }

    /// Set alpha.
    pub fn set_a(&mut self, a: f32) -> &mut Self {
        match self {
            Self::Rgba { alpha, .. }
            | Self::RgbaLinear { alpha, .. }
            | Self::Hsla { alpha, .. } => {
                *alpha = a;
            }
        }
        self
    }

    /// Converts a `Color` to variant `Color::Rgba`
    pub fn as_rgba(&self) -> Self {
        match self {
            Self::Rgba { .. } => *self,
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => Self::Rgba {
                red: red.linear_to_nonlinear_srgb(),
                green: green.linear_to_nonlinear_srgb(),
                blue: blue.linear_to_nonlinear_srgb(),
                alpha: *alpha,
            },
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                let [red, green, blue] =
                    HslRepresentation::hsl_to_nonlinear_srgb(*hue, *saturation, *lightness);
                Self::Rgba {
                    red,
                    green,
                    blue,
                    alpha: *alpha,
                }
            }
        }
    }

    /// Converts a `Color` to variant `Color::RgbaLinear`
    pub fn as_rgba_linear(&self) -> Self {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Self::RgbaLinear {
                red: red.nonlinear_to_linear_srgb(),
                green: green.nonlinear_to_linear_srgb(),
                blue: blue.nonlinear_to_linear_srgb(),
                alpha: *alpha,
            },
            Self::RgbaLinear { .. } => *self,
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                let [red, green, blue] =
                    HslRepresentation::hsl_to_nonlinear_srgb(*hue, *saturation, *lightness);
                Self::RgbaLinear {
                    red: red.nonlinear_to_linear_srgb(),
                    green: green.nonlinear_to_linear_srgb(),
                    blue: blue.nonlinear_to_linear_srgb(),
                    alpha: *alpha,
                }
            }
        }
    }

    /// Converts a `Color` to variant `Color::Hsla`
    pub fn as_hsla(&self) -> Self {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => {
                let (hue, saturation, lightness) =
                    HslRepresentation::nonlinear_srgb_to_hsl([*red, *green, *blue]);
                Self::Hsla {
                    hue,
                    saturation,
                    lightness,
                    alpha: *alpha,
                }
            }
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => {
                let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([
                    red.linear_to_nonlinear_srgb(),
                    green.linear_to_nonlinear_srgb(),
                    blue.linear_to_nonlinear_srgb(),
                ]);
                Self::Hsla {
                    hue,
                    saturation,
                    lightness,
                    alpha: *alpha,
                }
            }
            Self::Hsla { .. } => *self,
        }
    }

    /// Converts a `Color` to a `[f32; 4]` from sRGB colorspace
    pub fn as_rgba_f32(self) -> [f32; 4] {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => [red, green, blue, alpha],
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => [
                red.linear_to_nonlinear_srgb(),
                green.linear_to_nonlinear_srgb(),
                blue.linear_to_nonlinear_srgb(),
                alpha,
            ],
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                let [red, green, blue] =
                    HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
                [red, green, blue, alpha]
            }
        }
    }

    /// Converts a `Color` to a `[f32; 4]` from linear RBG colorspace
    #[inline]
    pub fn as_linear_rgba_f32(self) -> [f32; 4] {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => [
                red.nonlinear_to_linear_srgb(),
                green.nonlinear_to_linear_srgb(),
                blue.nonlinear_to_linear_srgb(),
                alpha,
            ],
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => [red, green, blue, alpha],
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                let [red, green, blue] =
                    HslRepresentation::hsl_to_nonlinear_srgb(hue, saturation, lightness);
                [
                    red.nonlinear_to_linear_srgb(),
                    green.nonlinear_to_linear_srgb(),
                    blue.nonlinear_to_linear_srgb(),
                    alpha,
                ]
            }
        }
    }

    /// Converts a `Color` to a `[f32; 4]` from HLS colorspace
    pub fn as_hlsa_f32(self) -> [f32; 4] {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => {
                let (hue, saturation, lightness) =
                    HslRepresentation::nonlinear_srgb_to_hsl([red, green, blue]);
                [hue, saturation, lightness, alpha]
            }
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => {
                let (hue, saturation, lightness) = HslRepresentation::nonlinear_srgb_to_hsl([
                    red.linear_to_nonlinear_srgb(),
                    green.linear_to_nonlinear_srgb(),
                    blue.linear_to_nonlinear_srgb(),
                ]);
                [hue, saturation, lightness, alpha]
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => [hue, saturation, lightness, alpha],
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl AddAssign<Self> for Color {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => {
                let rhs = rhs.as_rgba_f32();
                *red += rhs[0];
                *green += rhs[1];
                *blue += rhs[2];
                *alpha += rhs[3];
            }
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => {
                let rhs = rhs.as_linear_rgba_f32();
                *red += rhs[0];
                *green += rhs[1];
                *blue += rhs[2];
                *alpha += rhs[3];
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                let rhs = rhs.as_linear_rgba_f32();
                *hue += rhs[0];
                *saturation += rhs[1];
                *lightness += rhs[2];
                *alpha += rhs[3];
            }
        }
    }
}

impl Add<Self> for Color {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => {
                let rhs = rhs.as_rgba_f32();
                Self::Rgba {
                    red: red + rhs[0],
                    green: green + rhs[1],
                    blue: blue + rhs[2],
                    alpha: alpha + rhs[3],
                }
            }
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => {
                let rhs = rhs.as_linear_rgba_f32();
                Self::RgbaLinear {
                    red: red + rhs[0],
                    green: green + rhs[1],
                    blue: blue + rhs[2],
                    alpha: alpha + rhs[3],
                }
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                let rhs = rhs.as_linear_rgba_f32();
                Self::Hsla {
                    hue: hue + rhs[0],
                    saturation: saturation + rhs[1],
                    lightness: lightness + rhs[2],
                    alpha: alpha + rhs[3],
                }
            }
        }
    }
}

impl AddAssign<Vec4> for Color {
    fn add_assign(&mut self, rhs: Vec4) {
        let rhs: Self = rhs.into();
        *self += rhs;
    }
}

impl Add<Vec4> for Color {
    type Output = Self;

    fn add(self, rhs: Vec4) -> Self::Output {
        let rhs: Self = rhs.into();
        self + rhs
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.as_rgba_f32()
    }
}

impl From<[f32; 4]> for Color {
    fn from([r, g, b, a]: [f32; 4]) -> Self {
        Self::rgba(r, g, b, a)
    }
}

impl From<[f32; 3]> for Color {
    fn from([r, g, b]: [f32; 3]) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<Color> for Vec4 {
    fn from(color: Color) -> Self {
        let color: [f32; 4] = color.into();
        Self::new(color[0], color[1], color[2], color[3])
    }
}

impl From<Vec4> for Color {
    fn from(vec4: Vec4) -> Self {
        Self::rgba(vec4.x, vec4.y, vec4.z, vec4.w)
    }
}

/*impl From<Color> for wgpu::Color {
    fn from(color: Color) -> Self {
        if let Color::RgbaLinear {
            red,
            green,
            blue,
            alpha,
        } = color.as_rgba_linear()
        {
            wgpu::Color {
                r: red as f64,
                g: green as f64,
                b: blue as f64,
                a: alpha as f64,
            }
        } else {
            unreachable!()
        }
    }
}*/

impl Mul<f32> for Color {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Self::Rgba {
                red: red * rhs,
                green: green * rhs,
                blue: blue * rhs,
                alpha,
            },
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => Self::RgbaLinear {
                red: red * rhs,
                green: green * rhs,
                blue: blue * rhs,
                alpha,
            },
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => Self::Hsla {
                hue: hue * rhs,
                saturation: saturation * rhs,
                lightness: lightness * rhs,
                alpha,
            },
        }
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        match self {
            Self::Rgba {
                red, green, blue, ..
            }
            | Self::RgbaLinear {
                red, green, blue, ..
            } => {
                *red *= rhs;
                *green *= rhs;
                *blue *= rhs;
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                ..
            } => {
                *hue *= rhs;
                *saturation *= rhs;
                *lightness *= rhs;
            }
        }
    }
}

impl Mul<Vec4> for Color {
    type Output = Self;

    fn mul(self, rhs: Vec4) -> Self::Output {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Self::Rgba {
                red: red * rhs.x,
                green: green * rhs.y,
                blue: blue * rhs.z,
                alpha: alpha * rhs.w,
            },
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => Self::RgbaLinear {
                red: red * rhs.x,
                green: green * rhs.y,
                blue: blue * rhs.z,
                alpha: alpha * rhs.w,
            },
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => Self::Hsla {
                hue: hue * rhs.x,
                saturation: saturation * rhs.y,
                lightness: lightness * rhs.z,
                alpha: alpha * rhs.w,
            },
        }
    }
}

impl MulAssign<Vec4> for Color {
    fn mul_assign(&mut self, rhs: Vec4) {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            }
            | Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => {
                *red *= rhs.x;
                *green *= rhs.y;
                *blue *= rhs.z;
                *alpha *= rhs.w;
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                *hue *= rhs.x;
                *saturation *= rhs.y;
                *lightness *= rhs.z;
                *alpha *= rhs.w;
            }
        }
    }
}

impl Mul<Vec3> for Color {
    type Output = Self;

    fn mul(self, rhs: Vec3) -> Self::Output {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Self::Rgba {
                red: red * rhs.x,
                green: green * rhs.y,
                blue: blue * rhs.z,
                alpha,
            },
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => Self::RgbaLinear {
                red: red * rhs.x,
                green: green * rhs.y,
                blue: blue * rhs.z,
                alpha,
            },
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => Self::Hsla {
                hue: hue * rhs.x,
                saturation: saturation * rhs.y,
                lightness: lightness * rhs.z,
                alpha,
            },
        }
    }
}

impl MulAssign<Vec3> for Color {
    fn mul_assign(&mut self, rhs: Vec3) {
        match self {
            Self::Rgba {
                red, green, blue, ..
            }
            | Self::RgbaLinear {
                red, green, blue, ..
            } => {
                *red *= rhs.x;
                *green *= rhs.y;
                *blue *= rhs.z;
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                ..
            } => {
                *hue *= rhs.x;
                *saturation *= rhs.y;
                *lightness *= rhs.z;
            }
        }
    }
}

impl Mul<[f32; 4]> for Color {
    type Output = Self;

    fn mul(self, rhs: [f32; 4]) -> Self::Output {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Self::Rgba {
                red: red * rhs[0],
                green: green * rhs[1],
                blue: blue * rhs[2],
                alpha: alpha * rhs[3],
            },
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => Self::RgbaLinear {
                red: red * rhs[0],
                green: green * rhs[1],
                blue: blue * rhs[2],
                alpha: alpha * rhs[3],
            },
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => Self::Hsla {
                hue: hue * rhs[0],
                saturation: saturation * rhs[1],
                lightness: lightness * rhs[2],
                alpha: alpha * rhs[3],
            },
        }
    }
}

impl MulAssign<[f32; 4]> for Color {
    fn mul_assign(&mut self, rhs: [f32; 4]) {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            }
            | Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => {
                *red *= rhs[0];
                *green *= rhs[1];
                *blue *= rhs[2];
                *alpha *= rhs[3];
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => {
                *hue *= rhs[0];
                *saturation *= rhs[1];
                *lightness *= rhs[2];
                *alpha *= rhs[3];
            }
        }
    }
}

impl Mul<[f32; 3]> for Color {
    type Output = Self;

    fn mul(self, rhs: [f32; 3]) -> Self::Output {
        match self {
            Self::Rgba {
                red,
                green,
                blue,
                alpha,
            } => Self::Rgba {
                red: red * rhs[0],
                green: green * rhs[1],
                blue: blue * rhs[2],
                alpha,
            },
            Self::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => Self::RgbaLinear {
                red: red * rhs[0],
                green: green * rhs[1],
                blue: blue * rhs[2],
                alpha,
            },
            Self::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => Self::Hsla {
                hue: hue * rhs[0],
                saturation: saturation * rhs[1],
                lightness: lightness * rhs[2],
                alpha,
            },
        }
    }
}

impl MulAssign<[f32; 3]> for Color {
    fn mul_assign(&mut self, rhs: [f32; 3]) {
        match self {
            Self::Rgba {
                red, green, blue, ..
            }
            | Self::RgbaLinear {
                red, green, blue, ..
            } => {
                *red *= rhs[0];
                *green *= rhs[1];
                *blue *= rhs[2];
            }
            Self::Hsla {
                hue,
                saturation,
                lightness,
                ..
            } => {
                *hue *= rhs[0];
                *saturation *= rhs[1];
                *lightness *= rhs[2];
            }
        }
    }
}

#[derive(Debug)]
pub enum HexColorError {
    Length,
    Hex(hex::FromHexError),
}

fn decode_rgb(data: &[u8]) -> Result<Color, HexColorError> {
    let mut buf = [0; 3];
    match hex::decode_to_slice(data, &mut buf) {
        Ok(_) => {
            let r = f32::from(buf[0]) / 255.0;
            let g = f32::from(buf[1]) / 255.0;
            let b = f32::from(buf[2]) / 255.0;
            Ok(Color::rgb(r, g, b))
        }
        Err(err) => Err(HexColorError::Hex(err)),
    }
}

fn decode_rgba(data: &[u8]) -> Result<Color, HexColorError> {
    let mut buf = [0; 4];
    match hex::decode_to_slice(data, &mut buf) {
        Ok(_) => {
            let r = f32::from(buf[0]) / 255.0;
            let g = f32::from(buf[1]) / 255.0;
            let b = f32::from(buf[2]) / 255.0;
            let a = f32::from(buf[3]) / 255.0;
            Ok(Color::rgba(r, g, b, a))
        }
        Err(err) => Err(HexColorError::Hex(err)),
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8)) -> Self {
        Self::rgb_u8(val.0, val.1, val.2)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    fn from(val: (u8, u8, u8, u8)) -> Self {
        Self::rgba_u8(val.0, val.1, val.2, val.3)
    }
}

lgn_data_model::implement_primitive_type_def!(Color);
