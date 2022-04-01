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

#[cfg(test)]
mod test {
    #[test]
    fn angles() {
        use crate::Angle;

        let angle_45_rad = Angle::from_radians(std::f32::consts::FRAC_PI_4);
        let angle_45_deg = Angle::from_degrees(45.0);

        assert!((angle_45_rad.radians() - angle_45_deg.radians()).abs() < std::f32::EPSILON);
        assert!((angle_45_rad.degrees() - angle_45_deg.degrees()).abs() < std::f32::EPSILON);
    }
}
