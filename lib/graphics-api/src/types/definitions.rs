use super::*;

use crate::{Buffer, GfxApi, MAX_DESCRIPTOR_SET_LAYOUTS, ResourceFlags, Texture};
use legion_utils::decimal::DecimalF32;
use std::{
    hash::{Hash, Hasher},
    num::{NonZeroU32},
};

#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

/// General configuration that all APIs will make best effort to respect
#[derive(Default)]
pub struct ApiDef {
    // Don't have anything that's universal across APIs to add here yet
}

bitflags::bitflags! {
    pub struct ResourceUsage: u16 {        
        // buffer
        const HAS_CONST_BUFFER_VIEW = 0x0001;
        // buffer/texture
        const HAS_SHADER_RESOURCE_VIEW = 0x0002;
        // buffer/texture
        const HAS_UNORDERED_ACCESS_VIEW = 0x0004;
        // buffer/texture
        const HAS_RENDER_TARGET_VIEW = 0x0008;
        // texture
        const HAS_DEPTH_STENCIL_VIEW = 0x0010;        
        // buffer
        const HAS_VERTEX_BUFFER = 0x0020;
        // buffer
        const HAS_INDEX_BUFFER = 0x0040;
        // buffer
        const HAS_INDIRECT_BUFFER  = 0x0080;
        // meta
        const BUFFER_ONLY_USAGE_FLAGS = 
            Self::HAS_CONST_BUFFER_VIEW.bits|
            Self::HAS_VERTEX_BUFFER.bits|
            Self::HAS_INDEX_BUFFER.bits|
            Self::HAS_INDIRECT_BUFFER.bits;
        const TEXTURE_ONLY_USAGE_FLAGS = 
            Self::HAS_DEPTH_STENCIL_VIEW.bits;
    }
}

#[derive(Clone, Debug, Default)]
pub struct BufferElementData {
    // For storage buffers
    pub element_begin_index: u64,
    pub element_count: u64,
    pub element_stride: u64,
}

/// Used to create a `Buffer`
#[derive(Clone, Debug)]
pub struct BufferDef {
    pub size: u64,
    pub memory_usage: MemoryUsage,
    pub queue_type: QueueType,
    pub always_mapped: bool,
    pub usage_flags: ResourceUsage,
}

impl Default for BufferDef {
    fn default() -> Self {
        Self {
            size: 0,
            memory_usage: MemoryUsage::Unknown,
            queue_type: QueueType::Graphics,
            always_mapped: false,
            usage_flags: ResourceUsage::empty(),
        }
    }
}

impl BufferDef {
    pub fn verify(&self) {
        assert_ne!(self.size, 0);
        assert!(!self.usage_flags.intersects(ResourceUsage::TEXTURE_ONLY_USAGE_FLAGS));

    }

    pub fn for_staging_buffer(size: usize, usage_flags: ResourceUsage) -> Self {
        Self {
            size: size as u64,
            memory_usage: MemoryUsage::CpuToGpu,
            queue_type: QueueType::Graphics,
            always_mapped: false,
            usage_flags,
        }
    }

    pub fn for_staging_buffer_data<T: Copy>(data: &[T], usage_flags: ResourceUsage) -> Self {
        Self::for_staging_buffer(legion_utils::memory::slice_size_in_bytes(data), usage_flags)
    }

    pub fn for_staging_vertex_buffer(size: usize) -> Self {
        Self::for_staging_buffer(size, ResourceUsage::HAS_VERTEX_BUFFER)
    }

    pub fn for_staging_vertex_buffer_data<T: Copy>(data: &[T]) -> Self {
        Self::for_staging_buffer_data(data, ResourceUsage::HAS_VERTEX_BUFFER)
    }

    pub fn for_staging_index_buffer(size: usize) -> Self {
        Self::for_staging_buffer(size, ResourceUsage::HAS_INDEX_BUFFER)
    }

    pub fn for_staging_index_buffer_data<T: Copy>(data: &[T]) -> Self {
        Self::for_staging_buffer_data(data, ResourceUsage::HAS_INDEX_BUFFER)
    }

