use std::sync::Arc;

use exe::{
    local_data_execution_provider::LocalDataExecutionProvider, local_worker::LocalWorker,
    runtime::Runtime,
};
use service::{
    build_db::BuildDb,
    compiler_interface::{BuildParams, ResourceGuid, Services},
    content_store::ContentStore,
    resource_manager::ResourceManager,
    source_control::{CommitRoot, SourceControlBuilder},
};

pub async fn setup(
    source_data: &[(ResourceGuid, &str)],
) -> (Runtime, Services, BuildParams, CommitRoot) {
    let mut content_store = ContentStore::default();
    let build_params = BuildParams::default();

    let (commit_root, source_control) = source_data
        .iter()
        .fold(SourceControlBuilder::default(), |builder, &(guid, data)| {
            builder.add(guid, data)
        })
        .commit(&mut content_store)
        .await;

    let source_control = Arc::new(source_control);

    let content_store = Arc::new(content_store);
    let build_db = Arc::new(BuildDb::default());

    let data_execution_provider = Arc::new(LocalDataExecutionProvider::new());

    LocalWorker::start(
        content_store.clone(),
        source_control.clone(),
        build_db.clone(),
        data_execution_provider.clone(),
    );

    let resource_manager = ResourceManager::new(
        content_store.clone(),
        commit_root,
        source_control.clone(),
        data_execution_provider.clone(),
        build_db.clone(),
        build_params.clone(),
    );
    (
        Runtime { resource_manager },
        Services {
            content_store: content_store.clone(),
            source_control: source_control.clone(),
            build_db: build_db.clone(),
            data_execution_provider: data_execution_provider.clone(),
            tokio_runtime: tokio::runtime::Handle::current(),
        },
        build_params,
        commit_root,
    )
}

pub fn graph_eq<N, E, Ty, Ix>(
    a: &petgraph::Graph<N, E, Ty, Ix>,
    b: &petgraph::Graph<N, E, Ty, Ix>,
) -> bool
where
    N: PartialEq,
    E: PartialEq,
    Ty: petgraph::EdgeType,
    Ix: petgraph::graph::IndexType + PartialEq,
{
    let a_ns = a.raw_nodes().iter().map(|n| &n.weight);
    let b_ns = b.raw_nodes().iter().map(|n| &n.weight);
    let a_es = a
        .raw_edges()
        .iter()
        .map(|e| (e.source(), e.target(), &e.weight));
    let b_es = b
        .raw_edges()
        .iter()
        .map(|e| (e.source(), e.target(), &e.weight));
    a_ns.eq(b_ns) && a_es.eq(b_es)
}
