pub struct PhysicsSettings {
    pub(crate) enable_visual_debugger: bool,
    pub(crate) length_tolerance: f32,
    pub(crate) speed_tolerance: f32,
}

impl PhysicsSettings {
    pub fn new(enable_visual_debugger: bool, length_tolerance: f32, speed_tolerance: f32) -> Self {
        Self {
            enable_visual_debugger,
            length_tolerance,
            speed_tolerance,
        }
    }
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self::new(false, 1.0_f32, 1.0_f32)
    }
}
