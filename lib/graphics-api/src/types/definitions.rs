use std::{
    hash::{Hash, Hasher},
    num::NonZeroU32,
};

use legion_utils::decimal::DecimalF32;
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

use super::{
    AddressMode, BlendFactor, BlendOp, BlendStateTargets, ColorFlags, CompareOp, CullMode,
    Extents3D, FillMode, FilterType, Format, FrontFace, MemoryUsage, MipMapMode, PipelineType,
    PrimitiveTopology, SampleCount, ShaderStageFlags, StencilOp, TextureTiling,
    VertexAttributeRate,
};
use crate::{DescriptorSetLayout, ResourceFlags, RootSignature, Shader, ShaderModule};

#[cfg(feature = "vulkan")]
use crate::backends::vulkan::VkInstance;

/// Controls if an extension is enabled or not. The requirements/behaviors of validation is
/// API-specific.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ExtensionMode {
    /// Do not enable the related extensions
    Disabled,

    /// Enable extensions if available.
    EnabledIfAvailable,

    /// Enable validation, and fail if we cannot enable it or detect that it is not enabled through
    /// external means. (Details on this are API-specific)
    Enabled,
}

/// General configuration that all APIs will make best effort to respect
pub struct ApiDef {
    /// Used as a hint for drivers for what is being run. There are no special requirements for
    /// this. It is not visible to end-users.
    pub app_name: String,

    /// Used to enable/disable validation at runtime. Not all APIs allow this. Validation is helpful
    /// during development but very expensive. Applications should not ship with validation enabled.
    pub validation_mode: ExtensionMode,

    /// Don't enable Window interop extensions
    pub windowing_mode: ExtensionMode,

    /// Api Mode
    pub video_mode: ExtensionMode,
}

impl Default for ApiDef {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let validation_mode = ExtensionMode::EnabledIfAvailable;
        #[cfg(not(debug_assertions))]
        let validation_mode = ExtensionMode::Disabled;

        Self {
            app_name: "Legion Application".to_string(),
            validation_mode,
            windowing_mode: ExtensionMode::Enabled,
            video_mode: ExtensionMode::Disabled,
        }
    }
}

bitflags::bitflags! {
    pub struct ResourceUsage: u16 {
        // buffer
        const AS_CONST_BUFFER = 0x0001;
        // buffer/texture
        const AS_SHADER_RESOURCE = 0x0002;
        // buffer/texture
        const AS_UNORDERED_ACCESS = 0x0004;
        // buffer/texture
        const AS_RENDER_TARGET = 0x0008;
        // texture
        const AS_DEPTH_STENCIL = 0x0010;
        // buffer
        const AS_VERTEX_BUFFER = 0x0020;
        // buffer
        const AS_INDEX_BUFFER = 0x0040;
        // buffer
        const AS_INDIRECT_BUFFER  = 0x0080;
        // texture
        const AS_TRANSFERABLE = 0x0100;
        // meta
        const BUFFER_ONLY_USAGE_FLAGS =
            Self::AS_CONST_BUFFER.bits|
            Self::AS_VERTEX_BUFFER.bits|
            Self::AS_INDEX_BUFFER.bits|
            Self::AS_INDIRECT_BUFFER.bits;
        const TEXTURE_ONLY_USAGE_FLAGS =
            Self::AS_DEPTH_STENCIL.bits|
            Self::AS_TRANSFERABLE.bits;
    }
}

#[cfg(not(any(feature = "vulkan")))]
pub struct Instance {}

#[cfg(feature = "vulkan")]
pub struct Instance<'a> {
    #[cfg(feature = "vulkan")]
    pub(crate) platform_instance: &'a VkInstance,
}

/// Used to create a `Texture`
#[derive(Clone, Copy, Debug)]
pub struct TextureDef {
    pub extents: Extents3D,
    pub array_length: u32,
    pub mip_count: u32,
    pub format: Format,
    pub usage_flags: ResourceUsage,
    pub resource_flags: ResourceFlags,
    pub mem_usage: MemoryUsage,
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
            format: Format::UNDEFINED,
            usage_flags: ResourceUsage::empty(),
            resource_flags: ResourceFlags::empty(),
            mem_usage: MemoryUsage::GpuOnly,
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

