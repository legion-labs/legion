use std::{path::Path};
use graphics_api::{ShaderResource, ShaderResourceType, ShaderStageFlags, ShaderStageReflection};
use hassle_rs::{Dxc, compile_hlsl};
use anyhow::{Result, anyhow };
use spirv_reflect::types::{ReflectDescriptorBinding, ReflectShaderStageFlags};
use spirv_tools::{TargetEnv, opt::Optimizer};



pub struct CompileDefine {
    name: String,
    value: Option<String>
}

pub struct CompileParams<'a> {
    pub path : &'a Path,
    pub entry_point : &'a str,
    pub target_profile: &'a str,
    pub defines: Vec<CompileDefine>
}

pub struct CompileResult {
    pub bytecode : Vec<u8>,
    pub refl_info : Option<ShaderStageReflection>
}

pub struct HLSLCompiler {
    _dxc : Dxc    
}

impl HLSLCompiler {
    pub fn new() -> Result<Self> {

        let dxc = Dxc::new(None)?;

        Ok( HLSLCompiler{
            _dxc: dxc
        }) 
    }

    pub fn compile(&self, params: &CompileParams) -> Result<CompileResult> {

        use std::io::Read;

        // Read file
        let code = {
            let code = {
                match std::fs::File::open(params.path) {
                    Ok(mut f) => {
                        let mut content = String::new();
                        f.read_to_string(&mut content).unwrap();
                        Some(content)
                    }
                    Err(_) => None,
                }
            };
            code.ok_or( anyhow!("Cannot read source path") )?
        };
        // Compilation
        let bytecode = {
            let defines = 
                params.defines.iter().
                map( |x| (x.name.as_str(), x.value.as_ref().map(|y| y.as_str() ))  ).collect::<Vec<_>>();        
    
            compile_hlsl(
                params.path.to_str().unwrap(),
                &code,
                params.entry_point,
                params.target_profile.as_ref(),
                &["-Od", "-spirv"],
                &defines
            )?
        };
        // Reflection
        let refl_info = {       
            let shader_mod = spirv_reflect::create_shader_module(&bytecode).unwrap();            

            let mut shader_resources = Vec::new();
            let shader_stage = to_shader_stage_flags(shader_mod.get_shader_stage());
            
            let ivs = shader_mod.enumerate_input_variables(Some(params.entry_point)).unwrap();
            for iv in ivs {
                dbg!( &iv );
            }


            let em = shader_mod.get_spirv_execution_model();            
            let rc = em.required_capabilities();
            for c in rc {
                dbg!( &c );
            }

            for descriptor in &shader_mod.enumerate_descriptor_bindings(None).unwrap() {
                shader_resources.push( to_shader_resource(shader_stage, descriptor) );
            }

            ShaderStageReflection{
                shader_stage: shader_stage,
                resources: shader_resources,
                compute_threads_per_group: None,
                entry_point_name: params.entry_point.to_owned()
            }
        };
        // Optimize
        let bytecode = {
            let u32spirv = spirv_tools::binary::to_binary(&bytecode)?;
    
            let mut optimizer = spirv_tools::opt::create(Some(TargetEnv::Vulkan_1_2));        
            optimizer.register_performance_passes();

            let opt_binary = optimizer.optimize(u32spirv, &mut OptimizerCallback{}, None)?;
    
            opt_binary.as_bytes().to_vec()
        };
        // Finalize
        Ok(CompileResult{
            bytecode,
            refl_info : Some(refl_info)
        })
    }    
}

struct OptimizerCallback;
impl spirv_tools::error::MessageCallback for OptimizerCallback{
    fn on_message(&mut self, _msg: spirv_tools::error::Message) {
        unimplemented!();
    }
}

fn to_shader_stage_flags(flags : ReflectShaderStageFlags) -> ShaderStageFlags {    
    match flags {
        ReflectShaderStageFlags::VERTEX => ShaderStageFlags::VERTEX,
        ReflectShaderStageFlags::FRAGMENT => ShaderStageFlags::FRAGMENT,
        ReflectShaderStageFlags::COMPUTE => ShaderStageFlags::COMPUTE,
        _ => panic!()
    }
}

