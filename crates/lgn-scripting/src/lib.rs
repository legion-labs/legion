//! Scripting library - currently has an integration for the MUN language, Rune and Rhai.

mod labels;
pub use labels::ScriptingStage;

mod plugin;
pub use plugin::ScriptingPlugin;
