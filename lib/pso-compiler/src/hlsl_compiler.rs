use anyhow::{anyhow, Result};
use graphics_api::{
    PushConstant, ShaderResource, ShaderResourceType, ShaderStageFlags, ShaderStageReflection,
};
use hassle_rs::{compile_hlsl, Dxc};
use spirv_reflect::types::{
    ReflectBlockVariable, ReflectDecorationFlags, ReflectDescriptorBinding, ReflectShaderStageFlags,
};
use spirv_tools::{opt::Optimizer, TargetEnv};
use std::{io::Read, path::Path};

pub struct CompileDefine {
    name: String,
    value: Option<String>,
}

pub enum ShaderSource<'a> {
    Code(&'a str),
    Path(&'a Path),
}

pub struct CompileParams<'a> {
    pub shader_source: ShaderSource<'a>,
    pub entry_point: &'a str,
    pub target_profile: &'a str,
    pub defines: Vec<CompileDefine>,
}

impl<'a> CompileParams<'a> {
    fn path_as_string(&self) -> &str {
        match self.shader_source {
            ShaderSource::Code(_) => "_code.hlsl",
            ShaderSource::Path(path) => path.to_str().unwrap(),
        }
    }
}

pub struct CompileResult {
    pub bytecode: Vec<u8>,
    pub refl_info: Option<ShaderStageReflection>,
}

pub struct HlslCompiler {
    _dxc: Dxc,
}

impl HlslCompiler {
    pub fn new() -> Result<Self> {
        let dxc = Dxc::new(None)?;

        Ok(HlslCompiler { _dxc: dxc })
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
        Ok(CompileResult {
            bytecode: opt_bytecode,
            refl_info: Some(reflection_info),
        })
    }
}

fn optimize_spirv(bytecode: &[u8]) -> Result<Vec<u8>> {
    let u32spirv = spirv_tools::binary::to_binary(bytecode)?;
    let mut optimizer = spirv_tools::opt::create(Some(TargetEnv::Vulkan_1_2));
    optimizer.register_performance_passes();
    let opt_binary = optimizer.optimize(u32spirv, &mut OptimizerCallback {}, None)?;
    Ok(opt_binary.as_bytes().to_vec())
}

fn compile_to_unoptimized_spirv(params: &CompileParams, shader_code: &str) -> Result<Vec<u8>> {
    let defines = params
        .defines
        .iter()
        .map(|x| (x.name.as_str(), x.value.as_deref()))
        .collect::<Vec<_>>();

    compile_hlsl(
        params.path_as_string(),
        shader_code,
        params.entry_point,
        params.target_profile,
        &["-Od", "-spirv", "-fspv-target-env=vulkan1.1"],
        &defines,
    )
    .map_err(|err| anyhow!(err))
}

fn get_shader_source(params: &CompileParams) -> Result<String, anyhow::Error> {
    let mut shader_code = String::new();

    match &params.shader_source {
        ShaderSource::Code(code) => {
            shader_code = code.to_string();
        }
        ShaderSource::Path(path) => {
            let mut f = std::fs::File::open(path).unwrap();
            f.read_to_string(&mut shader_code)?;
        }
    };

    Ok(shader_code)
}

fn extract_reflection_info(bytecode: &[u8], params: &CompileParams) -> ShaderStageReflection {
    let shader_mod = spirv_reflect::create_shader_module(bytecode).unwrap();
    let shader_stage = to_shader_stage_flags(shader_mod.get_shader_stage());

    let mut shader_resources = Vec::new();
    for descriptor in &shader_mod
        .enumerate_descriptor_bindings(Some(params.entry_point))
        .unwrap()
    {
        shader_resources.push(to_shader_resource(shader_stage, descriptor));
    }

    let mut push_constants = Vec::new();
    for push_constant in &shader_mod
        .enumerate_push_constant_blocks(Some(params.entry_point))
        .unwrap()
    {
        push_constants.push(to_push_constant(shader_stage, push_constant));
    }

    ShaderStageReflection {
        shader_stage,
        shader_resources,
        push_constants,
        compute_threads_per_group: None,
        entry_point_name: params.entry_point.to_owned(),
    }
}

struct OptimizerCallback;
impl spirv_tools::error::MessageCallback for OptimizerCallback {
    fn on_message(&mut self, _msg: spirv_tools::error::Message) {
        unimplemented!();
    }
}

