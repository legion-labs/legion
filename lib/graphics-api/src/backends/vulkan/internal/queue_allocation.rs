use std::sync::{Arc, Mutex};

use ash::vk;
use crossbeam_channel::{Receiver, Sender};
use fnv::FnvHashMap;
use lgn_tracing::{debug, warn};

use crate::{backends::vulkan::VkQueueFamilyIndices, DeviceContext};

/// Has the indexes for all the queue families we will need. It's possible that
/// a single queue family will need to be shared across these usages
///
/// The graphics queue ALWAYS supports transfer and compute operations. The
/// queue families chosen here will try to be "dedicated" families. Sharing
/// resources across families is complex and has overhead. It's completely
/// reasonable to use the graphics queue family for everything for many
/// applications.
///
/// Present queue is not here because if we need a dedicated present queue, it
/// will be found and used by the swapchain directly. There is a single global
/// lock for all dedicated present queues on `DeviceContext`.

pub struct VkQueueInner {
    device_context: DeviceContext,
    unallocated_queue: VkUnallocatedQueue,
    drop_tx: Sender<VkUnallocatedQueue>,
}

impl PartialEq for VkQueueInner {
    fn eq(&self, other: &Self) -> bool {
        self.unallocated_queue.inner.raw_queue == other.unallocated_queue.inner.raw_queue
    }
}

impl Drop for VkQueueInner {
    fn drop(&mut self) {
        self.drop_tx.send(self.unallocated_queue.clone()).unwrap();
    }
}

/// Represents a single queue within a family. These can be safely cloned/shared
/// but all queues must be dropped before dropping their owning device. The
/// queue has a lock so it is thread-safe
#[derive(Clone)]
pub struct VkQueue {
    inner: Arc<VkQueueInner>,
}

impl std::fmt::Debug for VkQueue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VkQueue")
            .field(
                "queue_family_index",
                &self.inner.unallocated_queue.inner.queue_family_index,
            )
            .field(
                "queue_family",
                &self.inner.unallocated_queue.inner.queue_index,
            )
            .field("handle", &self.inner.unallocated_queue.inner.raw_queue)
            .finish()
    }
}

impl VkQueue {
    pub fn queue(&self) -> &Mutex<vk::Queue> {
        &self.inner.unallocated_queue.inner.locked_queue
    }

    pub fn queue_family_index(&self) -> u32 {
        self.inner.unallocated_queue.inner.queue_family_index
    }

    pub(crate) fn device_context(&self) -> &DeviceContext {
        &self.inner.device_context
    }
}

// Intentionally doesn't hold a DeviceContext as this is indirectly held by
// DeviceContext and would create a cyclical reference
struct UnallocatedQueueInner {
    queue_family_index: u32,
    queue_index: u32,
    raw_queue: vk::Queue,
    locked_queue: Mutex<vk::Queue>,
}

#[derive(Clone)]
pub struct VkUnallocatedQueue {
    inner: Arc<UnallocatedQueueInner>,
}

impl VkUnallocatedQueue {
    pub fn new(device: &ash::Device, queue_family_index: u32, queue_index: u32) -> Self {
        let raw_queue = unsafe { device.get_device_queue(queue_family_index, queue_index) };
        let inner = UnallocatedQueueInner {
            queue_family_index,
            queue_index,
            raw_queue,
            locked_queue: Mutex::new(raw_queue),
        };

        Self {
            inner: Arc::new(inner),
        }
    }
}

#[derive(Debug)]
pub struct VkQueueAllocationConfig {
    pub allocation_strategy: VkQueueAllocationStrategy,
    pub queue_family_index: u32,
    pub first_queue_index: u32,
}

pub struct VkQueueAllocator {
    allocator_config: VkQueueAllocationConfig,
    available_queues: Vec<VkUnallocatedQueue>,
    drop_tx: Sender<VkUnallocatedQueue>,
    drop_rx: Receiver<VkUnallocatedQueue>,
}

impl VkQueueAllocator {
    pub fn new(
        allocator_config: VkQueueAllocationConfig,
        available_queues: Vec<VkUnallocatedQueue>,
    ) -> Self {
        let (drop_tx, drop_rx) = crossbeam_channel::unbounded();

        Self {
            allocator_config,
            available_queues,
            drop_tx,
            drop_rx,
        }
    }

    fn create_queue(
        &self,
        device_context: &DeviceContext,
        unallocated_queue: VkUnallocatedQueue,
    ) -> VkQueue {
        let inner = VkQueueInner {
            device_context: device_context.clone(),
            drop_tx: self.drop_tx.clone(),
            unallocated_queue,
        };

        VkQueue {
            inner: Arc::new(inner),
        }
    }

