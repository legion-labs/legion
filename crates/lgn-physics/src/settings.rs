use lgn_math::prelude::Vec3;

pub struct PhysicsSettings {
    pub(crate) enable_visual_debugger: bool,
    pub(crate) length_tolerance: f32,
    pub(crate) speed_tolerance: f32,
    pub(crate) gravity: Vec3,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            enable_visual_debugger: false,
            length_tolerance: 1.0_f32,
            speed_tolerance: 1.0_f32,
            gravity: Vec3::new(0.0, -9.81, 0.0),
        }
    }
}

#[derive(Default)]
pub struct PhysicsSettingsBuilder(PhysicsSettings);

impl PhysicsSettingsBuilder {
    pub fn enable_visual_debugger(mut self, enable_visual_debugger: bool) -> Self {
        self.0.enable_visual_debugger = enable_visual_debugger;
        self
    }

    pub fn length_tolerance(mut self, length_tolerance: f32) -> Self {
        self.0.length_tolerance = length_tolerance;
        self
    }

    pub fn speed_tolerance(mut self, speed_tolerance: f32) -> Self {
        self.0.speed_tolerance = speed_tolerance;
        self
    }

    pub fn gravity(mut self, gravity: Vec3) -> Self {
        self.0.gravity = gravity;
        self
    }

    pub fn build(self) -> PhysicsSettings {
        self.0
    }
}
