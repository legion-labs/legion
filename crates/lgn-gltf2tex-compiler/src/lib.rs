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
use lgn_data_runtime::{AssetRegistryOptions, ResourceDescriptor, Transform};

pub static COMPILER_INFO: CompilerDescriptor = CompilerDescriptor {
    name: env!("CARGO_CRATE_NAME"),
    build_version: DATA_BUILD_VERSION,
    code_version: "1",
    data_version: "1",
    transform: &Transform::new(
        lgn_graphics_data::offline::Gltf::TYPE,
        lgn_graphics_data::runtime::RawTexture::TYPE,
    ),
    compiler_creator: || Box::new(Gltf2TexCompiler {}),
};

struct Gltf2TexCompiler();

#[async_trait]
impl Compiler for Gltf2TexCompiler {
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

        let resource = resources
            .load_async::<lgn_graphics_data::offline::Gltf>(context.source.resource_id())
            .await?;

        // minimize lock
        let content_id = {
            let gltf = resource.get().unwrap();
            gltf.content_id.clone()
        };
        let identifier = lgn_content_store::Identifier::from_str(&content_id)
            .map_err(|err| CompilerError::CompilationError(err.to_string()))?;

        // TODO: aganea - should we read from a Device directly?
        let bytes = context.persistent_provider.read(&identifier).await?;
        let gltf_file = GltfFile::from_bytes(&bytes)?;

        let outputs = {
            let source = context.source.clone();
            let target_unnamed = context.target_unnamed.clone();

            CompilerContext::execute_workload(move || {
                let mut compiled_resources = vec![];
                let textures = gltf_file.gather_textures();
                for texture in textures {
                    let mut compiled_asset = vec![];
                    lgn_data_runtime::to_binary_writer(&texture.0, &mut compiled_asset).map_err(
                        |err| {
                            CompilerError::CompilationError(format!(
                                "Writing to file '{}' failed: {}",
                                context.source.resource_id(),
                                err
                            ))
                        },
                    )?;
                    compiled_resources.push((target_unnamed.new_named(&texture.1), compiled_asset));
                }
                Ok(compiled_resources)
            })
            .await?
        };

        let mut compiled_resources = vec![];
        for (id, content) in outputs {
            compiled_resources.push(context.store_volatile(&content, id).await?);
        }

        Ok(CompilationOutput {
            compiled_resources,
            resource_references: vec![],
        })
    }
}
