//! Graphics code generation runtime

// BEGIN - Legion Labs lints v0.6
// do not change or add/remove here, but one can add exceptions after this section
#![deny(unsafe_code)]
#![warn(future_incompatible, nonstandard_style, rust_2018_idioms)]
// Rustdoc lints
#![warn(
    rustdoc::broken_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::private_intra_doc_links
)]
// Clippy pedantic lints, treat all as warnings by default, add exceptions in allow list
#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::if_not_else,
    clippy::items_after_statements,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::similar_names,
    clippy::shadow_unrelated,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix
)]
// Clippy nursery lints, still under development
#![warn(
    clippy::debug_assert_with_mut_call,
    clippy::disallowed_method,
    clippy::disallowed_type,
    clippy::fallible_impl_from,
    clippy::imprecise_flops,
    clippy::mutex_integer,
    clippy::path_buf_push_overwrite,
    clippy::string_lit_as_bytes,
    clippy::use_self,
    clippy::useless_transmute
)]
// Clippy restriction lints, usually not considered bad, but useful in specific cases
#![warn(
    clippy::dbg_macro,
    clippy::exit,
    clippy::float_cmp_const,
    clippy::map_err_ignore,
    clippy::mem_forget,
    clippy::missing_enforced_import_renames,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::string_to_string,
    clippy::todo,
    clippy::unimplemented,
    clippy::verbose_file_reads
)]
// END - Legion Labs lints v0.6
// crate-specific exceptions:
#![allow(unreachable_code)]

use std::{marker::PhantomData, sync::Arc};

use lgn_graphics_api::{
    DescriptorHeapPartition, DescriptorSetHandle, DescriptorSetLayout, DescriptorSetWriter,
    DeviceContext, MAX_DESCRIPTOR_SET_LAYOUTS,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy)]
pub struct Float1(f32);

#[derive(Default, Clone, Copy)]
pub struct Float2(glam::Vec2);

#[derive(Default, Clone, Copy)]
pub struct Float3(glam::Vec3);

#[derive(Default, Clone, Copy)]
pub struct Float4(glam::Vec4);

#[derive(Default, Clone, Copy)]
pub struct Float4x4(glam::Mat4);

