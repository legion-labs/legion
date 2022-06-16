use core::time;
use std::thread;

use async_trait::async_trait;
use service::{
    compiler_interface::{
        Compiler, CompilerContext, CompilerError, CompilerType, ResourceGuid, TEXTURE_A_CONTENT,
        TEXTURE_B_CONTENT, TEXTURE_COMPILER, TEXTURE_C_CONTENT,
    },
    CompilationInputs,
};

pub struct TextureCompiler;
#[async_trait]
impl Compiler for TextureCompiler {
    async fn compile(
        &self,
        compilation_inputs: CompilationInputs,
        context: &mut CompilerContext,
    ) -> Result<(), CompilerError> {
        thread::sleep(time::Duration::from_secs(1));

        let source_content = match compilation_inputs.output_id.source_resource {
            ResourceGuid::TextureA => TEXTURE_A_CONTENT,
            ResourceGuid::TextureB => TEXTURE_B_CONTENT,
            ResourceGuid::TextureC => TEXTURE_C_CONTENT,
            _ => "", // return CompilerError::WrongCompiler("".to_string()),
        }
        .to_string();

        context
            .store(compilation_inputs.output_id, source_content.clone())
            .await;

        Ok(())
    }

    fn get_compiler_type(&self) -> CompilerType {
        return TEXTURE_COMPILER.to_string();
    }
}
