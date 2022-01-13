// This is generated file. Do not edit manually

#[allow(unused_imports)]
use lgn_graphics_api::{
    BufferView, DescriptorRef, DescriptorSetDataProvider, DescriptorSetLayout, DeviceContext,
    Sampler, ShaderResourceType, TextureView,
};

#[allow(unused_imports)]
use lgn_graphics_cgen_runtime::{CGenDescriptorDef, CGenDescriptorSetDef, CGenDescriptorSetInfo};

static DESCRIPTOR_DEFS: [CGenDescriptorDef; 3] = [
    CGenDescriptorDef {
        name: "sb_omni_lights",
        shader_resource_type: ShaderResourceType::StructuredBuffer,
        flat_index_start: 0,
        flat_index_end: 1,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "sb_dir_lights",
        shader_resource_type: ShaderResourceType::StructuredBuffer,
        flat_index_start: 1,
        flat_index_end: 2,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "sb_spotlights",
        shader_resource_type: ShaderResourceType::StructuredBuffer,
        flat_index_start: 2,
        flat_index_end: 3,
        array_size: 0,
    },
];

static DESCRIPTOR_SET_DEF: CGenDescriptorSetDef = CGenDescriptorSetDef {
    name: "FrameDescriptorSet",
    id: 0,
    frequency: 0,
    descriptor_flat_count: 3,
    descriptor_defs: &DESCRIPTOR_DEFS,
};

static mut DESCRIPTOR_SET_LAYOUT: Option<DescriptorSetLayout> = None;

pub struct FrameDescriptorSet<'a> {
    descriptor_refs: [DescriptorRef<'a>; 3],
}

impl<'a> FrameDescriptorSet<'a> {
    #[allow(unsafe_code)]
    pub fn initialize(device_context: &DeviceContext) {
        unsafe {
            DESCRIPTOR_SET_LAYOUT =
                Some(DESCRIPTOR_SET_DEF.create_descriptor_set_layout(device_context));
        }
    }

    #[allow(unsafe_code)]
    pub fn shutdown() {
        unsafe {
            DESCRIPTOR_SET_LAYOUT = None;
        }
    }

    #[allow(unsafe_code)]
    pub fn descriptor_set_layout() -> &'static DescriptorSetLayout {
        unsafe {
            match &DESCRIPTOR_SET_LAYOUT {
                Some(dsl) => dsl,
                None => unreachable!(),
            }
        }
    }

    pub const fn id() -> u32 {
        0
    }

    pub const fn frequency() -> u32 {
        0
    }

    pub fn def() -> &'static CGenDescriptorSetDef {
        &DESCRIPTOR_SET_DEF
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_sb_omni_lights(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[0].validate(value));
        self.descriptor_refs[0] = DescriptorRef::BufferView(value);
    }

    pub fn set_sb_dir_lights(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[1].validate(value));
        self.descriptor_refs[1] = DescriptorRef::BufferView(value);
    }

    pub fn set_sb_spotlights(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[2].validate(value));
        self.descriptor_refs[2] = DescriptorRef::BufferView(value);
    }
}

impl<'a> Default for FrameDescriptorSet<'a> {
    fn default() -> Self {
        Self {
            descriptor_refs: [DescriptorRef::<'a>::default(); 3],
        }
    }
}

impl<'a> DescriptorSetDataProvider for FrameDescriptorSet<'a> {
    fn layout(&self) -> &'static DescriptorSetLayout {
        Self::descriptor_set_layout()
    }

    fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'a>] {
        &self.descriptor_refs[DESCRIPTOR_DEFS[descriptor_index].flat_index_start as usize
            ..DESCRIPTOR_DEFS[descriptor_index].flat_index_end as usize]
    }
}
