// This is generated file. Do not edit manually

use std::{mem, ptr};

use lgn_graphics_api::{
DeviceContext,
RootSignature,
DescriptorSetLayout,
DescriptorSetHandle,
Pipeline,
MAX_DESCRIPTOR_SET_LAYOUTS,
};

use lgn_graphics_cgen_runtime::{
CGenPipelineLayoutDef,
PipelineDataProvider,
};

use super::super::descriptor_set::ViewDescriptorSet;
use super::super::cgen_type::PushConstantData;

static PIPELINE_LAYOUT_DEF: CGenPipelineLayoutDef = CGenPipelineLayoutDef{ 
	name: "TmpPipelineLayout",
	id: 0,
	descriptor_set_layout_ids: [
	Some(ViewDescriptorSet::id()),
	None,
	None,
	None,
	],
	push_constant_type: None,
}; 

static mut PIPELINE_LAYOUT: Option<RootSignature> = None;

pub struct TmpPipelineLayout<'a> {
	pipeline: &'a Pipeline,
	descriptor_sets: [Option<DescriptorSetHandle>; MAX_DESCRIPTOR_SET_LAYOUTS],
	push_constant: PushConstantData
}

impl<'a> TmpPipelineLayout<'a> {
	
	#[allow(unsafe_code)]
	pub fn initialize(device_context: &DeviceContext, descriptor_set_layouts: &[&DescriptorSetLayout]) {
		unsafe { PIPELINE_LAYOUT = Some(PIPELINE_LAYOUT_DEF.create_pipeline_layout(device_context, descriptor_set_layouts)); }
	}
	
	#[allow(unsafe_code)]
	pub fn shutdown() {
		unsafe{ PIPELINE_LAYOUT = None; }
	}
	
	#[allow(unsafe_code)]
	pub fn root_signature() -> &'static RootSignature {
		unsafe{ match &PIPELINE_LAYOUT{
			Some(pl) => pl,
			None => unreachable!(),
		}}
	}
	
	pub fn new(pipeline: &'a Pipeline) -> Self {
		assert_eq!( pipeline.root_signature(), Self::root_signature());
		Self {
			pipeline,
			descriptor_sets: [None; MAX_DESCRIPTOR_SET_LAYOUTS],
			push_constant: PushConstantData::default(),
		}
	}
	
	pub fn set_view_descriptor_set(&mut self, descriptor_set_handle: DescriptorSetHandle) {
		self.descriptor_sets[0] = Some(descriptor_set_handle);
	}
	pub fn set_push_constant(&mut self, data: &PushConstantData) {
		self.push_constant = *data;
	}
}

impl<'a> PipelineDataProvider for TmpPipelineLayout<'a> {
	
	fn pipeline(&self) -> &Pipeline {
		self.pipeline
	}
	
	fn descriptor_set(&self, frequency: u32) -> Option<DescriptorSetHandle> {
		self.descriptor_sets[frequency as usize]
	}
	
	fn push_constant(&self) -> Option<&[u8]> {
		#![allow(unsafe_code)]
		let data_slice = unsafe {
		&*ptr::slice_from_raw_parts((&self.push_constant as *const PushConstantData).cast::<u8>(), mem::size_of::<PushConstantData>())
		};
		Some(data_slice)
	}
	
	fn set_descriptor_set(&mut self, frequency: u32, descriptor_set: Option<DescriptorSetHandle>) {
		self.descriptor_sets[frequency as usize] = descriptor_set;
	}
}