        assert!(!self
            .usage_flags
            .intersects(ResourceUsage::BUFFER_ONLY_USAGE_FLAGS));

        if self.resource_flags.contains(ResourceFlags::TEXTURE_CUBE) {
            assert_eq!(self.array_length % 6, 0);
        }

        // vdbdd: I think this validation is wrong
        assert!(
            !(self.format.has_depth()
                && self
                    .usage_flags
                    .intersects(ResourceUsage::AS_UNORDERED_ACCESS)),
            "Cannot use depth stencil as UAV"
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPUViewType {
    ConstantBufferView,
    ShaderResourceView,
    UnorderedAccessView,
    RenderTargetView,
    DepthStencilView,
}

bitflags::bitflags! {
    pub struct BufferViewFlags: u8 {
        const RAW_BUFFER = 0x01;
    }
}

#[derive(Clone, Debug)]
pub struct Descriptor {
    pub(crate) name: String,
    pub(crate) binding: u32,
    pub(crate) shader_resource_type: ShaderResourceType,
    #[cfg(feature = "vulkan")]
    pub(crate) vk_type: ash::vk::DescriptorType,
    pub(crate) element_count: u32,
    pub(crate) update_data_offset: u32,
}

#[derive(Clone, Copy)]
pub struct DescriptorSetHandle {
    #[cfg(feature = "vulkan")]
    pub vk_type: ash::vk::DescriptorSet,
}

#[derive(Clone, Copy, Debug)]
pub enum ViewDimension {
    _2D,
    _2DArray,
    Cube,
    CubeArray,
    _3D,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaneSlice {
    DefaultPlane,
    DepthPlane,
    StencilPlane,
}

#[derive(Clone, Copy, Debug)]
pub struct TextureViewDef {
    pub gpu_view_type: GPUViewType,
    pub view_dimension: ViewDimension,
    pub first_mip: u32,
    pub mip_count: u32,
    pub plane_slice: PlaneSlice,
    pub first_array_slice: u32,
    pub array_size: u32,
}

impl TextureViewDef {
    pub fn as_shader_resource_view(texture_def: &TextureDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::ShaderResourceView,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: texture_def.mip_count,
            plane_slice: PlaneSlice::DefaultPlane,
            first_array_slice: 0,
            array_size: texture_def.array_length,
        }
    }

    pub fn as_render_target_view(_texture: &TextureDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::RenderTargetView,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::DefaultPlane,
            first_array_slice: 0,
            array_size: 1,
        }
    }

    pub fn as_depth_stencil_view(_texture: &TextureDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::DepthStencilView,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::DefaultPlane,
            first_array_slice: 0,
            array_size: 1,
        }
    }

    pub fn verify(&self, texture_def: &TextureDef) {
        match self.view_dimension {
            ViewDimension::_2D | ViewDimension::_2DArray => {
                assert!(texture_def.is_2d() || texture_def.is_3d());
            }
            ViewDimension::Cube | ViewDimension::CubeArray => {
                assert!(texture_def.is_cube());
            }
            ViewDimension::_3D => {
                assert!(texture_def.is_3d());
            }
        }

        match self.gpu_view_type {
            GPUViewType::ShaderResourceView => {
                assert!(texture_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_SHADER_RESOURCE));

                match self.view_dimension {
                    ViewDimension::_2D => {
                        assert!(self.first_array_slice == 0);
                        assert!(self.array_size == 1);
                    }
                    ViewDimension::_2DArray => {}
                    ViewDimension::_3D | ViewDimension::Cube => {
                        assert!(self.plane_slice == PlaneSlice::DefaultPlane);
                        assert!(self.first_array_slice == 0);
                        assert!(self.array_size == 1);
                    }
                    ViewDimension::CubeArray => {
                        assert!(self.plane_slice == PlaneSlice::DefaultPlane);
                    }
                }
            }
            GPUViewType::UnorderedAccessView => {
                assert!(texture_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_UNORDERED_ACCESS));

                assert!(self.mip_count == 1);

                match self.view_dimension {
                    ViewDimension::_2D => {
                        assert!(self.first_array_slice == 0);
                        assert!(self.array_size == 1);
                    }
                    ViewDimension::_2DArray => {}
                    ViewDimension::_3D => {
                        assert!(self.plane_slice == PlaneSlice::DefaultPlane);
                    }
                    ViewDimension::Cube | ViewDimension::CubeArray => {
                        panic!();
                    }
                }
            }
            GPUViewType::RenderTargetView => {
                assert!(texture_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_RENDER_TARGET));

