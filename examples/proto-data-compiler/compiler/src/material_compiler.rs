use async_trait::async_trait;
use service::{
    compiler_interface::{
        Compiler, CompilerContext, CompilerError, CompilerType, ResourceGuid, ATLAS_COMPILER,
        MATERIAL_COMPILER, MATERIAL_CONTENT,
    },
    CompilationInputs, ResourcePathId,
};

pub struct MaterialCompiler;
#[async_trait]
impl Compiler for MaterialCompiler {
    async fn compile(
        &self,
        compilation_inputs: CompilationInputs,
        context: &mut CompilerContext,
    ) -> Result<(), CompilerError> {
        let _material_definition = context
            .load(compilation_inputs.output_id.path_dependency().unwrap())
            .await;

        context
            .store(
                compilation_inputs.output_id.clone(),
                MATERIAL_CONTENT.to_string(),
            )
            .await;

        context.add_runtime_references(
            compilation_inputs.output_id,
            &[ResourcePathId::new(ResourceGuid::TextureAtlas)
                .transform(ATLAS_COMPILER.to_string())],
        );
        Ok(())
    }

    fn get_compiler_type(&self) -> CompilerType {
        return MATERIAL_COMPILER.to_string();
    }
}