fn to_shader_resource(shader_stage_flags: ShaderStageFlags, descriptor_binding : &ReflectDescriptorBinding ) -> ShaderResource {
    ShaderResource{    
        shader_resource_type: to_shader_resource_type(descriptor_binding),
        set_index: descriptor_binding.set,
        binding: descriptor_binding.binding,   
        element_count: descriptor_binding.count,       
        size_in_bytes: 0,        
        used_in_shader_stages: shader_stage_flags,
        name: Some(descriptor_binding.name.clone()),
    }
}

fn to_shader_resource_type(descriptor_binding : &ReflectDescriptorBinding ) -> ShaderResourceType {

    println!( "Descriptor binding name {}-{} : {}", descriptor_binding.binding, descriptor_binding.set, descriptor_binding.name );

    println!( "Resource Type" );
    match descriptor_binding.resource_type {
        spirv_reflect::types::ReflectResourceType::Undefined => { println!( "Undefined" );},
        spirv_reflect::types::ReflectResourceType::Sampler => { println!( "Sampler" );},

        spirv_reflect::types::ReflectResourceType::ConstantBufferView => { println!( "ConstantBufferView" );},
        spirv_reflect::types::ReflectResourceType::ShaderResourceView => { println!( "ShaderResourceView" );},
        spirv_reflect::types::ReflectResourceType::UnorderedAccessView => { println!( "UnorderedAccessView" );},

        spirv_reflect::types::ReflectResourceType::CombinedImageSampler => { println!( "CombinedImageSampler" );},
    }

    println!( "Descriptor Type" );
    match descriptor_binding.descriptor_type {
        spirv_reflect::types::ReflectDescriptorType::Undefined => { println!( "Undefined" );},
        spirv_reflect::types::ReflectDescriptorType::Sampler => { println!( "Sampler" );},

        spirv_reflect::types::ReflectDescriptorType::SampledImage => { println!( "SampledImage" );},
        spirv_reflect::types::ReflectDescriptorType::StorageImage => { println!( "StorageImage" );},
        spirv_reflect::types::ReflectDescriptorType::UniformTexelBuffer => { println!( "UniformTexelBuffer" );},
        spirv_reflect::types::ReflectDescriptorType::StorageTexelBuffer => { println!( "StorageTexelBuffer" );},
        spirv_reflect::types::ReflectDescriptorType::UniformBuffer => { println!( "UniformBuffer" );},
        spirv_reflect::types::ReflectDescriptorType::StorageBuffer => { println!( "StorageBuffer" );},
        spirv_reflect::types::ReflectDescriptorType::UniformBufferDynamic => { println!( "UniformBufferDynamic" );},
        spirv_reflect::types::ReflectDescriptorType::StorageBufferDynamic => { println!( "StorageBufferDynamic" );},
        spirv_reflect::types::ReflectDescriptorType::InputAttachment => { println!( "InputAttachment" );},
        spirv_reflect::types::ReflectDescriptorType::AccelerationStructureNV => { println!( "AccelerationStructureNV" );},

        spirv_reflect::types::ReflectDescriptorType::CombinedImageSampler => { println!( "CombinedImageSampler" );},
    }


    // match descriptor_binding.resource_type {
    //     spirv_reflect::types::ReflectResourceType::Sampler => ResourceType::SAMPLER,
    //     spirv_reflect::types::ReflectResourceType::ConstantBufferView => ResourceType::UNIFORM_BUFFER,
    //     spirv_reflect::types::ReflectResourceType::ShaderResourceView => ResourceType::
    //     spirv_reflect::types::ReflectResourceType::UnorderedAccessView => todo!(),
    //     spirv_reflect::types::ReflectResourceType::Undefined |
    //     spirv_reflect::types::ReflectResourceType::CombinedImageSampler => panic!()
    // }    
    ShaderResourceType::Undefined
}
