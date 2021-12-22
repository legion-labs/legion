use bumpalo::Bump;
use lgn_graphics_api::{
    BufferView, DescriptorRef, DescriptorSetLayout, DescriptorSetWriter, ShaderResourceType,
};
use lgn_graphics_cgen_runtime::{CGenDescriptorSetDef};

pub struct DescriptorSetData<'rc> {
    info: &'rc CGenDescriptorSetDef,
    descriptor_refs: &'rc mut [DescriptorRef<'rc>],
}

impl<'rc> DescriptorSetData<'rc> {
    pub fn new(info: &'rc CGenDescriptorSetDef, bump: &'rc Bump) -> Self {
        Self {
            info,
            descriptor_refs: bump
                .alloc_slice_fill_default::<DescriptorRef>(info.descriptor_flat_count as usize),
        }
    }

    pub fn id(&self) -> u32 {
        self.info.id
    }

    pub fn frequency(&self) -> u32 {
        self.info.frequency
    }

    pub fn set_constant_buffer(&mut self, id: u32, const_buffer_view: &'rc BufferView) {
        let descriptor_index = id;
        let descriptor_def = &self.info.descriptor_defs[descriptor_index as usize];
        assert_eq!(
            descriptor_def.shader_resource_type,
            ShaderResourceType::ConstantBuffer
        );
        self.descriptor_refs[descriptor_def.flat_index as usize] =
            DescriptorRef::BufferView(const_buffer_view);
    }

    pub fn kick(&self) {
        // writer: DescriptorSetWriter<'rc>,
        //     writer: descriptor_heap_partition
        //         .write_descriptor_set(descriptor_set_layout, bump)
        //         .unwrap(),
        // self.writer
        //     .set_descriptors_by_index(
        //         descriptor_index,
        //         &[DescriptorRef::BufferView(const_buffer_view)],
        //     )
        //     .unwrap();
    }
}
