use std::any::Any;

use crate::App;

/// A collection of Legion App logic and configuration.
///
/// Plugins configure an [`App`](crate::App). When an [`App`](crate::App) registers
/// a plugin, the plugin's [`Plugin::build`] function is run.
pub trait Plugin: Any + Send + Sync {
    /// Configures the [`App`] to which this plugin is added.
    fn build(&self, app: &mut App);
    /// Configures a name for the [`Plugin`] which is primarily used for debugging.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// A type representing an unsafe function that returns a mutable pointer to a [`Plugin`].
/// It is used for dynamically loading plugins.
///
/// See `lgn_dynamic_plugin/src/loader.rs#dynamically_load_plugin`.
pub type CreatePlugin = unsafe fn() -> *mut dyn Plugin;
