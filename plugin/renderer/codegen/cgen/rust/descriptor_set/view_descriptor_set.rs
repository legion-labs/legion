// This is generated file. Do not edit manually

use lgn_graphics_api::DeviceContext;
use lgn_graphics_api::DescriptorSetLayout;
use lgn_graphics_api::ShaderResourceType;
use lgn_graphics_api::DescriptorRef;
use lgn_graphics_api::Sampler;
use lgn_graphics_api::BufferView;
use lgn_graphics_api::TextureView;
use lgn_graphics_cgen_runtime::CGenDescriptorSetInfo;
use lgn_graphics_cgen_runtime::CGenDescriptorDef;
use lgn_graphics_cgen_runtime::CGenDescriptorSetDef;

static descriptor_defs: [CGenDescriptorDef; 1] = [
	CGenDescriptorDef {
		name: "view_data",
		shader_resource_type: ShaderResourceType::ConstantBuffer,
		flat_index: 0,
		array_size: 0,
	}, 
];

static descriptor_set_def: CGenDescriptorSetDef = CGenDescriptorSetDef{ 
	name: "ViewDescriptorSet",
	id: 0,
	frequency: 0,
	descriptor_flat_count: 1,
	descriptor_defs: &descriptor_defs,
}; 

static mut descriptor_set_layout: Option<DescriptorSetLayout> = None;

pub struct ViewDescriptorSet<'a> {
	descriptor_refs: [DescriptorRef<'a>; 1],
}

impl<'a> ViewDescriptorSet<'a> {
	
	#![allow(unsafe_code)]
	pub fn initialize(device_context: &DeviceContext) {
		unsafe { descriptor_set_layout = Some(descriptor_set_def.create_descriptor_set_layout(device_context)); }
	}
	
	pub fn descriptor_set_layout() -> &'static DescriptorSetLayout {
		unsafe{ match &descriptor_set_layout{
		Some(dsl) => dsl,
		None => unreachable!(),
		}}
	}
	pub const fn id() -> u32 { 0  }
	pub const fn frequency() -> u32 { 0  }
	pub fn def() -> &'static CGenDescriptorSetDef { &descriptor_set_def }
	
	pub fn set_view_data(&mut self, value:  &'a BufferView) {
		assert!(descriptor_set_def.descriptor_defs[0].validate(value));
		self.descriptor_refs[0] = DescriptorRef::BufferView(value);
	}
	
}

