use std::ops::{Add, Sub};

#[derive(Clone, Copy)]
pub struct Angle(f32);

impl Angle {
    pub fn from_radians(radians: f32) -> Self {
        Self(radians)
    }

    pub fn from_degrees(degrees: f32) -> Self {
        Self(degrees / 180.0 * std::f32::consts::PI)
    }

    pub fn radians(self) -> f32 {
        self.0
    }

    pub fn degrees(self) -> f32 {
        self.0 / std::f32::consts::PI * 180.0
    }
}

impl Add for Angle {
    type Output = Self;
    fn add(self, other: Angle) -> Self {
        Angle::from_radians(self.0 + other.0)
    }
}

impl Sub for Angle {
    type Output = Self;
    fn sub(self, other: Angle) -> Self {
        Angle::from_radians(self.0 - other.0)
    }
}

impl std::fmt::Debug for Angle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Angle").field("radians", &self.0).finish()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn angles() {
        use crate::Angle;

        let angle_45_rad = Angle::from_radians(std::f32::consts::FRAC_PI_4);
        let angle_45_deg = Angle::from_degrees(45.0);

        assert!((angle_45_rad.radians() - angle_45_deg.radians()).abs() < std::f32::EPSILON);
        assert!((angle_45_rad.degrees() - angle_45_deg.degrees()).abs() < std::f32::EPSILON);

        let angle = Angle::from_degrees(25.0) + Angle::from_degrees(65.0);
        assert!((angle.radians() - std::f32::consts::FRAC_PI_2).abs() < std::f32::EPSILON);
        assert!((angle - Angle::from_degrees(90.0)).radians().abs() < std::f32::EPSILON);
    }
}
