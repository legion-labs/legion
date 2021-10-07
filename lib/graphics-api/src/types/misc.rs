#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

use crate::GfxApi;
use legion_utils::decimal::DecimalF32;
use std::hash::{Hash, Hasher};

/// Controls if validation is enabled or not. The requirements/behaviors of validation is
/// API-specific.
#[derive(Copy, Clone, Debug)]
pub enum ValidationMode {
    /// Do not enable validation. Even if validation is turned on through external means, do not
    /// intentionally fail initialization
    Disabled,

    /// Enable validation if possible. (Details on requirements to enable at runtime are
    /// API-specific)
    EnabledIfAvailable,

    /// Enable validation, and fail if we cannot enable it or detect that it is not enabled through
    /// external means. (Details on this are API-specific)
    Enabled,
}

impl Default for ValidationMode {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let validation_mode = Self::EnabledIfAvailable;
        #[cfg(not(debug_assertions))]
        let validation_mode = ValidationMode::Disabled;

        validation_mode
    }
}

/// Information about the device, mostly limits, requirements (like memory alignment), and flags to
/// indicate whether certain features are supported
pub struct DeviceInfo {
    pub supports_multithreaded_usage: bool,

    pub min_uniform_buffer_offset_alignment: u32,
    pub min_storage_buffer_offset_alignment: u32,
    pub upload_buffer_texture_alignment: u32,
    pub upload_buffer_texture_row_alignment: u32,

    // Requires iOS 14.0, macOS 10.12
    pub supports_clamp_to_border_color: bool,

    pub max_vertex_attribute_count: u32,
    //max_vertex_input_binding_count: u32,
    // max_root_signature_dwords: u32,
    // wave_lane_count: u32,
    // wave_ops_support_flags: u32,
    // gpu_vendor_preset: u32,
    // metal_argument_buffer_max_textures: u32,
    // metal_heaps: u32,
    // metal_placement_heaps: u32,
    // metal_draw_index_vertex_offset_supported: bool,
}

/// Used to indicate which type of queue to use. Some operations require certain types of queues.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum QueueType {
    /// Graphics queues generally supports all operations and are a safe default choice
    Graphics,

    /// Compute queues can be used for compute-based work.
    Compute,

    /// Transfer queues are generally limited to basic operations like copying data from buffers
    /// to images.
    Transfer,
}

/// The color space an image data is in. The correct color space often varies between texture types
/// (like normal maps vs. albedo maps).
#[derive(Copy, Clone, Debug)]
pub enum ColorType {
    Linear,
    Srgb,
}

// /// Texture will allocate its own memory (COMMITTED resource)
// TEXTURE_CREATION_FLAG_OWN_MEMORY_BIT = 0x01,
// /// Use on-tile memory to store this texture
// TEXTURE_CREATION_FLAG_ON_TILE = 0x20,
// /// Force 2D instead of automatically determining dimension based on width, height, depth
// TEXTURE_CREATION_FLAG_FORCE_2D = 0x80,
// /// Force 3D instead of automatically determining dimension based on width, height, depth
// TEXTURE_CREATION_FLAG_FORCE_3D = 0x100,
// /// Display target
// TEXTURE_CREATION_FLAG_ALLOW_DISPLAY_TARGET = 0x200,
// /// Create an sRGB texture.
// TEXTURE_CREATION_FLAG_SRGB = 0x400,

bitflags::bitflags! {
    /// The current state of a resource. When an operation is performed that references a resource,
    /// it must be in the correct state. Resources are moved between state using barriers.
    pub struct ResourceState: u32 {
        const UNDEFINED = 0;
        const VERTEX_AND_CONSTANT_BUFFER = 0x1;
        const INDEX_BUFFER = 0x2;
        /// Similar to vulkan's COLOR_ATTACHMENT_OPTIMAL image layout
        const RENDER_TARGET = 0x4;
        const UNORDERED_ACCESS = 0x8;
        /// Similar to vulkan's DEPTH_STENCIL_ATTACHMENT_OPTIMAL image layout
        const DEPTH_WRITE = 0x10;
        const DEPTH_READ = 0x20;
        const NON_PIXEL_SHADER_RESOURCE = 0x40;
        const PIXEL_SHADER_RESOURCE = 0x80;
        /// Similar to vulkan's SHADER_READ_ONLY_OPTIMAL image layout
        const SHADER_RESOURCE = 0x40 | 0x80;
        const STREAM_OUT = 0x100;
        const INDIRECT_ARGUMENT = 0x200;
        /// Similar to vulkan's TRANSFER_DST_OPTIMAL image layout
        const COPY_DST = 0x400;
        /// Similar to vulkan's TRANSFER_SRC_OPTIMAL image layout
        const COPY_SRC = 0x800;
        const GENERIC_READ = (((((0x1 | 0x2) | 0x40) | 0x80) | 0x200) | 0x800);
        /// Similar to vulkan's PRESENT_SRC_KHR image layout
        const PRESENT = 0x1000;
        /// Similar to vulkan's COMMON image layout
        const COMMON = 0x2000;
    }
}

