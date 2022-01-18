use lgn_data_offline::ResourcePathId;

#[resource]
struct Script {
    pub script: String,
}

#[component]
struct ScriptComponent {
    pub input_values: Vec<String>,
    pub entry_fn: String,
    //pub script_id: Option<ResourcePathId>,
    pub temp_script: String,
}
