use std::collections::HashMap;

use tokio::sync::Mutex;

use crate::{compiler_interface::CompilationOutput, source_control::CommitRoot, ResourcePathId};

pub type VersionHash = u128;

#[derive(Debug, Clone)]
struct Entry {
    build_deps: Vec<ResourcePathId>,
    output: CompilationOutput,
}
#[derive(Default, Debug)]
pub struct BuildDb {
    outputs: Mutex<HashMap<(ResourcePathId, VersionHash), Entry>>,
    dependencies: Mutex<HashMap<(ResourcePathId, CommitRoot), Vec<ResourcePathId>>>,
}

impl BuildDb {
    // this is invalidated per CommitRoot which is inefficient
    pub async fn find_dependencies(
        &self,
        id: ResourcePathId,
        commit_root: CommitRoot,
    ) -> Option<Vec<ResourcePathId>> {
        self.dependencies
            .lock()
            .await
            .get(&(id, commit_root))
            .cloned()
    }

    /// (id, commit_id, params) should probably be explicitly collapsed into a hash?
    pub async fn find(
        &self,
        id: ResourcePathId,
        version_hash: VersionHash,
    ) -> Option<(CompilationOutput, Vec<ResourcePathId>)> {
        let out = self
            .outputs
            .lock()
            .await
            .get(&(id.clone(), version_hash))
            .cloned()
            .map(|e| (e.output, e.build_deps));
        println!("finding: {}, {} = {:?}", id, version_hash, out);
        out
    }

    pub async fn store(
        &self,
        id: ResourcePathId,
        commit_root: CommitRoot,
        version_hash: VersionHash,
        output: CompilationOutput,
        build_deps: Vec<ResourcePathId>,
    ) {
        println!("store: {} = {}, {}", id, commit_root, version_hash);
        let entry = Entry {
            output,
            build_deps: build_deps.clone(),
        };
        self.outputs
            .lock()
            .await
            .insert((id.clone(), version_hash), entry);

        self.dependencies
            .lock()
            .await
            .insert((id, commit_root), build_deps);
    }
}
