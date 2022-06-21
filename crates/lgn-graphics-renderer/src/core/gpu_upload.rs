use crossbeam_channel::{Receiver, SendError, Sender};
use lgn_core::{Handle, ObjectPool};
use lgn_tracing::{dispatch::init_thread_stream, span_scope};
use parking_lot::RwLock;
use slotmap::{DefaultKey, SlotMap};

use std::{
    collections::VecDeque,
    future::Future,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Poll, Waker},
    thread,
    time::Duration,
};

use lgn_graphics_api::prelude::*;

use crate::{
    resources::{
        IndexAllocator, TextureData, TransientBufferAllocator, TransientCommandBufferAllocator,
    },
    GraphicsQueue,
};

use super::{RenderCommand, RenderResources};

#[derive(thiserror::Error, Debug, Clone)]
pub enum TransferError {
    #[error("Transfer channel disconnected: {0}")]
    TransferChannelDisconnected(String),
}

impl<T> From<SendError<T>> for TransferError {
    fn from(err: SendError<T>) -> Self {
        Self::TransferChannelDisconnected(err.to_string())
    }
}

pub struct UploadGPUBuffer {
    pub src_data: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u64,
}

pub struct UploadGPUTexture {
    pub src_data: TextureData,
    pub dst_texture: Texture,
}

pub enum UploadGPUResource {
    Buffer(UploadGPUBuffer),
    Texture(UploadGPUTexture),
}

pub struct UploadBufferCommand {
    pub src_buffer: Vec<u8>,
    pub dst_buffer: Buffer,
    pub dst_offset: u64,
}

impl RenderCommand<RenderResources> for UploadBufferCommand {
    fn execute(self, render_resources: &RenderResources) {
        let mng = render_resources.get::<GpuUploadManager>();
        mng.push(UploadGPUResource::Buffer(UploadGPUBuffer {
            src_data: self.src_buffer,
            dst_buffer: self.dst_buffer,
            dst_offset: self.dst_offset,
        }));
    }
}

impl RenderCommand<GpuUploadManager> for UploadBufferCommand {
    fn execute(self, _gpu_upload_manager: &GpuUploadManager) {}
}

pub struct UploadTextureCommand {
    pub src_data: TextureData,
    pub dst_texture: Texture,
}

impl RenderCommand<RenderResources> for UploadTextureCommand {
    fn execute(self, render_resources: &RenderResources) {
        let mng = render_resources.get::<GpuUploadManager>();
        mng.push(UploadGPUResource::Texture(UploadGPUTexture {
            src_data: self.src_data,
            dst_texture: self.dst_texture,
        }));
    }
}

struct TransferRequest {
    client_resource: UploadGPUResource,
    completion_slot: DefaultKey,
}

pub struct TransferFuture {
    completion_slot: DefaultKey,
    inner: ArcInner,
}

