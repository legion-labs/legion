use lgn_graphics_api::BufferView;
use lgn_graphics_cgen_runtime::CGenDescriptorSetDef;

pub struct DescriptorSetData {
    info: &'static CGenDescriptorSetDef,
}

impl DescriptorSetData {
    pub fn new(info: &'static CGenDescriptorSetDef) -> Self {
        Self { info }
    }

    pub fn frequency(&self) -> u32 {
        self.info.frequency
    }

    pub fn set_constant_buffer(&mut self, id: u32, const_buffer_view: &BufferView) {
        // let descriptor_index = id.to_index();
        // let descriptor_def = &self.cgen_descriptor_set_def.descriptor_defs[descriptor_index];
        // assert_eq!(
        //     descriptor_def.shader_resource_type,
        //     ShaderResourceType::ConstantBuffer
        // );

        // self.writer
        //     .set_descriptors_by_index(descriptor_index, &[DescriptorRef::BufferView(cbv)])
        //     .unwrap();
    }
}
