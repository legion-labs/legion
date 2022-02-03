use std::hash::Hash;

use serde::{Deserialize, Serialize};

use strum::IntoStaticStr;

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

bitflags::bitflags! {
    pub struct BufferViewFlags: u8 {
        const RAW_BUFFER = 0x01;
    }
}

#[derive(Clone, Debug)]
pub struct Descriptor {
    pub name: String,
    pub binding: u32,
    pub shader_resource_type: ShaderResourceType,
    pub element_count: u32,
    pub update_data_offset: u32,
}

impl Descriptor {
    pub fn element_count_normalized(&self) -> u32 {
        self.element_count.max(1)
    }
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
    Default,
    Depth,
    Stencil,
    // Multi-planar formats support
    Plane0,
    Plane1,
    Plane2,
}

#[derive(
    Copy, strum::Display, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, IntoStaticStr,
)]
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
