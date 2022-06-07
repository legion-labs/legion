use async_trait::async_trait;
use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_runtime::{
    AssetRegistryOptions, ResourceDescriptor, ResourcePathId, ResourceProcessor, Transform,
};
use lgn_graphics_data::offline::MaterialProcessor;

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline_gltf::GltfFile::TYPE,
        lgn_graphics_data::offline::Material::TYPE,
    ),
    compiler_creator: || Box::new(Gltf2MatCompiler {}),
};

struct Gltf2MatCompiler();

#[async_trait]
impl Compiler for Gltf2MatCompiler {
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

        let resource = resources
            .load_async::<lgn_graphics_data::offline_gltf::GltfFile>(context.source.resource_id())
            .await;
        let resource = resource
            .get(&resources)
            .ok_or_else(|| {
                CompilerError::CompilationError(format!(
                    "Failed to retrieve resource '{}'",
                    context.source.resource_id()
                ))
            })?
            .clone();

        let (outputs, resource_references) = {
            let source = context.source.clone();
            let target_unnamed = context.target_unnamed.clone();

            CompilerContext::execute_workload(move || {
                let mut compiled_resources: Vec<(ResourcePathId, Vec<u8>)> = vec![];
                let mut resource_references = vec![];
                let material_proc = MaterialProcessor {};

                let materials = resource.gather_materials(source.resource_id());
                for (material, name) in materials {
                    let mut compiled_asset = vec![];
                    material_proc
                        .write_resource(&material, &mut compiled_asset)
                        .map_err(|err| {
                            CompilerError::CompilationError(format!(
                                "Writing to file '{}' failed: {}",
                                source.resource_id(),
                                err
                            ))
                        })?;
                    let material_rpid = target_unnamed.new_named(&name);

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
            compiled_resources.push(context.store(&content, id.clone()).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references,
        })
    }
}
