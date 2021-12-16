#![allow(dead_code)]
#![allow(clippy::needless_update)]

use lgn_data_runtime::Component;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct EntityDc {
    pub name: String,
    pub components: Vec<Box<dyn Component>>,
}
impl EntityDc {
    const SIGNATURE_HASH: u64 = 2369000756644127862u64;
    pub fn get_default_instance() -> &'static Self {
        &__ENTITYDC_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for EntityDc {
    fn default() -> Self {
        Self {
            name: "unnamed".into(),
            components: Vec::new(),
        }
    }
}
impl lgn_data_reflection::TypeReflection for EntityDc {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection :: implement_struct_descriptor ! (EntityDc , vec ! [lgn_data_reflection :: FieldDescriptor { field_name : "name" . into () , offset : memoffset :: offset_of ! (EntityDc , name) , field_type : < String as lgn_data_reflection :: TypeReflection > :: get_type_def () , group : "" . into () } , lgn_data_reflection :: FieldDescriptor { field_name : "components" . into () , offset : memoffset :: offset_of ! (EntityDc , components) , field_type : < Vec < Box < dyn Component > > as lgn_data_reflection :: TypeReflection > :: get_type_def () , group : "" . into () } ,]);
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(EntityDc);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(EntityDc);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __ENTITYDC_DEFAULT : EntityDc = EntityDc { .. EntityDc :: default () } ; }
use lgn_data_runtime::{Asset, AssetLoader, Resource};
use std::{any::Any, io};
impl Resource for EntityDc {
    const TYPENAME: &'static str = "runtime_entitydc";
}
impl Asset for EntityDc {
    type Loader = EntityDcLoader;
}
#[derive(Default)]
pub struct EntityDcLoader {}
impl AssetLoader for EntityDcLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let output: EntityDc = bincode::deserialize_from(reader)
            .map_err(|_err| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse"))?;
        Ok(Box::new(output))
    }
    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
