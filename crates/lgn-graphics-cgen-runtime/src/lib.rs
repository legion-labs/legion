//! Graphics code generation runtime

// crate-specific lint exceptions:
#![allow(unreachable_code)]

use std::sync::Arc;

use lgn_graphics_api::{
    BufferViewFlags, DescriptorDef, DescriptorRef, DescriptorSetLayout, DescriptorSetLayoutDef,
    DeviceContext, GPUViewType, PushConstantDef, RootSignature, RootSignatureDef,
    ShaderResourceType, ShaderStageFlags, ViewDimension, MAX_DESCRIPTOR_SET_LAYOUTS,
};
use lgn_math::prelude::*;

use half::prelude::*;

macro_rules! impl_native_type_def {
    ( $x:ident ) => {
        impl $x {
            pub fn def() -> &'static CGenTypeDef {
                &CGenTypeDef {
                    name: stringify!($x),
                    size: std::mem::size_of::<$x>(),
                }
            }
        }
    };
}

///
/// Float1
///
#[derive(Default, Clone, Copy)]
pub struct Float1(f32);

impl_native_type_def!(Float1);

impl From<f32> for Float1 {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl From<Float1> for f32 {
    fn from(value: Float1) -> Self {
        value.0
    }
}

///
/// Float2
///
#[derive(Default, Clone, Copy)]
pub struct Float2([f32; 2]);

impl_native_type_def!(Float2);

impl From<Vec2> for Float2 {
    fn from(value: Vec2) -> Self {
        Self(value.to_array())
    }
}

impl From<Float2> for Vec2 {
    fn from(value: Float2) -> Self {
        Self::new(value.0[0], value.0[1])
    }
}

///
/// Float3
///
#[derive(Default, Clone, Copy)]
pub struct Float3([f32; 3]);

impl_native_type_def!(Float3);

impl From<Vec3> for Float3 {
    fn from(value: Vec3) -> Self {
        Self(value.to_array())
    }
}

impl From<Float3> for Vec3 {
    fn from(value: Float3) -> Self {
        Self::new(value.0[0], value.0[1], value.0[2])
    }
}

///
/// Float4
///
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Float4([f32; 4]);

impl_native_type_def!(Float4);

impl From<Vec4> for Float4 {
    fn from(value: Vec4) -> Self {
        Self(value.to_array())
    }
}

impl From<Float4> for Vec4 {
    fn from(value: Float4) -> Self {
        Self::new(value.0[0], value.0[1], value.0[2], value.0[3])
    }
}

///
/// Uint1
///
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Uint1(u32);

impl_native_type_def!(Uint1);

impl From<u32> for Uint1 {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Uint1> for u32 {
    fn from(value: Uint1) -> Self {
        value.0
    }
}

///
/// Uint2
///
#[derive(Default, Clone, Copy)]
pub struct Uint2([u32; 2]);

impl_native_type_def!(Uint2);

impl From<UVec2> for Uint2 {
    fn from(value: UVec2) -> Self {
        Self(value.to_array())
    }
}

impl From<Uint2> for UVec2 {
    fn from(value: Uint2) -> Self {
        Self::new(value.0[0], value.0[1])
    }
}

///
/// Uint3
///
#[derive(Default, Clone, Copy)]
pub struct Uint3([u32; 3]);

impl_native_type_def!(Uint3);

impl From<UVec3> for Uint3 {
    fn from(value: UVec3) -> Self {
        Self(value.to_array())
    }
}

impl From<Uint3> for UVec3 {
    fn from(value: Uint3) -> Self {
        Self::new(value.0[0], value.0[1], value.0[2])
    }
}

///
/// Uint4
///
#[derive(Default, Clone, Copy)]
pub struct Uint4([u32; 4]);

impl_native_type_def!(Uint4);

impl From<UVec4> for Uint4 {
    fn from(value: UVec4) -> Self {
        Self(value.to_array())
    }
}

impl From<Uint4> for UVec4 {
    fn from(value: Uint4) -> Self {
        Self::new(value.0[0], value.0[1], value.0[2], value.0[3])
    }
}

///
/// Half1
///
#[derive(Default, Clone, Copy)]
pub struct Half1(u16);

impl_native_type_def!(Half1);

impl From<f32> for Half1 {
    fn from(value: f32) -> Self {
        Self(half::f16::from_f32(value).to_bits())
    }
}

impl From<f16> for Half1 {
    fn from(value: f16) -> Self {
        Self(value.to_bits())
    }
}

impl From<Half1> for f32 {
    fn from(value: Half1) -> Self {
        half::f16::from_bits(value.0).to_f32()
    }
}

///
/// Half2
///
#[derive(Default, Clone, Copy)]
pub struct Half2([u16; 2]);

impl_native_type_def!(Half2);

impl From<Vec2> for Half2 {
    fn from(value: Vec2) -> Self {
        Self([
            half::f16::from_f32(value.x).to_bits(),
            half::f16::from_f32(value.y).to_bits(),
        ])
    }
}

impl From<Half2> for Vec2 {
    fn from(value: Half2) -> Self {
        Self::new(
            half::f16::from_bits(value.0[0]).to_f32(),
            half::f16::from_bits(value.0[1]).to_f32(),
        )
    }
}

///
/// Half3
///
#[derive(Default, Clone, Copy)]
pub struct Half3([u16; 3]);

impl_native_type_def!(Half3);

impl From<Vec3> for Half3 {
    fn from(value: Vec3) -> Self {
        Self([
            half::f16::from_f32(value.x).to_bits(),
            half::f16::from_f32(value.y).to_bits(),
            half::f16::from_f32(value.z).to_bits(),
        ])
    }
}

impl From<Half3> for Vec3 {
    fn from(value: Half3) -> Self {
        Self::new(
            half::f16::from_bits(value.0[0]).to_f32(),
            half::f16::from_bits(value.0[1]).to_f32(),
            half::f16::from_bits(value.0[2]).to_f32(),
        )
    }
}

///
/// Half4
///
#[derive(Default, Clone, Copy)]
pub struct Half4([u16; 4]);

impl_native_type_def!(Half4);

impl From<Vec4> for Half4 {
    fn from(value: Vec4) -> Self {
        Self([
            half::f16::from_f32(value.x).to_bits(),
            half::f16::from_f32(value.y).to_bits(),
            half::f16::from_f32(value.z).to_bits(),
            half::f16::from_f32(value.w).to_bits(),
        ])
    }
}

impl From<Half4> for Vec4 {
    fn from(value: Half4) -> Self {
        Self::new(
            half::f16::from_bits(value.0[0]).to_f32(),
            half::f16::from_bits(value.0[1]).to_f32(),
            half::f16::from_bits(value.0[2]).to_f32(),
            half::f16::from_bits(value.0[3]).to_f32(),
        )
    }
}

///
/// Float4x4
///
#[derive(Default, Clone, Copy)]
pub struct Float4x4([f32; 16]);

impl_native_type_def!(Float4x4);

impl From<Mat4> for Float4x4 {
    fn from(value: Mat4) -> Self {
        Self(value.to_cols_array())
    }
}

pub mod prelude {
    pub use crate::Float1;
    pub use crate::Float2;
    pub use crate::Float3;
    pub use crate::Float4;
    pub use crate::Float4x4;
    pub use crate::Half1;
    pub use crate::Half2;
    pub use crate::Half3;
    pub use crate::Half4;
    pub use crate::Uint1;
    pub use crate::Uint2;
    pub use crate::Uint3;
    pub use crate::Uint4;
}

//
// CGenTypeDef
//
pub struct CGenTypeDef {
    pub name: &'static str,
    pub size: usize,
}

//
// CGenDescriptorDef
//
#[derive(Debug, PartialEq)]
pub struct CGenDescriptorDef {
    pub name: &'static str,
    pub shader_resource_type: lgn_graphics_api::ShaderResourceType,
    pub bindless: bool,
    pub flat_index_start: u32,
    pub flat_index_end: u32,
    pub array_size: u32,
}

impl CGenDescriptorDef {
    pub fn validate(&self, descriptor_ref: &DescriptorRef) -> bool {
        match self.shader_resource_type {
            ShaderResourceType::Sampler => match descriptor_ref {
                DescriptorRef::Sampler(_) => true,
                DescriptorRef::Undefined
                | DescriptorRef::TransientBufferView(_)
                | DescriptorRef::BufferView(_)
                | DescriptorRef::TextureView(_) => {
                    panic!("Descriptor {}: expected Sampler", self.name)
                }
            },
            ShaderResourceType::ConstantBuffer => {
                let view_definition = Self::get_buffer_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ConstantBuffer {
                        true
                    } else {
                        panic!(
                            "Descriptor {}: expected view with type ConstantBuffer",
                            self.name
                        );
                    }
                } else {
                    panic!("Descriptor {}: expected view of type BufferView", self.name);
                }
            }
            ShaderResourceType::StructuredBuffer => {
                let view_definition = Self::get_buffer_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && !view_definition
                            .buffer_view_flags
                            .intersects(BufferViewFlags::RAW_BUFFER)
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and buffer flags not containing RAW_BUFFER", self.name);
                    }
                } else {
                    panic!("Descriptor {}: expected view of type BufferView", self.name);
                }
            }
            ShaderResourceType::ByteAddressBuffer => {
                let view_definition = Self::get_buffer_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && view_definition
                            .buffer_view_flags
                            .intersects(BufferViewFlags::RAW_BUFFER)
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and buffer flags containing RAW_BUFFER", self.name);
                    }
                } else {
                    panic!("Descriptor {}: expected view of type BufferView", self.name);
                }
            }

            ShaderResourceType::RWStructuredBuffer => {
                let view_definition = Self::get_buffer_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::UnorderedAccess
                        && !view_definition
                            .buffer_view_flags
                            .intersects(BufferViewFlags::RAW_BUFFER)
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type UnorderedAccess and buffer flags not containing RAW_BUFFER", self.name);
                    }
                } else {
                    panic!("Descriptor {}: expected view of type BufferView", self.name);
                }
            }
            ShaderResourceType::RWByteAddressBuffer => {
                let view_definition = Self::get_buffer_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::UnorderedAccess
                        && view_definition
                            .buffer_view_flags
                            .intersects(BufferViewFlags::RAW_BUFFER)
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type UnorderedAccess and buffer flags containing RAW_BUFFER", self.name);
                    }
                } else {
                    panic!("Descriptor {}: expected view of type BufferView", self.name);
                }
            }
            ShaderResourceType::Texture2D => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && view_definition.view_dimension == ViewDimension::_2D
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and view dimension 2D", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }

            ShaderResourceType::Texture2DArray => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && view_definition.view_dimension == ViewDimension::_2DArray
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and view dimension 2DArray", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }

            ShaderResourceType::Texture3D => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && view_definition.view_dimension == ViewDimension::_3D
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and view dimension 3D", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }

            ShaderResourceType::TextureCube => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && view_definition.view_dimension == ViewDimension::Cube
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and view dimension Cube", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }

            ShaderResourceType::TextureCubeArray => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::ShaderResource
                        && view_definition.view_dimension == ViewDimension::CubeArray
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type ShaderResource and view dimension CubeArray", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }
            ShaderResourceType::RWTexture2D => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::UnorderedAccess
                        && view_definition.view_dimension == ViewDimension::_2D
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type UnorderedAccess and view dimension 2D", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }

            ShaderResourceType::RWTexture2DArray => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::UnorderedAccess
                        && view_definition.view_dimension == ViewDimension::_2DArray
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type UnorderedAccess and view dimension 2DArray", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }

            ShaderResourceType::RWTexture3D => {
                let view_definition = Self::get_texture_view_def(descriptor_ref);
                if let Some(view_definition) = view_definition {
                    if view_definition.gpu_view_type == GPUViewType::UnorderedAccess
                        && view_definition.view_dimension == ViewDimension::_3D
                    {
                        true
                    } else {
                        panic!("Descriptor {}: expected view of type UnorderedAccess and view dimension 3D", self.name);
                    }
                } else {
                    panic!(
                        "Descriptor {}: expected view of type TextureView",
                        self.name
                    );
                }
            }
        }
    }

    #[allow(unsafe_code)]
    fn get_buffer_view_def(
        descriptor_ref: &DescriptorRef,
    ) -> Option<&lgn_graphics_api::BufferViewDef> {
        match descriptor_ref {
            DescriptorRef::TransientBufferView(view) => Some(view.definition()),
            DescriptorRef::BufferView(view) => Some(unsafe { view.as_ref().unwrap().definition() }),
            DescriptorRef::Undefined
            | DescriptorRef::Sampler(_)
            | DescriptorRef::TextureView(_) => None,
        }
    }

    #[allow(unsafe_code)]
    fn get_texture_view_def(
        descriptor_ref: &DescriptorRef,
    ) -> Option<&lgn_graphics_api::TextureViewDef> {
        match descriptor_ref {
            DescriptorRef::TextureView(view) => {
                Some(unsafe { view.as_ref().unwrap().definition() })
            }
            DescriptorRef::Undefined
            | DescriptorRef::Sampler(_)
            | DescriptorRef::BufferView(_)
            | DescriptorRef::TransientBufferView(_) => None,
        }
    }
}