/// A 2d size for windows, textures, etc.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Extents2D {
    pub width: u32,
    pub height: u32,
}

/// A 3d size for windows, textures, etc.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Extents3D {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
}

impl Extents3D {
    pub fn to_2d(self) -> Extents2D {
        Extents2D {
            width: self.width,
            height: self.height,
        }
    }
}

/// A 3d offset, textures, etc.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Offset3D {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Number of MSAA samples to use. 1xMSAA and 4xMSAA are most broadly supported
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum SampleCount {
    SampleCount1,
    SampleCount2,
    SampleCount4,
    SampleCount8,
    SampleCount16,
}

impl Default for SampleCount {
    fn default() -> Self {
        Self::SampleCount1
    }
}

bitflags::bitflags! {
    /// Indicates how a resource will be used. In some cases, multiple flags are allowed.
    #[derive(Default)]
    #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
    pub struct ResourceType: u32 {
        const UNDEFINED = 0;
        // const SAMPLER = 1<<0;
        /// Similar to DX12 SRV and vulkan SAMPLED image usage flag and SAMPLED_IMAGE descriptor type
        const TEXTURE = 1<<1;
        /// Similar to DX12 UAV and vulkan STORAGE image usage flag and STORAGE_IMAGE descriptor type
        const TEXTURE_READ_WRITE = 1<<2;
        /// Similar to DX12 SRV and vulkan STORAGE_BUFFER descriptor type
        // const BUFFER = 1<<3;
        /// Similar to DX12 UAV and vulkan STORAGE_BUFFER descriptor type
        // const BUFFER_READ_WRITE = 1<<5;
        /// Similar to vulkan UNIFORM_BUFFER descriptor type
        // const UNIFORM_BUFFER_ = 1<<7;
        // Push constant / Root constant
        /// Similar to DX12 root constants and vulkan push constants
        // const ROOT_CONSTANT = 1<<8;
        // Input assembler
        /// Similar to vulkan VERTEX_BUFFER buffer usage flag
        // const VERTEX_BUFFER = 1<<9;
        /// Similar to vulkan INDEX_BUFFER buffer usage flag
        // const INDEX_BUFFER = 1<<10;
        /// Similar to vulkan INDIRECT_BUFFER buffer usage flag
        // const INDIRECT_BUFFER = 1<<11;
        // Cubemap SRV
        /// Similar to vulkan's CUBE_COMPATIBLE image create flag and metal's Cube texture type
        const TEXTURE_CUBE = 1<<12 | Self::TEXTURE.bits();
        // RTV
        const RENDER_TARGET_MIP_SLICES = 1<<13;
        const RENDER_TARGET_ARRAY_SLICES = 1<<14;
        const RENDER_TARGET_DEPTH_SLICES = 1<<15;
        // Vulkan-only stuff
        const INPUT_ATTACHMENT = 1<<16;
        // const TEXEL_BUFFER = 1<<17;
        // const TEXEL_BUFFER_READ_WRITE = 1<<18;
        // Render target types
        /// A color attachment in a renderpass
        const RENDER_TARGET_COLOR = 1<<19;
        /// A depth/stencil attachment in a renderpass
        const RENDER_TARGET_DEPTH_STENCIL = 1<<20;
    }
}

    impl ResourceType {
//     pub fn is_uniform_buffer(self) -> bool {
//         self.intersects(Self::UNIFORM_BUFFER_)
//     }

//     pub fn is_storage_buffer(self) -> bool {
//         self.intersects(Self::BUFFER | Self::BUFFER_READ_WRITE)
//     }

        pub fn is_render_target(self) -> bool {
            self.intersects(Self::RENDER_TARGET_COLOR | Self::RENDER_TARGET_DEPTH_STENCIL)
        }

//     pub fn is_texture(self) -> bool {
//         self.intersects(Self::TEXTURE | Self::TEXTURE_READ_WRITE)
//     }
    }

