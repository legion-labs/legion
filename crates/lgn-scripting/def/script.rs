pub enum ScriptType {
    Mun,
    Rune,
    Rhai,
}

#[resource]
struct Script {
    #[legion(default=ScriptType::Rune)]
    pub script_type: ScriptType,

    #[legion(offline_only, editor_type = "script")]
    pub script: String,

    #[legion(runtime_only)]
    pub compiled_script: Vec<u8>,
}

#[component]
struct ScriptComponent {
    pub input_values: Vec<String>,
    pub entry_fn: String,

    #[legion(resource_type = Script)]
    pub script_id: Option<ResourcePathId>,

    pub temp_script: String,
}
