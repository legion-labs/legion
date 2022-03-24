use async_trait::async_trait;
use std::env;

use lgn_data_compiler::{
    compiler_api::{
        CompilationEnv, CompilationOutput, Compiler, CompilerContext, CompilerDescriptor,
        CompilerError, CompilerHash, DATA_BUILD_VERSION,
    },
    compiler_utils::hash_code_and_data,
};
use lgn_data_offline::resource::ResourceProcessor;
use lgn_data_offline::Transform;
use lgn_data_runtime::{AssetRegistryOptions, Resource};
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

        let mut resource_references = vec![];

        let outputs = {
            let resource = resources
                .load_async::<lgn_graphics_data::offline_gltf::GltfFile>(
                    context.source.resource_id(),
                )
                .await;
            let resource = resource.get(&resources).unwrap();

            let mut compiled_resources = vec![];
            let material_proc = MaterialProcessor {};

            let materials = resource.gather_materials(context.source.resource_id());
            for (material, name) in materials {
                let mut compiled_asset = vec![];
                material_proc
                    .write_resource(&material, &mut compiled_asset)
                    .unwrap_or_else(|_| panic!("writing to file {}", context.source.resource_id()));
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
            compiled_resources
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
