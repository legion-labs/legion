use lgn_app::Plugin;

use crate::core::render_object::RenderObjectSet;

pub type MeshRenderObjectSet = RenderObjectSet<MeshRenderObject>;

pub struct MeshRenderObject {
    pub tmp: u32,
}

#[derive(Default)]
pub struct MeshFeaturePlugin;

impl Plugin for MeshFeaturePlugin {
    fn build(&self, app: &mut lgn_app::App) {
        app.insert_resource(MeshRenderObjectSet::new(100 * 1024));
    }
}