                assert!(self.mip_count == 1);

                match self.view_dimension {
                    ViewDimension::_2D => {
                        assert!(self.first_array_slice == 0);
                        assert!(self.array_size == 1);
                    }
                    ViewDimension::_2DArray => {}
                    ViewDimension::_3D => {
                        assert!(self.plane_slice == PlaneSlice::DefaultPlane);
                    }
                    ViewDimension::Cube | ViewDimension::CubeArray => {
                        panic!();
                    }
                }
            }
            GPUViewType::DepthStencilView => {
                assert!(texture_def
                    .usage_flags
                    .intersects(ResourceUsage::AS_DEPTH_STENCIL));

                assert!(self.mip_count == 1);

                match self.view_dimension {
                    ViewDimension::_2D => {
                        assert!(self.first_array_slice == 0);
                        assert!(self.array_size == 1);
                    }
                    ViewDimension::_2DArray => {}
                    ViewDimension::_3D | ViewDimension::Cube | ViewDimension::CubeArray => {
                        panic!();
                    }
                }
            }
            GPUViewType::ConstantBufferView => {
                panic!();
            }
        }

        let last_mip = self.first_mip + self.mip_count;
        assert!(last_mip <= texture_def.mip_count);

        let last_array_slice = self.first_array_slice + self.array_size;

        let max_array_len = if texture_def.is_2d() {
            texture_def.array_length
        } else {
            texture_def.extents.depth
        };
        assert!(last_array_slice <= max_array_len);
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
#[derive(Clone)]
pub struct SwapchainDef {
    pub width: u32,
    pub height: u32,
    pub enable_vsync: bool,
    // image count?
}

/// Describes a single stage within a shader
#[derive(Clone)]
pub struct ShaderStageDef {
    pub entry_point: String,
    pub shader_stage: ShaderStageFlags,
    pub shader_module: ShaderModule,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShaderResourceType {
    Sampler = 0x00_01,
    ConstantBuffer = 0x00_02,
    StructuredBuffer = 0x00_04,
    RWStructuredBuffer = 0x00_08,
    ByteAdressBuffer = 0x00_10,
    RWByteAdressBuffer = 0x00_20,
    Texture2D = 0x00_40,
    RWTexture2D = 0x00_80,
    Texture2DArray = 0x01_00,
    RWTexture2DArray = 0x02_00,
    Texture3D = 0x04_00,
    RWTexture3D = 0x08_00,
    TextureCube = 0x10_00,
    TextureCubeArray = 0x20_00,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Copy)]
pub struct PushConstantDef {
    pub used_in_shader_stages: ShaderStageFlags,
    pub size: NonZeroU32,
}

pub struct RootSignatureDef {
    pub pipeline_type: PipelineType,
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub push_constant_def: Option<PushConstantDef>,
}

impl Clone for RootSignatureDef {
    fn clone(&self) -> Self {
        Self {
            pipeline_type: self.pipeline_type,
            descriptor_set_layouts: self.descriptor_set_layouts.clone(),
            push_constant_def: self.push_constant_def,
        }
    }
}