fn to_shader_stage_flags(flags: ReflectShaderStageFlags) -> ShaderStageFlags {
    match flags {
        ReflectShaderStageFlags::VERTEX => ShaderStageFlags::VERTEX,
        ReflectShaderStageFlags::FRAGMENT => ShaderStageFlags::FRAGMENT,
        ReflectShaderStageFlags::COMPUTE => ShaderStageFlags::COMPUTE,
        _ => unimplemented!(),
    }
}

fn to_shader_resource(
    shader_stage_flags: ShaderStageFlags,
    descriptor_binding: &ReflectDescriptorBinding,
) -> ShaderResource {
    ShaderResource {
        shader_resource_type: to_shader_resource_type(descriptor_binding),
        set_index: descriptor_binding.set,
        binding: descriptor_binding.binding,
        element_count: descriptor_binding.count,
        used_in_shader_stages: shader_stage_flags,
        name: descriptor_binding.name.clone(),
    }
}

/// Reference: https://github.com/Microsoft/DirectXShaderCompiler/blob/master/docs/SPIR-V.rst
fn to_shader_resource_type(descriptor_binding: &ReflectDescriptorBinding) -> ShaderResourceType {
    match descriptor_binding.descriptor_type {
        spirv_reflect::types::ReflectDescriptorType::Sampler => ShaderResourceType::Sampler,

        spirv_reflect::types::ReflectDescriptorType::UniformBuffer => {
            ShaderResourceType::ConstantBuffer
        }

        spirv_reflect::types::ReflectDescriptorType::StorageBuffer => {
            let byteaddressbuffer = descriptor_binding.block.members[0].padded_size == 0;
            let readonly = is_descriptor_readonly(descriptor_binding);
            match (byteaddressbuffer, readonly) {
                (true, true) => ShaderResourceType::ByteAdressBuffer,
                (true, false) => ShaderResourceType::RWByteAdressBuffer,
                (false, true) => ShaderResourceType::StructuredBuffer,
                (false, false) => ShaderResourceType::RWStructuredBuffer,
            }
        }
        spirv_reflect::types::ReflectDescriptorType::SampledImage => {
            match (
                descriptor_binding.image.dim,
                descriptor_binding.image.depth,
                descriptor_binding.image.arrayed,
                descriptor_binding.image.sampled,
            ) {
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 0, 1) => {
                    ShaderResourceType::Texture2D
                }
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 1, 1) => {
                    ShaderResourceType::Texture2DArray
                }
                (spirv_reflect::types::ReflectDimension::Type3d, 2, 0, 1) => {
                    ShaderResourceType::Texture3D
                }
                (spirv_reflect::types::ReflectDimension::Cube, 2, 0, 1) => {
                    ShaderResourceType::TextureCube
                }
                (spirv_reflect::types::ReflectDimension::Cube, 2, 1, 1) => {
                    ShaderResourceType::TextureCubeArray
                }
                _ => unimplemented!(),
            }
        }

        spirv_reflect::types::ReflectDescriptorType::StorageImage => {
            match (
                descriptor_binding.image.dim,
                descriptor_binding.image.depth,
                descriptor_binding.image.arrayed,
                descriptor_binding.image.sampled,
            ) {
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 0, 2) => {
                    ShaderResourceType::RWTexture2D
                }
                (spirv_reflect::types::ReflectDimension::Type2d, 2, 1, 2) => {
                    ShaderResourceType::RWTexture2DArray
                }
                (spirv_reflect::types::ReflectDimension::Type3d, 2, 0, 2) => {
                    ShaderResourceType::RWTexture3D
                }
                _ => unimplemented!(),
            }
        }

        _ => panic!(),
    }
}

fn is_descriptor_readonly(descriptor_binding: &ReflectDescriptorBinding) -> bool {
    ReflectDecorationFlags::NON_WRITABLE
        == (descriptor_binding.block.decoration_flags & ReflectDecorationFlags::NON_WRITABLE)
}

fn to_push_constant(
    shader_stage_flags: ShaderStageFlags,
    push_constant: &ReflectBlockVariable,
) -> PushConstant {
    PushConstant {
        used_in_shader_stages: shader_stage_flags,
        size: push_constant.size,
    }
}

#[cfg(test)]
mod tests {
    use graphics_api::ShaderStageFlags;

