use core::time;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use service::{
    build_db::BuildDb, content_store::ContentStore, data_execution_provider::DataExecutionProvider,
    source_control::SourceControl, worker::Worker,
};

#[derive(Default, Debug)]
pub struct LocalWorker {
    pub continue_execution: AtomicBool,
}

impl Drop for LocalWorker {
    fn drop(&mut self) {
        self.continue_execution.store(false, Ordering::Relaxed);
    }
}

impl LocalWorker {
    pub fn start(
        content_store: Arc<ContentStore>,
        source_control: Arc<SourceControl>,
        build_db: Arc<BuildDb>,
        data_execution: Arc<dyn DataExecutionProvider>,
    ) -> Arc<Self> {
        let worker = Arc::new(LocalWorker {
            continue_execution: AtomicBool::new(true),
        });

        let worker_clone = worker.clone();

        tokio::task::spawn(worker_clone.poll_work(
            content_store,
            source_control,
            build_db,
            data_execution,
        ));

        worker
    }

    async fn poll_work(
        self: Arc<LocalWorker>,
        content_store: Arc<ContentStore>,
        source_control: Arc<SourceControl>,
        build_db: Arc<BuildDb>,
        data_execution_provider: Arc<dyn DataExecutionProvider>,
    ) {
        loop {
            if !self.continue_execution.load(Ordering::Relaxed) {
                return;
            }

            match data_execution_provider.poll_compilation_work().await {
                Some((resource_path_id, build_params, commit_root)) => {
                    // Right now we spawn as many task as we can poll.
                    // Each time we await, it's possible that a new task is spawned.
                    tokio::task::spawn(Worker::spawn_compiler(
                        resource_path_id.clone(),
                        build_params.clone(),
                        commit_root,
                        build_db.clone(),
                        content_store.clone(),
                        data_execution_provider.clone(),
                        source_control.clone(),
                    ));
                }
                None => {
                    tokio::time::sleep(time::Duration::from_millis(10)).await;
                }
            }
        }
    }
}
