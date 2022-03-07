use std::sync::Arc;

use lgn_data_runtime::ResourceTypeAndId;
use lgn_ecs::prelude::Component;
use lgn_graphics_data::TextureFormat;

#[derive(Clone)]
pub struct TextureData {
    data: Arc<Vec<Vec<u8>>>,
}

impl TextureData {
    pub fn from_slice<T: Sized>(mip0_data: &[T]) -> Self {
        Self {
            data: Arc::new(vec![Self::to_vec_u8(mip0_data)]),
        }
    }

    pub fn from_slices<T: Sized>(mips_data: &[&[T]]) -> Self {
        let mut data = Vec::with_capacity(mips_data.len());
        for mip_data in mips_data.iter() {
            data.push(Self::to_vec_u8(*mip_data));
        }

        Self {
            data: Arc::new(data),
        }
    }

    pub fn data(&self) -> &Vec<Vec<u8>> {
        &self.data
    }

    pub fn mip_count(&self) -> usize {
        self.data.len()
    }

    #[allow(unsafe_code)]
    fn to_vec_u8<T: Sized>(mip_data: &[T]) -> Vec<u8> {
        let src_ptr = mip_data.as_ptr().cast::<u8>();
        let src_size = mip_data.len() * std::mem::size_of::<T>();
        unsafe {
            let dst_ptr =
                std::alloc::alloc(std::alloc::Layout::from_size_align(src_size, 16).unwrap());
            dst_ptr.copy_from_nonoverlapping(src_ptr, src_size);
            Vec::<u8>::from_raw_parts(dst_ptr, src_size, src_size)
        }
    }
}

#[derive(Component)]
pub struct TextureComponent {
    pub(crate) texture_id: ResourceTypeAndId,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) format: TextureFormat,
    pub(crate) texture_data: TextureData,
}

impl TextureComponent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        texture_id: ResourceTypeAndId,
        width: u32,
        height: u32,
        format: TextureFormat,
        texture_data: TextureData,
    ) -> Self {
        Self {
            texture_id,
            width,
            height,
            format,
            texture_data,
        }
    }
}
