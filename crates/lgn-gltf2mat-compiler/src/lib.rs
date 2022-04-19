use async_trait::async_trait;
use std::{env, str::FromStr};

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor, Transform};

use lgn_graphics_data::gltf_utils::GltfFile;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline::Gltf::TYPE,
        lgn_graphics_data::offline::Material::TYPE,
    ),
    compiler_creator: || Box::new(Gltf2MatCompiler {}),
};

struct Gltf2MatCompiler();

#[async_trait]
impl Compiler for Gltf2MatCompiler {
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

        let mut resource_references = vec![];
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
                let mut compiled_resources: Vec<(ResourcePathId, Vec<u8>)> = vec![];
                let mut resource_references = vec![];
                let materials = gltf.gather_materials(context.source.resource_id());
                for (material, name) in materials {
                    let mut compiled_asset = vec![];
                    lgn_data_offline::to_json_writer(&material, &mut compiled_asset)?;
                    let material_rpid = context.target_unnamed.new_named(&name);

                    compiled_resources.push((material_rpid.clone(), compiled_asset));
                    if let Some(albedo) = material.albedo {
                        resource_references.push((material_rpid.clone(), albedo));
                    }

                    if let Some(normal) = material.normal {
                        resource_references.push((material_rpid.clone(), normal));
                    }

                    if let Some(roughness) = material.roughness {
                        resource_references.push((material_rpid.clone(), roughness));
                    }

                    if let Some(metalness) = material.metalness {
                        resource_references.push((material_rpid.clone(), metalness));
                    }
                }
                Ok((compiled_resources, resource_references))
            })
            .await?
        };

        let mut compiled_resources = vec![];
        for (id, content) in outputs {
            compiled_resources.push(context.store_volatile(&content, id.clone()).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references,
        })
    }
}