    use crate::{CompileParams, HlslCompiler, ShaderSource};

    const SHADER: &str = "
    struct VertexIn {
        float3 pos : POSITION;
    };
    
    struct VertexOut {
        float4 hpos : SV_POSITION;
        float4 color : COLOR;
    };
    
    struct VertexColor {
        float3 foo;    
        float4 color;
        float bar;
    };
    
    [[vk::binding(0, 0)]]
    RWByteAddressBuffer rw_test_byteaddressbuffer[1];
    
    [[vk::binding(1, 0)]]
    ByteAddressBuffer test_byteaddressbuffer[2];
    
    [[vk::binding(2, 0)]]
    StructuredBuffer<VertexColor> vertex_color;
    
    [[vk::binding(3, 0)]]
    ConstantBuffer<VertexColor> cb_vertex_color[6];
    
    [[vk::binding(4, 0)]]
    SamplerState sampl[12];
    
    [[vk::binding(5, 0)]]
    Texture2D<float3> tex2d[2];
    
    [[vk::binding(6, 0)]]
    RWTexture2D<float3> rw_tex2d[2];
    
    [[vk::binding(7, 0)]]
    Texture2DArray<float3> tex2darray[2];
    
    [[vk::binding(8, 0)]]
    RWTexture2DArray<float3> rw_tex2darray[2];
    
    [[vk::binding(9, 0)]]
    Texture3D<float2> tex3d[2];
    
    [[vk::binding(10, 0)]]
    RWTexture3D<float2> rw_tex3d[2];
    
    [[vk::binding(11, 0)]]
    TextureCube<float> texcube[2];
    
    [[vk::binding(12, 0)]]
    TextureCubeArray<float> rw_texcube[2];
    
    [[vk::binding(13, 0)]]
    RWStructuredBuffer<VertexColor> rw_vertex_color;
    
    [[vk::push_constant]]
    ConstantBuffer<VertexColor> push_cst;
    
    VertexOut main_vs(in VertexIn vIn) {
    
        VertexOut vOut;
        vOut.hpos = float4(vIn.pos, 1.f);
        vOut.color = vertex_color[0].color;   
        vOut.color = push_cst.color;
        return vOut;
    }
    
    float4 main_ps(in VertexOut fIn) : SV_TARGET0  {
        return fIn.color;
    }";

    #[test]
    fn compile_vs_shader() {
        let compiler = HlslCompiler::new().expect(
            "dxcompiler dynamic library needs to be available in the default system search paths",
        );

        let compile_params = CompileParams {
            shader_source: ShaderSource::Code(SHADER),
            entry_point: "main_vs",
            target_profile: "vs_6_1",
            defines: Vec::new(),
        };

        let vs_out = compiler
            .compile(&compile_params)
            .expect("Shader compilation to succeed");
        let refl_info = vs_out.refl_info.expect("Valid reflection info");
        assert_eq!(refl_info.shader_stage, ShaderStageFlags::VERTEX);
        assert_eq!(&refl_info.entry_point_name, "main_vs");
        assert_eq!(refl_info.shader_resources.len(), 1);
        assert_eq!(refl_info.shader_resources[0].name, "vertex_color");
        assert_eq!(refl_info.shader_resources[0].element_count, 1);
        assert_eq!(refl_info.shader_resources[0].binding, 2);
        assert_eq!(refl_info.shader_resources[0].set_index, 0);
        assert_eq!(
            refl_info.shader_resources[0].used_in_shader_stages,
            ShaderStageFlags::VERTEX
        );
    }

    #[test]
    fn compile_ps_shader() {
        let compiler = HlslCompiler::new().expect(
            "dxcompiler dynamic library needs to be available in the default system search paths",
        );

        let compile_params = CompileParams {
            shader_source: ShaderSource::Code(SHADER),
            entry_point: "main_ps",
            target_profile: "ps_6_1",
            defines: Vec::new(),
        };

        let ps_out = compiler
            .compile(&compile_params)
            .expect("Shader compilation to succeed");
        let refl_info = ps_out.refl_info.expect("Valid reflection info");
        assert_eq!(&refl_info.entry_point_name, "main_ps");
        assert_eq!(refl_info.shader_stage, ShaderStageFlags::FRAGMENT);
        assert_eq!(refl_info.shader_resources.len(), 0);
    }
}
