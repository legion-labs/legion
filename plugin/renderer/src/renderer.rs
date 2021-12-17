#![allow(unsafe_code)]

use anyhow::Result;
use lgn_graphics_api::Queue;
use lgn_graphics_api::{
    ApiDef, BufferView, DescriptorHeap, DescriptorHeapDef, DeviceContext, Fence, FenceStatus,
    GfxApi, QueueType, Semaphore,
};
use lgn_graphics_cgen_runtime::CGenRuntime;

use lgn_pso_compiler::{FileSystem, HlslCompiler};

use parking_lot::{Mutex, RwLock, RwLockReadGuard};

    LightComponent, LightSettings, LightType, PickedComponent, RenderSurface, StaticMesh,
};
use crate::memory::{BumpAllocator, BumpAllocatorHandle};
use crate::resources::{
    CommandBufferPool, CommandBufferPoolHandle, CpuPool, DescriptorPool, DescriptorPoolHandle,
    EntityTransforms, GpuSafePool, TestStaticBuffer, TransientPagedBuffer, UnifiedStaticBuffer,
    UniformGPUData, UniformGPUDataUploadJobBlock,
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
    cgen_runtime: CGenRuntime,
    static_buffer: UnifiedStaticBuffer,
    // Temp for testing
    test_transform_data: TestStaticBuffer,
    bump_allocator_pool: Mutex<CpuPool<BumpAllocator>>,
    shader_compiler: HlslCompiler,
    // This should be last, as it must be destroyed last.
    api: GfxApi,
}

unsafe impl Send for Renderer {}

unsafe impl Sync for Renderer {}

impl Renderer {
    pub fn new() -> Result<Self> {
        #![allow(unsafe_code)]
        let num_render_frames = 2usize;
        let api = unsafe { GfxApi::new(&ApiDef::default()).unwrap() };
        let device_context = api.device_context();
        let filesystem = FileSystem::new("d:\\")?;
        filesystem.add_mount_point("renderer", env!("CARGO_MANIFEST_DIR"))?;

        let shader_compiler = HlslCompiler::new(filesystem).unwrap();

        // this is not compliant with the rules set for code generation, aka no binary files
        // TODO fix this
        let cgen_def = include_bytes!(concat!(env!("OUT_DIR"), "/codegen/cgen/blob/cgen_def.blob"));
        let cgen_runtime = CGenRuntime::new(cgen_def, device_context);
        let static_buffer = UnifiedStaticBuffer::new(device_context, 64 * 1024 * 1024, false);
        let test_transform_data = TestStaticBuffer::new(UniformGPUData::<EntityTransforms>::new(
            &static_buffer,
            64 * 1024,
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

        Ok(Self {
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
            cgen_runtime,
            transient_buffer: TransientPagedBuffer::new(device_context, 128, 64 * 1024),
            static_buffer,
            test_transform_data,
            bump_allocator_pool: Mutex::new(CpuPool::new()),
            shader_compiler,
            api,
        })
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

    pub fn graphics_queue_guard(&self, queue_type: QueueType) -> RwLockReadGuard<'_, Queue> {
        match queue_type {
            QueueType::Graphics => self.graphics_queue.read(),
            _ => unreachable!(),
        }
    }

    pub fn shader_compiler(&self) -> HlslCompiler {
        self.shader_compiler.clone()
    }

    // TMP: change that.
    pub(crate) fn transient_buffer(&self) -> TransientPagedBuffer {
        self.transient_buffer.clone()
    }

    pub fn aquire_transform_data(&mut self) -> TestStaticBuffer {
        self.test_transform_data.transfer()
    }

    pub fn release_transform_data(&mut self, test: TestStaticBuffer) {
        self.test_transform_data = test;
    }

    pub fn static_buffer(&self) -> &UnifiedStaticBuffer {
        &self.static_buffer
    }

    pub fn test_add_update_jobs(&self, job_blocks: &mut Vec<UniformGPUDataUploadJobBlock>) {
        self.static_buffer.add_update_job_block(job_blocks);
    }

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

    pub(crate) fn cgen_runtime(&self) -> &CGenRuntime {
        &self.cgen_runtime
    }

    pub(crate) fn release_descriptor_pool(&self, handle: DescriptorPoolHandle) {
        let mut pool = self.descriptor_pools.lock();
        pool.release(handle);
    }

    pub(crate) fn acquire_bump_allocator(&self) -> BumpAllocatorHandle {
        let mut pool = self.bump_allocator_pool.lock();
        pool.acquire_or_create(BumpAllocator::new)
    }

    pub(crate) fn release_bump_allocator(&self, handle: BumpAllocatorHandle) {
        let mut pool = self.bump_allocator_pool.lock();
        pool.release(handle);
    }

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
        device_context.free_gpu_memory().unwrap();

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
        {
            let mut pool = self.bump_allocator_pool.lock();
            pool.begin_frame();
        }

        // TMP: todo
        self.transient_buffer.begin_frame();
    }

    pub(crate) fn end_frame(&mut self) {
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
        {
            let mut pool = self.bump_allocator_pool.lock();
            pool.end_frame();
        }
    }

        (shader, root_signature)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        std::mem::drop(self.test_transform_data.take());

        let graphics_queue = self.queue(QueueType::Graphics);
        graphics_queue.wait_for_queue_idle().unwrap();
    }
}