impl Default for RootSignatureDef {
    fn default() -> Self {
        Self {
            pipeline_type: PipelineType::Graphics,
            descriptor_set_layouts: Vec::new(),
            push_constant_def: None,
        }
    }
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
#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
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
            front_depth_fail_op: StencilOp::default(),
            front_stencil_compare_op: CompareOp::Always,
            front_stencil_fail_op: StencilOp::default(),
            front_stencil_pass_op: StencilOp::default(),
            back_depth_fail_op: StencilOp::default(),
            back_stencil_compare_op: CompareOp::Always,
            back_stencil_fail_op: StencilOp::default(),
            back_stencil_pass_op: StencilOp::default(),
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
    pub depth_bias: f32,
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
            && DecimalF32(self.depth_bias) == DecimalF32(other.depth_bias)
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
        DecimalF32(self.depth_bias).hash(&mut state);
        DecimalF32(self.depth_bias_slope_scaled).hash(&mut state);
        self.depth_clamp_enable.hash(&mut state);
        self.multisample.hash(&mut state);
        self.scissor.hash(&mut state);
    }
}

impl Default for RasterizerState {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::default(),
            front_face: FrontFace::default(),
            fill_mode: FillMode::default(),
            depth_bias: 0.0,
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
        Self::default()
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
        if self.independent_blend {
            assert_eq!(self.render_target_blend_states.len(), color_attachment_count,
                "If BlendState::independent_blend is true, BlendState::render_target_blend_states length must match color attachment count");
        } else {
            assert_eq!(self.render_target_blend_states.len(), 1,
                "If BlendState::independent_blend is false, BlendState::render_target_blend_states must be 1");
        }
    }
}

/// Used to create a `Pipeline` for graphics operations
pub struct GraphicsPipelineDef<'a> {
    pub shader: &'a Shader,
    pub root_signature: &'a RootSignature,
    pub vertex_layout: &'a VertexLayout,
    pub blend_state: &'a BlendState,
    pub depth_state: &'a DepthState,
    pub rasterizer_state: &'a RasterizerState,
    pub primitive_topology: PrimitiveTopology,
    pub color_formats: &'a [Format],
    pub depth_stencil_format: Option<Format>,
    pub sample_count: SampleCount,
}

/// Used to create a `Pipeline` for compute operations
pub struct ComputePipelineDef<'a> {
    pub shader: &'a Shader,
    pub root_signature: &'a RootSignature,
}

/// Used to create a `DescriptorHeap`
#[derive(Default, Clone, Copy)]
pub struct DescriptorHeapDef {
    pub transient: bool,
    pub max_descriptor_sets: u32,
    pub sampler_count: u32,
    pub constant_buffer_count: u32,
    pub buffer_count: u32,
    pub rw_buffer_count: u32,
    pub texture_count: u32,
    pub rw_texture_count: u32,
}

impl DescriptorHeapDef {
    pub fn from_descriptor_set_layout_def(
        definition: &DescriptorSetLayoutDef,
        transient: bool,
        max_descriptor_sets: u32,
    ) -> Self {
        let mut result = Self {
            transient,
            max_descriptor_sets,
            ..Self::default()
        };

        for descriptor_def in &definition.descriptor_defs {
            let count = max_descriptor_sets * descriptor_def.array_size_normalized();
            match descriptor_def.shader_resource_type {
                ShaderResourceType::Sampler => result.sampler_count += count,
                ShaderResourceType::ConstantBuffer => result.constant_buffer_count += count,
                ShaderResourceType::StructuredBuffer | ShaderResourceType::ByteAdressBuffer => {
                    result.buffer_count += count;
                }
                ShaderResourceType::RWStructuredBuffer | ShaderResourceType::RWByteAdressBuffer => {
                    result.rw_buffer_count += count;
                }
                ShaderResourceType::Texture2D
                | ShaderResourceType::Texture2DArray
                | ShaderResourceType::Texture3D
                | ShaderResourceType::TextureCube => result.texture_count += count,
                ShaderResourceType::RWTexture2D
                | ShaderResourceType::RWTexture2DArray
                | ShaderResourceType::RWTexture3D
                | ShaderResourceType::TextureCubeArray => result.rw_texture_count += count,
            }
        }

        result
    }
}
