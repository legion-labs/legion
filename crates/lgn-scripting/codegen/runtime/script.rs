use lgn_data_offline::ResourcePathId;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct Script {
    pub script: String,
}
impl Script {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 1258477681462899259u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__SCRIPT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for Script {
    fn default() -> Self {
        Self {
            script: String::default(),
        }
    }
}
impl lgn_data_model::TypeReflection for Script {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    #[allow(clippy::too_many_lines)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            Script,
            vec![lgn_data_model::FieldDescriptor {
                field_name: "script".into(),
                offset: memoffset::offset_of!(Script, script),
                field_type: <String as lgn_data_model::TypeReflection>::get_type_def(),
                attributes: {
                    let mut attr = std::collections::HashMap::new();
                    attr
                }
            },]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(Script);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(Script);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __SCRIPT_DEFAULT : Script = Script :: default () ; }
impl lgn_data_runtime::Resource for Script {
    const TYPENAME: &'static str = "runtime_script";
}
impl lgn_data_runtime::Asset for Script {
    type Loader = ScriptLoader;
}
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct ScriptReferenceType(lgn_data_runtime::Reference<Script>);
lgn_data_model::implement_primitive_type_def!(ScriptReferenceType);
#[derive(Default)]
pub struct ScriptLoader {}
impl lgn_data_runtime::AssetLoader for ScriptLoader {
    fn load(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn std::any::Any + Send + Sync>> {
        let output: Script = bincode::deserialize_from(reader).map_err(|_err| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Failed to parse")
        })?;
        Ok(Box::new(output))
    }
    fn load_init(&mut self, _asset: &mut (dyn std::any::Any + Send + Sync)) {}
}
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct ScriptComponent {
    pub input_values: Vec<String>,
    pub entry_fn: String,
    pub temp_script: String,
}
impl ScriptComponent {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 406697386270205449u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__SCRIPTCOMPONENT_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for ScriptComponent {
    fn default() -> Self {
        Self {
            input_values: Vec::new(),
            entry_fn: String::default(),
            temp_script: String::default(),
        }
    }
}
impl lgn_data_model::TypeReflection for ScriptComponent {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    #[allow(clippy::too_many_lines)]
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            ScriptComponent,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "input_values".into(),
                    offset: memoffset::offset_of!(ScriptComponent, input_values),
                    field_type: <Vec<String> as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "entry_fn".into(),
                    offset: memoffset::offset_of!(ScriptComponent, entry_fn),
                    field_type: <String as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "temp_script".into(),
                    offset: memoffset::offset_of!(ScriptComponent, temp_script),
                    field_type: <String as lgn_data_model::TypeReflection>::get_type_def(),
                    attributes: {
                        let mut attr = std::collections::HashMap::new();
                        attr
                    }
                },
            ]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(ScriptComponent);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(ScriptComponent);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __SCRIPTCOMPONENT_DEFAULT : ScriptComponent = ScriptComponent :: default () ; }
#[typetag::serde(name = "Runtime_ScriptComponent")]
impl lgn_data_runtime::Component for ScriptComponent {}
