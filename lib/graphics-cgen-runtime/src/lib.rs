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

use lgn_graphics_api::{
    BufferView, DescriptorDef, DescriptorSetHandle, DescriptorSetLayout, DescriptorSetLayoutDef,
    DeviceContext, Pipeline, PushConstantDef, RootSignature, RootSignatureDef, Sampler,
    ShaderResourceType, TextureView, MAX_DESCRIPTOR_SET_LAYOUTS,
};

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

impl From<glam::Mat4> for Float4x4 {
    fn from(value: glam::Mat4) -> Self {
        Self(value)
    }
}

pub mod prelude {
    pub use crate::Float1;
    pub use crate::Float2;
    pub use crate::Float3;
    pub use crate::Float4;
    pub use crate::Float4x4;
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct CGenTypeId(pub u32);

#[derive(Default, Debug, PartialEq)]
pub struct CGenTypeDef {
    pub id: CGenTypeId,
}

#[derive(Debug, PartialEq)]
pub struct CGenDescriptorDef {
    pub name: &'static str,
    pub shader_resource_type: lgn_graphics_api::ShaderResourceType,
    pub flat_index_start: u32,
    pub flat_index_end: u32,
    pub array_size: u32,
}

pub trait ValueWrapper {
    fn validate(&self, def: &CGenDescriptorDef) -> bool;
}

impl ValueWrapper for &BufferView {
    fn validate(&self, def: &CGenDescriptorDef) -> bool {
        match def.shader_resource_type {
            ShaderResourceType::ConstantBuffer => {
                self.definition().gpu_view_type == lgn_graphics_api::GPUViewType::ConstantBufferView
            }
            ShaderResourceType::ByteAdressBuffer | ShaderResourceType::StructuredBuffer => {
                self.definition().gpu_view_type == lgn_graphics_api::GPUViewType::ShaderResourceView
            }
            ShaderResourceType::RWStructuredBuffer | ShaderResourceType::RWByteAdressBuffer => {
                self.definition().gpu_view_type
                    == lgn_graphics_api::GPUViewType::UnorderedAccessView
            }
            ShaderResourceType::Sampler
            | ShaderResourceType::Texture2D
            | ShaderResourceType::RWTexture2D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::Texture3D
            | ShaderResourceType::RWTexture3D
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => false,
        }
    }
}

impl ValueWrapper for &[&BufferView] {
    fn validate(&self, _def: &CGenDescriptorDef) -> bool {
        false
    }
}

impl ValueWrapper for &Sampler {
    fn validate(&self, _def: &CGenDescriptorDef) -> bool {
        false
    }
}

impl ValueWrapper for &[&Sampler] {
    fn validate(&self, _def: &CGenDescriptorDef) -> bool {
        false
    }
}

impl ValueWrapper for &TextureView {
    fn validate(&self, _def: &CGenDescriptorDef) -> bool {
        false
    }
}

impl ValueWrapper for &[&TextureView] {
    fn validate(&self, _def: &CGenDescriptorDef) -> bool {
        false
    }
}

impl CGenDescriptorDef {
    pub fn validate(&self, wrapper: impl ValueWrapper) -> bool {
        wrapper.validate(self)
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct CGenDescriptorSetDef {
    pub name: &'static str,
    pub id: u32,
    pub frequency: u32,
    pub descriptor_flat_count: u32,
    pub descriptor_defs: &'static [CGenDescriptorDef],
}

impl CGenDescriptorSetDef {
    pub fn create_descriptor_set_layout(
        &self,
        device_context: &DeviceContext,
    ) -> DescriptorSetLayout {
        let mut layout_def = DescriptorSetLayoutDef {
            frequency: self.frequency,
            ..DescriptorSetLayoutDef::default()
        };

        layout_def
            .descriptor_defs
            .reserve_exact(self.descriptor_defs.len());

        for (i, cgen_descriptor_def) in self.descriptor_defs.iter().enumerate() {
            let descriptor_def = DescriptorDef {
                name: cgen_descriptor_def.name.to_string(),
                binding: u32::try_from(i).unwrap(),
                shader_resource_type: cgen_descriptor_def.shader_resource_type,
                array_size: cgen_descriptor_def.array_size,
            };
            layout_def.descriptor_defs.push(descriptor_def);
        }
        device_context
            .create_descriptorset_layout(&layout_def)
            .unwrap()
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct CGenPipelineLayoutDef {
    pub name: &'static str,
    pub id: u32,
    pub descriptor_set_layout_ids: [Option<u32>; MAX_DESCRIPTOR_SET_LAYOUTS],
    pub push_constant_type: Option<CGenTypeId>,
}

impl CGenPipelineLayoutDef {
    pub fn create_pipeline_layout(
        &self,
        device_context: &DeviceContext,
        descriptor_set_layouts: &[&DescriptorSetLayout],
    ) -> RootSignature {
        let signature_def = RootSignatureDef {
            descriptor_set_layouts: self
                .descriptor_set_layout_ids
                .iter()
                .filter_map(|opt_id| opt_id.map(|id| descriptor_set_layouts[id as usize].clone()))
                .collect::<Vec<_>>(),
            push_constant_def: self.push_constant_type.as_ref().map(|_pc| PushConstantDef {
                used_in_shader_stages: todo!(),
                size: todo!(),
            }),
        };

        device_context
            .create_root_signature(&signature_def)
            .unwrap()
    }
}

pub trait CGenDescriptorSetInfo {    
    fn id() -> u32;
}

pub trait CGenPipelineLayoutInfo {
    fn id() -> u32;
}

pub trait PipelineDataProvider {
    fn pipeline(&self) -> &Pipeline;
    fn descriptor_set(&self, frequency: u32) -> Option<DescriptorSetHandle>;
    fn set_descriptor_set(&mut self, frequency: u32, descriptor_set: Option<DescriptorSetHandle>);
}
