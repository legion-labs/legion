use std::path::PathBuf;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::*;

#[derive(Clone, PartialEq)]
pub enum ECSScriptType {
    Mun,
    Rune,
    Rhai,
}

#[derive(Clone)]
pub enum ECSScriptPayload {
    LibPath(PathBuf), // FIXME: Replace with the payload/compiled assembly/IR once we switch to a different scripting language.
    ContainedScript(String),
    ScriptRef(ResourceTypeAndId),
}

#[derive(Component, Clone)]
pub struct ECSScriptComponent {
    pub script_type: ECSScriptType,
    pub input_values: Vec<String>,
    pub entry_fn: String,
    pub payload: ECSScriptPayload,
}
