use ash::vk;

use crate::{DeviceContext, Texture, TextureView, TextureViewDef};

#[derive(Clone, Debug)]
pub(crate) struct VulkanTextureView {
    vk_image_view: vk::ImageView,
}

impl VulkanTextureView {
    pub(crate) fn new(texture: &Texture, definition: TextureViewDef) -> Self {
        definition.verify(texture.definition());

        let device_context = texture.device_context();
        let device = device_context.vk_device();
        let texture_def = texture.definition();
        let aspect_mask = match definition.plane_slice {
            crate::PlaneSlice::Default => {
                super::internal::image_format_to_aspect_mask(texture_def.format)
            }
            crate::PlaneSlice::Depth => vk::ImageAspectFlags::DEPTH,
            crate::PlaneSlice::Stencil => vk::ImageAspectFlags::STENCIL,
            crate::PlaneSlice::Plane0 => vk::ImageAspectFlags::PLANE_0,
            crate::PlaneSlice::Plane1 => vk::ImageAspectFlags::PLANE_1,
            crate::PlaneSlice::Plane2 => vk::ImageAspectFlags::PLANE_2,
        };
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(definition.first_mip)
            .level_count(definition.mip_count)
            .base_array_layer(definition.first_array_slice)
            .layer_count(definition.array_size);
        let builder = vk::ImageViewCreateInfo::builder()
            .image(texture.vk_image())
            .components(vk::ComponentMapping::default())
            .view_type(definition.view_dimension.into())
            .format(texture_def.format.into())
            .subresource_range(subresource_range.build());
        let vk_image_view = unsafe { device.create_image_view(&builder.build(), None).unwrap() };

        Self { vk_image_view }
    }

    pub(crate) fn destroy(&self, device_context: &DeviceContext) {
        unsafe {
            device_context
                .vk_device()
                .destroy_image_view(self.vk_image_view, None);
        }
    }
}

impl TextureView {
    pub(crate) fn vk_image_view(&self) -> vk::ImageView {
        self.inner.backend_texture_view.vk_image_view
    }
}