//
// CGenDescriptorSetDef
//
#[derive(Default, Debug, PartialEq)]
pub struct CGenDescriptorSetDef {
    pub name: &'static str,
    pub id: u32,
    pub frequency: u32,
    pub descriptor_flat_count: u32,
    pub descriptor_defs: &'static [CGenDescriptorDef],
}

//
// CGenPipelineLayoutDef
//
#[derive(Default, Debug, PartialEq)]
pub struct CGenPipelineLayoutDef {
    pub name: &'static str,
    pub id: u32,
    pub descriptor_set_layout_ids: [Option<u32>; MAX_DESCRIPTOR_SET_LAYOUTS],
    pub push_constant_type: Option<u32>,
}

//
// CGenCrateID
//
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct CGenCrateID(pub u64);

//
// CGenShaderID
//
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CGenShaderID(pub u16);

impl CGenShaderID {
    pub const fn make(family_idx: u64) -> Self {
        let family_idx = family_idx as u16;
        Self(family_idx)
    }
}

//
// CGenShaderOptionMask
//
pub type CGenShaderOptionMask = u64;

//
// CGenShaderKey
//
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[repr(packed)]
pub struct CGenShaderKey(u64);

impl CGenShaderKey {
    // u16: family_id
    // u48: options
    const SHADER_FAMILY_ID_OFFSET: usize = 0;
    const SHADER_FAMILY_ID_BITCOUNT: usize = std::mem::size_of::<u16>() * 8;
    const SHADER_FAMILY_ID_MASK: u64 = (1 << Self::SHADER_FAMILY_ID_BITCOUNT) - 1;