    pub fn for_staging_uniform_buffer(size: usize) -> Self {
        Self::for_staging_buffer(size, ResourceUsage::HAS_CONST_BUFFER_VIEW)
    }

    pub fn for_staging_uniform_buffer_data<T: Copy>(data: &[T]) -> Self {
        Self::for_staging_buffer_data(data, ResourceUsage::HAS_CONST_BUFFER_VIEW)
    }
}

// /// Determines how many dimensions the texture will have.
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub enum TextureDimensions {
//     /// Assume 2D if depth = 1, otherwise 3d
//     // Auto,
//     // Dim1D,
//     Dim2D,
//     Dim3D,
// }

// impl TextureDimensions {
//     pub fn determine_dimensions(self, extents: Extents3D) -> Self {
//         match self {
//             Self::Dim2D => {
//                 assert_eq!(extents.depth, 1);
//                 Self::Dim2D
//             }
//             Self::Dim3D => Self::Dim3D,
//         }
//     }
// }

/// Used to create a `Texture`
#[derive(Clone, Debug)]
pub struct TextureDef {
    pub extents: Extents3D,
    pub array_length: u32,
    pub mip_count: u32,    
    pub format: Format,
    pub usage_flags: ResourceUsage,
    pub resource_flags: ResourceFlags,    
    pub mem_usage: MemoryUsage,
    // pub dimensions: TextureDimensions,
    pub tiling: TextureTiling,
}

impl Default for TextureDef {
    fn default() -> Self {
        Self {
            extents: Extents3D {
                width: 0,
                height: 0,
                depth: 0,
            },
            array_length: 1,
            mip_count: 1,
            // sample_count: SampleCount::SampleCount1,
            format: Format::UNDEFINED,
            usage_flags: ResourceUsage::empty(),
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
            // dimensions: TextureDimensions::Dim2D,
            tiling: TextureTiling::Optimal,
        }
    }
}

impl TextureDef {
    pub fn is_2d(&self) -> bool {
        self.extents.depth == 1
    }

    pub fn is_3d(&self) -> bool {
        self.extents.depth > 1
    }

    pub fn is_cube(&self) -> bool {
        self.resource_flags.contains(ResourceFlags::TEXTURE_CUBE)
    }

    pub fn verify(&self) {
        assert!(self.extents.width > 0);
        assert!(self.extents.height > 0);
        assert!(self.extents.depth > 0);
        assert!(self.array_length > 0);
        assert!(self.mip_count > 0);        

        assert!(!self.usage_flags.intersects(ResourceUsage::BUFFER_ONLY_USAGE_FLAGS));

        if self.resource_flags.contains(ResourceFlags::TEXTURE_CUBE) {
            assert_eq!(self.array_length % 6, 0);
        }

        // // we support only one or the other
        // assert!(
        //     !(self.resource_type.contains(
        //         ResourceType::RENDER_TARGET_ARRAY_SLICES | ResourceType::RENDER_TARGET_DEPTH_SLICES
        //     ))
        // );

        // vdbdd: I think this validation is wrong
        assert!(
            !(self.format.has_depth()
                && self.usage_flags.intersects(ResourceUsage::HAS_UNORDERED_ACCESS_VIEW)),
            "Cannot use depth stencil as UAV"
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BufferViewType {
    ConstantBufferView,
    ShaderResourceView,
    UnorderedAccessView,
}

#[derive(Clone, Copy, Debug)]
pub struct BufferViewDef {
    pub buffer_view_type: BufferViewType,
    pub offset: u64,
    pub size: u64,
}

impl BufferViewDef {
    pub(crate) fn verify<A: GfxApi>(&self, buffer: &A::Buffer) {
        
        assert_ne!(self.size, 0);
        
        let buffer_def = buffer.buffer_def();
        let upper_bound = self.offset + self.size;        
        assert!(upper_bound <= buffer_def.size);

        match self.buffer_view_type {
            BufferViewType::ConstantBufferView => {
                assert!( buffer_def.usage_flags.intersects( ResourceUsage::HAS_CONST_BUFFER_VIEW ));
            }
            BufferViewType::ShaderResourceView => {
                assert!( buffer_def.usage_flags.intersects( ResourceUsage::HAS_SHADER_RESOURCE_VIEW ));
            }
            BufferViewType::UnorderedAccessView => {
                assert!( buffer_def.usage_flags.intersects( ResourceUsage::HAS_UNORDERED_ACCESS_VIEW ));
            }
        }

    }
}

#[derive(Clone, Copy, Debug)]
pub enum TextureViewType {
    ShaderResourceView,
    UnorderedAccessView,
    RenderTargetView,
    DepthStencilView
}


#[derive(Clone, Copy, Debug)]
pub enum ViewType {
    ViewType2d,
    ViewType2darray,
    ViewTypeCube,
    ViewTypeCubeArray,
    ViewType3d,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureViewDef {
    pub texture_view_type: TextureViewType,    
    pub view_type : ViewType,
    pub first_mip : u32,
    pub mip_count : u32,
    pub first_slice : u32,    
    pub slice_count : u32,    
}

impl TextureViewDef {
    pub fn verify<A: GfxApi> (&self, texture: &A::Texture) {
        
        let texture_def = texture.texture_def();

        let last_mip = self.first_mip + self.mip_count;
        assert!( last_mip <= texture_def.mip_count );

        let last_slice = self.first_slice + self.slice_count;
        assert!( last_slice <= texture_def.array_length );

        match self.texture_view_type {
            TextureViewType::ShaderResourceView => {
                assert!( texture_def.usage_flags.intersects(ResourceUsage::HAS_SHADER_RESOURCE_VIEW) );
            }
            TextureViewType::UnorderedAccessView => {
                assert!( texture_def.usage_flags.intersects(ResourceUsage::HAS_UNORDERED_ACCESS_VIEW) );
            }
            TextureViewType::RenderTargetView => {
                assert!( texture_def.usage_flags.intersects(ResourceUsage::HAS_RENDER_TARGET_VIEW) );
                assert!( self.mip_count == 1 );
                assert!( self.slice_count == 1 );
            }
            TextureViewType::DepthStencilView => {
                assert!( texture_def.usage_flags.intersects(ResourceUsage::HAS_DEPTH_STENCIL_VIEW) );
                assert!( self.mip_count == 1 );
                assert!( self.slice_count == 1 );
            }
        }

        match self.view_type {
            ViewType::ViewType2d => {
                assert!( texture_def.is_2d() || texture_def.is_3d() );
            }
            ViewType::ViewType2darray => {
                assert!( texture_def.is_2d() || texture_def.is_3d() );
            }
            ViewType::ViewTypeCube => {
                assert!( texture_def.is_cube() );
            }
            ViewType::ViewTypeCubeArray => {
                assert!( texture_def.is_cube() );
            }
            ViewType::ViewType3d => {
                assert!( texture_def.is_3d() );
            }
        }



    }
}

/// Used to create a `CommandPool`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommandPoolDef {
    /// Set to true if the command buffers allocated from the pool are expected to have very short
    /// lifetimes
    pub transient: bool,
}

/// Used to create a `CommandBuffer`
#[derive(Debug, Clone, PartialEq)]
pub struct CommandBufferDef {
    /// Secondary command buffers are used to encode a single pass on multiple threads
    pub is_secondary: bool,
}

/// Used to create a `Swapchain`
#[derive(Clone, Debug)]
pub struct SwapchainDef {
    pub width: u32,
    pub height: u32,
    pub enable_vsync: bool,
    // image count?
}

/// Describes a single stage within a shader
#[derive(Clone, Debug)]
pub struct ShaderStageDef<A: GfxApi> {
    pub shader_module: A::ShaderModule,
    pub reflection: ShaderStageReflection,
}

impl<A: GfxApi> ShaderStageDef<A> {
    pub fn hash_definition<HasherT: std::hash::Hasher, ShaderModuleHashT: Hash>(
        hasher: &mut HasherT,
        reflection_data: &[&ShaderStageReflection],
        shader_module_hashes: &[ShaderModuleHashT],
    ) {
        assert_eq!(reflection_data.len(), shader_module_hashes.len());
        fn hash_stage<HasherT: std::hash::Hasher, ShaderModuleHashT: Hash>(
            hasher: &mut HasherT,
            stage_flag: ShaderStageFlags,
            reflection_data: &[&ShaderStageReflection],
            shader_module_hashes: &[ShaderModuleHashT],
        ) {
            for (reflection, shader_module_hash) in reflection_data.iter().zip(shader_module_hashes)
            {
                if reflection.shader_stage.intersects(stage_flag) {
                    reflection.shader_stage.hash(hasher);
                    reflection.entry_point_name.hash(hasher);
                    reflection.shader_resources.hash(hasher);
                    shader_module_hash.hash(hasher);
                    break;
                }
            }
        }

        // Hash stages in a deterministic order
        for stage_flag in &super::ALL_SHADER_STAGE_FLAGS {
            hash_stage(hasher, *stage_flag, reflection_data, shader_module_hashes);
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShaderResourceType {
    Undefined,
    Sampler,
    ConstantBuffer,
    StructuredBuffer,
    RWStructuredBuffer,
    ByteAdressBuffer,
    RWByteAdressBuffer,
    Texture2D,
    RWTexture2D,
    Texture2DArray,
    RWTexture2DArray,
    Texture3D,
    RWTexture3D,
    TextureCube,
    TextureCubeArray,
}

impl Default for ShaderResourceType {
    fn default() -> Self {
        ShaderResourceType::Undefined
    }
}

pub struct DescriptorDef {
    pub name: String,
    pub binding: u32,
    pub shader_resource_type: ShaderResourceType,
    pub array_size: u32,
}

impl DescriptorDef {
    pub fn array_size_normalized(&self) -> u32 {
        self.array_size.max(1u32)
    }
}

pub struct DescriptorSetLayoutDef {
    pub frequency: u32,
    pub descriptor_defs: Vec<DescriptorDef>,
}

impl DescriptorSetLayoutDef {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            descriptor_defs: Vec::new(),
        }
    }
}

impl Default for DescriptorSetLayoutDef {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PushConstantDef {
    pub used_in_shader_stages: ShaderStageFlags,
    pub size: NonZeroU32,
}

pub struct RootSignatureDef<A: GfxApi> {
    pub pipeline_type: PipelineType,
    pub descriptor_set_layouts: [Option<A::DescriptorSetLayout>; MAX_DESCRIPTOR_SET_LAYOUTS],
    pub push_constant_def: Option<PushConstantDef>,
}

/// Used to create a `Sampler`
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct SamplerDef {
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub min_filter: FilterType,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub mag_filter: FilterType,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub mip_map_mode: MipMapMode,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub address_mode_u: AddressMode,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub address_mode_v: AddressMode,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub address_mode_w: AddressMode,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub mip_lod_bias: f32,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub max_anisotropy: f32,
    #[cfg_attr(feature = "serde-support", serde(default))]
    pub compare_op: CompareOp,
    //NOTE: Custom hash impl, don't forget to add changes there too!
}

impl Eq for SamplerDef {}
impl PartialEq for SamplerDef {
    fn eq(&self, other: &Self) -> bool {
        self.min_filter == other.min_filter
            && self.mag_filter == other.mag_filter
            && self.mip_map_mode == other.mip_map_mode
            && self.address_mode_u == other.address_mode_u
            && self.address_mode_v == other.address_mode_v
            && self.address_mode_w == other.address_mode_w
            && DecimalF32(self.mip_lod_bias) == DecimalF32(other.mip_lod_bias)
            && DecimalF32(self.max_anisotropy) == DecimalF32(other.max_anisotropy)
            && self.compare_op == other.compare_op
    }
}

impl Hash for SamplerDef {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.min_filter.hash(&mut state);
        self.mag_filter.hash(&mut state);
        self.mip_map_mode.hash(&mut state);
        self.address_mode_u.hash(&mut state);
        self.address_mode_v.hash(&mut state);
        self.address_mode_w.hash(&mut state);
        DecimalF32(self.mip_lod_bias).hash(&mut state);
        DecimalF32(self.max_anisotropy).hash(&mut state);
        self.compare_op.hash(&mut state);
    }
}

/// Describes an attribute within a `VertexLayout`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexLayoutAttribute {
    /// Format of the attribute
    pub format: Format,
    /// Which buffer the attribute is contained in
    pub buffer_index: u32,
    /// Affects what input variable within the shader the attribute is assigned
    pub location: u32,
    /// The byte offset of the attribute within the buffer
    pub byte_offset: u32,

    /// name of the attribute in the shader, only required for GL
    pub gl_attribute_name: Option<String>,
}

/// Describes a buffer that provides vertex attribute data (See `VertexLayout`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexLayoutBuffer {
    pub stride: u32,
    pub rate: VertexAttributeRate,
}

/// Describes how vertex attributes are laid out within one or more buffers
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VertexLayout {
    pub attributes: Vec<VertexLayoutAttribute>,
    pub buffers: Vec<VertexLayoutBuffer>,
}

/// Affects depth testing and stencil usage. Commonly used to enable "Z-buffering".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct DepthState {
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub depth_compare_op: CompareOp,
    pub stencil_test_enable: bool,
    pub stencil_read_mask: u8,
    pub stencil_write_mask: u8,
    pub front_depth_fail_op: StencilOp,
    pub front_stencil_compare_op: CompareOp,
    pub front_stencil_fail_op: StencilOp,
    pub front_stencil_pass_op: StencilOp,
    pub back_depth_fail_op: StencilOp,
    pub back_stencil_compare_op: CompareOp,
    pub back_stencil_fail_op: StencilOp,
    pub back_stencil_pass_op: StencilOp,
}

impl Default for DepthState {
    fn default() -> Self {
        Self {
            depth_test_enable: false,
            depth_write_enable: false,
            depth_compare_op: CompareOp::LessOrEqual,
            stencil_test_enable: false,
            stencil_read_mask: 0xFF,
            stencil_write_mask: 0xFF,
            front_depth_fail_op: Default::default(),
            front_stencil_compare_op: CompareOp::Always,
            front_stencil_fail_op: Default::default(),
            front_stencil_pass_op: Default::default(),
            back_depth_fail_op: Default::default(),
            back_stencil_compare_op: CompareOp::Always,
            back_stencil_fail_op: Default::default(),
            back_stencil_pass_op: Default::default(),
        }
    }
}

/// Affects rasterization, commonly used to enable backface culling or wireframe rendering
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct RasterizerState {
    pub cull_mode: CullMode,
    pub front_face: FrontFace,
    pub fill_mode: FillMode,
    pub depth_bias: i32,
    pub depth_bias_slope_scaled: f32,
    pub depth_clamp_enable: bool,
    pub multisample: bool,
    pub scissor: bool,
    // Hash implemented manually below, don't forget to update it!
}

impl Eq for RasterizerState {}

impl PartialEq for RasterizerState {
    fn eq(&self, other: &Self) -> bool {
        self.cull_mode == other.cull_mode
            && self.front_face == other.front_face
            && self.fill_mode == other.fill_mode
            && self.depth_bias == other.depth_bias
            && DecimalF32(self.depth_bias_slope_scaled) == DecimalF32(other.depth_bias_slope_scaled)
            && self.depth_clamp_enable == other.depth_clamp_enable
            && self.multisample == other.multisample
            && self.scissor == other.scissor
    }
}

impl Hash for RasterizerState {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        self.cull_mode.hash(&mut state);
        self.front_face.hash(&mut state);
        self.fill_mode.hash(&mut state);
        self.depth_bias.hash(&mut state);
        DecimalF32(self.depth_bias_slope_scaled).hash(&mut state);
        self.depth_clamp_enable.hash(&mut state);
        self.multisample.hash(&mut state);
        self.scissor.hash(&mut state);
    }
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::None,
            front_face: Default::default(),
            fill_mode: Default::default(),
            depth_bias: 0,
            depth_bias_slope_scaled: 0.0,
            depth_clamp_enable: false,
            multisample: false,
            scissor: false,
        }
    }
}

