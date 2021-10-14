use legion_app::Plugin;

#[derive(Default)]
pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, _app: &mut legion_app::App) {}
}
