use legion_app::{PluginGroup, PluginGroupBuilder};

use legion_app::ScheduleRunnerPlugin;
use legion_core::CorePlugin;
use legion_transform::TransformPlugin;

/// This plugin group will add all the default plugins:
/// * [`CorePlugin`]
/// * [`TransformPlugin`]
///
/// See also [`MinimalPlugins`] for a slimmed down option
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CorePlugin::default());
        group.add(TransformPlugin::default());
    }
}

/// Minimal plugin group that will add the following plugins:
/// * [`CorePlugin`]
/// * [`ScheduleRunnerPlugin`]
///
/// See also [`DefaultPlugins`] for a more complete set of plugins
pub struct MinimalPlugins;

impl PluginGroup for MinimalPlugins {
    fn build(&mut self, group: &mut PluginGroupBuilder) {
        group.add(CorePlugin::default());
        group.add(ScheduleRunnerPlugin::default());
    }
}