/// Configures blend state for a particular render target
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct BlendStateRenderTarget {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub src_factor_alpha: BlendFactor,
    pub dst_factor_alpha: BlendFactor,
    pub blend_op: BlendOp,
    pub blend_op_alpha: BlendOp,
    pub masks: ColorFlags,
}

impl Default for BlendStateRenderTarget {
    fn default() -> Self {
        Self {
            blend_op: BlendOp::Add,
            blend_op_alpha: BlendOp::Add,
            src_factor: BlendFactor::One,
            src_factor_alpha: BlendFactor::One,
            dst_factor: BlendFactor::Zero,
            dst_factor_alpha: BlendFactor::Zero,
            masks: ColorFlags::ALL,
        }
    }
}

impl BlendStateRenderTarget {
    pub fn default_alpha_disabled() -> Self {
        Default::default()
    }

    pub fn default_alpha_enabled() -> Self {
        Self {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            src_factor_alpha: BlendFactor::One,
            dst_factor_alpha: BlendFactor::Zero,
            blend_op: BlendOp::Add,
            blend_op_alpha: BlendOp::Add,
            masks: ColorFlags::ALL,
        }
    }
}

impl BlendStateRenderTarget {
    pub fn blend_enabled(&self) -> bool {
        self.src_factor != BlendFactor::One
            || self.src_factor_alpha != BlendFactor::One
            || self.dst_factor != BlendFactor::Zero
            || self.dst_factor_alpha != BlendFactor::Zero
    }
}

