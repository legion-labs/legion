use lgn_math::{Mat4, Vec3, Vec4};

use crate::Color;

use super::ColorRgb565;

pub(crate) struct PcaVectors {
    mean: Vec3,
    principal_axis: Vec3,
}

impl PcaVectors {
    pub fn create(colors: &[Color]) -> Self {
        let mut vectors = Vec::with_capacity(colors.len());
        convert_to_vec4(colors, &mut vectors);

        let (cov, mean) = calculate_covariance(&mut vectors);

        let mean = Vec3::new(mean.x, mean.y, mean.z);

        let pa = calculate_principal_axis(cov);
        let mut principal_axis = Vec3::new(pa.x, pa.y, pa.z);

        if principal_axis.dot(principal_axis) == 0.0 {
            principal_axis = Vec3::Y;
        } else {
            principal_axis = principal_axis.normalize();
        }

        Self {
            mean,
            principal_axis,
        }
    }

    pub fn get_min_max_color_565(&self, colors: &[Color]) -> (ColorRgb565, ColorRgb565) {
        const C565_5_MASK: i32 = 0xF8;
        const C565_6_MASK: i32 = 0xFC;

        let mut min_d = 0.0f32;
        let mut max_d = 0.0f32;

        for color in colors {
            let color_vec = Vec3::new(
                f32::from(color.r) / 255.0,
                f32::from(color.g) / 255.0,
                f32::from(color.b) / 255.0,
            );

            let v = color_vec - self.mean;
            let d = v.dot(self.principal_axis);

            if d < min_d {
                min_d = d;
            };
            if d > max_d {
                max_d = d;
            };
        }

        //Inset
        min_d *= 15.0 / 16.0;
        max_d *= 15.0 / 16.0;

        let min_vec = self.mean + self.principal_axis * min_d;
        let max_vec = self.mean + self.principal_axis * max_d;

        let mut min_r = (min_vec.x * 255.0) as i32;
        let mut min_g = (min_vec.y * 255.0) as i32;
        let mut min_b = (min_vec.z * 255.0) as i32;

        let mut max_r = (max_vec.x * 255.0) as i32;
        let mut max_g = (max_vec.y * 255.0) as i32;
        let mut max_b = (max_vec.z * 255.0) as i32;

        min_r = if min_r >= 0 { min_r } else { 0 };
        min_g = if min_g >= 0 { min_g } else { 0 };
        min_b = if min_b >= 0 { min_b } else { 0 };

        max_r = if max_r <= 255 { max_r } else { 255 };
        max_g = if max_g <= 255 { max_g } else { 255 };
        max_b = if max_b <= 255 { max_b } else { 255 };

        // Optimal round
        min_r = (min_r & C565_5_MASK) | (min_r >> 5);
        min_g = (min_g & C565_6_MASK) | (min_g >> 6);
        min_b = (min_b & C565_5_MASK) | (min_b >> 5);

        max_r = (max_r & C565_5_MASK) | (max_r >> 5);
        max_g = (max_g & C565_6_MASK) | (max_g >> 6);
        max_b = (max_b & C565_5_MASK) | (max_b >> 5);

        let min = ColorRgb565::new(min_r as u8, min_g as u8, min_b as u8);
        let max = ColorRgb565::new(max_r as u8, max_g as u8, max_b as u8);

        (min, max)
    }
}

fn convert_to_vec4(colors: &[Color], vectors: &mut [Vec4]) {
    for i in 0..colors.len() {
        vectors[i].x += f32::from(colors[i].r);
        vectors[i].y += f32::from(colors[i].g);
        vectors[i].z += f32::from(colors[i].b);
        vectors[i].w = 0.0;
    }
}

#[allow(clippy::cast_precision_loss)]
fn calculate_covariance(values: &mut [Vec4]) -> (Mat4, Vec4) {
    let mean = calculate_mean(values);
    (0..values.len()).for_each(|i| {
        values[i] -= mean;
    });

    // 4x4 matrix
    let mut mat = Mat4::ZERO;

    (0..values.len()).for_each(|i| {
        mat.x_axis.x += values[i].x * values[i].x;
        mat.x_axis.y += values[i].x * values[i].y;
        mat.x_axis.z += values[i].x * values[i].z;
        mat.x_axis.w += values[i].x * values[i].w;

        mat.y_axis.y += values[i].y * values[i].y;
        mat.y_axis.z += values[i].y * values[i].z;
        mat.y_axis.w += values[i].y * values[i].w;

        mat.z_axis.z += values[i].z * values[i].z;
        mat.z_axis.w += values[i].z * values[i].w;

        mat.w_axis.w += values[i].w * values[i].w;
    });

    mat = mat.mul_scalar(1.0 / (values.len() - 1) as f32);

    mat.y_axis.x = mat.x_axis.y;
    mat.z_axis.x = mat.x_axis.z;
    mat.z_axis.y = mat.y_axis.z;
    mat.w_axis.x = mat.x_axis.w;
    mat.w_axis.y = mat.y_axis.w;
    mat.w_axis.z = mat.z_axis.w;

    (mat, mean)
}

#[allow(clippy::cast_precision_loss)]
fn calculate_mean(colors: &[Vec4]) -> Vec4 {
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    let mut a = 0.0f32;

    (0..colors.len()).for_each(|i| {
        r += colors[i].x;
        g += colors[i].y;
        b += colors[i].z;
        a += colors[i].w;
    });

    Vec4::new(
        r / colors.len() as f32,
        g / colors.len() as f32,
        b / colors.len() as f32,
        a / colors.len() as f32,
    )
}

/// <summary>
/// Calculate principal axis with the power-method
/// </summary>
/// <param name="covarianceMatrix"></param>
/// <returns></returns>
fn calculate_principal_axis(covariance_matrix: Mat4) -> Vec4 {
    let mut last_da = Vec4::Y;

    for _i in 0..30 {
        let mut da = covariance_matrix.mul_vec4(last_da);

        if da.dot(da) == 0.0 {
            break;
        }

        da = da.normalize();
        if last_da.dot(da) > 0.999999 {
            last_da = da;
            break;
        }

        last_da = da;
    }
    last_da
}
