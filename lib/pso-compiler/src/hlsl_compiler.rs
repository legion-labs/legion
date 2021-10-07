use std::{io::Read, path::Path};
use graphics_api::{PushConstant, ShaderResource, ShaderResourceType, ShaderStageFlags, ShaderStageReflection};
use hassle_rs::{Dxc, compile_hlsl};
use anyhow::{Result, anyhow};
use spirv_reflect::types::{ReflectBlockVariable, ReflectDecorationFlags, ReflectDescriptorBinding, ReflectShaderStageFlags};
use spirv_tools::{TargetEnv, opt::Optimizer};

pub struct CompileDefine {
    name: String,
    value: Option<String>
}

pub enum ShaderSource<'a> {
    Code(String),
    Path(&'a Path)
}

pub struct CompileParams<'a> {
    pub shader_source : ShaderSource<'a>,
    pub entry_point : &'a str,
    pub target_profile: &'a str,
    pub defines: Vec<CompileDefine>
}

impl<'a> CompileParams<'a> {
    fn path_as_string(&self) -> &str {
        match self.shader_source {
            ShaderSource::Code(_) => "_code.hlsl",
            ShaderSource::Path(path) =>  path.to_str().unwrap()
        }
    }
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

        // Shader source
        let shader_code = get_shader_source(params)?;

        // Compilation
        let unopt_bytecode = compile_to_unoptimized_spirv(params, &shader_code)?;
        
        // Reflection
        let reflection_info = extract_reflection_info(&unopt_bytecode, params);

        // Optimize
        let opt_bytecode = optimize_spirv(&unopt_bytecode)?;

        // Finalize
        Ok(CompileResult{
            bytecode: opt_bytecode,
            refl_info : Some(reflection_info)
        })
    }    
}

fn optimize_spirv(bytecode: &[u8]) -> Result<Vec<u8>> {

    let u32spirv = spirv_tools::binary::to_binary(&bytecode)?;
    let mut optimizer = spirv_tools::opt::create(Some(TargetEnv::Vulkan_1_2));
    optimizer.register_performance_passes();
    let opt_binary = optimizer.optimize(u32spirv, &mut OptimizerCallback{}, None)?;
    Ok(opt_binary.as_bytes().to_vec())
}

fn compile_to_unoptimized_spirv(params: &CompileParams, shader_code: &str) -> Result<Vec<u8>> {
    
    let defines = 
        params.defines.iter().
        map( |x| (x.name.as_str(), x.value.as_ref().map(|y| y.as_str() ))  ).collect::<Vec<_>>();

    compile_hlsl(
        params.path_as_string(),
        &shader_code,
        params.entry_point,
        params.target_profile.as_ref(),
        &[
            "-Od", 
            "-spirv", 
            "-fspv-target-env=vulkan1.1", 
            // "-fspv-reflect", 
            // "-fspv-extension=SPV_GOOGLE_hlsl_functionality1"
        ],
        &defines
    ).map_err( |err| anyhow!(err) )
}

fn get_shader_source(params: &CompileParams) -> Result<String, anyhow::Error> {
    
    let mut shader_code = String::new();

    match &params.shader_source{
        ShaderSource::Code(code) => { 
            shader_code = code.clone();
        }
        ShaderSource::Path(path) => {
            let mut f = std::fs::File::open(path).unwrap();                                
            f.read_to_string(&mut shader_code)?;
        }            
    };

    Ok(shader_code)
}

fn extract_reflection_info(bytecode: &Vec<u8>, params: &CompileParams) -> ShaderStageReflection {

    let shader_mod = spirv_reflect::create_shader_module(bytecode).unwrap();
    let shader_stage = to_shader_stage_flags(shader_mod.get_shader_stage());
    let mut shader_resources = Vec::new();

    for descriptor in &shader_mod.enumerate_descriptor_bindings(Some(params.entry_point) ).unwrap() {
        shader_resources.push(to_shader_resource(shader_stage, descriptor) );
    }    
    dbg!(&shader_resources);

    let mut push_constants = Vec::new();
    for push_constant in &shader_mod.enumerate_push_constant_blocks(Some(params.entry_point)).unwrap() {
        push_constants.push(to_push_constant(shader_stage, push_constant));
    }
    dbg!(&push_constants);

    ShaderStageReflection{
        shader_stage,
        shader_resources,
        push_constants,
        compute_threads_per_group: None,
        entry_point_name: params.entry_point.to_owned()
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
        _ => unimplemented!()
    }
}