/// Affects the way the result of a pixel shader is blended with a value it will overwrite. Commonly
/// used to enable "alpha-blending".
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct BlendState {
    /// Individual blend states for blend targets
    pub render_target_blend_states: Vec<BlendStateRenderTarget>,

    /// Indicates which blend targets to affect. Blend targets with unset bits are left in default
    /// state.
    pub render_target_mask: BlendStateTargets,

    /// If false, `render_target_blend_states[0]` will apply to all render targets indicated by
    /// `render_target_mask`. If true, we index into `render_target_blend_states` based on the
    /// render target's index.
    pub independent_blend: bool,
}

impl BlendState {
    pub fn default_alpha_disabled() -> Self {
        Self {
            render_target_blend_states: vec![BlendStateRenderTarget::default_alpha_disabled()],
            render_target_mask: BlendStateTargets::BLEND_STATE_TARGET_ALL,
            independent_blend: false,
        }
    }

    pub fn default_alpha_enabled() -> Self {
        Self {
            render_target_blend_states: vec![BlendStateRenderTarget::default_alpha_enabled()],
            render_target_mask: BlendStateTargets::BLEND_STATE_TARGET_ALL,
            independent_blend: false,
        }
    }
}

impl Default for BlendState {
    fn default() -> Self {
        Self::default_alpha_disabled()
    }
}

