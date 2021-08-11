use cgmath::Vector3 as cg_Vector3;

pub type Vector3 = cg_Vector3<f32>;

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
}
