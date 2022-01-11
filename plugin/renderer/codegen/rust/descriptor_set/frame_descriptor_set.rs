// This is generated file. Do not edit manually

#[allow(unused_imports)]
use lgn_graphics_api::{
    BufferView, DescriptorRef, DescriptorSetDataProvider, DescriptorSetLayout, DeviceContext,
    Sampler, ShaderResourceType, TextureView,
};

#[allow(unused_imports)]
use lgn_graphics_cgen_runtime::{CGenDescriptorDef, CGenDescriptorSetDef, CGenDescriptorSetInfo};

static DESCRIPTOR_DEFS: [CGenDescriptorDef; 18] = [
    CGenDescriptorDef {
        name: "smp",
        shader_resource_type: ShaderResourceType::Sampler,
        flat_index_start: 0,
        flat_index_end: 1,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "smp_arr",
        shader_resource_type: ShaderResourceType::Sampler,
        flat_index_start: 1,
        flat_index_end: 11,
        array_size: 10,
    },
    CGenDescriptorDef {
        name: "cb",
        shader_resource_type: ShaderResourceType::ConstantBuffer,
        flat_index_start: 11,
        flat_index_end: 12,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "cb_tr",
        shader_resource_type: ShaderResourceType::ConstantBuffer,
        flat_index_start: 12,
        flat_index_end: 13,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "sb",
        shader_resource_type: ShaderResourceType::StructuredBuffer,
        flat_index_start: 13,
        flat_index_end: 14,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "sb2",
        shader_resource_type: ShaderResourceType::StructuredBuffer,
        flat_index_start: 14,
        flat_index_end: 15,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "sb_arr",
        shader_resource_type: ShaderResourceType::StructuredBuffer,
        flat_index_start: 15,
        flat_index_end: 25,
        array_size: 10,
    },
    CGenDescriptorDef {
        name: "rw_sb",
        shader_resource_type: ShaderResourceType::RWStructuredBuffer,
        flat_index_start: 25,
        flat_index_end: 26,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "bab",
        shader_resource_type: ShaderResourceType::ByteAdressBuffer,
        flat_index_start: 26,
        flat_index_end: 27,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "rw_bab",
        shader_resource_type: ShaderResourceType::RWByteAdressBuffer,
        flat_index_start: 27,
        flat_index_end: 28,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "tex2d",
        shader_resource_type: ShaderResourceType::Texture2D,
        flat_index_start: 28,
        flat_index_end: 29,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "rw_tex2d",
        shader_resource_type: ShaderResourceType::RWTexture2D,
        flat_index_start: 29,
        flat_index_end: 30,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "tex3d",
        shader_resource_type: ShaderResourceType::Texture3D,
        flat_index_start: 30,
        flat_index_end: 31,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "rw_tex3d",
        shader_resource_type: ShaderResourceType::RWTexture3D,
        flat_index_start: 31,
        flat_index_end: 32,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "tex2darr",
        shader_resource_type: ShaderResourceType::Texture2DArray,
        flat_index_start: 32,
        flat_index_end: 33,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "rw_tex2darr",
        shader_resource_type: ShaderResourceType::RWTexture2DArray,
        flat_index_start: 33,
        flat_index_end: 34,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "rw_texcube",
        shader_resource_type: ShaderResourceType::TextureCube,
        flat_index_start: 34,
        flat_index_end: 35,
        array_size: 0,
    },
    CGenDescriptorDef {
        name: "rw_texcubearr",
        shader_resource_type: ShaderResourceType::TextureCubeArray,
        flat_index_start: 35,
        flat_index_end: 36,
        array_size: 0,
    },
];

static DESCRIPTOR_SET_DEF: CGenDescriptorSetDef = CGenDescriptorSetDef {
    name: "FrameDescriptorSet",
    id: 1,
    frequency: 1,
    descriptor_flat_count: 36,
    descriptor_defs: &DESCRIPTOR_DEFS,
};

static mut DESCRIPTOR_SET_LAYOUT: Option<DescriptorSetLayout> = None;

pub struct FrameDescriptorSet<'a> {
    descriptor_refs: [DescriptorRef<'a>; 36],
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
        1
    }

    pub const fn frequency() -> u32 {
        1
    }

    pub fn def() -> &'static CGenDescriptorSetDef {
        &DESCRIPTOR_SET_DEF
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_smp(&mut self, value: &'a Sampler) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[0].validate(value));
        self.descriptor_refs[0] = DescriptorRef::Sampler(value);
    }

    pub fn set_smp_arr(&mut self, value: &[&'a Sampler; 10]) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[1].validate(&value.as_slice()));
        for i in 0..10 {
            self.descriptor_refs[1 + i] = DescriptorRef::Sampler(value[i]);
        }
    }

    pub fn set_cb(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[2].validate(value));
        self.descriptor_refs[11] = DescriptorRef::BufferView(value);
    }

    pub fn set_cb_tr(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[3].validate(value));
        self.descriptor_refs[12] = DescriptorRef::BufferView(value);
    }

    pub fn set_sb(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[4].validate(value));
        self.descriptor_refs[13] = DescriptorRef::BufferView(value);
    }

    pub fn set_sb2(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[5].validate(value));
        self.descriptor_refs[14] = DescriptorRef::BufferView(value);
    }

    pub fn set_sb_arr(&mut self, value: &[&'a BufferView; 10]) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[6].validate(&value.as_slice()));
        for i in 0..10 {
            self.descriptor_refs[15 + i] = DescriptorRef::BufferView(value[i]);
        }
    }

    pub fn set_rw_sb(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[7].validate(value));
        self.descriptor_refs[25] = DescriptorRef::BufferView(value);
    }

    pub fn set_bab(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[8].validate(value));
        self.descriptor_refs[26] = DescriptorRef::BufferView(value);
    }

    pub fn set_rw_bab(&mut self, value: &'a BufferView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[9].validate(value));
        self.descriptor_refs[27] = DescriptorRef::BufferView(value);
    }

    pub fn set_tex2d(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[10].validate(value));
        self.descriptor_refs[28] = DescriptorRef::TextureView(value);
    }

    pub fn set_rw_tex2d(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[11].validate(value));
        self.descriptor_refs[29] = DescriptorRef::TextureView(value);
    }

    pub fn set_tex3d(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[12].validate(value));
        self.descriptor_refs[30] = DescriptorRef::TextureView(value);
    }

    pub fn set_rw_tex3d(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[13].validate(value));
        self.descriptor_refs[31] = DescriptorRef::TextureView(value);
    }

    pub fn set_tex2darr(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[14].validate(value));
        self.descriptor_refs[32] = DescriptorRef::TextureView(value);
    }

    pub fn set_rw_tex2darr(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[15].validate(value));
        self.descriptor_refs[33] = DescriptorRef::TextureView(value);
    }

    pub fn set_rw_texcube(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[16].validate(value));
        self.descriptor_refs[34] = DescriptorRef::TextureView(value);
    }

    pub fn set_rw_texcubearr(&mut self, value: &'a TextureView) {
        assert!(DESCRIPTOR_SET_DEF.descriptor_defs[17].validate(value));
        self.descriptor_refs[35] = DescriptorRef::TextureView(value);
    }
}

impl<'a> Default for FrameDescriptorSet<'a> {
    fn default() -> Self {
        Self {
            descriptor_refs: [DescriptorRef::<'a>::default(); 36],
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
