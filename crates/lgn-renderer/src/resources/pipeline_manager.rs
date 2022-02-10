use lgn_graphics_api::{
    DeviceContext, Pipeline, Shader, ShaderPackage, ShaderStage, ShaderStageDef,
};

use lgn_graphics_cgen_runtime::{
    CGenRegistry, CGenShaderFamily, CGenShaderInstance, CGenShaderKey,
};
use lgn_pso_compiler::{
    CompileDefine, CompileParams, EntryPoint, HlslCompiler, ShaderSource, TargetProfile,
};
use lgn_tracing::{error, span_fn};
use parking_lot::RwLock;
use smallvec::SmallVec;
use strum::{EnumCount, IntoEnumIterator};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PipelineHandle(usize);

struct PipelineInfo {
    key: CGenShaderKey,
    create_pipeline: Box<dyn Fn(&DeviceContext, &Shader) -> Pipeline + Send + Sync + 'static>,
}

pub struct PipelineManager {
    device_context: DeviceContext,
    shader_compiler: HlslCompiler,
    shader_families: Vec<&'static CGenShaderFamily>,
    infos: RwLock<Vec<PipelineInfo>>,
    pipelines: Vec<Option<Pipeline>>,
}

impl PipelineManager {
    pub(crate) fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            shader_compiler: HlslCompiler::new().unwrap(),
            shader_families: Vec::new(),
            infos: RwLock::new(Vec::new()),
            pipelines: Vec::new(),
        }
    }

    pub fn register_shader_families(&mut self, registry: &CGenRegistry) {
        self.shader_families
            .extend_from_slice(&registry.shader_families);
    }

    pub fn get_pipeline(&self, handle: PipelineHandle) -> Option<&Pipeline> {
        if handle.0 >= self.pipelines.len() {
            None
        } else {
            self.pipelines[handle.0].as_ref()
        }
    }

    pub fn register_pipeline<F: Fn(&DeviceContext, &Shader) -> Pipeline + Send + Sync + 'static>(
        &self,
        key: CGenShaderKey,
        func: F,
    ) -> PipelineHandle {
        self.shader_instance(key).expect("Invalid shader key");
        {
            let mut infos = self.infos.write();
            infos.push(PipelineInfo {
                key,
                create_pipeline: Box::new(func),
            });
            PipelineHandle(infos.len() - 1)
        }
    }

    #[span_fn]
    pub fn update(&mut self) {
        let infos = self.infos.read();

        self.pipelines.resize(infos.len(), None);

        for i in 0..self.pipelines.len() {
            if self.pipelines[i].is_none() {
                let info = &infos[i];
                let shader = self.create_shader(info.key);
                if let Some(shader) = shader {
                    let pipeline = (info.create_pipeline)(&self.device_context, &shader);
                    self.pipelines[i] = Some(pipeline);
                }
            }
        }
    }

    #[span_fn]
    fn create_shader(&self, key: CGenShaderKey) -> Option<Shader> {
        // get the instance
        let shader_instance = self.shader_instance(key).unwrap();

        // build the define list from options
        let mut defines: SmallVec<[CompileDefine<'_>; CGenShaderKey::MAX_SHADER_OPTIONS]> =
            SmallVec::new();

        let shader_family = self.shader_family(key).unwrap();
        let mut shader_option_mask = key.shader_option_mask();
        while shader_option_mask != 0 {
            let trailing_zeros = shader_option_mask.trailing_zeros();
            shader_option_mask >>= trailing_zeros + 1;
            let option_index = trailing_zeros as u8;

            for shader_option in shader_family.options {
                if shader_option.index == option_index {
                    defines.push(CompileDefine {
                        name: shader_option.name,
                        value: None,
                    });
                }
            }
        }

        // build the entry point list
        let mut entry_points: SmallVec<[EntryPoint<'_>; ShaderStage::COUNT]> = SmallVec::new();
        for shader_stage in ShaderStage::iter() {
            let shader_stage_flag = shader_stage.into();
            if (shader_instance.stage_flags & shader_stage_flag) == shader_stage_flag {
                entry_points.push(EntryPoint {
                    defines: &defines,
                    name: Self::entry_point(shader_stage),
                    target_profile: Self::target_profile(shader_stage),
                });
            }
        }

        // compile
        let shader_build_result = self.shader_compiler.compile(&CompileParams {
            shader_source: ShaderSource::Path(shader_family.path),
            global_defines: &[CompileDefine {
                name: &shader_family.name.to_uppercase(),
                value: None,
            }],
            entry_points: &entry_points,
        });

        let shader_build_result = match shader_build_result {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to compile shader '{}' with error:\n{}",
                    shader_family.name, e
                );
                return None;
            }
        };

        // build the final shader
        let mut shader_stage_defs: SmallVec<[ShaderStageDef; ShaderStage::COUNT]> = SmallVec::new();
        let mut entry_point_index = 0;
        for shader_stage in ShaderStage::iter() {
            let shader_stage_flag = shader_stage.into();
            if (shader_instance.stage_flags & shader_stage_flag) == shader_stage_flag {
                let shader_module = self.device_context.create_shader_module(
                    ShaderPackage::SpirV(
                        shader_build_result.spirv_binaries[entry_point_index]
                            .bytecode
                            .clone(),
                    )
                    .module_def(),
                );
                let shader_module = match shader_module {
                    Ok(r) => r,
                    Err(e) => {
                        error!(
                            "Failed to create module for shader '{}' with error:\n{}",
                            shader_family.name, e
                        );
                        return None;
                    }
                };

                shader_stage_defs.push(ShaderStageDef {
                    entry_point: Self::entry_point(shader_stage).to_string(),
                    shader_stage,
                    shader_module,
                });

                entry_point_index += 1;
            }
        }

        Some(
            self.device_context
                .create_shader(shader_stage_defs.to_vec()),
        )
    }

    fn shader_family(&self, key: CGenShaderKey) -> Option<&CGenShaderFamily> {
        let shader_family_id = key.shader_family_id();
        for shader_family in &self.shader_families {
            if shader_family.id == shader_family_id {
                return Some(shader_family);
            }
        }
        None
    }

    fn shader_instance(&self, key: CGenShaderKey) -> Option<&CGenShaderInstance> {
        let shader_family = self.shader_family(key)?;
        for shader_instance in shader_family.instances {
            if shader_instance.key == key {
                return Some(shader_instance);
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
}
