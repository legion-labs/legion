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
use lgn_data_runtime::{AssetRegistryOptions, Resource, ResourceDescriptor, Transform};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline::Gltf::TYPE,
        lgn_graphics_data::offline::Gltf::TYPE,
    ),
    compiler_creator: || Box::new(GltfCompiler {}),
};

struct GltfCompiler();

#[async_trait]
impl Compiler for GltfCompiler {
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
        lgn_tracing::async_span_scope!("compiler_gltf");

        let resources = context.registry();

        let resource = {
            lgn_tracing::async_span_scope!("load-resource-def");
            resources
                .load_async::<lgn_graphics_data::offline::Gltf>(context.source.resource_id())
                .await?
        };

        // minimize lock
        let content_id = {
            let gltf = resource.get().unwrap();
            gltf.content_id.clone()
        };

        let raw_data = {
            lgn_tracing::async_span_scope!("content store read");
            let identifier = lgn_content_store::Identifier::from_str(&content_id)
                .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

            // TODO: aganea - should we read from a Device directly?
            context.persistent_provider.read(&identifier).await?
        };

        let outputs = {
            let source = context.source.clone();
            let target_unnamed =
                lgn_data_runtime::ResourcePathId::from(context.target_unnamed.source_resource());

            CompilerContext::execute_workload(move || {
                let gltf = {
                    lgn_tracing::span_scope!("GltfFile::from_bytes");
                    GltfFile::from_bytes(&raw_data)?
                };

                let mut compiled_resources = vec![];
                let mut resource_references = vec![];

                {
                    // Extract Textures
                    lgn_tracing::span_scope!("Extract Textires");
                    let textures = {
                        lgn_tracing::span_scope!("gather_texture");
                        gltf.gather_textures()
                    };
                    for (texture, texture_name) in textures {
                        lgn_tracing::span_scope!("RawTexture write");
                        let mut compiled_asset = vec![];
                        lgn_data_runtime::to_binary_writer(&texture, &mut compiled_asset).map_err(
                            |err| {
                                CompilerError::CompilationError(format!(
                                    "Writing to file '{}' failed: {}",
                                    source.resource_id(),
                                    err
                                ))
                            },
                        )?;
                        compiled_resources.push((
                            target_unnamed.push_named(texture.get_resource_type(), &texture_name),
                            compiled_asset,
                        ));
                    }
                }

                {
                    // Extract Materials
                    lgn_tracing::span_scope!("Extract Material");
                    let (materials, texture_references) =
                        gltf.gather_materials(source.resource_id());
                    resource_references.extend(texture_references);

                    for (material, name) in materials {
                        let mut compiled_asset = vec![];
                        lgn_data_runtime::to_binary_writer(&material, &mut compiled_asset)?;
                        let material_rpid =
                            target_unnamed.push_named(material.get_resource_type(), &name);
                        compiled_resources.push((material_rpid.clone(), compiled_asset));
                    }
                }

                {
                    // Extract Models
                    lgn_tracing::span_scope!("Extract Models");
                    let (models, model_references) = gltf.gather_models(source.resource_id());
                    resource_references.extend(model_references);
                    for (model, name) in models {
                        lgn_tracing::span_scope!("Model Write");
                        let mut compiled_asset = vec![];
                        lgn_data_runtime::to_binary_writer(&model, &mut compiled_asset)?;
                        let model_rpid =
                            target_unnamed.push_named(model.get_resource_type(), &name);
                        compiled_resources.push((model_rpid.clone(), compiled_asset));
                    }
                }

                Ok(compiled_resources)
            })
            .await?
        };

        let compiled_resources = {
            lgn_tracing::async_span_scope!("Store Write");
            let mut compiled_resources = vec![];
            for (id, content) in outputs {
                compiled_resources.push(context.store_volatile(&content, id).await?);
            }
            compiled_resources
        };

        Ok(CompilationOutput {
            compiled_resources,
            resource_references: vec![],
        })
    }
}
