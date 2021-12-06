use std::{
    num::NonZeroU32,
    sync::{Arc, Mutex},
};

use graphics_api::{
    Buffer, BufferAllocation, BufferDef, ComputePipelineDef, DescriptorDef, DescriptorSetLayoutDef,
    DeviceContext, MemoryPagesAllocation, PagedBufferAllocation, Pipeline, PipelineType,
    PushConstantDef, Queue, QueueType, ResourceCreation, ResourceUsage, RootSignature,
    RootSignatureDef, Semaphore, ShaderPackage, ShaderStageDef, ShaderStageFlags,
    MAX_DESCRIPTOR_SET_LAYOUTS,
};

use legion_math::Mat4;
use legion_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource};

use super::{RangeAllocator, SparseBindingManager, TransientBufferAllocator, TransientPagedBuffer};

pub(crate) struct UnifiedStaticBufferInner {
    buffer: Buffer,
    segment_allocator: RangeAllocator,
    binding_manager: SparseBindingManager,
    page_size: u64,
}

#[derive(Clone)]
pub(crate) struct UnifiedStaticBuffer {
    inner: Arc<Mutex<UnifiedStaticBufferInner>>,
}

impl UnifiedStaticBuffer {
    pub fn new(device_context: &DeviceContext, virtual_buffer_size: u64) -> Self {
        let buffer_def = BufferDef {
            size: virtual_buffer_size,
            queue_type: QueueType::Graphics,
            usage_flags: ResourceUsage::AS_SHADER_RESOURCE,
            creation_flags: ResourceCreation::SPARSE_BINDING,
        };

        let buffer = device_context.create_buffer(&buffer_def);
        let required_alignment = buffer.required_alignment();

        assert!(virtual_buffer_size % required_alignment == 0);

        Self {
            inner: Arc::new(Mutex::new(UnifiedStaticBufferInner {
                buffer,
                segment_allocator: RangeAllocator::new(virtual_buffer_size / required_alignment),
                binding_manager: SparseBindingManager::new(),
                page_size: required_alignment,
            })),
        }
    }

    pub fn allocate_segment(&self, segment_size: u64) -> PagedBufferAllocation {
        let inner = &mut *self.inner.lock().unwrap();

        let page_size = inner.page_size;
        let page_count =
            legion_utils::memory::round_size_up_to_alignment_u64(segment_size, page_size)
                / page_size;

        let location = inner.segment_allocator.allocate(page_count).unwrap();
        let allocation = MemoryPagesAllocation::for_sparse_buffer(
            inner.buffer.device_context(),
            &inner.buffer,
            page_count,
        );

        let paged_allocation = PagedBufferAllocation {
            buffer: inner.buffer.clone(),
            memory: allocation,
            range: location,
        };

        inner
            .binding_manager
            .add_sparse_binding(paged_allocation.clone());

        paged_allocation
    }

    pub fn free_segment(&self, segment: PagedBufferAllocation) {
        let inner = &mut *self.inner.lock().unwrap();

        inner.segment_allocator.free(segment.range);
        inner.binding_manager.add_sparse_unbinding(segment);
    }

    pub fn commmit_segment_memory<'a>(
        &mut self,
        queue: &Queue,
        prev_frame_semaphore: &'a Semaphore,
        unbind_semaphore: &'a Semaphore,
        bind_semaphore: &'a Semaphore,
    ) -> &'a Semaphore {
        let inner = &mut *self.inner.lock().unwrap();

        inner.binding_manager.commmit_sparse_bindings(
            queue,
            prev_frame_semaphore,
            unbind_semaphore,
            bind_semaphore,
        )
    }
}

pub(crate) struct UnifiedStaticBufferUpdater {
    static_buffer: UnifiedStaticBuffer,
    root_signature: RootSignature,
    pipeline: Pipeline,
}

