use std::fmt;
use std::ops::Add;

#[derive(Copy, Clone, PartialEq)]
pub struct Vector3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T> Vector3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: Add<Output = T>> Add for Vector3<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl<T: fmt::Debug> fmt::Debug for Vector3<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Vector3
    #[test]
    fn add_vec3() {
        assert_eq!(
            Vector3::new(0., 1., 2.) + Vector3::new(2., 3., 4.),
            Vector3::new(2., 4., 6.)
        );
    }

    // Vector3
    #[test]
    fn debug_vec3() {
        assert_eq!(
            format!("{:?}", Vector3::new(1.1, 2.2, 3.3)),
            "(1.1, 2.2, 3.3)"
        );
        assert_eq!(
            format!("{:#?}", Vector3::new(1.1, 2.2, 3.3)),
            "(
    1.1,
    2.2,
    3.3,
)"
        );
    }
}