impl Future for TransferFuture {
    type Output = Result<(), TransferError>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        {
            let slotmap = self.inner.slotmap.read();
            let completion_data = slotmap.get(self.completion_slot).unwrap();

            if completion_data.is_completed {
                return Poll::Ready(Ok(()));
            }
        }
        {
            let mut slotmap = self.inner.slotmap.write();
            let completion_data = slotmap.get_mut(self.completion_slot).unwrap();
            completion_data.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct TransferBufferId {
    bucket: u32,
    index: u32,
}

struct TransferBufferBucket {
    id: u32,
    device_context: DeviceContext,
    buffers: Vec<Buffer>,
    allocator: IndexAllocator,
}

impl TransferBufferBucket {
    fn new(id: u32, device_context: &DeviceContext) -> Self {
        Self {
            id,
            device_context: device_context.clone(),
            buffers: Vec::new(),
            allocator: IndexAllocator::new(256),
        }
    }

    fn acquire(&mut self) -> u32 {
        let index = self.allocator.allocate();
        if index as usize >= self.buffers.len() {
            let debug_name = "transfer_buffer";
            self.buffers.resize(
                index as usize + 1,
                self.device_context.create_buffer(
                    BufferDef {
                        size: self.bucket_size() as u64,
                        usage_flags: ResourceUsage::AS_TRANSFERABLE,
                        create_flags: BufferCreateFlags::empty(),
                        memory_usage: MemoryUsage::CpuToGpu,
                        always_mapped: true,
                    },
                    debug_name,
                ),
            );
        }
        index
    }

    fn release(&mut self, index: u32) {
        self.allocator.free(index);
    }

    fn buffer(&self, index: u32) -> &Buffer {
        &self.buffers[index as usize]
    }

    fn bucket_size(&self) -> usize {
        TransferBufferBucket::bucket_size_from_id(self.id)
    }

    fn bucket_size_from_id(id: u32) -> usize {
        1 << id
    }

    pub(crate) fn id_from_mem_requirement(mem_requirement: usize) -> u32 {
        let bucket_size = mem_requirement.next_power_of_two();
        let id = bucket_size.trailing_zeros();
        println!("{} : {} : {}", mem_requirement, bucket_size, id);
        id
    }
}

struct TransferBufferAllocator {
    buckets: Vec<TransferBufferBucket>,
    device_context: DeviceContext,
}

impl TransferBufferAllocator {
    fn new(device_context: &DeviceContext) -> Self {
        Self {
            buckets: Vec::new(),
            device_context: device_context.clone(),
        }
    }

    fn buffer(&self, id: &TransferBufferId) -> &Buffer {
        self.buckets[id.bucket as usize].buffer(id.index)
    }

    fn allocate(&mut self, memory_requirement: usize) -> TransferBufferId {
        assert!(memory_requirement > 0);

        let bucket_id = TransferBufferBucket::id_from_mem_requirement(memory_requirement);
        if bucket_id as usize >= self.buckets.len() {
            let additionnal_buckets = 1 + bucket_id as usize - self.buckets.len();
            if additionnal_buckets > 0 {
                self.buckets.reserve(additionnal_buckets);
                let mut next_id = self.buckets.len();
                for _ in 0..additionnal_buckets {
                    self.buckets.push(TransferBufferBucket::new(
                        u32::try_from(next_id).unwrap(),
                        &self.device_context,
                    ));
                    next_id += 1;
                }
            }
        }
        let bucket = &mut self.buckets[bucket_id as usize];

        TransferBufferId {
            bucket: bucket_id,
            index: bucket.acquire(),
        }
    }

    fn release(&mut self, id: TransferBufferId) {
        let bucket = &mut self.buckets[id.bucket as usize];
        bucket.release(id.index);
        drop(id);
    }
}

struct UploadItem {
    client_resource: UploadGPUResource,
    completion_slot: DefaultKey,
}

struct UploadBatch {
    upload_items: Vec<UploadItem>,
    cmd_buffer: Handle<CommandBuffer>,
    transfer_buffer_id: Option<TransferBufferId>,
    commit_gpu_epoch: Option<u64>,
}

struct CompletionData {
    is_completed: bool,
    waker: Option<Waker>,
}

struct Inner {
    device_context: DeviceContext,
    updates: RwLock<Vec<UploadGPUResource>>,
    sender: Sender<TransferRequest>,
    receiver: Receiver<TransferRequest>,
    slotmap: RwLock<SlotMap<DefaultKey, CompletionData>>,
    exit_required: AtomicBool,
}

type ArcInner = Arc<Inner>;

#[derive(Clone)]
pub struct GpuUploadManager {
    inner: ArcInner,
}

impl GpuUploadManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(10 * 1024);

        let inner = Arc::new(Inner {
            device_context: device_context.clone(),
            updates: RwLock::new(Vec::new()),
            sender,
            receiver,
            slotmap: RwLock::new(SlotMap::new()),
            exit_required: AtomicBool::new(false),
        });

        let thread_inner = inner.clone();

        thread::Builder::new()
            .name("TransferThread".to_string())
            .spawn(move || {
                Self::transfer_thread(thread_inner);
            })
            .unwrap();

        Self { inner }
    }

