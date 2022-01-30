#[component()]
pub struct View {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub projection_type: usize,
}

/* TODO: Add Enum support
pub enum ProjectionType {
    Orthogonal,
    Perspective,
}
*/
