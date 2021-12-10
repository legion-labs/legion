use std::{marker::PhantomData, sync::Arc};

use lgn_graphics_api::{BufferView, DescriptorHeap, DescriptorSetHandle};
use serde::{Deserialize, Serialize};

pub struct Float1(f32);
pub struct Float2(glam::Vec2);

pub struct Float3(glam::Vec3);

pub struct Float4(glam::Vec4);

pub struct Float4x4(glam::Mat4);

pub mod prelude {
    pub use crate::Float1;
    pub use crate::Float2;
    pub use crate::Float3;
    pub use crate::Float4;
    pub use crate::Float4x4;
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenTypeDef {}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenDescriptorDef {
    pub name: String,
    pub shader_resource_type: lgn_graphics_api::ShaderResourceType,
    pub array_size: u32,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenDescriptorSetDef {
    pub name: String,
    pub frequency: u32,
    pub descriptor_defs: Vec<CGenDescriptorDef>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenPipelineLayoutDef {
    pub name: String,
    pub descriptor_set_layout_ids: Vec<u32>,
    pub push_constant_type: Option<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenDef {
    pub type_defs: Vec<CGenTypeDef>,
    pub descriptor_set_layout_defs: Vec<CGenDescriptorSetDef>,
    pub root_signature_defs: Vec<CGenPipelineLayoutDef>,
}

struct CGenRuntimeInner {
    definition: CGenDef,
    descriptor_set_layouts: Vec<lgn_graphics_api::DescriptorSetLayout>,
    root_signatures: Vec<lgn_graphics_api::RootSignature>,
}

pub struct CGenRuntime {
    inner: Arc<CGenRuntimeInner>,
}

impl CGenRuntime {
    pub fn new(cgen_def: &[u8], device_context: &lgn_graphics_api::DeviceContext) -> Self {
        let definition: CGenDef = bincode::deserialize(&cgen_def).unwrap();

        let mut descriptor_set_layouts =
            Vec::with_capacity(definition.descriptor_set_layout_defs.len());
        for cgen_layout_def in definition.descriptor_set_layout_defs.iter() {
            let mut layout_def = lgn_graphics_api::DescriptorSetLayoutDef::default();
            layout_def.frequency = cgen_layout_def.frequency;
            layout_def
                .descriptor_defs
                .reserve_exact(cgen_layout_def.descriptor_defs.len());
            for (i, cgen_descriptor_def) in cgen_layout_def.descriptor_defs.iter().enumerate() {
                let descriptor_def = lgn_graphics_api::DescriptorDef {
                    name: cgen_descriptor_def.name.clone(),
                    binding: u32::try_from(i).unwrap(),
                    shader_resource_type: cgen_descriptor_def.shader_resource_type,
                    array_size: cgen_descriptor_def.array_size,
                };
                layout_def.descriptor_defs.push(descriptor_def);
            }

            descriptor_set_layouts.push(
                device_context
                    .create_descriptorset_layout(&layout_def)
                    .unwrap(),
            );
        }

        let mut root_signatures = Vec::with_capacity(definition.root_signature_defs.len());
        for cgen_signature_def in &definition.root_signature_defs {
            let signature_def = lgn_graphics_api::RootSignatureDef {
                descriptor_set_layouts: cgen_signature_def
                    .descriptor_set_layout_ids
                    .iter()
                    .map(|id| descriptor_set_layouts[*id as usize].clone())
                    .collect::<Vec<_>>(),
                push_constant_def: cgen_signature_def.push_constant_type.map(|pc| {
                    lgn_graphics_api::PushConstantDef {
                        used_in_shader_stages: todo!(),
                        size: todo!(),
                    }
                }),
            };

            root_signatures.push(
                device_context
                    .create_root_signature(&signature_def)
                    .unwrap(),
            );
        }

        Self {
            inner: Arc::new(CGenRuntimeInner {
                definition,
                descriptor_set_layouts,
                root_signatures,
            }),
        }
    }
}

pub trait DescriptorSetLayoutStaticInfo {
    type DescriptorID;
}

pub struct Fake;

pub enum FakeDescriptorID {
    A,
    B,
    C,
}

pub type FakeDescriptorSetData = DescriptorSetData<Fake>;

impl DescriptorSetLayoutStaticInfo for Fake {
    type DescriptorID = FakeDescriptorID;
}

// T : CGenDescriptorDef
pub struct DescriptorSetData<T: DescriptorSetLayoutStaticInfo> {
    _phantom: PhantomData<T>,
}

impl<T: DescriptorSetLayoutStaticInfo> DescriptorSetData<T> {
    pub fn new(bump: &bumpalo::Bump) -> Self {
        Self {
            _phantom: PhantomData::default(),
        }
    }

    pub fn set_constant_buffer(self, id: T::DescriptorID, cbv: BufferView) -> Self {
        self
    }

    pub fn build(self, descriptor_heap: &DescriptorHeap) -> () { // DescriptorSetHandle {
                                                                 // let writer = descriptor_heap.allocate_descriptor_set().unwrap();
                                                                 //         writer.flush(vulkan_device_context)
    }
}
