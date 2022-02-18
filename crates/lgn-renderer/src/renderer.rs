#![allow(unsafe_code)]

use lgn_core::Handle;
use lgn_graphics_api::Queue;
use lgn_graphics_api::{
    ApiDef, BufferView, DescriptorHeap, DescriptorHeapDef, DeviceContext, Fence, FenceStatus,
    GfxApi, QueueType, Semaphore,
};

use lgn_graphics_cgen_runtime::CGenRegistryList;
use lgn_tracing::span_fn;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};

use crate::cgen::cgen_type::{DirectionalLight, OmniDirectionalLight, SpotLight};

use crate::debug_display::DebugDisplay;
use crate::resources::{
    CommandBufferPool, CommandBufferPoolHandle, DescriptorPool, DescriptorPoolHandle, GpuSafePool,
    MeshManager, PipelineManager, TransientPagedBuffer, UnifiedStaticBuffer, UniformGPUData,
};
use crate::RenderContext;

pub struct Renderer {
    frame_idx: usize,
    render_frame_idx: usize,
    num_render_frames: usize,
    prev_frame_sems: Vec<Semaphore>,
    sparse_unbind_sems: Vec<Semaphore>,
    sparse_bind_sems: Vec<Semaphore>,
    frame_fences: Vec<Fence>,
    graphics_queue: RwLock<Queue>,
    descriptor_heap: DescriptorHeap,
    command_buffer_pools: Mutex<GpuSafePool<CommandBufferPool>>,
    descriptor_pools: Mutex<GpuSafePool<DescriptorPool>>,
    transient_buffer: TransientPagedBuffer,
    static_buffer: UnifiedStaticBuffer,
    omnidirectional_lights_data: OmniDirectionalLightsStaticBuffer,
    directional_lights_data: DirectionalLightsStaticBuffer,
    spotlights_data: SpotLightsStaticBuffer,
    pipeline_manager: PipelineManager,
    mesh_manager: MeshManager,
    cgen_registry_list: CGenRegistryList,
    debug_display: DebugDisplay,
    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

pub type OmniDirectionalLightsStaticBuffer = Handle<UniformGPUData<OmniDirectionalLight>>;
pub type DirectionalLightsStaticBuffer = Handle<UniformGPUData<DirectionalLight>>;
pub type SpotLightsStaticBuffer = Handle<UniformGPUData<SpotLight>>;

macro_rules! impl_static_buffer_accessor {
    ($name:ident, $buffer_type:ty, $type:ty) => {
        paste::paste! {
            pub fn [<acquire_ $name>](&mut self) -> $buffer_type {
                self.$name.transfer()
            }
            pub fn [<release_ $name>](&mut self, $name: $buffer_type) {
                self.$name = $name;
            }
            pub fn [<$name _structured_buffer_view>](&self) -> BufferView{
                self.$name.structured_buffer_view($type::SIZE as u64)
            }
        }
    };
}

impl Renderer {
    pub fn new() -> Self {
        #![allow(unsafe_code)]
        let num_render_frames = 2usize;
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();
        let static_buffer = UnifiedStaticBuffer::new(device_context, 64 * 1024 * 1024, false);
        let omnidirectional_lights_data =
            OmniDirectionalLightsStaticBuffer::new(UniformGPUData::<OmniDirectionalLight>::new(
                &static_buffer,
                OmniDirectionalLight::PAGE_SIZE,
            ));
        let directional_lights_data =
            DirectionalLightsStaticBuffer::new(UniformGPUData::<DirectionalLight>::new(
                &static_buffer,
                DirectionalLight::PAGE_SIZE,
            ));
        let spotlights_data = SpotLightsStaticBuffer::new(UniformGPUData::<SpotLight>::new(
            &static_buffer,
            SpotLight::PAGE_SIZE,
        ));
        let descriptor_heap_def = DescriptorHeapDef {
            max_descriptor_sets: 32 * 4096,
            sampler_count: 32 * 128,
            constant_buffer_count: 32 * 1024,
            buffer_count: 32 * 1024,
            rw_buffer_count: 32 * 1024,
            texture_count: 32 * 1024,
            rw_texture_count: 32 * 1024,
        };
        let transient_buffer = TransientPagedBuffer::new(device_context, 512, 64 * 1024);
        let pipeline_manager = PipelineManager::new(device_context);
        let mesh_manager = MeshManager::new(&static_buffer, &transient_buffer);
        let cgen_registry_list = CGenRegistryList::new();
        let debug_display = DebugDisplay::default();

        Self {
            frame_idx: 0,
            render_frame_idx: 0,
            num_render_frames,
            prev_frame_sems: (0..num_render_frames)
                .map(|_| device_context.create_semaphore())
                .collect(),
            sparse_unbind_sems: (0..num_render_frames)
                .map(|_| device_context.create_semaphore())
                .collect(),
            sparse_bind_sems: (0..num_render_frames)
                .map(|_| device_context.create_semaphore())
                .collect(),
            frame_fences: (0..num_render_frames)
                .map(|_| device_context.create_fence().unwrap())
                .collect(),
            graphics_queue: RwLock::new(device_context.create_queue(QueueType::Graphics).unwrap()),
            descriptor_heap: device_context
                .create_descriptor_heap(&descriptor_heap_def)
                .unwrap(),
            command_buffer_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
            descriptor_pools: Mutex::new(GpuSafePool::new(num_render_frames)),
            transient_buffer,
            static_buffer,
            omnidirectional_lights_data,
            directional_lights_data,
            spotlights_data,
            pipeline_manager,
            mesh_manager,
            cgen_registry_list,
            debug_display,
            api,
        }
    }