    pub fn allocate_queue(&mut self, device_context: &DeviceContext) -> Option<VkQueue> {
        if self.allocator_config.allocation_strategy
            == VkQueueAllocationStrategy::ShareFirstQueueInFamily
        {
            // Just wipe out anything that gets returned to us. We don't need these
            // notifications.
            if self.drop_rx.try_recv().is_ok() {
                // Not needed, all of these are instance of available_queues[0]
            }

            // Return the 0th (and only) queue
            Some(self.create_queue(device_context, self.available_queues[0].clone()))
        } else {
            // If we are notified of a queue no longer in use, return it to the pool
            if let Ok(free_queue_index) = self.drop_rx.try_recv() {
                self.available_queues.push(free_queue_index);
            }

            // Try to take a queue from the pool
            let unallocated_queue = self.available_queues.pop();
            unallocated_queue
                .map(|unallocated_queue| self.create_queue(device_context, unallocated_queue))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VkQueueAllocationStrategy {
    /// Find an appropriate queue family and use the 0th queue. Allocating a
    /// queue returns the same one each time. These instances are
    /// shared/ref-counted
    ShareFirstQueueInFamily,
    /// Set aside N queues and treat them as an allocated/freed resource.
    /// Allocation may fail if allocated queue count exceeds size of the
    /// pool
    Pool(u32),
}

pub struct VkQueueRequirements {
    // key: family index, value: queue index
    pub queue_counts: FnvHashMap<u32, u32>,
    pub graphics_allocation_config: VkQueueAllocationConfig,
    pub compute_allocation_config: VkQueueAllocationConfig,
    pub transfer_allocation_config: VkQueueAllocationConfig,
    pub decode_allocation_config: Option<VkQueueAllocationConfig>,
    pub encode_allocation_config: Option<VkQueueAllocationConfig>,
}

impl VkQueueRequirements {
    fn determine_queue_allocation_strategy(
        all_queue_families: &[ash::vk::QueueFamilyProperties],
        queue_counts: &mut FnvHashMap<u32, u32>,
        queue_family: u32,
        strategy: VkQueueAllocationStrategy,
    ) -> VkQueueAllocationConfig {
        if let VkQueueAllocationStrategy::Pool(count) = strategy {
            let count_in_family = queue_counts.entry(queue_family).or_insert(0);
            if *count_in_family + count <= all_queue_families[queue_family as usize].queue_count {
                // Increase queue_counts for this family and assign the next N queues
                let first_queue_index = *count_in_family;
                *count_in_family += count;

                // Success, bail out early
                return VkQueueAllocationConfig {
                    allocation_strategy: VkQueueAllocationStrategy::Pool(count),
                    queue_family_index: queue_family,
                    first_queue_index,
                };
            }
            warn!(
                "Not enough available queues in queue family {} to create pool of size {}. Falling back to ShareFirstQueueInFamily behavior",
                queue_family,
                count
            );
        }

        // Default safe behavior. Works as long as a queue exists.
        queue_counts.entry(queue_family).or_insert(1);
        VkQueueAllocationConfig {
            allocation_strategy: VkQueueAllocationStrategy::ShareFirstQueueInFamily,
            queue_family_index: queue_family,
            first_queue_index: 0,
        }
    }

    pub(crate) fn determine_required_queue_counts(
        queue_family_indices: &VkQueueFamilyIndices,
        all_queue_families: &[ash::vk::QueueFamilyProperties],
        graphics_allocation_strategy: VkQueueAllocationStrategy,
        queue_allocation_strategy: VkQueueAllocationStrategy,
        transfer_allocation_strategy: VkQueueAllocationStrategy,
        decode_allocation_strategy: VkQueueAllocationStrategy,
        encode_allocation_strategy: VkQueueAllocationStrategy,
    ) -> Self {
        debug!(
            "Determine required queue counts. Allocation strategies: Graphics: {:?}, Compute: {:?}, Transfer: {:?}",
            graphics_allocation_strategy,
            queue_allocation_strategy,
            transfer_allocation_strategy
        );
        debug!("Queue family indices: {:?}", queue_family_indices);
        debug!("Queue families: {:?}", all_queue_families);

        let mut queue_counts = FnvHashMap::default();
        let graphics_allocation_config = Self::determine_queue_allocation_strategy(
            all_queue_families,
            &mut queue_counts,
            queue_family_indices.graphics_queue_family_index,
            graphics_allocation_strategy,
        );
        let compute_allocation_config = Self::determine_queue_allocation_strategy(
            all_queue_families,
            &mut queue_counts,
            queue_family_indices.compute_queue_family_index,
            queue_allocation_strategy,
        );
        let transfer_allocation_config = Self::determine_queue_allocation_strategy(
            all_queue_families,
            &mut queue_counts,
            queue_family_indices.transfer_queue_family_index,
            transfer_allocation_strategy,
        );
        let decode_allocation_config =
            queue_family_indices
                .decode_queue_family_index
                .map(|queue_family| {
                    Self::determine_queue_allocation_strategy(
                        all_queue_families,
                        &mut queue_counts,
                        queue_family,
                        decode_allocation_strategy,
                    )
                });
        let encode_allocation_config =
            queue_family_indices
                .encode_queue_family_index
                .map(|queue_family| {
                    Self::determine_queue_allocation_strategy(
                        all_queue_families,
                        &mut queue_counts,
                        queue_family,
                        encode_allocation_strategy,
                    )
                });

        debug!("Queue counts: {:?}", queue_counts);
        debug!(
            "Graphics queue allocation config: {:?}",
            graphics_allocation_config
        );
        debug!(
            "Compute queue allocation config: {:?}",
            compute_allocation_config
        );
        debug!(
            "Transfer queue allocation config: {:?}",
            transfer_allocation_config
        );
        debug!(
            "Transfer queue allocation config: {:?}",
            decode_allocation_config
        );
        debug!(
            "Transfer queue allocation config: {:?}",
            encode_allocation_config
        );

        Self {
            queue_counts,
            graphics_allocation_config,
            compute_allocation_config,
            transfer_allocation_config,
            decode_allocation_config,
            encode_allocation_config,
        }
    }
}

/// Created by `VulkanApi`, provides logic for allocating/releasing queues
pub struct VkQueueAllocatorSet {
    graphics_queue_allocator: Mutex<VkQueueAllocator>,
    compute_queue_allocator: Mutex<VkQueueAllocator>,
    transfer_queue_allocator: Mutex<VkQueueAllocator>,
    decode_queue_allocator: Option<Mutex<VkQueueAllocator>>,
    encode_queue_allocator: Option<Mutex<VkQueueAllocator>>,
}

impl VkQueueAllocatorSet {
    pub fn new(
        device: &ash::Device,
        all_queue_families: &[ash::vk::QueueFamilyProperties],
        queue_requirements: VkQueueRequirements,
    ) -> Self {
        debug!("Creating queue allocators");

        // let mut queue_allocators = FnvHashMap::default();
        let mut all_queues = FnvHashMap::default();
        for (&queue_family_index, &queue_count) in &queue_requirements.queue_counts {
            assert!(queue_count <= all_queue_families[queue_family_index as usize].queue_count);

            let mut queues = Vec::with_capacity(queue_count as usize);
            for queue_index in 0..queue_count {
                queues.push(VkUnallocatedQueue::new(
                    device,
                    queue_family_index,
                    queue_index,
                ));
            }

            all_queues.insert(queue_family_index, queues);
        }

        fn create_allocator(
            all_queues: &FnvHashMap<u32, Vec<VkUnallocatedQueue>>,
            allocation_config: VkQueueAllocationConfig,
        ) -> Mutex<VkQueueAllocator> {
            let available_queues = match allocation_config.allocation_strategy {
                VkQueueAllocationStrategy::ShareFirstQueueInFamily => {
                    // Get the 0th queue in the queue family
                    vec![all_queues[&allocation_config.queue_family_index][0].clone()]
                }
                VkQueueAllocationStrategy::Pool(count) => {
                    let begin = allocation_config.first_queue_index as usize;
                    let end = (allocation_config.first_queue_index + count) as usize;

                    all_queues[&allocation_config.queue_family_index][begin..end].to_vec()
                }
            };

            Mutex::new(VkQueueAllocator::new(allocation_config, available_queues))
        }

        let graphics_queue_allocator =
            create_allocator(&all_queues, queue_requirements.graphics_allocation_config);
        let compute_queue_allocator =
            create_allocator(&all_queues, queue_requirements.compute_allocation_config);
        let transfer_queue_allocator =
            create_allocator(&all_queues, queue_requirements.transfer_allocation_config);
        let decode_queue_allocator = queue_requirements
            .decode_allocation_config
            .map(|allocation_config| create_allocator(&all_queues, allocation_config));
        let encode_queue_allocator = queue_requirements
            .encode_allocation_config
            .map(|allocation_config| create_allocator(&all_queues, allocation_config));

        Self {
            graphics_queue_allocator,
            compute_queue_allocator,
            transfer_queue_allocator,
            decode_queue_allocator,
            encode_queue_allocator,
        }
    }

    pub fn allocate_graphics_queue(&self, device_context: &DeviceContext) -> Option<VkQueue> {
        self.graphics_queue_allocator
            .lock()
            .unwrap()
            .allocate_queue(device_context)
    }

    pub fn allocate_compute_queue(&self, device_context: &DeviceContext) -> Option<VkQueue> {
        self.compute_queue_allocator
            .lock()
            .unwrap()
            .allocate_queue(device_context)
    }

    pub fn allocate_transfer_queue(&self, device_context: &DeviceContext) -> Option<VkQueue> {
        self.transfer_queue_allocator
            .lock()
            .unwrap()
            .allocate_queue(device_context)
    }

    pub fn allocate_decode_queue(&self, device_context: &DeviceContext) -> Option<VkQueue> {
        self.decode_queue_allocator
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .allocate_queue(device_context)
    }

    pub fn allocate_encode_queue(&self, device_context: &DeviceContext) -> Option<VkQueue> {
        self.encode_queue_allocator
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .allocate_queue(device_context)
    }
}
