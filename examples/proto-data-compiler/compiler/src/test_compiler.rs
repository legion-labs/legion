use async_trait::async_trait;
use service::{
    compiler_interface::{
        Compiler, CompilerContext, CompilerError, CompilerType, ResourceGuid,
        TEST_COMPILATION_APPEND, TEST_COMPILER,
    },
    CompilationInputs, ResourcePathId,
};

pub struct TestCompiler;
#[async_trait]
impl Compiler for TestCompiler {
    async fn compile(
        &self,
        compilation_inputs: CompilationInputs,
        context: &mut CompilerContext,
    ) -> Result<(), CompilerError> {
        let mut compiled_resource = compilation_inputs.data_input.clone();

        // Resource A has a runtime dependency on B
        // Resource C has a runtime dependency on D
        // Resource D has a runtime dependency on E
        // Resource F has a build dependency on G
        // Resource H has a build dependency on I
        // Resource I has a build dependency on J
        // Resource K has a runtime dependency on L
        // Resource L has a build dependency on M
        // Resource N has a build dependency on O
        // Resource O has a runtime dependency on P
        match compilation_inputs.output_id.source_resource {
            ResourceGuid::ResourceF => {
                compiled_resource = compiled_resource
                    + context
                        .load(
                            ResourcePathId::new(ResourceGuid::ResourceG)
                                .transform(TEST_COMPILER.to_string()),
                        )
                        .await
                        .unwrap()
                        .as_str();
            }
            ResourceGuid::ResourceH => {
                compiled_resource = compiled_resource
                    + context
                        .load(
                            ResourcePathId::new(ResourceGuid::ResourceI)
                                .transform(TEST_COMPILER.to_string()),
                        )
                        .await
                        .unwrap()
                        .as_str();
            }
            ResourceGuid::ResourceI => {
                compiled_resource = compiled_resource
                    + context
                        .load(
                            ResourcePathId::new(ResourceGuid::ResourceJ)
                                .transform(TEST_COMPILER.to_string()),
                        )
                        .await
                        .unwrap()
                        .as_str();
            }
            ResourceGuid::ResourceL => {
                compiled_resource = compiled_resource
                    + context
                        .load(
                            ResourcePathId::new(ResourceGuid::ResourceM)
                                .transform(TEST_COMPILER.to_string()),
                        )
                        .await
                        .unwrap()
                        .as_str();
            }
            ResourceGuid::ResourceN => {
                compiled_resource = compiled_resource
                    + context
                        .load(
                            ResourcePathId::new(ResourceGuid::ResourceO)
                                .transform(TEST_COMPILER.to_string()),
                        )
                        .await
                        .unwrap()
                        .as_str();
            }
            _ => {}
        }

        compiled_resource = compiled_resource + TEST_COMPILATION_APPEND;

        context
            .store(compilation_inputs.output_id.clone(), compiled_resource)
            .await;

        match compilation_inputs.output_id.source_resource {
            ResourceGuid::ResourceA => {
                context.add_runtime_references(
                    compilation_inputs.output_id.clone(),
                    &[ResourcePathId::new(ResourceGuid::ResourceB)
                        .transform(TEST_COMPILER.to_string())],
                );
            }
            ResourceGuid::ResourceC => {
                context.add_runtime_references(
                    compilation_inputs.output_id.clone(),
                    &[ResourcePathId::new(ResourceGuid::ResourceD)
                        .transform(TEST_COMPILER.to_string())],
                );
            }
            ResourceGuid::ResourceD => {
                context.add_runtime_references(
                    compilation_inputs.output_id.clone(),
                    &[ResourcePathId::new(ResourceGuid::ResourceE)
                        .transform(TEST_COMPILER.to_string())],
                );
            }
            ResourceGuid::ResourceK => {
                context.add_runtime_references(
                    compilation_inputs.output_id.clone(),
                    &[ResourcePathId::new(ResourceGuid::ResourceL)
                        .transform(TEST_COMPILER.to_string())],
                );
            }
            ResourceGuid::ResourceO => {
                context.add_runtime_references(
                    compilation_inputs.output_id.clone(),
                    &[ResourcePathId::new(ResourceGuid::ResourceP)
                        .transform(TEST_COMPILER.to_string())],
                );
            }
            _ => {}
        }

        Ok(())
    }

    fn get_compiler_type(&self) -> CompilerType {
        return TEST_COMPILER.to_string().to_string();
    }
}
