use std::hash::{Hash, Hasher};

use lgn_utils::decimal::DecimalF32;
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

use crate::{Buffer, BufferView, PlaneSlice, Sampler, Texture, TextureView};

/// Information about the device, mostly limits, requirements (like memory
/// alignment), and flags to indicate whether certain features are supported
#[derive(Clone, Copy)]
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

/// Used to indicate which type of queue to use. Some operations require certain
/// types of queues.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum QueueType {
    /// Graphics queues generally supports all operations and are a safe default
    /// choice
    Graphics,

    /// Compute queues can be used for compute-based work.
    Compute,

    /// Transfer queues are generally limited to basic operations like copying
    /// data from buffers to images.
    Transfer,

    /// Decode queues are not available on all device but allow use of dedicated
    /// hardware to encode videos
    Decode,

    /// Encode queues are not available on all device but allow use of dedicated
    /// hardware to encode videos
    Encode,
}

/// The color space an image data is in. The correct color space often varies
/// between texture types (like normal maps vs. albedo maps).
#[derive(Copy, Clone, Debug)]
pub enum ColorType {
    Linear,
    Srgb,
}

// /// Texture will allocate its own memory (COMMITTED resource)
// TEXTURE_CREATION_FLAG_OWN_MEMORY_BIT = 0x01,
// /// Use on-tile memory to store this texture
// TEXTURE_CREATION_FLAG_ON_TILE = 0x20,
// /// Force 2D instead of automatically determining dimension based on width,
// height, depth TEXTURE_CREATION_FLAG_FORCE_2D = 0x80,
// /// Force 3D instead of automatically determining dimension based on width,
// height, depth TEXTURE_CREATION_FLAG_FORCE_3D = 0x100,
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
    pub struct ResourceFlags: u32 {
        const TEXTURE_CUBE = 1<<12;
    }
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

/// Indicates how the memory will be accessed and affects where in memory it
/// needs to be allocated.
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

    /// The image was shown and the swapchain can continue to be used. However,
    /// this result also hints that there is a more optimal configuration
    /// for the swapchain to be in. This is vague because the precise
    /// meaning varies between platform. For example, windows may return this
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
    /// The fence was submitted to the command buffer and signaled as completed
    /// by the GPU
    Complete,
    /// The fence will be signaled as complete later by the GPU
    Incomplete,
    /// The fence was never submitted, or was submitted and already returned
    /// complete once, putting it back into the unsubmitted state
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
    #[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
    pub struct ShaderStageFlags : u32 {
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

/// Affects how quickly vertex attributes are consumed from buffers, similar to
/// `vkVertexInputRate`
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

/// Determines if the contents of an image attachment in a renderpass begins
/// with its previous contents, a clear value, or undefined data. Similar to
/// `vkAttachmentLoadOp`
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

/// Determines if the contents of an image attachment in a render pass will
/// store the resulting state for use after the render pass
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

/// How to intepret vertex data into a form of geometry. Similar to
/// `vkPrimitiveTopology`
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

/// Determines if we cull polygons that are front-facing or back-facing. Facing
/// direction is determined by `FrontFace`, sometimes called "winding order".
/// Similar to `vkCullModeFlags`
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

/// Determines what winding order is considered the front face of a polygon.
/// Similar to `vkFrontFace`
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
    /// Finds the closest value in the texture and uses it. Commonly used for
    /// "pixel-perfect" assets.
    Nearest,

    /// "Averages" color values of the texture. A common choice for most cases
    /// but may make some "pixel-perfect" assets appear blurry
    Linear,
}

impl Default for FilterType {
    fn default() -> Self {
        Self::Nearest
    }
}

/// Affects image sampling, particularly for UV coordinates outside the [0, 1]
/// range. Similar to `vkSamplerAddressMode`
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

/// A clear values for depth/stencil attachments. One or both values may be used
/// depending on the format of the attached image
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

