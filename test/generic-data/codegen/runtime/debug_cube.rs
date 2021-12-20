use lgn_graphics_data::Color;
use lgn_math::prelude::*;
#[derive(serde :: Serialize, serde :: Deserialize)]
pub struct DebugCube {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub mesh_id: usize,
    pub color: Color,
    pub rotation_speed: Vec3,
}
impl DebugCube {
    #[allow(dead_code)]
    const SIGNATURE_HASH: u64 = 13358266263809831249u64;
    #[allow(dead_code)]
    pub fn get_default_instance() -> &'static Self {
        &__DEBUGCUBE_DEFAULT
    }
}
#[allow(clippy::derivable_impls)]
impl Default for DebugCube {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0, 0.0).into(),
            rotation: Quat::IDENTITY,
            scale: (1.0, 1.0, 1.0).into(),
            mesh_id: 1,
            color: (255, 0, 0).into(),
            rotation_speed: (0.0, 0.0, 0.0).into(),
        }
    }
}
impl lgn_data_model::TypeReflection for DebugCube {
    fn get_type(&self) -> lgn_data_model::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_struct_descriptor!(
            DebugCube,
            vec![
                lgn_data_model::FieldDescriptor {
                    field_name: "position".into(),
                    offset: memoffset::offset_of!(DebugCube, position),
                    field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "rotation".into(),
                    offset: memoffset::offset_of!(DebugCube, rotation),
                    field_type: <Quat as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "scale".into(),
                    offset: memoffset::offset_of!(DebugCube, scale),
                    field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "mesh_id".into(),
                    offset: memoffset::offset_of!(DebugCube, mesh_id),
                    field_type: <usize as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "color".into(),
                    offset: memoffset::offset_of!(DebugCube, color),
                    field_type: <Color as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_model::FieldDescriptor {
                    field_name: "rotation_speed".into(),
                    offset: memoffset::offset_of!(DebugCube, rotation_speed),
                    field_type: <Vec3 as lgn_data_model::TypeReflection>::get_type_def(),
                    group: "".into()
                },
            ]
        );
        lgn_data_model::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_option_descriptor!(DebugCube);
        lgn_data_model::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_model::TypeDefinition {
        lgn_data_model::implement_array_descriptor!(DebugCube);
        lgn_data_model::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { # [allow (clippy :: needless_update)] static ref __DEBUGCUBE_DEFAULT : DebugCube = DebugCube :: default () ; }
use lgn_data_runtime::{Asset, AssetLoader, Resource};
use std::{any::Any, io};
impl Resource for DebugCube {
    const TYPENAME: &'static str = "runtime_debugcube";
}
impl Asset for DebugCube {
    type Loader = DebugCubeLoader;
}
#[derive(Default)]
pub struct DebugCubeLoader {}
impl AssetLoader for DebugCubeLoader {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let output: DebugCube = bincode::deserialize_from(reader)
            .map_err(|_err| io::Error::new(io::ErrorKind::InvalidData, "Failed to parse"))?;
        Ok(Box::new(output))
    }
    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
