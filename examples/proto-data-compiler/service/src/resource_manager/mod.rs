use std::{collections::VecDeque, sync::Arc};

use tokio::sync::Mutex;

use crate::{
    build_db::{BuildDb, VersionHash},
    compiler_interface::{BuildParams, CompilerError, ResourceGuid},
    content_store::ContentStore,
    data_execution_provider::DataExecutionProvider,
    minimal_hash,
    source_control::{CommitRoot, SourceControl},
    ResourcePathId,
};

#[derive(Debug, Clone)]
struct Workspace {
    commit_root: CommitRoot,
    source_control: Arc<SourceControl>,
}

impl Workspace {
    pub fn from_root(commit_root: CommitRoot, source_control: Arc<SourceControl>) -> Self {
        Self {
            commit_root,
            source_control,
        }
    }

    pub async fn get(&self, content_store: &ContentStore, guid: ResourceGuid) -> Option<String> {
        self.source_control
            .get(content_store, guid, self.commit_root)
            .await
    }

    pub async fn update(
        &mut self,
        content_store: &ContentStore,
        guid: ResourceGuid,
        new_content: &str,
    ) -> Result<(), CompilerError> {
        self.source_control
            .update(content_store, guid, new_content, self.commit_root)
            .await
            .map(|(new_commit_root, _)| {
                self.commit_root = new_commit_root;
                ()
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadCompletion {
    SourceResource,
    Cached(VersionHash),
    Compiled,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct LoadEntry {
    pub id: ResourcePathId,
    pub commit_root: CommitRoot,
    pub result: LoadCompletion,
}

#[derive(Debug)]
pub struct ResourceManager {
    content_store: Arc<ContentStore>,
    build_db: Arc<BuildDb>,
    data_execution_provider: Arc<dyn DataExecutionProvider>,
    build_params: BuildParams,
    workspace: Workspace,
    load_log: Arc<Mutex<VecDeque<LoadEntry>>>,
}

impl Clone for ResourceManager {
    fn clone(&self) -> Self {
        Self {
            content_store: self.content_store.clone(),
            build_db: self.build_db.clone(),
            data_execution_provider: self.data_execution_provider.clone(),
            build_params: self.build_params.clone(),
            workspace: self.workspace.clone(),
            load_log: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

impl ResourceManager {
    pub fn new(
        content_store: Arc<ContentStore>,
        commit_root: CommitRoot,
        source_control: Arc<SourceControl>,
        data_execution_provider: Arc<dyn DataExecutionProvider>,
        build_db: Arc<BuildDb>,
        build_params: BuildParams,
    ) -> Self {
        Self {
            content_store,
            build_db,
            data_execution_provider,
            build_params,
            workspace: Workspace::from_root(commit_root, source_control),
            load_log: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn change(&mut self, guid: ResourceGuid, content: &str) -> Result<(), CompilerError> {
        self.workspace
            .update(&self.content_store, guid, content)
            .await
    }

    // 1. Check if it's a source-resource. If it is, send the content-store link
    // 2. Check in the Build-database if this resource is already in the content-store.
    // 3. If it's not there, request the data executor provider to compile it
    pub async fn load(&self, id: ResourcePathId) -> Result<String, CompilerError> {
        let mut log_entry = LoadEntry {
            id: id.clone(),
            commit_root: self.workspace.commit_root,
            result: LoadCompletion::Failed("Uninit".to_string()),
        };

        if id.is_source_resource() {
            let result = self
                .workspace
                .get(&self.content_store, id.source_resource)
                .await
                .ok_or(CompilerError::NotFound);

            log_entry.result = match result {
                Ok(_) => LoadCompletion::SourceResource,
                Err(_) => LoadCompletion::Failed("Source Resource Not Found".to_string()),
            };

            self.load_log.lock().await.push_back(log_entry);
            return result;
        }

        if let Some(version_hash) = minimal_hash(
            id.clone(),
            self.workspace.commit_root,
            &self.build_params,
            &self.workspace.source_control,
            &self.build_db,
        )
        .await
        {
            if let Some((compilation_output, _)) =
                self.build_db.find(id.clone(), version_hash).await
            {
                log_entry.result = LoadCompletion::Cached(version_hash);
                self.load_log.lock().await.push_back(log_entry);
                return Ok(self
                    .content_store
                    .find(compilation_output.content[0].addr)
                    .await
                    .unwrap());
            }
        }

        if let Ok(compilation_output) = self
            .data_execution_provider
            .compile(
                id.clone(),
                self.build_params.clone(),
                self.workspace.commit_root,
            )
            .await
        {
            log_entry.result = LoadCompletion::Compiled;
            self.load_log.lock().await.push_back(log_entry);
            return Ok(self
                .content_store
                .find(compilation_output.content[0].addr)
                .await
                .unwrap());
        }

        log_entry.result = LoadCompletion::Failed("Not Found".to_string());
        self.load_log.lock().await.push_back(log_entry);
        Err(CompilerError::NotFound)
    }

    pub async fn pop_log(&self) -> Option<LoadEntry> {
        self.load_log.lock().await.pop_front()
    }
}