/// Determines if a barrier is transferring a resource from one queue to
/// another.
pub enum BarrierQueueTransition {
    /// No queue transition will take place
    None,

    /// A barrier for the "sending" queue. Contains the "receiving" queue. (the
    /// "sending" queue is inferred by the queue on which the barrier is
    /// submitted)
    ReleaseTo(QueueType),

    /// A barrier for the "receiving" queue. Contains the "sending" queue. (the
    /// "receiving" queue is inferred by the queue on which the barrier is
    /// submitted)
    AcquireFrom(QueueType),
}

impl Default for BarrierQueueTransition {
    fn default() -> Self {
        Self::None
    }
}

/// A memory barrier for buffers. This is used to transition buffers between
/// resource states and possibly from one queue to another
pub struct BufferBarrier<'a> {
    pub buffer: &'a Buffer,
    pub src_state: ResourceState,
    pub dst_state: ResourceState,
    pub queue_transition: BarrierQueueTransition,
}

/// A memory barrier for textures. This is used to transition textures between
/// resource states and possibly from one queue to another.
pub struct TextureBarrier<'a> {
    pub texture: &'a Texture,
    pub src_state: ResourceState,
    pub dst_state: ResourceState,
    pub queue_transition: BarrierQueueTransition,
    /// If set, only the specified array element is included
    pub array_slice: Option<u16>,
    /// If set, only the specified mip level is included
    pub mip_slice: Option<u8>,
}

impl<'a> TextureBarrier<'a> {
    /// Creates a simple state transition
    pub fn state_transition(
        texture: &'a Texture,
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
pub struct SwapchainImage {
    pub texture: Texture,
    pub render_target_view: TextureView,
    pub swapchain_image_index: u32,
}

impl Clone for SwapchainImage {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            render_target_view: self.render_target_view.clone(),
            swapchain_image_index: self.swapchain_image_index,
        }
    }
}

/// A color render target bound during a renderpass
#[derive(Debug)]
pub struct ColorRenderTargetBinding<'a> {
    pub texture_view: &'a TextureView,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_value: ColorClearValue,
}

/// A depth/stencil render target to be bound during a renderpass
#[derive(Debug)]
pub struct DepthStencilRenderTargetBinding<'a> {
    pub texture_view: &'a TextureView,
    pub depth_load_op: LoadOp,
    pub stencil_load_op: LoadOp,
    pub depth_store_op: StoreOp,
    pub stencil_store_op: StoreOp,
    pub clear_value: DepthStencilClearValue,
}

/// A vertex buffer to be bound during a renderpass
pub struct VertexBufferBinding<'a> {
    pub buffer: &'a Buffer,
    pub byte_offset: u64,
}

/// An index buffer to be bound during a renderpass
pub struct IndexBufferBinding<'a> {
    pub buffer: &'a Buffer,
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
    pub src_plane_slice: PlaneSlice,
    pub dst_plane_slice: PlaneSlice,
    pub extent: Extents3D,
}

/// Wraps all the possible types used to fill a `DescriptorSet`
#[derive(Clone, Copy)]
pub enum DescriptorRef<'a> {
    Undefined,
    Sampler(&'a Sampler),
    BufferView(&'a BufferView),
    TextureView(&'a TextureView),
}

impl<'a> Default for DescriptorRef<'a> {
    fn default() -> Self {
        Self::Undefined
    }
}

/// Set the texture tiling (internally swizzled, linear)
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TextureTiling {
    /// Optimal for the underlying format, probably swizzled for efficient
    /// sampling
    Optimal,
    /// Linear, usefull
    Linear,
}

// notes: this should probably have a mut version (see how to be generic over
// mutability) notes: having drop implement unmap would be wise, and do the same
// for buffer map/unmap
/// Used when mapping a texture
pub struct TextureSubResource<'a> {
    pub data: &'a [u8],
    pub row_pitch: u32,
    pub array_pitch: u32,
    pub depth_pitch: u32,
}
