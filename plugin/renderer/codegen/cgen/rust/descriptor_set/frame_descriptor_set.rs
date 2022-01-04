// This is generated file. Do not edit manually

use lgn_graphics_api::DeviceContext;
use lgn_graphics_api::DescriptorSetLayout;
use lgn_graphics_api::ShaderResourceType;
use lgn_graphics_api::DescriptorRef;
use lgn_graphics_api::Sampler;
use lgn_graphics_api::BufferView;
use lgn_graphics_api::TextureView;
use lgn_graphics_api::DescriptorSetDataProvider;
use lgn_graphics_cgen_runtime::CGenDescriptorSetInfo;
use lgn_graphics_cgen_runtime::CGenDescriptorDef;
use lgn_graphics_cgen_runtime::CGenDescriptorSetDef;

static descriptor_defs: [CGenDescriptorDef; 17] = [
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
		name: "sb_arr",
		shader_resource_type: ShaderResourceType::StructuredBuffer,
		flat_index_start: 14,
		flat_index_end: 24,
		array_size: 10,
	}, 
	CGenDescriptorDef {
		name: "rw_sb",
		shader_resource_type: ShaderResourceType::RWStructuredBuffer,
		flat_index_start: 24,
		flat_index_end: 25,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "bab",
		shader_resource_type: ShaderResourceType::ByteAdressBuffer,
		flat_index_start: 25,
		flat_index_end: 26,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "rw_bab",
		shader_resource_type: ShaderResourceType::RWByteAdressBuffer,
		flat_index_start: 26,
		flat_index_end: 27,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "tex2d",
		shader_resource_type: ShaderResourceType::Texture2D,
		flat_index_start: 27,
		flat_index_end: 28,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "rw_tex2d",
		shader_resource_type: ShaderResourceType::RWTexture2D,
		flat_index_start: 28,
		flat_index_end: 29,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "tex3d",
		shader_resource_type: ShaderResourceType::Texture3D,
		flat_index_start: 29,
		flat_index_end: 30,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "rw_tex3d",
		shader_resource_type: ShaderResourceType::RWTexture3D,
		flat_index_start: 30,
		flat_index_end: 31,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "tex2darr",
		shader_resource_type: ShaderResourceType::Texture2DArray,
		flat_index_start: 31,
		flat_index_end: 32,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "rw_tex2darr",
		shader_resource_type: ShaderResourceType::RWTexture2DArray,
		flat_index_start: 32,
		flat_index_end: 33,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "rw_texcube",
		shader_resource_type: ShaderResourceType::TextureCube,
		flat_index_start: 33,
		flat_index_end: 34,
		array_size: 0,
	}, 
	CGenDescriptorDef {
		name: "rw_texcubearr",
		shader_resource_type: ShaderResourceType::TextureCubeArray,
		flat_index_start: 34,
		flat_index_end: 35,
		array_size: 0,
	}, 
];

static descriptor_set_def: CGenDescriptorSetDef = CGenDescriptorSetDef{ 
	name: "FrameDescriptorSet",
	id: 1,
	frequency: 1,
	descriptor_flat_count: 35,
	descriptor_defs: &descriptor_defs,
}; 

static mut descriptor_set_layout: Option<DescriptorSetLayout> = None;

pub struct FrameDescriptorSet<'a> {
	descriptor_refs: [DescriptorRef<'a>; 35],
}

impl<'a> FrameDescriptorSet<'a> {
	
	#[allow(unsafe_code)]
	pub fn initialize(device_context: &DeviceContext) {
		unsafe { descriptor_set_layout = Some(descriptor_set_def.create_descriptor_set_layout(device_context)); }
	}
	
	#[allow(unsafe_code)]
	pub fn shutdown() {
		unsafe{ descriptor_set_layout = None; }
	}
	
	#[allow(unsafe_code)]
	pub fn descriptor_set_layout() -> &'static DescriptorSetLayout {
		unsafe{ match &descriptor_set_layout{
		Some(dsl) => dsl,
		None => unreachable!(),
		}}
	}
	
	pub const fn id() -> u32 { 1  }
	
	pub const fn frequency() -> u32 { 1  }
	
	pub fn def() -> &'static CGenDescriptorSetDef { &descriptor_set_def }
	
	pub fn new() -> Self { Self::default() }
	
	pub fn set_smp(&mut self, value:  &'a Sampler) {
		assert!(descriptor_set_def.descriptor_defs[0].validate(value));
		self.descriptor_refs[0] = DescriptorRef::Sampler(value);
	}
	
	pub fn set_smp_arr(&mut self, value:  &[&'a Sampler; 10]) {
		assert!(descriptor_set_def.descriptor_defs[1].validate(value.as_ref()));
		for i in 0..10 {
			self.descriptor_refs[1+i] = DescriptorRef::Sampler(value[i]);
		}
	}
	
	pub fn set_cb(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[2].validate(value));
		self.descriptor_refs[11] = DescriptorRef::BufferView(value);
	}
	
	pub fn set_cb_tr(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[3].validate(value));
		self.descriptor_refs[12] = DescriptorRef::BufferView(value);
	}
	
	pub fn set_sb(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[4].validate(value));
		self.descriptor_refs[13] = DescriptorRef::BufferView(value);
	}
	
	pub fn set_sb_arr(&mut self, value:  &[&'a BufferView; 10]) {
		assert!(descriptor_set_def.descriptor_defs[5].validate(value.as_ref()));
		for i in 0..10 {
			self.descriptor_refs[14+i] = DescriptorRef::BufferView(value[i]);
		}
	}
	
	pub fn set_rw_sb(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[6].validate(value));
		self.descriptor_refs[24] = DescriptorRef::BufferView(value);
	}
	
	pub fn set_bab(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[7].validate(value));
		self.descriptor_refs[25] = DescriptorRef::BufferView(value);
	}
	
	pub fn set_rw_bab(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[8].validate(value));
		self.descriptor_refs[26] = DescriptorRef::BufferView(value);
	}
	
	pub fn set_tex2d(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[9].validate(value));
		self.descriptor_refs[27] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_rw_tex2d(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[10].validate(value));
		self.descriptor_refs[28] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_tex3d(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[11].validate(value));
		self.descriptor_refs[29] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_rw_tex3d(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[12].validate(value));
		self.descriptor_refs[30] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_tex2darr(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[13].validate(value));
		self.descriptor_refs[31] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_rw_tex2darr(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[14].validate(value));
		self.descriptor_refs[32] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_rw_texcube(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[15].validate(value));
		self.descriptor_refs[33] = DescriptorRef::TextureView(value);
	}
	
	pub fn set_rw_texcubearr(&mut self, value:  &'a TextureView) {
		assert!(descriptor_set_def.descriptor_defs[16].validate(value));
		self.descriptor_refs[34] = DescriptorRef::TextureView(value);
	}
	
}

impl<'a> Default for FrameDescriptorSet<'a> {
	fn default() -> Self {
		Self {descriptor_refs: [DescriptorRef::<'a>::default(); 35], }
	}
}

impl<'a> DescriptorSetDataProvider for FrameDescriptorSet<'a> {
	fn layout(&self) -> &'static DescriptorSetLayout {
		Self::descriptor_set_layout()
	}
	
	fn descriptor_refs(&self, descriptor_index: usize) -> &[DescriptorRef<'a>] {
		&self.descriptor_refs[
                descriptor_defs[descriptor_index].flat_index_start as usize .. descriptor_defs[descriptor_index].flat_index_end as usize
             ]
	}
	
}