fn to_shader_resource(shader_stage_flags: ShaderStageFlags, descriptor_binding : &ReflectDescriptorBinding ) -> ShaderResource {
    ShaderResource{    
        shader_resource_type: to_shader_resource_type(descriptor_binding),
        set_index: descriptor_binding.set,
        binding: descriptor_binding.binding,   
        element_count: descriptor_binding.count,               
        used_in_shader_stages: shader_stage_flags,
        name: descriptor_binding.name.clone(),
    }
}

/// Reference: https://github.com/Microsoft/DirectXShaderCompiler/blob/master/docs/SPIR-V.rst
fn to_shader_resource_type(descriptor_binding : &ReflectDescriptorBinding ) -> ShaderResourceType {    
    
    match descriptor_binding.descriptor_type {
        
        spirv_reflect::types::ReflectDescriptorType::Sampler => {              
            ShaderResourceType::Sampler
        },                
        
        spirv_reflect::types::ReflectDescriptorType::UniformBuffer => {       
            ShaderResourceType::ConstantBuffer
        },
        
        spirv_reflect::types::ReflectDescriptorType::StorageBuffer => {             
            let readonly = is_descriptor_readonly(descriptor_binding);
            if descriptor_binding.block.members[0].padded_size == 0 {
                if readonly {
                    ShaderResourceType::ByteAdressBuffer
                } else {
                    ShaderResourceType::RWByteAdressBuffer
                }
            } else {        
                if readonly {
                    ShaderResourceType::StructuredBuffer
                } else {
                    ShaderResourceType::RWStructuredBuffer
                }
            }
        },
        spirv_reflect::types::ReflectDescriptorType::SampledImage => {
            match (descriptor_binding.image.dim, descriptor_binding.image.depth, descriptor_binding.image.arrayed, descriptor_binding.image.sampled) {
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 0, 1) => ShaderResourceType::Texture2D,
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 1, 1) => ShaderResourceType::Texture2DArray,
                (spirv_reflect::types::ReflectDimension::Type3d, 2, 0, 1) => ShaderResourceType::Texture3D,
                (spirv_reflect::types::ReflectDimension::Cube, 2, 0, 1) => ShaderResourceType::TextureCube,
                (spirv_reflect::types::ReflectDimension::Cube, 2, 1, 1) => ShaderResourceType::TextureCubeArray,
                
                
                _ => unimplemented!()
            }
            
        }

        spirv_reflect::types::ReflectDescriptorType::StorageImage => { 
            match (descriptor_binding.image.dim, descriptor_binding.image.depth, descriptor_binding.image.arrayed, descriptor_binding.image.sampled) {
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 0, 2) => ShaderResourceType::RWTexture2D,   
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 1, 2) => ShaderResourceType::RWTexture2DArray,                             
                (spirv_reflect::types::ReflectDimension::Type3d, 2, 0, 2) => ShaderResourceType::RWTexture3D,
                
                _ => unimplemented!()
            }
        },                
        // 
        _ => panic!() 
    }
}

fn is_descriptor_readonly(descriptor_binding : &ReflectDescriptorBinding ) -> bool {
    ReflectDecorationFlags::NON_WRITABLE == (descriptor_binding.block.decoration_flags & ReflectDecorationFlags::NON_WRITABLE)
}

fn to_push_constant(shader_stage_flags: ShaderStageFlags, push_constant : &ReflectBlockVariable ) -> PushConstant {
    PushConstant {
        used_in_shader_stages: shader_stage_flags,
        size: push_constant.size
    }
}