    pub const MAX_SHADER_OPTIONS: usize =
        std::mem::size_of::<u64>() * 8 - Self::SHADER_OPTIONS_OFFSET;

    const SHADER_OPTIONS_OFFSET: usize = Self::SHADER_FAMILY_ID_BITCOUNT;
    const SHADER_OPTIONS_BITCOUNT: usize = Self::MAX_SHADER_OPTIONS;
    const SHADER_OPTIONS_MASK: u64 = (1 << Self::SHADER_OPTIONS_BITCOUNT) - 1;

    pub const fn make(shader_id: CGenShaderID, shader_option_mask: CGenShaderOptionMask) -> Self {
        let shader_family_id = shader_id.0 as u64;
        // static_assertions::const_assert_eq!(shader_family_id & Self::SHADER_FAMILY_MASK, 0);
        let shader_option_mask = shader_option_mask as u64;
        // static_assertions::const_assert_eq!(shader_option_mask & Self::SHADER_OPTIONS_MASK, 0);
        Self(
            ((shader_family_id & Self::SHADER_FAMILY_ID_MASK) << Self::SHADER_FAMILY_ID_OFFSET)
                | ((shader_option_mask & Self::SHADER_OPTIONS_MASK) << Self::SHADER_OPTIONS_OFFSET),
        )
    }

