use crate::runtime::MeshScale;

// Clone is not derived for struct used in components
impl Clone for MeshScale {
    fn clone(&self) -> Self {
        Self {
            scale: self.scale,
            rotation: self.rotation,
        }
    }
}
