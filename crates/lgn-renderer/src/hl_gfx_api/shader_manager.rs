use lgn_graphics_api::{DeviceContext, Shader, ShaderPackage, ShaderStage, ShaderStageDef};

use lgn_graphics_cgen_runtime::{
    CGenRegistry, CGenShaderFamily, CGenShaderInstance, CGenShaderKey, CGenShaderOption,
};
use lgn_pso_compiler::{
    CompileDefine, CompileParams, EntryPoint, HlslCompiler, ShaderSource, TargetProfile,
};
use lgn_tracing::span_fn;

use strum::IntoEnumIterator;

#[derive(Default)]
pub struct ShaderRegistry {
    shader_options: Vec<&'static CGenShaderOption>,
    shader_families: Vec<&'static CGenShaderFamily>,
    shader_instances: Vec<&'static CGenShaderInstance>,
}

pub struct ShaderManager {
    device_context: DeviceContext,
    shader_compiler: HlslCompiler,
    shader_registry: ShaderRegistry,
}

impl ShaderManager {
    pub(crate) fn new(device_context: DeviceContext) -> Self {
        Self {
            device_context,
            shader_compiler: HlslCompiler::new().unwrap(),
            shader_registry: ShaderRegistry::default(),
        }
    }

    pub fn register(&mut self, registry: &CGenRegistry) {
        self.shader_registry
            .shader_options
            .extend_from_slice(&registry.shader_options);
        self.shader_registry
            .shader_families
            .extend_from_slice(&registry.shader_families);
        self.shader_registry
            .shader_instances
            .extend_from_slice(&registry.shader_instances);
    }

    fn shader_family(&self, key: CGenShaderKey) -> Option<&CGenShaderFamily> {
        let shader_family_id = key.shader_family_id();
        for shader_family in &self.shader_registry.shader_families {
            if shader_family.id == shader_family_id {
                return Some(shader_family);
            }
        }
        None
    }

    fn entry_point(shader_stage: ShaderStage) -> &'static str {
        match shader_stage {
            ShaderStage::Vertex => "main_vs",
            ShaderStage::Fragment => "main_ps",
            ShaderStage::Compute => "main_cs",
        }
    }

    fn target_profile(shader_stage: ShaderStage) -> TargetProfile {
        match shader_stage {
            ShaderStage::Vertex => TargetProfile::Vertex,
            ShaderStage::Fragment => TargetProfile::Pixel,
            ShaderStage::Compute => TargetProfile::Compute,
        }
    }

    pub fn get_shader(&self, key: CGenShaderKey) -> Shader {
        let shader_family_id = key.shader_family_id();
        let shader_option_mask = key.shader_option_mask();

        for shader_instance in &self.shader_registry.shader_instances {
            if shader_instance.shader_family_id == shader_family_id
                && shader_instance.shader_option_mask == shader_option_mask
            {
                let shader_family = self.shader_family(key).unwrap();
                let mut defines = Vec::new();

                let mut shader_option_mask = shader_option_mask;
                while shader_option_mask != 0 {
                    let trailing_zeros = shader_option_mask.trailing_zeros();
                    shader_option_mask >>= trailing_zeros + 1;
                    let option_index = trailing_zeros as u8;
                    for shader_option in &self.shader_registry.shader_options {
                        if shader_option.shader_family_id == shader_family_id
                            && shader_option.index == option_index
                        {
                            defines.push(CompileDefine {
                                name: shader_option.name,
                                value: None,
                            });
                        }
                    }
                }

                let mut entry_points = Vec::new();

                for shader_stage in ShaderStage::iter() {
                    let shader_stage_flag = shader_stage.into();
                    if (shader_instance.shader_stage_flags & shader_stage_flag) == shader_stage_flag
                    {
                        entry_points.push(EntryPoint {
                            defines: &defines,
                            name: Self::entry_point(shader_stage),
                            target_profile: Self::target_profile(shader_stage),
                        });
                    }
                }

                let compile_params = CompileParams {
                    shader_source: ShaderSource::Path(shader_family.path),
                    global_defines: &[CompileDefine {
                        name: &shader_family.name.to_uppercase(),
                        value: None,
                    }],
                    entry_points: &entry_points,
                };

                let shader_build_result = self.shader_compiler.compile(&compile_params).unwrap();

                let mut shader_stage_defs = Vec::new();

                let mut binary_index = 0;
                for shader_stage in ShaderStage::iter() {
                    let shader_stage_flag = shader_stage.into();
                    if (shader_instance.shader_stage_flags & shader_stage_flag) == shader_stage_flag
                    {
                        let shader_module = self
                            .device_context
                            .create_shader_module(
                                ShaderPackage::SpirV(
                                    shader_build_result.spirv_binaries[binary_index]
                                        .bytecode
                                        .clone(),
                                )
                                .module_def(),
                            )
                            .unwrap();

                        shader_stage_defs.push(ShaderStageDef {
                            entry_point: Self::entry_point(shader_stage).to_string(),
                            shader_stage,
                            shader_module,
                        });

                        binary_index += 1;
                    }
                }

                return self.device_context.create_shader(shader_stage_defs);
            }
        }

        panic!();
    }

    #[span_fn]
    pub fn prepare_vs_ps(&self, shader_path: &str) -> Shader {
        let shader_build_result = self
            .shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Path(shader_path),
                global_defines: &[],
                entry_points: &[
                    EntryPoint {
                        defines: &[],
                        name: "main_vs",
                        target_profile: TargetProfile::Vertex,
                    },
                    EntryPoint {
                        defines: &[],
                        name: "main_ps",
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
                shader_stage: ShaderStage::Vertex,
                shader_module: vert_shader_module,
            },
            ShaderStageDef {
                entry_point: "main_ps".to_owned(),
                shader_stage: ShaderStage::Fragment,
                shader_module: frag_shader_module,
            },
        ])
    }

    #[span_fn]
    pub fn prepare_cs(&self, shader_path: &str) -> Shader {
        let shader_build_result = self
            .shader_compiler
            .compile(&CompileParams {
                shader_source: ShaderSource::Path(shader_path),
                global_defines: &[],
                entry_points: &[EntryPoint {
                    defines: &[],
                    name: "main_cs",
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
            shader_stage: ShaderStage::Compute,
            shader_module: compute_shader_module,
        }])
    }
}