    pub fn shader_id(self) -> CGenShaderID {
        let shader_id = (self.0 >> Self::SHADER_FAMILY_ID_OFFSET) & Self::SHADER_FAMILY_ID_MASK;
        CGenShaderID(shader_id.try_into().unwrap())
    }

    pub fn shader_option_mask(self) -> CGenShaderOptionMask {
        (self.0 >> Self::SHADER_OPTIONS_OFFSET) & Self::SHADER_OPTIONS_MASK
    }
}

//
// CGenShader
//
pub struct CGenShaderDef {
    pub id: CGenShaderID,
    pub name: &'static str,
    pub path: &'static str,
    pub options: &'static [CGenShaderOption],
    pub instances: &'static [CGenShaderInstance],
}

//
// CGenShaderOption
//
pub struct CGenShaderOption {
    pub index: u8,
    pub name: &'static str,
}

//
// CGenShaderInstance
//
pub struct CGenShaderInstance {
    pub key: CGenShaderKey,
    pub stage_flags: ShaderStageFlags,
}

//
// CGenRegistry
//
pub struct CGenRegistry {
    pub crate_id: CGenCrateID,
    pub shutdown_fn: fn(),

    // static
    pub type_defs: Vec<&'static CGenTypeDef>,
    pub shader_defs: Vec<&'static CGenShaderDef>,
    // dynamic
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub pipeline_layouts: Vec<RootSignature>,
}

