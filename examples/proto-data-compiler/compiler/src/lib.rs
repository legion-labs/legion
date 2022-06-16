use async_ffi::{FfiFuture, FutureExt};
use atlas_compiler::AtlasCompiler;
use futures::future::join_all;
use material_compiler::MaterialCompiler;
use service::{
    compiler_interface::{
        BuildParams, Compiler, CompilerContext, CompilerError, CompilerType, Services,
        ATLAS_COMPILER, MATERIAL_COMPILER, TEST_COMPILER, TEXTURE_COMPILER,
    },
    minimal_hash_internal,
    resource_manager::ResourceManager,
    source_control::CommitRoot,
    CompilationInputs, ResourcePathId,
};
use test_compiler::TestCompiler;
use texture_compiler::TextureCompiler;

pub mod atlas_compiler;
pub mod material_compiler;
pub mod test_compiler;
pub mod texture_compiler;

#[no_mangle]
pub fn compile(
    id: ResourcePathId, // String parsing of resource path id in real code
    build_params: BuildParams,
    commit_root: CommitRoot,
    // Services is only for this prototype. It would not exist in the real context
    // We would reconstruct those informations inside the Compile
    services: Services,
) -> FfiFuture<()> {
    println!("DLL compile function called {}", id);

    async move {
        println!("DLL spawned task {}", id);
        assert!(id.transformations.len() > 0);

        let _guard = services.tokio_runtime.enter();

        let resource_manager = ResourceManager::new(
            services.content_store.clone(),
            commit_root,
            services.source_control.clone(),
            services.data_execution_provider.clone(),
            services.build_db.clone(),
            build_params.clone(),
        );

        // Recursively load all ResourcePathID dependencies
        let data_input = resource_manager
            .clone()
            .load(id.path_dependency().unwrap())
            .await
            .unwrap();

        let compilation_inputs = CompilationInputs {
            output_id: id.clone(),
            data_input,
        };

        let compiler_version = 0; // get_compiler_id(id.transformations[0], commit_root);

        let compiler = find_compiler(
            compilation_inputs.output_id.last_transformation().unwrap(),
            compiler_version,
        )
        .unwrap();

        let mut compiler_context = CompilerContext::new(
            compilation_inputs.output_id.clone(),
            services.content_store.clone(),
            resource_manager.clone(),
        );

        compiler
            .compile(compilation_inputs, &mut compiler_context)
            .await
            .unwrap();

        let version_hash = minimal_hash_internal(
            id.clone(),
            compiler_context.loaded_resources.clone(),
            commit_root,
            &build_params,
            &services.source_control,
        )
        .await
        .unwrap(); // it has been compiled so minimal_hash must exist

        services
            .build_db
            .store(
                id.clone(),
                commit_root,
                version_hash,
                compiler_context.output.clone(),
                compiler_context.loaded_resources.clone(),
            )
            .await;

        let all_references: Vec<ResourcePathId> = compiler_context
            .output
            .content
            .iter()
            .map(|x| x.clone().references)
            .flatten()
            .collect();

        let mut all_futures = Vec::new();

        for reference in all_references {
            if services
                .build_db
                .find(reference.clone(), version_hash)
                .await
                .is_none()
            {
                let data_execution_provider_clone = services.data_execution_provider.clone();
                let build_params_clone = build_params.clone();

                // The runtime_reference is not built, compile it.
                all_futures.push(tokio::task::spawn(async move {
                    data_execution_provider_clone
                        .compile(reference.clone(), build_params_clone, commit_root)
                        .await
                        .unwrap();
                }));
            }
        }
        join_all(all_futures).await;

        services
            .data_execution_provider
            .compilation_completed(
                id,
                build_params.clone(),
                commit_root,
                compiler_context.clone().output,
            )
            .await;
    }
    .into_ffi()
}

pub fn find_compiler(
    compiler_type: CompilerType,
    _commit_root: i32,
) -> Result<Box<dyn Compiler>, CompilerError> {
    // TODO:
    // map commit_root -> compiler_version
    // return the compiler of the right compiler_version.

    if compiler_type == ATLAS_COMPILER {
        Ok(Box::new(AtlasCompiler {}))
    } else if compiler_type == TEXTURE_COMPILER {
        Ok(Box::new(TextureCompiler {}))
    } else if compiler_type == MATERIAL_COMPILER {
        Ok(Box::new(MaterialCompiler {}))
    } else if compiler_type == TEST_COMPILER {
        Ok(Box::new(TestCompiler {}))
    } else {
        Err(CompilerError::CompilerNotFound(compiler_type))
    }
}