bitflags::bitflags! {
    /// Flags for enabling/disabling color channels, used with `BlendState`
    #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
    pub struct ColorFlags: u8 {
        const RED = 1;
        const GREEN = 2;
        const BLUE = 4;
        const ALPHA = 8;
        const ALL = 0x0F;
    }
}

impl Default for ColorFlags {
    fn default() -> Self {
        Self::ALL
    }
}

/// Indicates how the memory will be accessed and affects where in memory it needs to be allocated.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MemoryUsage {
    Unknown,

    /// The memory is only accessed by the GPU
    GpuOnly,

    /// The memory is only accessed by the CPU
    CpuOnly,

    /// The memory is written by the CPU and read by the GPU
    CpuToGpu,

    /// The memory is written by the GPU and read by the CPU
    GpuToCpu,
}

/// Indicates the result of presenting a swapchain image
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PresentSuccessResult {
    /// The image was shown and the swapchain can continue to be used.
    Success,

    /// The image was shown and the swapchain can continue to be used. However, this result also
    /// hints that there is a more optimal configuration for the swapchain to be in. This is vague
    /// because the precise meaning varies between platform. For example, windows may return this
    /// when the application is minimized.
    SuccessSuboptimal,

    // While this is an "error" being returned as success, it is expected and recoverable while
    // other errors usually aren't. This way the ? operator can still be used to bail out the
    // unrecoverable errors and the different flavors of "success" should be explicitly handled
    // in a match
    /// Indicates that the swapchain can no longer be used
    DeviceReset,
}

/// Indicates the current state of a fence.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum FenceStatus {
    /// The fence was submitted to the command buffer and signaled as completed by the GPU
    Complete,
    /// The fence will be signaled as complete later by the GPU
    Incomplete,
    /// The fence was never submitted, or was submitted and already returned complete once, putting
    /// it back into the unsubmitted state
    Unsubmitted,
}

bitflags::bitflags! {
    /// Indicates what render targets are affected by a blend state
    #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
    pub struct BlendStateTargets : u8 {
        const BLEND_STATE_TARGET_0 = 0x01;
        const BLEND_STATE_TARGET_1 = 0x02;
        const BLEND_STATE_TARGET_2 = 0x04;
        const BLEND_STATE_TARGET_3 = 0x08;
        const BLEND_STATE_TARGET_4 = 0x10;
        const BLEND_STATE_TARGET_5 = 0x20;
        const BLEND_STATE_TARGET_6 = 0x40;
        const BLEND_STATE_TARGET_7 = 0x80;
        const BLEND_STATE_TARGET_ALL = 0xFF;
    }
}

bitflags::bitflags! {
    /// Indicates a particular stage of a shader, or set of stages in a shader. Similar to
    /// VkShaderStageFlagBits
    #[derive(Default)]
    #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
    pub struct ShaderStageFlags : u32 {
        const NONE = 0;
        const VERTEX = 1;
        const TESSELLATION_CONTROL = 2;
        const TESSELLATION_EVALUATION = 4;
        const GEOMETRY = 8;
        const FRAGMENT = 16;
        const COMPUTE = 32;
        const ALL_GRAPHICS = 0x1F;
        const ALL = 0x7FFF_FFFF;
    }
}

/// Contains all the individual stages
pub const ALL_SHADER_STAGE_FLAGS: [ShaderStageFlags; 6] = [
    ShaderStageFlags::VERTEX,
    ShaderStageFlags::TESSELLATION_CONTROL,
    ShaderStageFlags::TESSELLATION_EVALUATION,
    ShaderStageFlags::GEOMETRY,
    ShaderStageFlags::FRAGMENT,
    ShaderStageFlags::COMPUTE,
];

/// Indicates the type of pipeline, roughly corresponds with `QueueType`
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PipelineType {
    Graphics = 0,
    Compute = 1,
}

/// Affects how quickly vertex attributes are consumed from buffers, similar to `vkVertexInputRate`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttributeRate {
    Vertex,
    Instance,
}

