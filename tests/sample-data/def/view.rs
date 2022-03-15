pub enum ProjectionType {
    Orthogonal,
    Perspective,
}
#[component()]
pub struct View {
    pub fov: f32,
    pub near: f32,
    pub far: f32,
    pub projection_type: ProjectionType,
}
