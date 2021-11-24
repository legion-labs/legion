use crate::MemoryUsage;

pub struct MemoryAllocationDef {
    pub memory_usage: MemoryUsage,
    pub always_mapped: bool,
}

impl Default for MemoryAllocationDef {
    fn default() -> Self {
        Self {
            size: 0,
            memory_usage: MemoryUsage::Unknown,
            queue_type: QueueType::Graphics,
            always_mapped: false,
            usage_flags: ResourceUsage::empty(),
        }
    }
}

struct MemoryAllocation {}
