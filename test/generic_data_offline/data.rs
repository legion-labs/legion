use legion_graphics_data::Color;
use legion_math::prelude::*;

#[data_container()]
pub struct TestEntity {
    // Default with string literal
    #[legion(default = "string literal", readonly, category = "Name")]
    test_string: String,

    #[legion(default = (255,0,0,255))]
    test_color: Color,

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

#[data_container()]
struct DebugCube {
    #[legion(default=(0.0,0.0,0.0))]
    pub position: Vec3,

    #[legion(default= Quat::IDENTITY)]
    pub rotation: Quat,

    #[legion(default=(1.0,1.0,1.0))]
    pub scale: Vec3,
}

#[data_container()]
struct InstanceDc {
    #[legion(resource_type = EntityDc)]
    pub original: Option<ResourcePathId>,
}

#[data_container()]
struct EntityDc {
    #[legion(default = "unnamed")]
    pub name: String,

    #[legion(resource_type = EntityDc)]
    pub children: Vec<ResourcePathId>,

    #[legion(resource_type = EntityDc)]
    pub parent: Option<ResourcePathId>,
    //pub components: Vec<Box<dyn Component>>,
}