pub struct TmpRenderPass {
    static_meshes: Vec<StaticMeshRenderData>,
    root_signature: RootSignature,
    pipeline: Pipeline,
    pub color: [f32; 4],
    pub speed: f32,
}

impl TmpRenderPass {
    #![allow(clippy::too_many_lines)]
    pub fn new(renderer: &Renderer) -> Self {
        let device_context = renderer.device_context();

        let (shader, root_signature) =
            renderer.prepare_vs_ps(String::from("crate://renderer/shaders/shader.hlsl"));

                    cull_mode: CullMode::Back,
                    ..RasterizerState::default()
                },
            StaticMeshRenderData::new_sphere(0.25, 20, 20),
    #[allow(clippy::too_many_arguments)]
        lights: &[(&Transform, &LightComponent)],
        light_settings: &LightSettings,
        const NUM_LIGHTS: usize = 8;

        // Lights
        let mut directional_lights_data = Vec::<f32>::with_capacity(32 * NUM_LIGHTS);
        let mut omnidirectional_lights_data = Vec::<f32>::with_capacity(32 * NUM_LIGHTS);
        let mut spotlights_data = Vec::<f32>::with_capacity(64 * NUM_LIGHTS);
        let mut num_directional_lights = 0;
        let mut num_omnidirectional_lights = 0;
        let mut num_spotlights = 0;
        for (transform, light) in lights {
            if !light.enabled {
                continue;
            }
            match light.light_type {
                LightType::Directional { direction } => {
                    let direction_in_view = view_matrix.mul_vec4(direction.extend(0.0));

                    directional_lights_data.push(direction_in_view.x);
                    directional_lights_data.push(direction_in_view.y);
                    directional_lights_data.push(direction_in_view.z);
                    directional_lights_data.push(light.radiance);
                    directional_lights_data.push(light.color.0);
                    directional_lights_data.push(light.color.1);
                    directional_lights_data.push(light.color.2);
                    directional_lights_data.push(0.0);
                    num_directional_lights += 1;
                }
                LightType::Omnidirectional { attenuation } => {
                    let transform_in_view = view_matrix.mul_vec4(transform.translation.extend(1.0));

                    omnidirectional_lights_data.push(transform_in_view.x);
                    omnidirectional_lights_data.push(transform_in_view.y);
                    omnidirectional_lights_data.push(transform_in_view.z);
                    omnidirectional_lights_data.push(light.radiance);
                    omnidirectional_lights_data.push(attenuation);
                    omnidirectional_lights_data.push(light.color.0);
                    omnidirectional_lights_data.push(light.color.1);
                    omnidirectional_lights_data.push(light.color.2);
                    num_omnidirectional_lights += 1;
                }
                LightType::Spotlight {
                    direction,
                    cone_angle,
                    attenuation,
                } => {
                    let transform_in_view = view_matrix.mul_vec4(transform.translation.extend(1.0));
                    let direction_in_view = view_matrix.mul_vec4(direction.extend(0.0));

                    spotlights_data.push(transform_in_view.x);
                    spotlights_data.push(transform_in_view.y);
                    spotlights_data.push(transform_in_view.z);
                    spotlights_data.push(light.radiance);
                    spotlights_data.push(direction_in_view.x);
                    spotlights_data.push(direction_in_view.y);
                    spotlights_data.push(direction_in_view.z);
                    spotlights_data.push(cone_angle);
                    spotlights_data.push(attenuation);
                    spotlights_data.push(light.color.0);
                    spotlights_data.push(light.color.1);
                    spotlights_data.push(light.color.2);
                    num_spotlights += 1;
                    unsafe {
                        spotlights_data.set_len(64 * num_spotlights as usize);
                    }
                }
            }
        }
        unsafe {
            directional_lights_data.set_len(32 * NUM_LIGHTS);
            omnidirectional_lights_data.set_len(32 * NUM_LIGHTS);
            spotlights_data.set_len(64 * NUM_LIGHTS);
        }

        let directional_lights_buffer_view = transient_allocator
            .copy_data(&directional_lights_data, ResourceUsage::AS_SHADER_RESOURCE)
            .structured_buffer_view(32, true);

        let omnidirectional_lights_buffer_view = transient_allocator
            .copy_data(
                &omnidirectional_lights_data,
                ResourceUsage::AS_SHADER_RESOURCE,
            )
            .structured_buffer_view(32, true);

        let spotlights_buffer_view = transient_allocator
            .copy_data(&spotlights_data, ResourceUsage::AS_SHADER_RESOURCE)
            .structured_buffer_view(64, true);

            constant_data[36] = f32::from_bits(num_directional_lights);
            constant_data[37] = f32::from_bits(num_omnidirectional_lights);
            constant_data[38] = f32::from_bits(num_spotlights);
            constant_data[39] = f32::from_bits(light_settings.diffuse as u32);
            constant_data[40] = f32::from_bits(light_settings.specular as u32);
                )
                .unwrap();
            descriptor_set_writer
                .set_descriptors_by_name(
                    "directional_lights",
                    &[DescriptorRef::BufferView(&directional_lights_buffer_view)],
                )
                .unwrap();

            descriptor_set_writer
                .set_descriptors_by_name(
                    "omnidirectional_lights",
                    &[DescriptorRef::BufferView(
                        &omnidirectional_lights_buffer_view,
                    )],
                )
                .unwrap();

            descriptor_set_writer
                .set_descriptors_by_name(
                    "spotlights",
                    &[DescriptorRef::BufferView(&spotlights_buffer_view)],
            }