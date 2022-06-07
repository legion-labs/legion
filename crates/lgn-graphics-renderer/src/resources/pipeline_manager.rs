use lgn_graphics_api::{
    ComputePipelineDef, DeviceContext, GraphicsPipelineDef, Pipeline, Shader, ShaderPackage,
    ShaderStage, ShaderStageDef,
};
use std::{collections::HashMap, sync::Arc};

use lgn_graphics_cgen_runtime::{
    CGenCrateID, CGenRegistry, CGenShaderDef, CGenShaderInstance, CGenShaderKey,
};
use lgn_pso_compiler::{
    CompileDefine, CompileParams, EntryPoint, HlslCompiler, ShaderSource, TargetProfile,
};
use lgn_tracing::{error, span_fn, span_scope};
use parking_lot::RwLock;
use smallvec::SmallVec;
use strum::{EnumCount, IntoEnumIterator};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PipelineHandle(usize);

#[allow(clippy::large_enum_variant)]
#[derive(Clone, PartialEq)]
pub enum PipelineDef {
    Graphics(GraphicsPipelineDef),
    Compute(ComputePipelineDef),
}

struct PipelineInfo {
    pipeline_def: PipelineDef,
}

pub struct PipelineManager {
    device_context: DeviceContext,
    shader_compiler: HlslCompiler,
    cgen_registries: Vec<Arc<CGenRegistry>>,
    infos: RwLock<Vec<PipelineInfo>>,
    pipelines: Vec<Option<Pipeline>>,
    shaders: RwLock<HashMap<(CGenCrateID, CGenShaderKey), Option<Shader>>>,
}

impl PipelineManager {
    pub fn new(device_context: &DeviceContext) -> Self {
        Self {
            device_context: device_context.clone(),
            shader_compiler: HlslCompiler::new().unwrap(),
            cgen_registries: Vec::new(),
            infos: RwLock::new(Vec::new()),
            pipelines: Vec::new(),
            shaders: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_shader_families(&mut self, registry: &Arc<CGenRegistry>) {
        self.cgen_registries.push(registry.clone());
    }

    pub fn get_pipeline(&self, handle: PipelineHandle) -> Option<&Pipeline> {
        if handle.0 >= self.pipelines.len() {
            None
        } else {
            self.pipelines[handle.0].as_ref()
        }
    }

    pub fn register_pipeline(&self, pipeline_def: PipelineDef) -> PipelineHandle {
        {
            let infos = self.infos.read();
            for (i, info) in infos.iter().enumerate() {
                if pipeline_def == info.pipeline_def {
                    return PipelineHandle(i);
                }
            }
        }
        let mut infos = self.infos.write();
        infos.push(PipelineInfo { pipeline_def });
        PipelineHandle(infos.len() - 1)
    }

    #[span_fn]
    pub fn frame_update(&mut self, device_context: &DeviceContext) {
        let infos = self.infos.read();

        self.pipelines.resize(infos.len(), None);

        for i in 0..self.pipelines.len() {
            if self.pipelines[i].is_none() {
                let info = &infos[i];
                let pipeline = match &info.pipeline_def {
                    PipelineDef::Graphics(graphics_pipeline_def) => {
                        device_context.create_graphics_pipeline(graphics_pipeline_def.clone())
                    }
                    PipelineDef::Compute(compute_pipeline_def) => {
                        device_context.create_compute_pipeline(compute_pipeline_def.clone())
                    }
                };
                self.pipelines[i] = Some(pipeline);
            }
        }
    }

    pub fn create_shader(&self, crate_id: CGenCrateID, key: CGenShaderKey) -> Option<Shader> {
        {
            span_scope!("create_shader_check_cache");
            let shaders = self.shaders.read();
            if let Some(shader) = shaders.get(&(crate_id, key)) {
                return shader.clone();
            };
        }

        span_scope!("create_shader");

        // get the instance
        let shader_instance = self.shader_instance(crate_id, key).unwrap();

        // build the define list from options
        let mut defines: SmallVec<[CompileDefine<'_>; CGenShaderKey::MAX_SHADER_OPTIONS]> =
            SmallVec::new();

        let shader_family = self.shader_family(crate_id, key).unwrap();
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

        let shader = self
            .device_context
            .create_shader(shader_stage_defs.to_vec());

        let shader = {
            let mut shaders = self.shaders.write();
            shaders.insert((crate_id, key), Some(shader));
            shaders.entry((crate_id, key)).or_default().clone() // or_default will always return the shader we just inserted, not default.
        };
        shader
    }

    fn cgen_registry(&self, crate_id: CGenCrateID) -> Option<&CGenRegistry> {
        for cgen_registry in &self.cgen_registries {
            if cgen_registry.crate_id == crate_id {
                return Some(cgen_registry);
            }
        }
        None
    }

    fn shader_family(&self, crate_id: CGenCrateID, key: CGenShaderKey) -> Option<&CGenShaderDef> {
        let registry = self.cgen_registry(crate_id)?;

        let shader_family_id = key.shader_id();
        for shader_family in &registry.shader_defs {
            if shader_family.id == shader_family_id {
                return Some(shader_family);
            }
        }
        None
    }

    fn shader_instance(
        &self,
        crate_id: CGenCrateID,
        key: CGenShaderKey,
    ) -> Option<&CGenShaderInstance> {
        let shader_family = self.shader_family(crate_id, key)?;
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
