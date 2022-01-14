// This is generated file. Do not edit manually

#[allow(unused_imports)]
use lgn_graphics_api::{
    BufferView, DescriptorRef, DescriptorSetDataProvider, DescriptorSetLayout, DeviceContext,
    Sampler, ShaderResourceType, TextureView,
};

#[allow(unused_imports)]
use lgn_graphics_cgen_runtime::{CGenDescriptorDef, CGenDescriptorSetDef, CGenDescriptorSetInfo};

static DESCRIPTOR_DEFS: [CGenDescriptorDef; 2] = [
    CGenDescriptorDef {
        name: "font_texture",
        shader_resource_type: ShaderResourceType::Texture2D,
        flat_index_start: 0,
        flat_index_end: 1,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "font_sampler",
        shader_resource_type: ShaderResourceType::Sampler,
        flat_index_start: 1,
        flat_index_end: 2,
        array_size: 0,
    },
];

static DESCRIPTOR_SET_DEF: CGenDescriptorSetDef = CGenDescriptorSetDef {
    name: "EguiDescriptorSet",
    id: 2,
    frequency: 0,
    descriptor_flat_count: 2,
    descriptor_defs: &DESCRIPTOR_DEFS,
};

static mut DESCRIPTOR_SET_LAYOUT: Option<DescriptorSetLayout> = None;

pub struct EguiDescriptorSet<'a> {
    descriptor_refs: [DescriptorRef<'a>; 2],
}

impl<'a> EguiDescriptorSet<'a> {
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
        2
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

    pub fn set_font_texture(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[0].validate(value));
        self.descriptor_refs[0] = DescriptorRef::TextureView(value);
    }

    pub fn set_font_sampler(&mut self, value: &'a Sampler) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[1].validate(value));
        self.descriptor_refs[1] = DescriptorRef::Sampler(value);
    }
}

impl<'a> Default for EguiDescriptorSet<'a> {
    fn default() -> Self {
        Self {
            descriptor_refs: [DescriptorRef::<'a>::default(); 2],
        }
    }
}

impl<'a> DescriptorSetDataProvider for EguiDescriptorSet<'a> {
    fn layout(&self) -> &'static DescriptorSetLayout {
        Self::descriptor_set_layout()
    }

    fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'a>] {
        &self.descriptor_refs[DESCRIPTOR_DEFS[descriptor_index].flat_index_start as usize
            ..DESCRIPTOR_DEFS[descriptor_index].flat_index_end as usize]
    }
}
