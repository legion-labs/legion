use async_trait::async_trait;

use crate::{LocalChange, PendingBranchMerge, ResolvePending, Result};

#[async_trait]
pub trait WorkspaceBackend: Send + Sync {
    async fn get_current_branch(&self) -> Result<(String, String)>;
    async fn set_current_branch(&self, branch_name: &str, commit_id: &str) -> Result<()>;
    async fn get_local_changes(&self) -> Result<Vec<LocalChange>>;
    async fn find_local_change(&self, canonical_relative_path: &str)
        -> Result<Option<LocalChange>>;
    async fn save_local_change(&self, change_spec: &LocalChange) -> Result<()>;
    async fn clear_local_changes(&self, changes: &[LocalChange]) -> Result<()>;
    async fn read_pending_branch_merges(&self) -> Result<Vec<PendingBranchMerge>>;
    async fn clear_pending_branch_merges(&self) -> Result<()>;
    async fn save_pending_branch_merge(&self, merge_spec: &PendingBranchMerge) -> Result<()>;
    async fn save_resolve_pending(&self, resolve_pending: &ResolvePending) -> Result<()>;
    async fn clear_resolve_pending(&self, resolve_pending: &ResolvePending) -> Result<()>;
    async fn find_resolve_pending(
        &self,
        canonical_relative_path: &str,
    ) -> Result<Option<ResolvePending>>;
    async fn read_resolves_pending(&self) -> Result<Vec<ResolvePending>>;
}
