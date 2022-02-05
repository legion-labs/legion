use std::hash::Hash;
use strum::{Display, IntoStaticStr};

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
        // buffer/texture
        const AS_TRANSFERABLE = 0x0100;
        // meta
        const BUFFER_ONLY_USAGE_FLAGS =
            Self::AS_CONST_BUFFER.bits|
            Self::AS_VERTEX_BUFFER.bits|
            Self::AS_INDEX_BUFFER.bits|
            Self::AS_INDIRECT_BUFFER.bits;
        const TEXTURE_ONLY_USAGE_FLAGS =
            Self::AS_DEPTH_STENCIL.bits;
    }
}

bitflags::bitflags! {
    pub struct ResourceCreation: u16 {
        const SPARSE_BINDING = 0x0001;
        const SPARSE_RESIDENCY = 0x0002;
        const SPARSE_ALIASED = 0x0004;
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GPUViewType {
    ConstantBuffer,
    ShaderResource,
    UnorderedAccess,
    RenderTarget,
    DepthStencil,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlaneSlice {
    Default,
    Depth,
    Stencil,
    // Multi-planar formats support
    Plane0,
    Plane1,
    Plane2,
}

#[derive(Display, Copy, Clone, Debug, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum ShaderResourceType {
    Sampler,
    ConstantBuffer,
    StructuredBuffer,
    RWStructuredBuffer,
    ByteAddressBuffer,
    RWByteAddressBuffer,
    Texture2D,
    RWTexture2D,
    Texture2DArray,
    RWTexture2DArray,
    Texture3D,
    RWTexture3D,
    TextureCube,
    TextureCubeArray,
}