impl Default for VertexAttributeRate {
    fn default() -> Self {
        Self::Vertex
    }
}

/// Determines if the contents of an image attachment in a renderpass begins with its previous
/// contents, a clear value, or undefined data. Similar to `vkAttachmentLoadOp`
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum LoadOp {
    DontCare,
    Load,
    Clear,
}

impl Default for LoadOp {
    fn default() -> Self {
        Self::DontCare
    }
}

/// Determines if the contents of an image attachment in a render pass will store the resulting
/// state for use after the render pass
#[derive(Copy, Clone, Debug, Hash, PartialEq)]
pub enum StoreOp {
    /// Do not store the image, leaving the contents of it undefined
    DontCare,

    /// Persist the image's content after a render pass completes
    Store,
}

impl Default for StoreOp {
    fn default() -> Self {
        Self::Store
    }
}

/// How to intepret vertex data into a form of geometry. Similar to `vkPrimitiveTopology`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum PrimitiveTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
    PatchList,
}

/// The size of index buffer elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum IndexType {
    Uint32,
    Uint16,
}

impl Default for IndexType {
    fn default() -> Self {
        Self::Uint32
    }
}

/// Affects blending. Similar to `vkBlendFactor`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum BlendFactor {
    Zero,
    One,
    SrcColor,
    OneMinusSrcColor,
    DstColor,
    OneMinusDstColor,
    SrcAlpha,
    OneMinusSrcAlpha,
    DstAlpha,
    OneMinusDstAlpha,
    SrcAlphaSaturate,
    ConstantColor,
    OneMinusConstantColor,
}

impl Default for BlendFactor {
    fn default() -> Self {
        Self::Zero
    }
}

/// Affects blending. Similar to `vkBlendOp`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum BlendOp {
    Add,
    Subtract,
    ReverseSubtract,
    Min,
    Max,
}

impl Default for BlendOp {
    fn default() -> Self {
        Self::Add
    }
}

/// Affects depth testing and sampling. Similar to `vkCompareOp`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum CompareOp {
    Never,
    Less,
    Equal,
    LessOrEqual,
    Greater,
    NotEqual,
    GreaterOrEqual,
    Always,
}

impl Default for CompareOp {
    fn default() -> Self {
        Self::Never
    }
}

/// Similar to `vkStencilOp`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    IncrementAndClamp,
    DecrementAndClamp,
    Invert,
    IncrementAndWrap,
    DecrementAndWrap,
}

impl Default for StencilOp {
    fn default() -> Self {
        Self::Keep
    }
}

/// Determines if we cull polygons that are front-facing or back-facing. Facing direction is
/// determined by `FrontFace`, sometimes called "winding order". Similar to `vkCullModeFlags`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum CullMode {
    None,
    Back,
    Front,
}

impl Default for CullMode {
    fn default() -> Self {
        Self::None
    }
}

/// Determines what winding order is considered the front face of a polygon. Similar to
/// `vkFrontFace`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum FrontFace {
    CounterClockwise,
    Clockwise,
}

impl Default for FrontFace {
    fn default() -> Self {
        Self::CounterClockwise
    }
}

/// Whether to fill in polygons or not. Similar to `vkPolygonMode`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum FillMode {
    Solid,
    Wireframe,
}

impl Default for FillMode {
    fn default() -> Self {
        Self::Solid
    }
}

/// Filtering method when sampling. Similar to `vkFilter`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum FilterType {
    /// Finds the closest value in the texture and uses it. Commonly used for "pixel-perfect"
    /// assets.
    Nearest,

    /// "Averages" color values of the texture. A common choice for most cases but may make some
    /// "pixel-perfect" assets appear blurry
    Linear,
}

impl Default for FilterType {
    fn default() -> Self {
        Self::Nearest
    }
}

/// Affects image sampling, particularly for UV coordinates outside the [0, 1] range. Similar to
/// `vkSamplerAddressMode`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum AddressMode {
    Mirror,
    Repeat,
    ClampToEdge,
    ClampToBorder,
}

impl Default for AddressMode {
    fn default() -> Self {
        Self::Mirror
    }
}

/// Similar to `vkSamplerMipmapMode`
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub enum MipMapMode {
    Nearest,
    Linear,
}

impl Default for MipMapMode {
    fn default() -> Self {
        Self::Nearest
    }
}

