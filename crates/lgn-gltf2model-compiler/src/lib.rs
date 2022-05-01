use async_trait::async_trait;
use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor, ResourceProcessor, Transform};
use lgn_graphics_data::offline::ModelProcessor;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_gltf::GltfFile::TYPE,
        lgn_graphics_data::offline::Model::TYPE,
    ),
    compiler_creator: || Box::new(Gltf2ModelCompiler {}),
};

struct Gltf2ModelCompiler();

#[async_trait]
impl Compiler for Gltf2ModelCompiler {
    async fn init(&self, registry: AssetRegistryOptions) -> AssetRegistryOptions {
        registry.add_loader::<lgn_graphics_data::offline_gltf::GltfFile>()
    }

    async fn hash(
        &self,
        code: &'static str,
        data: &'static str,
        env: &CompilationEnv,
    ) -> CompilerHash {
        hash_code_and_data(code, data, env)
    }

    async fn compile(
        &self,
        mut context: CompilerContext<'_>,
    ) -> Result<CompilationOutput, CompilerError> {
        let resources = context.registry();

        let mut resource_references = Vec::new();

        let outputs = {
            let resource = resources
                .load_async::<lgn_graphics_data::offline_gltf::GltfFile>(
                    context.source.resource_id(),
                )
                .await;
            let resource = resource.get(&resources).ok_or_else(|| {
                CompilerError::CompilationError(format!(
                    "Failed to retrieve resource '{}'",
                    context.source.resource_id()
                ))
            })?;
            let mut compiled_resources = vec![];
            let model_proc = ModelProcessor {};

            let models = resource.gather_models(context.source.resource_id());
            for (model, name) in models {
                let mut compiled_asset = vec![];
                model_proc
                    .write_resource(&model, &mut compiled_asset)
                    .map_err(|err| {
                        CompilerError::CompilationError(format!(
                            "Writing to file '{}' failed: {}",
                            context.source.resource_id(),
                            err
                        ))
                    })?;
                let model_rpid = context.target_unnamed.new_named(&name);
                compiled_resources.push((model_rpid.clone(), compiled_asset));
                for mesh in model.meshes {
                    if let Some(material_rpid) = mesh.material {
                        resource_references.push((model_rpid.clone(), material_rpid));
                    }
                }
            }
            compiled_resources
        };

        let mut compiled_resources = vec![];
        for (id, content) in outputs {
            compiled_resources.push(context.store(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references,
        })
    }
}
