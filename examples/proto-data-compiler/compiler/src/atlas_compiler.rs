use core::time;
use std::thread;

use async_trait::async_trait;
use service::{
    compiler_interface::{
        Compiler, CompilerContext, CompilerError, CompilerType, ResourceGuid, ATLAS_COMPILER,
    },
    CompilationInputs, ResourcePathId,
};

pub struct AtlasCompiler;
#[async_trait]
impl Compiler for AtlasCompiler {
    async fn compile(
        &self,
        compilation_inputs: CompilationInputs,
        context: &mut CompilerContext,
    ) -> Result<(), CompilerError> {
        thread::sleep(time::Duration::from_secs(1)); // Emulates work being done

        if compilation_inputs.output_id.source_resource.clone() == ResourceGuid::TextureAtlas {
            let all_textures = context
                .load_many(&[
                    ResourcePathId::new(ResourceGuid::TextureA),
                    ResourcePathId::new(ResourceGuid::TextureB),
                    ResourcePathId::new(ResourceGuid::TextureC),
                ])
                .await?;

            let atlas = all_textures.concat();

            context.store(compilation_inputs.output_id, atlas).await;

            Ok(())
        } else {
            Err(CompilerError::WrongCompiler("not implemented".to_string()))
        }
    }

    fn get_compiler_type(&self) -> CompilerType {
        return ATLAS_COMPILER.to_string();
    }
}
