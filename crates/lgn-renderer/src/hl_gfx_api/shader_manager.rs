use lgn_graphics_api::{DeviceContext, Shader, ShaderPackage, ShaderStageDef, ShaderStageFlags};

use lgn_graphics_cgen_runtime::CGenShaderDef;
use lgn_pso_compiler::{CompileParams, EntryPoint, HlslCompiler, ShaderSource, TargetProfile};
use lgn_tracing::span_fn;

pub struct ShaderManager {
    device_context: DeviceContext,
    shader_compiler: HlslCompiler,
}

impl ShaderManager {
    pub(crate) fn new(device_context: DeviceContext) -> Self {
        Self {
            device_context,
            shader_compiler: HlslCompiler::new().unwrap(),
        }
    }

    pub fn load(&self, _shader_def: &CGenShaderDef) {}

    #[span_fn]
    pub fn prepare_vs_ps(&self, shader_path: &str) -> Shader {
        let shader_build_result = self
            .shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Path(shader_path.to_string()),
                global_defines: Vec::new(),
                entry_points: vec![
                    EntryPoint {
                        defines: Vec::new(),
                        name: "main_vs".to_owned(),
                        target_profile: TargetProfile::Vertex,
                    },
                    EntryPoint {
                        defines: Vec::new(),
                        name: "main_ps".to_owned(),
                        target_profile: TargetProfile::Pixel,
                    },
                ],
            })
            .unwrap();

        let vert_shader_module = self
            .device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        let frag_shader_module = self
            .device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[1].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        self.device_context.create_shader(vec![
            ShaderStageDef {
                entry_point: "main_vs".to_owned(),
                shader_stage: ShaderStageFlags::VERTEX,
                shader_module: vert_shader_module,
            },
            ShaderStageDef {
                entry_point: "main_ps".to_owned(),
                shader_stage: ShaderStageFlags::FRAGMENT,
                shader_module: frag_shader_module,
            },
        ])
    }

    #[span_fn]
    pub fn prepare_cs(&self, shader_path: &str) -> Shader {
        let shader_build_result = self
            .shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Path(shader_path.to_string()),
                global_defines: Vec::new(),
                entry_points: vec![EntryPoint {
                    defines: Vec::new(),
                    name: "main_cs".to_owned(),
                    target_profile: TargetProfile::Compute,
                }],
            })
            .unwrap();

        let compute_shader_module = self
            .device_context
            .create_shader_module(
                ShaderPackage::SpirV(shader_build_result.spirv_binaries[0].bytecode.clone())
                    .module_def(),
            )
            .unwrap();

        self.device_context.create_shader(vec![ShaderStageDef {
            entry_point: "main_cs".to_owned(),
            shader_stage: ShaderStageFlags::COMPUTE,
            shader_module: compute_shader_module,
        }])
    }
}