    pub fn push(&self, update: UploadGPUResource) {
        let mut updates = self.inner.updates.write();
        updates.push(update);
    }

    pub fn async_upload(&self, upload: UploadGPUResource) -> Result<TransferFuture, TransferError> {
        let completion_slot = {
            let mut slotmap = self.inner.slotmap.write();
            slotmap.insert(CompletionData {
                waker: None,
                is_completed: false,
            })
        };

        self.inner.sender.send(TransferRequest {
            client_resource: upload,
            completion_slot,
        })?;

        Ok(TransferFuture {
            inner: self.inner.clone(),
            completion_slot,
        })
    }

    pub fn upload(
        &self,
        transient_commandbuffer_allocator: &mut TransientCommandBufferAllocator,
        transient_buffer_allocator: &mut TransientBufferAllocator,
        graphics_queue: &GraphicsQueue,
    ) {
        let mut updates = self.inner.updates.write();

        if updates.is_empty() {
            return;
        }

        let mut cmd_buffer_handle = transient_commandbuffer_allocator.acquire();
        let cmd_buffer = cmd_buffer_handle.as_mut();

        cmd_buffer.begin();

        for update in updates.drain(..) {
            match update {
                UploadGPUResource::Buffer(upload_buf) => {
                    let transient_alloc = transient_buffer_allocator
                        .copy_data_slice(&upload_buf.src_data, ResourceUsage::empty());

                    cmd_buffer.cmd_resource_barrier(
                        &[BufferBarrier {
                            buffer: &upload_buf.dst_buffer,
                            src_state: ResourceState::SHADER_RESOURCE,
                            dst_state: ResourceState::COPY_DST,
                            queue_transition: BarrierQueueTransition::None,
                        }],
                        &[],
                    );

                    cmd_buffer.cmd_copy_buffer_to_buffer(
                        transient_alloc.buffer(),
                        &upload_buf.dst_buffer,
                        &[BufferCopy {
                            src_offset: transient_alloc.byte_offset(),
                            dst_offset: upload_buf.dst_offset,
                            size: upload_buf.src_data.len() as u64,
                        }],
                    );

                    cmd_buffer.cmd_resource_barrier(
                        &[BufferBarrier {
                            buffer: &upload_buf.dst_buffer,
                            src_state: ResourceState::COPY_DST,
                            dst_state: ResourceState::SHADER_RESOURCE,
                            queue_transition: BarrierQueueTransition::None,
                        }],
                        &[],
                    );
                }
                UploadGPUResource::Texture(upload_tex) => {
                    let texture = upload_tex.dst_texture;
                    let texture_data = upload_tex.src_data;

                    cmd_buffer.cmd_resource_barrier(
                        &[],
                        &[TextureBarrier::state_transition(
                            &texture,
                            ResourceState::UNDEFINED,
                            ResourceState::COPY_DST,
                        )],
                    );

                    for (mip_level, mip_data) in texture_data.mips().iter().enumerate() {
                        let transient_alloc = transient_buffer_allocator
                            .copy_data_slice(mip_data, ResourceUsage::empty());

                        cmd_buffer.cmd_copy_buffer_to_texture(
                            transient_alloc.buffer(),
                            &texture,
                            &CmdCopyBufferToTextureParams {
                                buffer_offset: transient_alloc.byte_offset(),
                                array_layer: 0,
                                mip_level: mip_level as u8,
                            },
                        );
                    }

                    cmd_buffer.cmd_resource_barrier(
                        &[],
                        &[TextureBarrier::state_transition(
                            &texture,
                            ResourceState::COPY_DST,
                            ResourceState::SHADER_RESOURCE,
                        )],
                    );
                }
            }
        }

        cmd_buffer.end();

        graphics_queue
            .queue_mut()
            .submit(&[cmd_buffer], &[], &[], None);

        transient_commandbuffer_allocator.release(cmd_buffer_handle);
    }

