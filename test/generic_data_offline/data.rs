use legion_math::prelude::*;

#[data_container()]
pub struct TestEntity {
    // Default with string literal
    #[legion(default = "string literal", readonly, category = "Name")]
    test_string: String,

    // Default with Tuple()
    #[legion(default=(0.0,0.0,0.0), hidden)]
    pub test_position: Vec3,

    // Default with Constant value
    #[legion(default= Quat::IDENTITY, tooltip = "Rotation Tooltip")]
    pub test_rotation: Quat,

    // Default with bool constant
    #[legion(default = false)]
    test_bool: bool,

    // Default with Float constant
    #[legion(default = 32.32f32)]
    test_float32: f32,

    #[legion(default = 64.64f64, offline)]
    test_float64: f64,

    // Default with Integer constant
    #[legion(default = 123)]
    test_int: i32,

    // Default with Array
    #[legion(default=[0,1,2,3])]
    test_blob: Vec<u8>,
}
