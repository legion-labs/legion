use async_trait::async_trait;
use service::{
    compiler_interface::{Compiler, CompilerContext, CompilerError, CompilerType, ENTITY_COMPILER},
    CompilationInputs,
};

pub struct EntityCompiler;
#[async_trait]
impl Compiler for EntityCompiler {
    async fn compile(
        &self,
        _compilation_inputs: CompilationInputs,
        _context: &mut CompilerContext,
    ) -> Result<(), CompilerError> {
        Ok(())
    }

    fn get_compiler_type(&self) -> CompilerType {
        return ENTITY_COMPILER.to_string();
    }
}
