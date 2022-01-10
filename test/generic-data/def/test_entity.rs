use lgn_data_runtime::Component;
use lgn_graphics_data::Color;
use lgn_math::prelude::*;

#[resource()]
pub struct TestEntity {
    // Default with string literal
    #[legion(default = "string literal", readonly)]
    test_string: String,

    #[legion(default = (255,0,0,255), group = "GroupTest1")]
    test_color: Color,

    // Default with Tuple()
    #[legion(default=(0.0,0.0,0.0), hidden, group = "GroupTest1")]
    pub test_position: Vec3,

    // Default with Constant value
    #[legion(default= Quat::IDENTITY, group = "GroupTest1", tooltip = "Rotation Tooltip")]
    pub test_rotation: Quat,

    // Default with bool constant
    #[legion(default = false, group = "GroupTest2")]
    test_bool: bool,

    // Default with Float constant
    #[legion(default = 32.32f32, group = "GroupTest2")]
    test_float32: f32,

    #[legion(default = 64.64f64, offline)]
    test_float64: f64,

    // Default with Integer constant
    #[legion(default = 123, group = "GroupTest2")]
    test_int: i32,

    // Default with Array
    #[legion(default=[0,1,2,3])]
    test_blob: Vec<u8>,

    // Default SubType
    test_sub_type: TestSubType1,

    test_option_set: Option<TestSubType2>,
    test_option_none: Option<TestSubType2>,

    #[legion(resource_type = TestEntity)]
    test_resource_path_option: Option<ResourcePathId>,

    #[legion(resource_type = TestEntity)]
    test_resource_path_vec: Vec<ResourcePathId>,
}

#[component]
pub struct TestComponent {
    test_i32: i32,
}

pub struct TestSubType1 {
    /// Test Dynamic Array Box Component
    test_components: Vec<Box<dyn Component>>,
    test_string: String,
    test_sub_type: TestSubType2,
}

pub struct TestSubType2 {
    pub test_vec: Vec3,
}
