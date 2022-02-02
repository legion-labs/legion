#[resource]
struct Script {
    #[legion(offline_only, editor_type = "script")]
    pub script: String,

    #[legion(runtime_only)]
    pub compiled_script: Vec<u8>,
}

/*pub enum ScriptType {
    Mun, // 1
    Rune, // 2
    Rhai // 3
}*/

#[component]
struct ScriptComponent {
    pub script_type: usize, //ScriptType,
    pub input_values: Vec<String>,
    pub entry_fn: String,

    #[legion(resource_type = Script)]
    pub script_id: Option<ResourcePathId>,

    pub temp_script: String,
}