pub mod prelude {
    pub use crate::Float1;
    pub use crate::Float2;
    pub use crate::Float3;
    pub use crate::Float4;
    pub use crate::Float4x4;
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CGenTypeId(pub u32);

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenTypeDef {
    pub id: CGenTypeId,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct CGenDescriptorId(pub u32);

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenDescriptorDef {
    pub name: String,
    pub shader_resource_type: lgn_graphics_api::ShaderResourceType,
    pub flat_index: u32,
    pub array_size: u32,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Clone, Copy)]
pub struct CGenDescriptorSetId(pub u32);

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenDescriptorSetDef {
    pub name: String,
    pub id: CGenDescriptorSetId,
    pub frequency: u32,
    pub descriptor_flat_count: u32,
    pub descriptor_defs: Vec<CGenDescriptorDef>,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub struct CGenPipelineLayoutId(pub u32);

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenPipelineLayoutDef {
    pub name: String,
    pub id: CGenPipelineLayoutId,
    pub descriptor_set_layout_ids: [Option<CGenDescriptorSetId>; MAX_DESCRIPTOR_SET_LAYOUTS],
    pub push_constant_type: Option<CGenTypeId>,
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

#[derive(Clone)]
pub struct CGenRuntime {
    inner: Arc<CGenRuntimeInner>,
}

impl CGenRuntime {
    #[allow(clippy::todo)]
    pub fn new(cgen_def: &[u8], device_context: &lgn_graphics_api::DeviceContext) -> Self {
        let definition: CGenDef = bincode::deserialize(cgen_def).unwrap();

        let mut descriptor_set_layouts =
            Vec::with_capacity(definition.descriptor_set_layout_defs.len());
        for cgen_layout_def in &definition.descriptor_set_layout_defs {
            let mut layout_def = lgn_graphics_api::DescriptorSetLayoutDef {
                frequency: cgen_layout_def.frequency,
                ..lgn_graphics_api::DescriptorSetLayoutDef::default()
            };
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
                    .filter_map(|opt_id| {
                        opt_id
                            .as_ref()
                            .map(|id| descriptor_set_layouts[id.0 as usize].clone())
                    })
                    .collect::<Vec<_>>(),
                push_constant_def: cgen_signature_def.push_constant_type.as_ref().map(|_pc| {
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

    // pub fn new_descriptor_set<'frame, T>(
    //     &self,
    //     bump: &'frame bumpalo::Bump,
    //     descriptor_heap_partition: &DescriptorHeapPartition,
    // ) -> DescriptorSetData<'_, T>
    // where
    //     T: CGenDescriptorSetInfo,
    //     T::DescriptorID: ToIndex + Copy,
    // {
    //     let cgen_descriptor_set_def =
    // self.get_descriptor_set_def(T::descriptor_set_layout_id());
    //     let descriptor_set_layout =
    // self.get_descriptor_set_layout(T::descriptor_set_layout_id());
    //     DescriptorSetData::<'_, T>::new(self, bump, descriptor_heap_partition)
    // }

    pub fn get_descriptor_set_def(&self, id: CGenDescriptorSetId) -> &CGenDescriptorSetDef {
        &self.inner.definition.descriptor_set_layout_defs[id.0 as usize]
    }

    pub fn get_descriptor_set_layout(&self, id: CGenDescriptorSetId) -> &DescriptorSetLayout {
        &self.inner.descriptor_set_layouts[id.0 as usize]
    }

    pub fn get_pipeline_layout_def(
        &self,
        pipeline_layout_id: CGenPipelineLayoutId,
    ) -> &CGenPipelineLayoutDef {
        &self.inner.definition.root_signature_defs[pipeline_layout_id.0 as usize]
    }

    pub fn get_pipeline_layout(
        &self,
        pipeline_layout_id: CGenPipelineLayoutId,
    ) -> &lgn_graphics_api::RootSignature {
        &self.inner.root_signatures[pipeline_layout_id.0 as usize]
    }
}

pub trait CGenDescriptorSetInfo {
    // type DescriptorID;
    fn id() -> CGenDescriptorSetId;
}

pub trait CGenPipelineLayoutInfo {
    fn id() -> CGenPipelineLayoutId;
}

pub struct DescriptorSetData_<'renderer, 'frame, T>
where
    T: CGenDescriptorSetInfo,
    // T::DescriptorID: ToIndex + Copy,
{
    cgen_descriptor_set_def: &'renderer CGenDescriptorSetDef,
    writer: DescriptorSetWriter<'frame>,
    _phantom: PhantomData<T>,
}

pub trait ToIndex {
    fn to_index(self) -> usize;
}

impl<'renderer, 'frame, T: CGenDescriptorSetInfo> DescriptorSetData_<'renderer, 'frame, T>
// where
//     T::DescriptorID: ToIndex + Copy,
{
    pub fn new(
        cgen_runtime: &'renderer CGenRuntime,
        bump: &'frame bumpalo::Bump,
        descriptor_heap_partition: &DescriptorHeapPartition,
    ) -> Self {
        let cgen_descriptor_set_def = cgen_runtime.get_descriptor_set_def(T::id());
        let descriptor_set_layout = cgen_runtime.get_descriptor_set_layout(T::id());

        Self {
            cgen_descriptor_set_def,
            writer: descriptor_heap_partition
                .write_descriptor_set(descriptor_set_layout, bump)
                .unwrap(),
            _phantom: PhantomData,
        }
    }

    // pub fn set_constant_buffer(&mut self, id: T::DescriptorID, cbv: &'frame BufferView) {
    //     let descriptor_index = id.to_index();
    //     let descriptor_def = &self.cgen_descriptor_set_def.descriptor_defs[descriptor_index];
    //     assert_eq!(
    //         descriptor_def.shader_resource_type,
    //         ShaderResourceType::ConstantBuffer
    //     );

    //     self.writer
    //         .set_descriptors_by_index(descriptor_index, &[DescriptorRef::BufferView(cbv)])
    //         .unwrap();
    // }

    pub fn build(self, device_context: &DeviceContext) -> DescriptorSetHandle {
        self.writer.flush(device_context)
    }
}

pub mod fake {

    use crate::{CGenDescriptorSetId, CGenDescriptorSetInfo, DescriptorSetData_, ToIndex};

    pub struct Fake;

    #[derive(Clone, Copy)]
    pub enum FakeDescriptorID {
        A = 0,
        B = 1,
        C = 2,
    }

    impl ToIndex for FakeDescriptorID {
        fn to_index(self) -> usize {
            match self {
                FakeDescriptorID::A => 0,
                FakeDescriptorID::B => 1,
                FakeDescriptorID::C => 2,
            }
        }
    }

    pub type FakeDescriptorSetData<'renderer, 'frame> = DescriptorSetData_<'renderer, 'frame, Fake>;

    impl CGenDescriptorSetInfo for Fake {
        // type DescriptorID = FakeDescriptorID;

        fn id() -> CGenDescriptorSetId {
            CGenDescriptorSetId(0)
        }
    }
}