impl CGenRegistry {
    pub fn new(crate_id: CGenCrateID, shutdown_fn: fn()) -> Self {
        Self {
            crate_id,
            shutdown_fn,
            type_defs: Vec::new(),
            descriptor_set_layouts: Vec::new(),
            pipeline_layouts: Vec::new(),
            shader_defs: Vec::new(),
        }
    }

    pub fn shutdown(&self) {
        (self.shutdown_fn)();
    }

    pub fn add_type(&mut self, def: &'static CGenTypeDef) {
        self.type_defs.push(def);
    }

    pub fn add_descriptor_set(
        &mut self,
        device_context: &DeviceContext,
        def: &CGenDescriptorSetDef,
    ) {
        let mut layout_def = DescriptorSetLayoutDef {
            frequency: def.frequency,
            ..DescriptorSetLayoutDef::default()
        };

        layout_def
            .descriptor_defs
            .reserve_exact(def.descriptor_defs.len());

        for cgen_descriptor_def in def.descriptor_defs {
            let descriptor_def = DescriptorDef {
                name: cgen_descriptor_def.name.to_string(),
                bindless: cgen_descriptor_def.bindless,
                shader_resource_type: cgen_descriptor_def.shader_resource_type,
                array_size: cgen_descriptor_def.array_size,
            };
            layout_def.descriptor_defs.push(descriptor_def);
        }

        self.descriptor_set_layouts
            .push(device_context.create_descriptorset_layout(layout_def));
    }

    pub fn descriptor_set_layout(&self, id: u32) -> &DescriptorSetLayout {
        &self.descriptor_set_layouts[id as usize]
    }

    pub fn add_pipeline_layout(
        &mut self,
        device_context: &DeviceContext,
        def: &CGenPipelineLayoutDef,
    ) {
        let push_constant_def = def.push_constant_type.map(|ty_id| PushConstantDef {
            size: u32::try_from(self.type_defs[ty_id as usize].size).unwrap(),
        });

        let signature_def = RootSignatureDef {
            descriptor_set_layouts: def
                .descriptor_set_layout_ids
                .iter()
                .filter_map(|opt_id| {
                    opt_id.map(|id| self.descriptor_set_layouts[id as usize].clone())
                })
                .collect::<Vec<_>>(),
            push_constant_def,
        };

        self.pipeline_layouts
            .push(device_context.create_root_signature(signature_def));
    }

    pub fn pipeline_layout(&self, id: u32) -> &RootSignature {
        &self.pipeline_layouts[id as usize]
    }

    pub fn add_shader_def(&mut self, def: &'static CGenShaderDef) {
        self.shader_defs.push(def);
    }
}

//
// CGenRegistryList
//
#[derive(Default)]
pub struct CGenRegistryList {
    registry_list: Vec<Arc<CGenRegistry>>,
}

impl CGenRegistryList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, registry: Arc<CGenRegistry>) {
        self.registry_list.push(registry);
    }
}

impl Drop for CGenRegistryList {
    fn drop(&mut self) {
        for registry in self.registry_list.drain(..) {
            registry.shutdown();
        }
    }
}