/// A clear value for color attachments
#[derive(Copy, Clone, Debug, Default)]
pub struct ColorClearValue(pub [f32; 4]);

impl Hash for ColorClearValue {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        for &value in &self.0 {
            DecimalF32(value).hash(&mut state);
        }
    }
}

/// A clear values for depth/stencil attachments. One or both values may be used depending on the
/// format of the attached image
#[derive(Clone, Copy, Debug)]
pub struct DepthStencilClearValue {
    pub depth: f32,
    pub stencil: u32,
}

impl Default for DepthStencilClearValue {
    fn default() -> Self {
        Self {
            depth: 0.0,
            stencil: 0,
        }
    }
}

impl Hash for DepthStencilClearValue {
    fn hash<H: Hasher>(&self, mut state: &mut H) {
        DecimalF32(self.depth).hash(&mut state);
        self.stencil.hash(&mut state);
    }
}

/// Determines if a barrier is transferring a resource from one queue to another.
pub enum BarrierQueueTransition {
    /// No queue transition will take place
    None,

    /// A barrier for the "sending" queue. Contains the "receiving" queue. (the "sending" queue is
    /// inferred by the queue on which the barrier is submitted)
    ReleaseTo(QueueType),

    /// A barrier for the "receiving" queue. Contains the "sending" queue. (the "receiving" queue is
    /// inferred by the queue on which the barrier is submitted)
    AcquireFrom(QueueType),
}

impl Default for BarrierQueueTransition {
    fn default() -> Self {
        Self::None
    }
}

/// A memory barrier for buffers. This is used to transition buffers between resource states and
/// possibly from one queue to another
pub struct BufferBarrier<'a, A: GfxApi> {
    pub buffer: &'a A::Buffer,
    pub src_state: ResourceState,
    pub dst_state: ResourceState,
    pub queue_transition: BarrierQueueTransition,
}

/// A memory barrier for textures. This is used to transition textures between resource states and
/// possibly from one queue to another.
pub struct TextureBarrier<'a, A: GfxApi> {
    pub texture: &'a A::Texture,
    pub src_state: ResourceState,
    pub dst_state: ResourceState,
    pub queue_transition: BarrierQueueTransition,
    /// If set, only the specified array element is included
    pub array_slice: Option<u16>,
    /// If set, only the specified mip level is included
    pub mip_slice: Option<u8>,
}

impl<'a, A: GfxApi> TextureBarrier<'a, A> {
    /// Creates a simple state transition
    pub fn state_transition(
        texture: &'a A::Texture,
        src_state: ResourceState,
        dst_state: ResourceState,
    ) -> Self {
        Self {
            texture,
            src_state,
            dst_state,
            queue_transition: BarrierQueueTransition::None,
            array_slice: None,
            mip_slice: None,
        }
    }
}

/// Represents an image owned by the swapchain
pub struct SwapchainImage<A: GfxApi> {
    pub texture: A::Texture,
    pub swapchain_image_index: u32,
}

impl<A: GfxApi> Clone for SwapchainImage<A> {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            swapchain_image_index: self.swapchain_image_index,
        }
    }
}

/// A color render target bound during a renderpass
#[derive(Debug)]
pub struct ColorRenderTargetBinding<'a, A: GfxApi> {
    pub texture: &'a A::Texture,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub mip_slice: Option<u8>,
    pub array_slice: Option<u16>,
    pub clear_value: ColorClearValue,
    pub resolve_target: Option<&'a A::Texture>,
    pub resolve_store_op: StoreOp,
    pub resolve_mip_slice: Option<u8>,
    pub resolve_array_slice: Option<u16>,
}

/// A depth/stencil render target to be bound during a renderpass
#[derive(Debug)]
pub struct DepthStencilRenderTargetBinding<'a, A: GfxApi> {
    pub texture: &'a A::Texture,
    pub depth_load_op: LoadOp,
    pub stencil_load_op: LoadOp,
    pub depth_store_op: StoreOp,
    pub stencil_store_op: StoreOp,
    pub mip_slice: Option<u8>,
    pub array_slice: Option<u16>,
    pub clear_value: DepthStencilClearValue,
}

/// A vertex buffer to be bound during a renderpass
pub struct VertexBufferBinding<'a, A: GfxApi> {
    pub buffer: &'a A::Buffer,
    pub byte_offset: u64,
}

