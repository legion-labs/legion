use std::sync::Arc;

use async_ffi::FfiFuture;

use crate::{
    build_db::BuildDb,
    compiler_interface::{BuildParams, Services},
    content_store::ContentStore,
    data_execution_provider::DataExecutionProvider,
    source_control::{CommitRoot, SourceControl},
    ResourcePathId,
};

pub struct Worker;
impl Worker {
    pub async fn spawn_compiler(
        id: ResourcePathId,
        build_params: BuildParams,
        commit_root: CommitRoot,
        build_db: Arc<BuildDb>,
        content_store: Arc<ContentStore>,
        data_execution_provider: Arc<dyn DataExecutionProvider>,
        source_control: Arc<SourceControl>,
    ) {
        let mut dll_path = std::env::current_exe().unwrap();
        dll_path.pop();
        dll_path.push("compiler.dll");

        let dll_as_str = dll_path.to_str().unwrap();
        println!("Loading {}", dll_as_str);

        unsafe {
            let lib = libloading::Library::new(dll_as_str).unwrap();
            let compile_fn: libloading::Symbol<
                fn(
                    id: ResourcePathId,
                    build_params: BuildParams,
                    commit_root: CommitRoot,
                    services: Services,
                ) -> FfiFuture<()>,
            > = lib.get(b"compile").unwrap();

            let services = Services {
                build_db,
                content_store,
                data_execution_provider,
                source_control,
                tokio_runtime: tokio::runtime::Handle::current(),
            };

            compile_fn(id, build_params, commit_root, services).await
        }
    }
}
