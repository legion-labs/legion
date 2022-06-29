use bumpalo::Bump;
use bumpalo_herd::Herd;
use lgn_graphics_api::{
    CommandBuffer, DescriptorRef, DescriptorSetHandle, DescriptorSetLayout, DeviceContext,
};
use lgn_transform::prelude::GlobalTransform;

use crate::{
    components::ManipulatorComponent,
    debug_display::DebugDisplay,
    egui::Egui,
    picking::PickingManager,
    resources::{
        DescriptorPoolHandle, PipelineManager, TransientBufferAllocator,
        TransientCommandBufferAllocator, UnifiedStaticBuffer,
    },
    GraphicsQueue,
};

pub struct RenderContext<'frame> {
    pub device_context: &'frame DeviceContext,
    pub graphics_queue: &'frame GraphicsQueue,
    pub descriptor_pool: &'frame DescriptorPoolHandle,
    pub pipeline_manager: &'frame mut PipelineManager,
    pub transient_commandbuffer_allocator: &'frame mut TransientCommandBufferAllocator,
    pub transient_buffer_allocator: &'frame mut TransientBufferAllocator,
    pub static_buffer: &'frame UnifiedStaticBuffer,
    pub herd: &'frame Herd,
    pub bump: &'frame Bump,
    pub picking_manager: &'frame PickingManager,
    pub debug_display: &'frame DebugDisplay,
    pub manipulator_drawables: &'frame [(&'frame GlobalTransform, &'frame ManipulatorComponent)],
    pub egui: &'frame Egui,
    // tmp
    persistent_descriptor_set: Option<(&'frame DescriptorSetLayout, DescriptorSetHandle)>,
    frame_descriptor_set: Option<(&'frame DescriptorSetLayout, DescriptorSetHandle)>,
    view_descriptor_set: Option<(&'frame DescriptorSetLayout, DescriptorSetHandle)>,
}

impl<'frame> RenderContext<'frame> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        device_context: &'frame DeviceContext,
        graphics_queue: &'frame GraphicsQueue,
        descriptor_pool: &'frame DescriptorPoolHandle,
        pipeline_manager: &'frame mut PipelineManager,
        transient_commandbuffer_allocator: &'frame mut TransientCommandBufferAllocator,
        transient_buffer_allocator: &'frame mut TransientBufferAllocator,
        static_buffer: &'frame UnifiedStaticBuffer,
        herd: &'frame Herd,
        bump: &'frame Bump,
        picking_manager: &'frame PickingManager,
        debug_display: &'frame DebugDisplay,
        manipulator_drawables: &'frame [(&'frame GlobalTransform, &'frame ManipulatorComponent)],
        egui: &'frame Egui,
    ) -> Self {
        Self {
            device_context,
            graphics_queue,
            descriptor_pool,
            pipeline_manager,
            transient_commandbuffer_allocator,
            transient_buffer_allocator,
            static_buffer,
            herd,
            bump,
            picking_manager,
            debug_display,
            manipulator_drawables,
            egui,
            persistent_descriptor_set: None,
            frame_descriptor_set: None,
            view_descriptor_set: None,
        }
    }

    #[allow(clippy::todo)]
    pub fn write_descriptor_set(
        &self,
        layout: &DescriptorSetLayout,
        descriptors: &[DescriptorRef],
    ) -> DescriptorSetHandle {
        self.descriptor_pool
            .write_descriptor_set(layout, descriptors)
    }

    pub fn persistent_descriptor_set(&self) -> (&DescriptorSetLayout, DescriptorSetHandle) {
        self.persistent_descriptor_set.unwrap()
    }

    pub fn set_persistent_descriptor_set(
        &mut self,
        layout: &'frame DescriptorSetLayout,
        handle: DescriptorSetHandle,
    ) {
        self.persistent_descriptor_set = Some((layout, handle));
    }

    pub fn frame_descriptor_set(&self) -> (&DescriptorSetLayout, DescriptorSetHandle) {
        self.frame_descriptor_set.unwrap()
    }

    pub fn set_frame_descriptor_set(
        &mut self,
        layout: &'frame DescriptorSetLayout,
        handle: DescriptorSetHandle,
    ) {
        self.frame_descriptor_set = Some((layout, handle));
    }

    pub fn view_descriptor_set(&self) -> (&DescriptorSetLayout, DescriptorSetHandle) {
        self.view_descriptor_set.unwrap()
    }

    pub fn set_view_descriptor_set(
        &mut self,
        layout: &'frame DescriptorSetLayout,
        handle: DescriptorSetHandle,
    ) {
        self.view_descriptor_set = Some((layout, handle));
    }

    pub fn bind_default_descriptor_sets(&self, cmd_buffer: &mut CommandBuffer) {
        cmd_buffer.cmd_bind_descriptor_set_handle(
            self.persistent_descriptor_set.unwrap().0,
            self.persistent_descriptor_set.unwrap().1,
        );
        cmd_buffer.cmd_bind_descriptor_set_handle(
            self.frame_descriptor_set.unwrap().0,
            self.frame_descriptor_set.unwrap().1,
        );
        cmd_buffer.cmd_bind_descriptor_set_handle(
            self.view_descriptor_set.unwrap().0,
            self.view_descriptor_set.unwrap().1,
        );
    }
}
