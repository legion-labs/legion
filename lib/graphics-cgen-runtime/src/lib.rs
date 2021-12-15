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

use std::{marker::PhantomData, sync::Arc};

use lgn_graphics_api::{
    BufferView, DescriptorHeapPartition, DescriptorRef, DescriptorSetHandle, DescriptorSetLayout,
    DescriptorSetWriter, DeviceContext,
};
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
    pub flat_index: u32,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct CGenDescriptorSetDef {
    pub name: String,
    pub frequency: u32,
    pub descriptor_defs: Vec<CGenDescriptorDef>,
    pub flat_sampler_count: u32,
    pub flat_texture_count: u32,
    pub flat_buffer_count: u32,
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

#[derive(Clone)]
pub struct CGenRuntime {
    inner: Arc<CGenRuntimeInner>,
}

impl CGenRuntime {
    pub fn new(cgen_def: &[u8], device_context: &lgn_graphics_api::DeviceContext) -> Self {
        let definition: CGenDef = bincode::deserialize(cgen_def).unwrap();

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
                push_constant_def: cgen_signature_def.push_constant_type.map(|_pc| {
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
    //     T: DescriptorSetLayoutStaticInfo,
    //     T::DescriptorID: ToIndex + Copy,
    // {
    //     let cgen_descriptor_set_def = self.get_descriptor_set_def(T::descriptor_set_layout_id());
    //     let descriptor_set_layout = self.get_descriptor_set_layout(T::descriptor_set_layout_id());
    //     DescriptorSetData::<'_, T>::new(self, bump, descriptor_heap_partition)
    // }

    pub fn get_descriptor_set_def(&self, descriptor_set_layout_id: u32) -> &CGenDescriptorSetDef {
        &self.inner.definition.descriptor_set_layout_defs[descriptor_set_layout_id as usize]
    }

    pub fn get_descriptor_set_layout(&self, descriptor_set_layout_id: u32) -> &DescriptorSetLayout {
        &self.inner.descriptor_set_layouts[descriptor_set_layout_id as usize]
    }
}

pub trait DescriptorSetLayoutStaticInfo {
    type DescriptorID;
    fn descriptor_set_layout_id() -> u32;
}

pub struct DescriptorSetData<'renderer, 'frame, T>
where
    T: DescriptorSetLayoutStaticInfo,
    T::DescriptorID: ToIndex + Copy,
{
    cgen_descriptor_set_def: &'renderer CGenDescriptorSetDef,
    descriptor_set_layout: &'renderer DescriptorSetLayout,
    writer: DescriptorSetWriter<'frame>,
    // descriptors: RefCell<&'frame mut [Descriptor_<'frame>]>,
    _phantom: PhantomData<T>,
}

pub trait ToIndex {
    fn to_index(self) -> usize;
}

impl<'renderer, 'frame, T: DescriptorSetLayoutStaticInfo> DescriptorSetData<'renderer, 'frame, T>
where
    T::DescriptorID: ToIndex + Copy,
{
    pub fn new(
        cgen_runtime: &'renderer CGenRuntime,
        bump: &'frame bumpalo::Bump,
        descriptor_heap_partition: &DescriptorHeapPartition,
    ) -> Self {
        let cgen_descriptor_set_def =
            cgen_runtime.get_descriptor_set_def(T::descriptor_set_layout_id());
        let descriptor_set_layout =
            cgen_runtime.get_descriptor_set_layout(T::descriptor_set_layout_id());
        // let buffer_descriptors = bump.alloc_slice_fill_default::<BufferDescriptor>(
        //     cgen_descriptor_set_def.flat_buffer_count as usize,
        // );
        // let texture_descriptors = bump.alloc_slice_fill_default::<TextureDescriptor>(
        //     cgen_descriptor_set_def.flat_texture_count as usize,
        // );
        // let sampler_descriptors = bump.alloc_slice_fill_default::<SamplerDescriptor>(
        //     cgen_descriptor_set_def.flat_sampler_count as usize,
        // );
        // let descriptors = bump
        //     .alloc_slice_fill_default::<Descriptor_>(cgen_descriptor_set_def.descriptor_defs.len());

        // for (i, cgen_descriptor_def) in cgen_descriptor_set_def.descriptor_defs.iter().enumerate() {
        //     let first_index = cgen_descriptor_def.flat_index as usize;
        //     let count = usize::max(1, cgen_descriptor_def.array_size as usize);
        //     let descriptor_def = descriptor_set_layout.descriptor(i).unwrap();
        //     let descriptor_array = match descriptor_def.shader_resource_type {
        //         ShaderResourceType::Sampler => {
        //             // DescriptorArray::Sampler(sampler_descriptors [first_index..count].as_mut_ptr())
        //             DescriptorArray::Sampler(unsafe {
        //                 sampler_descriptors.as_mut_ptr().add(first_index)
        //             })
        //         }
        //         ShaderResourceType::ConstantBuffer
        //         | ShaderResourceType::StructuredBuffer
        //         | ShaderResourceType::RWStructuredBuffer
        //         | ShaderResourceType::ByteAdressBuffer
        //         | ShaderResourceType::RWByteAdressBuffer => {
        //             // DescriptorArray::Buffer(&buffer_descriptors[first_index..count].as_mut_ptr())
        //             DescriptorArray::Buffer(unsafe {
        //                 buffer_descriptors.as_mut_ptr().add(first_index)
        //             })
        //         }
        //         ShaderResourceType::Texture2D
        //         | ShaderResourceType::RWTexture2D
        //         | ShaderResourceType::Texture2DArray
        //         | ShaderResourceType::RWTexture2DArray
        //         | ShaderResourceType::Texture3D
        //         | ShaderResourceType::RWTexture3D
        //         | ShaderResourceType::TextureCube
        //         | ShaderResourceType::TextureCubeArray => DescriptorArray::Texture(unsafe {
        //             texture_descriptors.as_mut_ptr().add(first_index)
        //         }),
        //     };
        //     descriptors[i] = Descriptor_::new(descriptor_def, descriptor_array);
        // }

        Self {
            descriptor_set_layout,
            cgen_descriptor_set_def,
            // descriptors: RefCell::new(descriptors),
            writer: descriptor_heap_partition
                .write_descriptor_set(descriptor_set_layout, bump)
                .unwrap(),
            _phantom: PhantomData,
        }
    }

    pub fn set_constant_buffer(&mut self, id: T::DescriptorID, cbv: &'frame BufferView) {
        let descriptor_index = id.to_index();
        // let descriptor_def = &self.cgen_descriptor_set_def.descriptor_defs[descriptor_index];
        // let mut descriptor = &mut self.descriptors.borrow_mut()[descriptor_index];
        // descriptor.set_constant_buffer(cbv);
        self.writer
            .set_descriptors_by_index(descriptor_index, &[DescriptorRef::BufferView(cbv)]);
    }

    pub fn build(self, device_context: &DeviceContext) -> DescriptorSetHandle {
        // descriptor_heap_partition
        //     .write_descriptor_set(self.descriptor_set_layout, &*self.descriptors.borrow())
        //     .unwrap()
        self.writer.flush(device_context)
    }
}

// impl<'renderer, 'frame, T: DescriptorSetLayoutStaticInfo> DescriptorSetWriter
//     for DescriptorSetData<'renderer, 'frame, T>
// {
//     fn descriptor_set_layout(&self) -> &DescriptorSetLayout {
//         self.descriptor_set_layout
//     }

//     fn descriptors(&self) -> &[Descriptor_] {
//         let x = self.descriptors.borrow();

//     }
// }

pub mod Fake {

    use crate::{DescriptorSetData, DescriptorSetLayoutStaticInfo, ToIndex};

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

    pub type FakeDescriptorSetData<'renderer, 'frame> = DescriptorSetData<'renderer, 'frame, Fake>;

    impl DescriptorSetLayoutStaticInfo for Fake {
        type DescriptorID = FakeDescriptorID;

        fn descriptor_set_layout_id() -> u32 {
            0
        }
    }
}
