use std::hash::Hash;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShaderResourceType {
    Sampler = 0x00_01,
    ConstantBuffer = 0x00_02,
    StructuredBuffer = 0x00_04,
    RWStructuredBuffer = 0x00_08,
    ByteAddressBuffer = 0x00_10,
    RWByteAddressBuffer = 0x00_20,
    Texture2D = 0x00_40,
    RWTexture2D = 0x00_80,
    Texture2DArray = 0x01_00,
    RWTexture2DArray = 0x02_00,
    Texture3D = 0x04_00,
    RWTexture3D = 0x08_00,
    TextureCube = 0x10_00,
    TextureCubeArray = 0x20_00,
}

impl ShaderResourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            ShaderResourceType::Sampler => "Sampler",
            ShaderResourceType::ConstantBuffer => "ConstantBuffer",
            ShaderResourceType::StructuredBuffer => "StructuredBuffer",
            ShaderResourceType::RWStructuredBuffer => "RWStructuredBuffer",
            ShaderResourceType::ByteAddressBuffer => "ByteAddressBuffer",
            ShaderResourceType::RWByteAddressBuffer => "RWByteAddressBuffer",
            ShaderResourceType::Texture2D => "Texture2D",
            ShaderResourceType::RWTexture2D => "RWTexture2D",
            ShaderResourceType::Texture2DArray => "Texture2DArray",
            ShaderResourceType::RWTexture2DArray => "RWTexture2DArray",
            ShaderResourceType::Texture3D => "Texture3D",
            ShaderResourceType::RWTexture3D => "RWTexture3D",
            ShaderResourceType::TextureCube => "TextureCube",
            ShaderResourceType::TextureCubeArray => "TextureCubeArray",
    }
    }
}