impl UnifiedStaticBufferUpdater {
    pub fn new(device_context: &DeviceContext, static_buffer: &UnifiedStaticBuffer) -> Self {
        //
        // shader
        //
        let shader_compiler = HlslCompiler::new().unwrap();

        let shader_source =
            String::from_utf8(include_bytes!("../../shaders/static_buffer_transfer.hlsl").to_vec())
                .unwrap();

        let shader_build_result = shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Code(shader_source),
                glob_defines: Vec::new(),
                entry_points: vec![EntryPoint {
                    defines: Vec::new(),
                    name: "main_cs".to_owned(),
                    target_profile: "vs_6_0".to_owned(),
                }],
            })
            .unwrap();

        let compute_shader_module = device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        let shader = device_context
            .create_shader(
                vec![ShaderStageDef {
                    entry_point: "main_cs".to_owned(),
                    shader_stage: ShaderStageFlags::COMPUTE,
                    shader_module: compute_shader_module,
                }],
                &shader_build_result.pipeline_reflection,
            )
            .unwrap();

        //
        // Root signature
        //
        let mut descriptor_set_layouts = Vec::new();
        for set_index in 0..MAX_DESCRIPTOR_SET_LAYOUTS {
            let shader_resources: Vec<_> = shader_build_result
                .pipeline_reflection
                .shader_resources
                .iter()
                .filter(|x| x.set_index as usize == set_index)
                .collect();

            if !shader_resources.is_empty() {
                let descriptor_defs = shader_resources
                    .iter()
                    .map(|sr| DescriptorDef {
                        name: sr.name.clone(),
                        binding: sr.binding,
                        shader_resource_type: sr.shader_resource_type,
                        array_size: sr.element_count,
                    })
                    .collect();

                let def = DescriptorSetLayoutDef {
                    frequency: set_index as u32,
                    descriptor_defs,
                };
                let descriptor_set_layout =
                    device_context.create_descriptorset_layout(&def).unwrap();
                descriptor_set_layouts.push(descriptor_set_layout);
            }
        }

        let root_signature_def = RootSignatureDef {
            pipeline_type: PipelineType::Graphics,
            descriptor_set_layouts: descriptor_set_layouts.clone(),
            push_constant_def: shader_build_result
                .pipeline_reflection
                .push_constant
                .map(|x| PushConstantDef {
                    used_in_shader_stages: x.used_in_shader_stages,
                    size: NonZeroU32::new(x.size).unwrap(),
                }),
        };

        let root_signature = device_context
            .create_root_signature(&root_signature_def)
            .unwrap();

        //
        // Pipeline state
        //

        let pipeline = device_context
            .create_compute_pipeline(&ComputePipelineDef {
                shader: &shader,
                root_signature: &root_signature,
            })
            .unwrap();

        Self {
            static_buffer: static_buffer.clone(),
            root_signature,
            pipeline,
        }
    }
}

enum UniformGPUDataType {
    EntityTransforms = 0,
}

struct EntityTransforms {
    world: Mat4,
}

struct UniformGPUData<T> {
    static_baffer: UnifiedStaticBuffer,
    allocated_pages: Vec<PagedBufferAllocation>,
    page_size: u64,
    element_size: u64,
    marker: std::marker::PhantomData<T>,
}

impl<T> UniformGPUData<T> {
    fn new(static_baffer: &UnifiedStaticBuffer, min_page_size: u64) -> Self {
        let page = static_baffer.allocate_segment(min_page_size);
        let page_size = page.size();
        Self {
            static_baffer: static_baffer.clone(),
            allocated_pages: vec![page],
            page_size,
            element_size: std::mem::size_of::<T>() as u64,
            marker: ::std::marker::PhantomData,
        }
    }

    fn alloc_offset_for_index(&mut self, index: u64) -> u64 {
        let elements_per_page = self.page_size / self.element_size;
        let required_pages = (index / elements_per_page) + 1;

        while (self.allocated_pages.len() as u64) < required_pages {
            self.allocated_pages
                .push(self.static_baffer.allocate_segment(self.page_size));
        }

        let index_of_page = index / elements_per_page;
        let page_in_index = index % elements_per_page;

        self.allocated_pages[index_of_page as usize].offset() + (page_in_index * self.element_size)
    }
}

struct UniformGPUDataUploadJob {
    src_offset: u32,
    dst_offset: u32,
    size: u32,
}

struct UniformGPUDataUploadJobBlock {
    upload_allocation: BufferAllocation,
    upload_jobs: Vec<UniformGPUDataUploadJob>,
    offset: u32,
}

impl UniformGPUDataUploadJobBlock {
    fn new(upload_allocation: BufferAllocation) -> Self {
        Self {
            upload_allocation,
            upload_jobs: Vec::new(),
            offset: 0,
        }
    }

    fn add_update_jobs<T>(&mut self, data: &[T], dst_offset: u32) -> bool {
        let upload_size_in_bytes = legion_utils::memory::slice_size_in_bytes(data) as u32;
        if self.offset + upload_size_in_bytes <= self.upload_allocation.size() as u32 {
            let src = data.as_ptr().cast::<u8>();
            {
                #[allow(unsafe_code)]
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        src,
                        self.upload_allocation
                            .memory
                            .mapped_ptr()
                            .add((self.upload_allocation.offset() as u32 + self.offset) as usize),
                        upload_size_in_bytes as usize,
                    );
                }
            }

            for i in 0..data.len() as u32 {
                let data_size = std::mem::size_of::<T>() as u32;
                self.upload_jobs.push(UniformGPUDataUploadJob {
                    src_offset: self.offset,
                    dst_offset: dst_offset + (i * data_size),
                    size: data_size,
                });
                self.offset += data_size;
            }
            true
        } else {
            false
        }
    }
}

struct UniformGPUDataUpdater {
    paged_buffer: TransientPagedBuffer,
    job_blocks: Vec<UniformGPUDataUploadJobBlock>,
    block_size: u64,
}

impl UniformGPUDataUpdater {
    fn new(paged_buffer: TransientPagedBuffer, block_size: u64) -> Self {
        Self {
            paged_buffer,
            job_blocks: Vec::new(),
            block_size,
        }
    }

    fn add_update_jobs<T>(&mut self, data: &[T], dst_offset: u32) {
        while self.job_blocks.is_empty()
            || !self
                .job_blocks
                .last_mut()
                .unwrap()
                .add_update_jobs(data, dst_offset)
        {
            self.job_blocks.push(UniformGPUDataUploadJobBlock::new(
                self.paged_buffer.allocate_page(self.block_size),
            ));
        }
    }
}
