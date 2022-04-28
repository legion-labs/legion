pub enum ScriptType {
    Rune,
    Rhai,
}

#[resource]
struct Script {
    #[legion(default=ScriptType::Rune)]
    pub script_type: ScriptType,

    #[legion(editor_type = "script")]
    pub script: String,
}

#[component]
struct ScriptComponent {
    pub input_values: Vec<String>,
    pub entry_fn: String,

    #[legion(resource_type = Script)]
    pub script_id: Option<ResourcePathId>,
}