    pub fn device_context(&self) -> &DeviceContext {
        self.api.device_context()
    }

    pub fn num_render_frames(&self) -> usize {
        self.num_render_frames
    }

    pub fn render_frame_idx(&self) -> usize {
        self.render_frame_idx
    }

    pub fn pipeline_manager(&self) -> &PipelineManager {
        &self.pipeline_manager
    }

    pub fn pipeline_manager_mut(&mut self) -> &mut PipelineManager {
        &mut self.pipeline_manager
    }

    pub fn cgen_registry_list(&self) -> &CGenRegistryList {
        &self.cgen_registry_list
    }

    pub fn cgen_registry_list_mut(&mut self) -> &mut CGenRegistryList {
        &mut self.cgen_registry_list
    }

    pub fn mesh_manager(&self) -> &MeshManager {
        &self.mesh_manager
    }

    pub fn debug_display(&self) -> &DebugDisplay {
        &self.debug_display
    }

    pub fn graphics_queue_guard(&self, queue_type: QueueType) -> RwLockReadGuard<'_, Queue> {
        match queue_type {
            QueueType::Graphics => self.graphics_queue.read(),
            _ => unreachable!(),
        }
    }

    // TMP: change that.
    pub(crate) fn transient_buffer(&self) -> &TransientPagedBuffer {
        &self.transient_buffer
    }

    impl_static_buffer_accessor!(
        omnidirectional_lights_data,
        OmniDirectionalLightsStaticBuffer,
        OmniDirectionalLight
    );

    impl_static_buffer_accessor!(
        directional_lights_data,
        DirectionalLightsStaticBuffer,
        DirectionalLight
    );

    impl_static_buffer_accessor!(spotlights_data, SpotLightsStaticBuffer, SpotLight);

    pub fn static_buffer(&self) -> &UnifiedStaticBuffer {
        &self.static_buffer
    }

    #[span_fn]
    pub fn flush_update_jobs(&self, render_context: &RenderContext<'_>) {
        let prev_frame_semaphore = &self.prev_frame_sems[self.render_frame_idx];
        let unbind_semaphore = &self.sparse_unbind_sems[self.render_frame_idx];
        let bind_semaphore = &self.sparse_bind_sems[self.render_frame_idx];

        self.static_buffer.flush_updater(
            prev_frame_semaphore,
            unbind_semaphore,
            bind_semaphore,
            render_context,
        );
    }

    pub fn static_buffer_ro_view(&self) -> BufferView {
        self.static_buffer.read_only_view()
    }

    //    pub fn prev_frame_semaphore(&self)

    pub(crate) fn acquire_command_buffer_pool(
        &self,
        queue_type: QueueType,
    ) -> CommandBufferPoolHandle {
        let queue = self.graphics_queue_guard(queue_type);
        let mut pool = self.command_buffer_pools.lock();
        pool.acquire_or_create(|| CommandBufferPool::new(&*queue))
    }

    pub(crate) fn release_command_buffer_pool(&self, handle: CommandBufferPoolHandle) {
        let mut pool = self.command_buffer_pools.lock();
        pool.release(handle);
    }

    pub(crate) fn acquire_descriptor_pool(
        &self,
        heap_def: &DescriptorHeapDef,
    ) -> DescriptorPoolHandle {
        let mut pool = self.descriptor_pools.lock();
        pool.acquire_or_create(|| DescriptorPool::new(self.descriptor_heap.clone(), heap_def))
    }

    pub(crate) fn release_descriptor_pool(&self, handle: DescriptorPoolHandle) {
        let mut pool = self.descriptor_pools.lock();
        pool.release(handle);
    }

    #[span_fn]
    pub(crate) fn begin_frame(&mut self) {
        //
        // Update frame indices
        //
        self.frame_idx += 1;
        self.render_frame_idx = self.frame_idx % self.num_render_frames;

        //
        // Wait for the next cpu frame to be available
        //
        let signal_fence = &self.frame_fences[self.render_frame_idx];
        if signal_fence.get_fence_status().unwrap() == FenceStatus::Incomplete {
            signal_fence.wait().unwrap();
        }

        //
        // Now, it is safe to free memory
        //
        let device_context = self.api.device_context();
        device_context.free_gpu_memory();

        //
        // Broadcast begin frame event
        //
        {
            let mut pool = self.command_buffer_pools.lock();
            pool.begin_frame();
        }
        {
            let mut pool = self.descriptor_pools.lock();
            pool.begin_frame();
        }

        // TMP: todo
        self.transient_buffer.begin_frame();
    }

    #[span_fn]
    pub(crate) fn end_frame(&mut self) {
        self.debug_display.end_frame();

        let graphics_queue = self.graphics_queue.write();
        let frame_fence = &self.frame_fences[self.render_frame_idx];

        graphics_queue
            .submit(&[], &[], &[], Some(frame_fence))
            .unwrap();

        //
        // Broadcast end frame event
        //

        {
            let mut pool = self.command_buffer_pools.lock();
            pool.end_frame();
        }
        {
            let mut pool = self.descriptor_pools.lock();
            pool.end_frame();
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        {
            let graphics_queue = self.graphics_queue_guard(QueueType::Graphics);
            graphics_queue.wait_for_queue_idle().unwrap();
        }
        std::mem::drop(self.spotlights_data.take());
        std::mem::drop(self.directional_lights_data.take());
        std::mem::drop(self.omnidirectional_lights_data.take());
    }
}
