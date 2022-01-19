use std::path::PathBuf;

use lgn_ecs::prelude::*;

#[derive(Component, Clone)]
pub struct ECSScriptComponent {
    pub input_values: Vec<String>,
    pub entry_fn: String,
    pub lib_path: PathBuf, // FIXME: Replace with the payload/compiled assembly/IR once we switch to a different scripting language.
}
