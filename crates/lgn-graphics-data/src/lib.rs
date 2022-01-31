//! Graphics Data Definition

// crate-specific lint exceptions:

//! Graphics

// crate-specific lint exceptions:

//! Generic data codegen test (offline)

// crate-specific lint exceptions:
// #![allow()]

// generated from def\*.rs
include!(concat!(env!("OUT_DIR"), "/data_def.rs"));

mod colorspace;

/// Plugin module to register support types
pub mod plugin;
pub use plugin::*;

#[cfg(feature = "runtime")]
#[path = "runtime/texture.rs"]
pub mod runtime_texture;

#[cfg(feature = "offline")]
#[path = "offline/psd.rs"]
pub mod offline_psd;

#[cfg(feature = "offline")]
#[path = "offline/texture.rs"]
pub mod offline_texture;
pub use colorspace::*;

mod color;
pub use color::*;

#[cfg(test)]
mod tests {
    use lgn_math::{Vec3, Vec4};

    use super::*;

    #[test]
    fn hex_color() {
        assert_eq!(Color::hex("FFF").unwrap(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::hex("000").unwrap(), Color::rgb(0.0, 0.0, 0.0));
        assert!(Color::hex("---").is_err());

        assert_eq!(Color::hex("FFFF").unwrap(), Color::rgba(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::hex("0000").unwrap(), Color::rgba(0.0, 0.0, 0.0, 0.0));
        assert!(Color::hex("----").is_err());

        assert_eq!(Color::hex("FFFFFF").unwrap(), Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::hex("000000").unwrap(), Color::rgb(0.0, 0.0, 0.0));
        assert!(Color::hex("------").is_err());

        assert_eq!(
            Color::hex("FFFFFFFF").unwrap(),
            Color::rgba(1.0, 1.0, 1.0, 1.0)
        );
        assert_eq!(
            Color::hex("00000000").unwrap(),
            Color::rgba(0.0, 0.0, 0.0, 0.0)
        );
        assert!(Color::hex("--------").is_err());

        assert!(Color::hex("1234567890").is_err());
    }

    #[test]
    fn conversions_vec4() {
        let starting_vec4 = Vec4::new(0.4, 0.5, 0.6, 1.0);
        let starting_color = Color::from(starting_vec4);

        assert_eq!(starting_vec4, Vec4::from(starting_color),);

        let transformation = Vec4::new(0.5, 0.5, 0.5, 1.0);

        assert_eq!(
            starting_color * transformation,
            Color::from(starting_vec4 * transformation),
        );
    }

    #[test]
    fn mul_and_mulassign_f32() {
        let transformation = 0.5;
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(
            starting_color * transformation,
            Color::rgba(0.4 * 0.5, 0.5 * 0.5, 0.6 * 0.5, 1.0),
        );

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_f32by3() {
        let transformation = [0.4, 0.5, 0.6];
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(
            starting_color * transformation,
            Color::rgba(0.4 * 0.4, 0.5 * 0.5, 0.6 * 0.6, 1.0),
        );

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_f32by4() {
        let transformation = [0.4, 0.5, 0.6, 0.9];
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(
            starting_color * transformation,
            Color::rgba(0.4 * 0.4, 0.5 * 0.5, 0.6 * 0.6, 1.0 * 0.9),
        );

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_vec3() {
        let transformation = Vec3::new(0.2, 0.3, 0.4);
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(
            starting_color * transformation,
            Color::rgba(0.4 * 0.2, 0.5 * 0.3, 0.6 * 0.4, 1.0),
        );

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }

    #[test]
    fn mul_and_mulassign_vec4() {
        let transformation = Vec4::new(0.2, 0.3, 0.4, 0.5);
        let starting_color = Color::rgba(0.4, 0.5, 0.6, 1.0);

        assert_eq!(
            starting_color * transformation,
            Color::rgba(0.4 * 0.2, 0.5 * 0.3, 0.6 * 0.4, 1.0 * 0.5),
        );

        let mut mutated_color = starting_color;
        mutated_color *= transformation;

        assert_eq!(starting_color * transformation, mutated_color,);
    }
}