/// An index buffer to be bound during a renderpass
pub struct IndexBufferBinding<'a, A: GfxApi> {
    pub buffer: &'a A::Buffer,
    pub byte_offset: u64,
    pub index_type: IndexType,
}

/// Parameters for copying a buffer to a texture
#[derive(Default)]
pub struct CmdCopyBufferToTextureParams {
    pub buffer_offset: u64,
    pub array_layer: u16,
    pub mip_level: u8,
}

/// Parameters for blitting one image to another (vulkan backend only)
pub struct CmdBlitParams {
    pub src_state: ResourceState,
    pub dst_state: ResourceState,
    pub src_offsets: [Offset3D; 2],
    pub dst_offsets: [Offset3D; 2],
    pub src_mip_level: u8,
    pub dst_mip_level: u8,
    pub array_slices: Option<[u16; 2]>,
    pub filtering: FilterType,
}

/// Parameters for copying one image data to another (vulkan backend only)
pub struct CmdCopyTextureParams {
    pub src_state: ResourceState,
    pub dst_state: ResourceState,
    pub src_offset: Offset3D,
    pub dst_offset: Offset3D,
    pub src_mip_level: u8,
    pub dst_mip_level: u8,
    pub src_array_slice: u16,
    pub dst_array_slice: u16,
    pub extent: Extents3D,
}

/// A legion-specific index that refers to a particular binding. Instead of doing name/binding lookups
/// every frame, query the descriptor index during startup and use it instead. This is a more
/// efficient way to address descriptors.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct DescriptorIndex(pub(crate) u32);

/// Selects a particular descriptor in a descriptor set
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum DescriptorKey<'a> {
    Undefined,
    Name(&'a str),
}

impl<'a> Default for DescriptorKey<'a> {
    fn default() -> Self {
        DescriptorKey::Undefined
    }
}

/// Used when binding buffers to descriptor sets
#[derive(Default, Clone, Copy, Debug)]
pub struct OffsetSize {
    pub byte_offset: u64,
    pub size: u64,
}

/// Specifies what value to assign to a descriptor set
#[derive(Debug)]
pub struct DescriptorElements<'a, A: GfxApi> {
    pub samplers: Option<&'a [&'a A::Sampler]>,
    pub buffer_views: Option<&'a [&'a A::BufferView]>,
}

impl<'a, A: GfxApi> Default for DescriptorElements<'a, A> {
    fn default() -> Self {
        Self {
            // textures: None,
            samplers: None,
            // buffers: None,
            // buffer_offset_sizes: None,
            buffer_views : None,
            // srvs : None
        }
    }
}
// 
// /// Used when binding a texture to select between different ways to bind the texture
// #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
// pub enum TextureBindType {
//     // Color or depth only
//     Srv,
//     // stencil?
//     SrvStencil,
//     // Bind all mip levels of the 0th provided texture
//     UavMipChain,
//     // Bind a particular mip slice of all provided textures
//     UavMipSlice(u32),
// }

/// Describes how to update a single descriptor
#[derive(Debug)]
pub struct DescriptorUpdate<'a, A: GfxApi> {
    pub array_index: u32,
    pub descriptor_key: DescriptorKey<'a>,
    pub elements: DescriptorElements<'a, A>,
    pub dst_element_offset: u32,
    // Srv when read-only, UavMipSlice(0) when read-write
    // pub texture_bind_type: Option<TextureBindType>,
}

impl<'a, A: GfxApi> Default for DescriptorUpdate<'a, A> {
    fn default() -> Self {
        DescriptorUpdate {
            array_index: 0,
            descriptor_key: DescriptorKey::Undefined,
            elements: DescriptorElements::default(),
            dst_element_offset: 0,            
        }
    }
}

/// Set the texture tiling (internally swizzled, linear)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextureTiling {
    /// Optimal for the underlying format, probably swizzled for efficient sampling
    Optimal,
    /// Linear, usefull
    Linear,
}

// notes: this should probably have a mut version (see how to be generic over mutability)
// notes: having drop implement unmap would be wise, and do the same for buffer map/unmap
/// Used when mapping a texture
pub struct TextureSubResource<'a> {
    pub data: &'a [u8],
    pub row_pitch: u32,
    pub array_pitch: u32,
    pub depth_pitch: u32,
}
