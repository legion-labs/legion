#![allow(dead_code)]
#![allow(clippy::needless_update)]

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
    const SIGNATURE_HASH: u64 = 13358266263809831249u64;
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
impl lgn_data_reflection::TypeReflection for DebugCube {
    fn get_type(&self) -> lgn_data_reflection::TypeDefinition {
        Self::get_type_def()
    }
    fn get_type_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_struct_descriptor!(
            DebugCube,
            vec![
                lgn_data_reflection::FieldDescriptor {
                    field_name: "position".into(),
                    offset: memoffset::offset_of!(DebugCube, position),
                    field_type: <Vec3 as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "rotation".into(),
                    offset: memoffset::offset_of!(DebugCube, rotation),
                    field_type: <Quat as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "scale".into(),
                    offset: memoffset::offset_of!(DebugCube, scale),
                    field_type: <Vec3 as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "mesh_id".into(),
                    offset: memoffset::offset_of!(DebugCube, mesh_id),
                    field_type: <usize as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "color".into(),
                    offset: memoffset::offset_of!(DebugCube, color),
                    field_type: <Color as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
                lgn_data_reflection::FieldDescriptor {
                    field_name: "rotation_speed".into(),
                    offset: memoffset::offset_of!(DebugCube, rotation_speed),
                    field_type: <Vec3 as lgn_data_reflection::TypeReflection>::get_type_def(),
                    group: "".into()
                },
            ]
        );
        lgn_data_reflection::TypeDefinition::Struct(&TYPE_DESCRIPTOR)
    }
    fn get_option_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_option_descriptor!(DebugCube);
        lgn_data_reflection::TypeDefinition::Option(&OPTION_DESCRIPTOR)
    }
    fn get_array_def() -> lgn_data_reflection::TypeDefinition {
        lgn_data_reflection::implement_array_descriptor!(DebugCube);
        lgn_data_reflection::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
    }
}
lazy_static::lazy_static! { static ref __DEBUGCUBE_DEFAULT : DebugCube = DebugCube { .. DebugCube :: default () } ; }
use lgn_data_offline::resource::{OfflineResource, ResourceProcessor};
use lgn_data_runtime::{Asset, AssetLoader, Resource};
use std::{any::Any, io};
impl Resource for DebugCube {
    const TYPENAME: &'static str = "offline_debugcube";
}
impl Asset for DebugCube {
    type Loader = DebugCubeProcessor;
}
impl OfflineResource for DebugCube {
    type Processor = DebugCubeProcessor;
}
#[derive(Default)]
pub struct DebugCubeProcessor {}
impl AssetLoader for DebugCubeProcessor {
    fn load(&mut self, reader: &mut dyn io::Read) -> io::Result<Box<dyn Any + Send + Sync>> {
        let mut instance = DebugCube {
            ..DebugCube::default()
        };
        let values: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        lgn_data_reflection::json_utils::reflection_apply_json_edit::<DebugCube>(
            &mut instance,
            &values,
        )
        .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(Box::new(instance))
    }
    fn load_init(&mut self, _asset: &mut (dyn Any + Send + Sync)) {}
}
impl ResourceProcessor for DebugCubeProcessor {
    fn new_resource(&mut self) -> Box<dyn Any + Send + Sync> {
        Box::new(DebugCube {
            ..DebugCube::default()
        })
    }
    fn extract_build_dependencies(
        &mut self,
        _resource: &dyn Any,
    ) -> Vec<lgn_data_offline::ResourcePathId> {
        vec![]
    }
    #[allow(clippy::float_cmp, clippy::too_many_lines)]
    fn write_resource(
        &mut self,
        resource: &dyn Any,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<usize> {
        let instance = resource.downcast_ref::<DebugCube>().unwrap();
        let values = lgn_data_reflection::json_utils::reflection_save_relative_json(
            instance,
            DebugCube::get_default_instance(),
        )
        .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        serde_json::to_writer_pretty(writer, &values)
            .map_err(|_err| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json"))?;
        Ok(1)
    }
    fn read_resource(
        &mut self,
        reader: &mut dyn std::io::Read,
    ) -> std::io::Result<Box<dyn Any + Send + Sync>> {
        self.load(reader)
    }
    fn get_resource_reflection<'a>(
        &self,
        resource: &'a dyn Any,
    ) -> Option<&'a dyn lgn_data_reflection::TypeReflection> {
        if let Some(instance) = resource.downcast_ref::<DebugCube>() {
            return Some(instance);
        }
        None
    }
    fn get_resource_reflection_mut<'a>(
        &self,
        resource: &'a mut dyn Any,
    ) -> Option<&'a mut dyn lgn_data_reflection::TypeReflection> {
        if let Some(instance) = resource.downcast_mut::<DebugCube>() {
            return Some(instance);
        }
        None
    }
}
