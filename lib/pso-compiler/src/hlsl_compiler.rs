use std::sync::Arc;

use anyhow::{anyhow, Result};
use hassle_rs::Dxc;
use lgn_graphics_api::{
    PipelineReflection, PushConstant, ShaderResource, ShaderResourceType, ShaderStageFlags,
};
use spirv_reflect::types::{
    ReflectBlockVariable, ReflectDecorationFlags, ReflectDescriptorBinding, ReflectShaderStageFlags,
};
use spirv_tools::{opt::Optimizer, TargetEnv};

use crate::{file_server::FileServerIncludeHandler, FileSystem};

pub struct CompileDefine {
    name: String,
    value: Option<String>,
}

pub enum ShaderSource {
    Code(String),
    Path(String),
}

pub enum TargetProfile {
    Vertex,
    Pixel,
    Compute,
}

impl TargetProfile {
    fn to_profile_string(&self) -> &str {
        match self {
            TargetProfile::Vertex => "vs_6_2",
            TargetProfile::Pixel => "ps_6_2",
            TargetProfile::Compute => "cs_6_2",
        }
    }
}

pub struct EntryPoint {
    pub defines: Vec<CompileDefine>,
    pub name: String,
    pub target_profile: TargetProfile,
}

pub struct CompileParams {
    pub shader_source: ShaderSource,
    pub glob_defines: Vec<CompileDefine>,
    pub entry_points: Vec<EntryPoint>,
}

pub struct SpirvBinary {
    pub bytecode: Vec<u8>,
}

pub struct CompileResult {
    pub pipeline_reflection: PipelineReflection,
    pub spirv_binaries: Vec<SpirvBinary>,
}

struct HlslCompilerInner {
    dxc: Dxc,
    filesystem: FileSystem,
}

#[derive(Clone)]
pub struct HlslCompiler {
    inner: Arc<HlslCompilerInner>,
}

impl HlslCompiler {
    /// Create a new HLSL compiler.
    ///
    /// # Errors
    /// fails if the Dxc library cannot be loaded.
    ///
    pub fn new(filesystem: FileSystem) -> Result<Self> {
        Ok(Self {
            inner: Arc::new(HlslCompilerInner {
                dxc: Dxc::new(None)?,
                filesystem,
            }),
        })
    }

    pub fn filesystem(&self) -> FileSystem {
        self.inner.filesystem.clone()
    }

    /// Compile an HLSL shader.
    ///
    /// # Errors
    /// fails if the shader cannot be compiled.
    ///
    pub fn compile(&self, params: &CompileParams) -> Result<CompileResult> {
        // For each compilation target
        let mut spirv_binaries = Vec::with_capacity(params.entry_points.len());
        let mut pipeline_reflection = PipelineReflection::default();

        for (entry_point_idx, _) in params.entry_points.iter().enumerate() {
            // Compilation
            let unopt_spirv = self.compile_to_unoptimized_spirv(params, entry_point_idx)?;

            // Reflection
            let shader_reflection =
                Self::extract_reflection_info(&unopt_spirv, params, entry_point_idx);
            pipeline_reflection =
                PipelineReflection::merge(&pipeline_reflection, &shader_reflection);

            // Optimize
            let opt_spirv = Self::optimize_spirv(&unopt_spirv)?;

            // Push in the same order
            spirv_binaries.push(opt_spirv);
        }

        // Finalize
        Ok(CompileResult {
            pipeline_reflection,
            spirv_binaries,
        })
    }

    fn compile_to_unoptimized_spirv(
        &self,
        params: &CompileParams,
        entry_point_idx: usize,
    ) -> Result<SpirvBinary> {
        let shader_product = &params.entry_points[entry_point_idx];

        let mut defines = params
            .glob_defines
            .iter()
            .map(|x| (x.name.as_str(), x.value.as_deref()))
            .collect::<Vec<_>>();

        defines.extend(
            shader_product
                .defines
                .iter()
                .map(|x| (x.name.as_str(), x.value.as_deref())),
        );

        // dxc.exe -Od -spirv -fspv-target-env=vulkan1.1 -I d:\\temp\\ -E main_vs -H -T
        // vs_6_0 shaders\shader.hlsl

        let bytecode = self
            .compile_internal(
                &params.shader_source,
                &shader_product.name,
                shader_product.target_profile.to_profile_string(),
                &[
                    "-Od",
                    "-spirv",
                    "-fspv-target-env=vulkan1.1",
                    "-enable-16bit-types",
                    "-HV 2021",
                ],
                &defines,
            )
            .map_err(|err| anyhow!(err))?;

        Ok(SpirvBinary { bytecode })
    }

    fn optimize_spirv(spirv: &SpirvBinary) -> Result<SpirvBinary> {
        let u32spirv = spirv_tools::binary::to_binary(&spirv.bytecode)?;
        let mut optimizer = spirv_tools::opt::create(Some(TargetEnv::Vulkan_1_2));
        optimizer.register_performance_passes();
        let opt_binary = optimizer.optimize(u32spirv, &mut OptimizerCallback {}, None)?;
        Ok(SpirvBinary {
            bytecode: opt_binary.as_bytes().to_vec(),
        })
    }

