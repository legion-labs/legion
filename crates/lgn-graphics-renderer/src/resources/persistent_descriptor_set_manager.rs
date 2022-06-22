use egui::mutex::RwLock;
use lgn_graphics_api::{
    DescriptorHeapDef, DescriptorHeapPartition, DescriptorRef, DescriptorSet, DescriptorSetWriter,
    DeviceContext, Sampler, TextureView,
};

use std::sync::Arc;

use crate::cgen;

use super::{DescriptorHeapManager, IndexAllocator};

const TEXTURE_ARRAY_LEN: u32 = 10 * 1024;
const SAMPLER_ARRAY_LEN: u32 = 64; // When changing this number make sure to make a corresponding change to material_samplers in root.rn

#[derive(Clone, Copy)]
pub struct TextureSlot(u32);

impl TextureSlot {
    pub fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for TextureSlot {
    fn from(val: u32) -> Self {
        TextureSlot(val)
    }
}

#[derive(Clone, Copy)]
pub struct SamplerSlot(u32);

impl SamplerSlot {
    pub fn index(self) -> u32 {
        self.0
    }
}

impl From<u32> for SamplerSlot {
    fn from(val: u32) -> Self {
        SamplerSlot(val)
    }
}

struct BindlessAllocator {
    index_allocator: IndexAllocator,
    removed_slots: Vec<Vec<u32>>,
    render_frame: u64,
    num_render_frames: u64,
}

impl BindlessAllocator {
    fn new(capacity: u32, num_render_frames: u64) -> Self {
        Self {
            index_allocator: IndexAllocator::new(capacity),
            removed_slots: (0..num_render_frames)
                .map(|_| Vec::new())
                .collect::<Vec<_>>(),
            render_frame: 0,
            num_render_frames,
        }
    }

    fn allocate(&mut self) -> u32 {
        self.index_allocator.allocate()
    }

    fn free(&mut self, slot: u32) {
        let removed_indices = &mut self.removed_slots[self.render_frame as usize];
        removed_indices.push(slot);
    }

    fn frame_update(&mut self) {
        self.render_frame = (self.render_frame + 1) % self.num_render_frames;
        self.removed_slots[self.render_frame as usize]
            .iter()
            .for_each(|slot| self.index_allocator.free(*slot));
        self.removed_slots[self.render_frame as usize].clear();
    }
}

struct Inner {
    device_context: DeviceContext,
    descriptor_set: DescriptorSet,
    texture_allocator: RwLock<BindlessAllocator>,
    sampler_allocator: RwLock<BindlessAllocator>,
    material_textures_index: u32,
    material_samplers_index: u32,
}

#[derive(Clone)]
pub struct PersistentDescriptorSetManager {
    inner: Arc<Inner>,
}

impl PersistentDescriptorSetManager {
    pub fn new(
        device_context: &DeviceContext,
        descriptor_heap_manager: &DescriptorHeapManager,
        num_render_frames: u64,
    ) -> Self {
        // todo: cgen must be initialized at this point
        let layout = cgen::descriptor_set::PersistentDescriptorSet::descriptor_set_layout();

        // todo: make the size runtime-driven
        assert_eq!(
            layout
                .find_descriptor_by_name("material_textures")
                .unwrap()
                .element_count
                .get(),
            TEXTURE_ARRAY_LEN
        );

        let material_textures_index = layout
            .find_descriptor_index_by_name("material_textures")
            .unwrap();
        let material_samplers_index = layout
            .find_descriptor_index_by_name("material_samplers")
            .unwrap();

        let persistent_partition = DescriptorHeapPartition::new(
            descriptor_heap_manager.descriptor_heap(),
            false,
            DescriptorHeapDef::from_descriptor_set_layout_def(layout.definition(), 1),
        );

        Self {
            inner: Arc::new(Inner {
                device_context: device_context.clone(),
                descriptor_set: persistent_partition.alloc(layout).unwrap(),
                material_textures_index,
                material_samplers_index,
                texture_allocator: RwLock::new(BindlessAllocator::new(
                    TEXTURE_ARRAY_LEN,
                    num_render_frames,
                )),
                sampler_allocator: RwLock::new(BindlessAllocator::new(
                    SAMPLER_ARRAY_LEN,
                    num_render_frames,
                )),
            }),
        }
    }

    pub fn allocate_texture_slot(&self, texture_view: &TextureView) -> TextureSlot {
        let mut allocator = self.inner.texture_allocator.write();
        let slot = allocator.allocate();

        let mut writer = DescriptorSetWriter::new(
            &self.inner.device_context,
            self.inner.descriptor_set.handle(),
            self.inner.descriptor_set.layout(),
        );

        writer.set_descriptors_by_index_and_offset(
            self.inner.material_textures_index,
            slot,
            &[DescriptorRef::TextureView(texture_view)],
        );

        TextureSlot(slot)
    }

    pub fn free_texture_slot(&self, slot: TextureSlot) {
        let mut allocator = self.inner.texture_allocator.write();
        allocator.free(slot.0);
    }

    pub fn allocate_sampler_slot(&self, sampler: &Sampler) -> SamplerSlot {
        let mut allocator = self.inner.sampler_allocator.write();
        let slot = allocator.allocate();

        let mut writer = DescriptorSetWriter::new(
            &self.inner.device_context,
            self.inner.descriptor_set.handle(),
            self.inner.descriptor_set.layout(),
        );

        writer.set_descriptors_by_index_and_offset(
            self.inner.material_samplers_index,
            slot,
            &[DescriptorRef::Sampler(sampler)],
        );

        SamplerSlot(slot)
    }

    pub fn free_sampler_slot(&self, slot: SamplerSlot) {
        let mut allocator = self.inner.sampler_allocator.write();
        allocator.free(slot.0);
    }

    pub fn descriptor_set(&self) -> &DescriptorSet {
        &self.inner.descriptor_set
    }

    pub fn frame_update(&mut self) {
        {
            let mut allocator = self.inner.texture_allocator.write();
            allocator.frame_update();
        }
        {
            let mut allocator = self.inner.sampler_allocator.write();
            allocator.frame_update();
        }
    }
}
