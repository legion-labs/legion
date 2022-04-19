use async_trait::async_trait;
use lgn_graphics_data::gltf_utils::GltfFile;
use std::{env, str::FromStr};

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_runtime::prelude::*;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline::Gltf::TYPE,
        lgn_graphics_data::offline::Model::TYPE,
    ),
    compiler_creator: || Box::new(Gltf2ModelCompiler {}),
};

struct Gltf2ModelCompiler();

#[async_trait]
impl Compiler for Gltf2ModelCompiler {
    async fn init(&self, mut registry: AssetRegistryOptions) -> AssetRegistryOptions {
        lgn_graphics_data::register_types(&mut registry);
        registry
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

        let gltf_resource = resources
            .load_async::<lgn_graphics_data::offline::Gltf>(context.source.resource_id())
            .await?;

        let content_id = {
            let gltf = gltf_resource.get().unwrap();
            gltf.content_id.clone()
        };
        let identifier = lgn_content_store::Identifier::from_str(&content_id)
            .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

        // TODO: aganea - should we read from a Device directly?
        let bytes = context.persistent_provider.read(&identifier).await?;

        let gltf = GltfFile::from_bytes(&bytes)?;
        let (outputs, resource_references) = {
            let source = context.source.clone();
            let target_unnamed = context.target_unnamed.clone();

            CompilerContext::execute_workload(move || {
                let mut compiled_resources = vec![];
                let mut resource_references = Vec::new();

                let models = gltf.gather_models(context.source.resource_id());
                for (model, name) in models {
                    let mut compiled_asset = vec![];
                    lgn_data_offline::to_json_writer(&model, &mut compiled_asset)?;
                    let model_rpid = context.target_unnamed.new_named(&name);
                    compiled_resources.push((model_rpid.clone(), compiled_asset));
                    for mesh in model.meshes {
                        if let Some(material_rpid) = mesh.material {
                            resource_references.push((model_rpid.clone(), material_rpid));
                        }
                    }
                }
                Ok((compiled_resources, resource_references))
            })
            .await?
        };

        let mut compiled_resources = vec![];
        for (id, content) in outputs {
            compiled_resources.push(context.store_volatile(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references,
        })
    }
}
