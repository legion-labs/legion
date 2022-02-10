use crate::backends::BackendTextureView;
use crate::{deferred_drop::Drc, Texture};
use crate::{Descriptor, GPUViewType, PlaneSlice, ResourceUsage, ShaderResourceType, TextureDef};

#[derive(Clone, Copy, Debug)]
pub enum ViewDimension {
    _2D,
    _2DArray,
    Cube,
    CubeArray,
    _3D,
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
            gpu_view_type: GPUViewType::ShaderResource,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: texture_def.mip_count,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: texture_def.array_length,
        }
    }

    pub fn as_render_target_view(_texture: &TextureDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::RenderTarget,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
            first_array_slice: 0,
            array_size: 1,
        }
    }

    pub fn as_depth_stencil_view(_texture: &TextureDef) -> Self {
        Self {
            gpu_view_type: GPUViewType::DepthStencil,
            view_dimension: ViewDimension::_2D,
            first_mip: 0,
            mip_count: 1,
            plane_slice: PlaneSlice::Default,
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
            GPUViewType::ShaderResource => {
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
                        assert!(self.plane_slice == PlaneSlice::Default);
                        assert!(self.first_array_slice == 0);
                        assert!(self.array_size == 1);
                    }
                    ViewDimension::CubeArray => {
                        assert!(self.plane_slice == PlaneSlice::Default);
                    }
                }
            }
            GPUViewType::UnorderedAccess => {
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
                        assert!(self.plane_slice == PlaneSlice::Default);
                    }
                    ViewDimension::Cube | ViewDimension::CubeArray => {
                        panic!();
                    }
                }
            }
            GPUViewType::RenderTarget => {
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
                        assert!(self.plane_slice == PlaneSlice::Default);
                    }
                    ViewDimension::Cube | ViewDimension::CubeArray => {
                        panic!();
                    }
                }
            }
            GPUViewType::DepthStencil => {
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
            GPUViewType::ConstantBuffer => {
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

#[derive(Clone, Debug)]
pub(crate) struct TextureViewInner {
    definition: TextureViewDef,
    texture: Texture,
    pub(crate) backend_texture_view: BackendTextureView,
}

impl Drop for TextureViewInner {
    fn drop(&mut self) {
        self.backend_texture_view
            .destroy(self.texture.device_context());
    }
}

#[derive(Clone, Debug)]
pub struct TextureView {
    pub(crate) inner: Drc<TextureViewInner>,
}

impl TextureView {
    pub(crate) fn new(texture: &Texture, view_def: &TextureViewDef) -> Self {
        let device_context = texture.device_context();
        let backend_texture_view = BackendTextureView::new(texture, view_def);

        Self {
            inner: device_context.deferred_dropper().new_drc(TextureViewInner {
                definition: *view_def,
                texture: texture.clone(),
                backend_texture_view,
            }),
        }
    }

    pub fn definition(&self) -> &TextureViewDef {
        &self.inner.definition
    }

    pub(crate) fn texture(&self) -> &Texture {
        &self.inner.texture
    }

    pub(crate) fn is_compatible_with_descriptor(&self, descriptor: &Descriptor) -> bool {
        match descriptor.shader_resource_type {
            ShaderResourceType::ConstantBuffer
            | ShaderResourceType::StructuredBuffer
            | ShaderResourceType::ByteAddressBuffer
            | ShaderResourceType::RWStructuredBuffer
            | ShaderResourceType::RWByteAddressBuffer
            | ShaderResourceType::Sampler => false,

            ShaderResourceType::Texture2D
            | ShaderResourceType::Texture3D
            | ShaderResourceType::Texture2DArray
            | ShaderResourceType::TextureCube
            | ShaderResourceType::TextureCubeArray => {
                self.inner.definition.gpu_view_type == GPUViewType::ShaderResource
                    && self.inner.definition.array_size == 1
            }

            ShaderResourceType::RWTexture2D
            | ShaderResourceType::RWTexture2DArray
            | ShaderResourceType::RWTexture3D => {
                self.inner.definition.gpu_view_type == GPUViewType::UnorderedAccess
                    && self.inner.definition.array_size == 1
            }
        }
    }
}