    fn extract_reflection_info(
        spirv: &SpirvBinary,
        params: &CompileParams,
        entry_point_idx: usize,
    ) -> PipelineReflection {
        let shader_product = &params.entry_points[entry_point_idx];
        let shader_mod = spirv_reflect::create_shader_module(&spirv.bytecode).unwrap();
        let shader_stage = Self::to_shader_stage_flags(shader_mod.get_shader_stage());

        let mut shader_resources = Vec::new();
        for descriptor in &shader_mod
            .enumerate_descriptor_bindings(Some(&shader_product.name))
            .unwrap()
        {
            shader_resources.push(Self::to_shader_resource(shader_stage, descriptor));
        }

        let mut push_constant = None;
        for push_constant_block in &shader_mod
            .enumerate_push_constant_blocks(Some(&shader_product.name))
            .unwrap()
        {
            push_constant = Some(Self::to_push_constant(shader_stage, push_constant_block));
        }

        PipelineReflection {
            shader_resources,
            push_constant,
            compute_threads_per_group: None,
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
            shader_resource_type: Self::to_shader_resource_type(descriptor_binding),
            set_index: descriptor_binding.set,
            binding: descriptor_binding.binding,
            element_count: descriptor_binding.count,
            used_in_shader_stages: shader_stage_flags,
            name: descriptor_binding.name.clone(),
        }
    }

    /// Reference: <https://github.com/Microsoft/DirectXShaderCompiler/blob/master/docs/SPIR-V.rst>
    fn to_shader_resource_type(
        descriptor_binding: &ReflectDescriptorBinding,
    ) -> ShaderResourceType {
        match descriptor_binding.descriptor_type {
            spirv_reflect::types::ReflectDescriptorType::Sampler => ShaderResourceType::Sampler,

            spirv_reflect::types::ReflectDescriptorType::UniformBuffer => {
                ShaderResourceType::ConstantBuffer
            }

            spirv_reflect::types::ReflectDescriptorType::StorageBuffer => {
                let byteaddressbuffer = descriptor_binding.block.members[0].padded_size == 0;
                let readonly = Self::is_descriptor_readonly(descriptor_binding);
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

    fn compile_internal(
        &self,
        shader_source: &ShaderSource,
        entry_point: &str,
        target_profile: &str,
        args: &[&str],
        defines: &[(&str, Option<&str>)],
    ) -> Result<Vec<u8>> {
        let dxc = &self.inner.dxc;
        let compiler = dxc.create_compiler()?;
        let library = dxc.create_library()?;

        let (shader_path, shader_text) = match shader_source {
            ShaderSource::Code(text) => ("_code.hlsl".to_owned(), text.clone()),
            ShaderSource::Path(path) => (
                self.inner
                    .filesystem
                    .translate_path(path)?
                    .as_path()
                    .display()
                    .to_string(),
                self.inner.filesystem.get_file_content(path)?,
            ),
        };

        let blob = library
            .create_blob_with_encoding_from_str(&shader_text)
            .map_err(|x| {
                anyhow!(
                    "Failed to create blob with encoding from string (HRESULT: {})",
                    x
                )
            })?;

        let result = compiler.compile(
            &blob,
            &shader_path,
            entry_point,
            target_profile,
            args,
            Some(&mut FileServerIncludeHandler(self.inner.filesystem.clone())),
            defines,
        );

        match result {
            Err(result) => {
                let error_blob = result.0.get_error_buffer().unwrap();
                Err(anyhow!(library.get_blob_as_string(&error_blob)))
            }
            Ok(result) => {
                let result_blob = result.get_result().unwrap();

                Ok(result_blob.to_vec())
            }
        }
    }
}

struct OptimizerCallback;
impl spirv_tools::error::MessageCallback for OptimizerCallback {
    fn on_message(&mut self, _msg: spirv_tools::error::Message) {
        unimplemented!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let filesystem = FileSystem::new(".").unwrap();
        let compiler = HlslCompiler::new(filesystem).expect(
            "dxcompiler dynamic library needs to be available in the default system search paths",
        );

        let compile_params = CompileParams {
            shader_source: ShaderSource::Code(SHADER.to_owned()),
            glob_defines: Vec::new(),
            entry_points: vec![EntryPoint {
                defines: Vec::new(),
                name: "main_vs".to_owned(),
                target_profile: TargetProfile::Vertex,
            }],
        };

        let vs_out = compiler
            .compile(&compile_params)
            .expect("Shader compilation to succeed");
        let refl_info = vs_out.pipeline_reflection;
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
        let filesystem = FileSystem::new(".").unwrap();
        let compiler = HlslCompiler::new(filesystem).expect(
            "dxcompiler dynamic library needs to be available in the default system search paths",
        );

        let compile_params = CompileParams {
            shader_source: ShaderSource::Code(SHADER.to_owned()),
            glob_defines: Vec::new(),
            entry_points: vec![EntryPoint {
                defines: Vec::new(),
                name: "main_ps".to_owned(),
                target_profile: TargetProfile::Pixel,
            }],
        };

        let ps_out = compiler
            .compile(&compile_params)
            .expect("Shader compilation to succeed");
        let pipeline_reflection = &ps_out.pipeline_reflection;
        assert_eq!(pipeline_reflection.shader_resources.len(), 0);
    }
}