    fn transfer_thread(inner: ArcInner) {
        init_thread_stream();

        let mut transfer_queue = inner.device_context.create_queue(QueueType::Transfer);
        let mut command_pool =
            transfer_queue.create_command_pool(CommandPoolDef { transient: false });
        let mut transfer_buffer_allocator = TransferBufferAllocator::new(&inner.device_context);

        let mut pending_batches = VecDeque::<Handle<UploadBatch>>::new();
        let mut batch_pool = ObjectPool::<UploadBatch>::new();
        let mut commandbuffer_pool = ObjectPool::<CommandBuffer>::new();
        let mut next_gpu_epoch = 0;
        let timeline_sem = inner.device_context.create_semaphore(SemaphoreDef {
            usage_flags: SemaphoreUsage::TIMELINE,
            initial_value: next_gpu_epoch,
        });
        let upload_alignment = inner
            .device_context
            .device_info()
            .upload_buffer_texture_alignment as usize;
        let upload_align = |size| -> usize { (size + upload_alignment - 1) & !upload_alignment };

        loop {
            // 1) read gpu timestamp
            let gpu_epoch = timeline_sem.get_timeline_value();

            // 2) try flush pending batches
            while let Some(batch) = pending_batches.front_mut() {
                span_scope!("try flush pending scopes");
                if batch.commit_gpu_epoch.unwrap() <= gpu_epoch {
                    span_scope!("flush pending scope");
                    let mut batch = pending_batches.pop_front().unwrap();
                    let mut slotmap = inner.slotmap.write();
                    for item in batch.upload_items.drain(..) {
                        let mut completion_data = slotmap.get_mut(item.completion_slot).unwrap();
                        completion_data.is_completed = true;
                        completion_data.waker.iter().for_each(Waker::wake_by_ref);
                    }
                    commandbuffer_pool.release(batch.cmd_buffer.transfer());
                    transfer_buffer_allocator.release(batch.transfer_buffer_id.take().unwrap());
                    batch.commit_gpu_epoch = None;
                    batch_pool.release(batch);
                } else {
                    // We can leave this loop as we are sure that next scopes won't be processed by the gpu
                    break;
                }
            }

            // 3) if no new uploads
            if inner.receiver.is_empty() {
                if pending_batches.is_empty() {
                    if inner.exit_required.load(Ordering::SeqCst) {
                        break;
                    }
                    span_scope!("no pending batches");
                    thread::sleep(Duration::from_millis(2));
                }
                // Jump to 1)
                continue;
            }

            // 4) allocate a new batch
            let mut batch = batch_pool.acquire_or_create(|| UploadBatch {
                upload_items: Vec::new(),
                cmd_buffer: Handle::invalid(),
                commit_gpu_epoch: None,
                transfer_buffer_id: None,
            });

            // 5) setup this new batch
            next_gpu_epoch += 1;

            batch.cmd_buffer = commandbuffer_pool.acquire_or_create(|| {
                command_pool.create_command_buffer(CommandBufferDef {
                    is_secondary: false,
                })
            });
            batch.commit_gpu_epoch = Some(next_gpu_epoch);

            for transfer_request in inner.receiver.try_iter() {
                batch.upload_items.push(UploadItem {
                    client_resource: transfer_request.client_resource,
                    completion_slot: transfer_request.completion_slot,
                });
            }

            let mut requirement_capacity = 0;
            for item in &mut batch.upload_items {
                match &item.client_resource {
                    UploadGPUResource::Buffer(buf) => {
                        requirement_capacity += upload_align(buf.src_data.len());
                    }
                    UploadGPUResource::Texture(tex) => {
                        for i in 0..tex.src_data.mip_count() {
                            requirement_capacity += upload_align(tex.src_data.mips()[i].len());
                        }
                    }
                }
            }

            batch.transfer_buffer_id =
                Some(transfer_buffer_allocator.allocate(requirement_capacity));

            let transfer_buffer =
                transfer_buffer_allocator.buffer(batch.transfer_buffer_id.as_ref().unwrap());
            let mapped_ptr = transfer_buffer.mapped_ptr();

            let mut src_offset = 0;
            for item in &mut batch.upload_items {
                match &item.client_resource {
                    UploadGPUResource::Buffer(buf) => {
                        #[allow(unsafe_code)]
                        unsafe {
                            mapped_ptr.add(src_offset).copy_from_nonoverlapping(
                                buf.src_data.as_ptr(),
                                buf.src_data.len(),
                            );
                        };
                        src_offset += upload_align(buf.src_data.len());
                    }
                    UploadGPUResource::Texture(tex) => {
                        for i in 0..tex.src_data.mip_count() {
                            #[allow(unsafe_code)]
                            unsafe {
                                mapped_ptr.add(src_offset).copy_from_nonoverlapping(
                                    tex.src_data.mips()[i].as_ptr(),
                                    tex.src_data.mips()[i].len(),
                                );
                            };
                            src_offset += upload_align(tex.src_data.mips()[i].len());
                        }
                    }
                }
            }

            // 6) build command buffer
            let mut cmd_buffer = batch.cmd_buffer.transfer();
            let mut src_offset = 0;

            cmd_buffer.reset();
            cmd_buffer.begin();

            for item in &mut batch.upload_items {
                match &item.client_resource {
                    UploadGPUResource::Buffer(buf) => {
                        cmd_buffer.cmd_resource_barrier(
                            &[BufferBarrier {
                                buffer: &buf.dst_buffer,
                                src_state: ResourceState::UNDEFINED,
                                dst_state: ResourceState::COPY_DST,
                                queue_transition: BarrierQueueTransition::None,
                            }],
                            &[],
                        );

                        cmd_buffer.cmd_copy_buffer_to_buffer(
                            transfer_buffer,
                            &buf.dst_buffer,
                            &[BufferCopy {
                                src_offset: src_offset as u64,
                                dst_offset: buf.dst_offset,
                                size: buf.src_data.len() as u64,
                            }],
                        );

                        cmd_buffer.cmd_resource_barrier(
                            &[BufferBarrier {
                                buffer: &buf.dst_buffer,
                                src_state: ResourceState::COPY_DST,
                                dst_state: ResourceState::SHADER_RESOURCE,
                                queue_transition: BarrierQueueTransition::None,
                            }],
                            &[],
                        );

                        src_offset += upload_align(buf.src_data.len());
                    }
                    UploadGPUResource::Texture(tex) => {
                        cmd_buffer.cmd_resource_barrier(
                            &[],
                            &[TextureBarrier::state_transition(
                                &tex.dst_texture,
                                ResourceState::UNDEFINED,
                                ResourceState::COPY_DST,
                            )],
                        );

                        for mip_level in 0..tex.src_data.mip_count() {
                            cmd_buffer.cmd_copy_buffer_to_texture(
                                transfer_buffer,
                                &tex.dst_texture,
                                &CmdCopyBufferToTextureParams {
                                    buffer_offset: src_offset as u64,
                                    array_layer: 0,
                                    mip_level: mip_level as u8,
                                },
                            );
                            src_offset += upload_align(tex.src_data.mips()[mip_level].len());
                        }

                        cmd_buffer.cmd_resource_barrier(
                            &[],
                            &[TextureBarrier::state_transition(
                                &tex.dst_texture,
                                ResourceState::COPY_DST,
                                ResourceState::SHADER_RESOURCE,
                            )],
                        );
                    }
                }
            }
            assert_eq!(requirement_capacity, src_offset);

            cmd_buffer.end();

            timeline_sem.set_next_timeline_value(batch.commit_gpu_epoch.unwrap());

            transfer_queue.submit(&[&cmd_buffer], &[], &[&timeline_sem], None);

            batch.cmd_buffer = cmd_buffer.transfer();

            pending_batches.push_back(batch);
        }

        drop(inner);
    }
}

impl Drop for GpuUploadManager {
    fn drop(&mut self) {
        self.inner.exit_required.store(true, Ordering::SeqCst);
    }
}
