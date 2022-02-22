use lgn_config::Config;

pub struct PhysicsSettings {
    pub(crate) enable_visual_debugger: bool,
    pub(crate) length_tolerance: f32,
    pub(crate) speed_tolerance: f32,
}

impl PhysicsSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            enable_visual_debugger: config.get_or("physics.enable_visual_debugger", false),
            length_tolerance: config.get_or("physics.length_tolerance", 1.0_f32),
            speed_tolerance: config.get_or("physics.speed_tolerance", 1.0_f32),
        }
    }
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            enable_visual_debugger: false,
            length_tolerance: 1.0_f32,
            speed_tolerance: 1.0_f32,
        }
    }
}