impl BlendState {
    pub fn verify(&self, color_attachment_count: usize) {
        if !self.independent_blend {
            assert_eq!(self.render_target_blend_states.len(), 1, "If BlendState::independent_blend is false, BlendState::render_target_blend_states must be 1");
        } else {
            assert_eq!(self.render_target_blend_states.len(), color_attachment_count, "If BlendState::independent_blend is true, BlendState::render_target_blend_states length must match color attachment count");
        }
    }
}

/// Used to create a `Pipeline` for graphics operations
#[derive(Debug)]
pub struct GraphicsPipelineDef<'a, A: GfxApi> {
    pub shader: &'a A::Shader,
    pub root_signature: &'a A::RootSignature,
    pub vertex_layout: &'a VertexLayout,
    pub blend_state: &'a BlendState,
    pub depth_state: &'a DepthState,
    pub rasterizer_state: &'a RasterizerState,
    pub primitive_topology: PrimitiveTopology,
    pub color_formats: &'a [Format],
    pub depth_stencil_format: Option<Format>,
    pub sample_count: SampleCount,
    //indirect_commands_enable: bool
}

/// Used to create a `Pipeline` for compute operations
#[derive(Debug)]
pub struct ComputePipelineDef<'a, A: GfxApi> {
    pub shader: &'a A::Shader,
    pub root_signature: &'a A::RootSignature,
}

/// Used to create a `DescriptorSetArray`
pub struct DescriptorSetArrayDef<'a, A: GfxApi> {
    pub descriptor_set_layout: &'a A::DescriptorSetLayout,
    /// The number of descriptor sets in the array
    pub array_length: usize,
}
