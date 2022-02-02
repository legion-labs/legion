use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::{CanonicalPath, Change, PendingBranchMerge, ResolvePending, Result};

#[async_trait]
pub trait WorkspaceBackend: Send + Sync {
    async fn get_current_branch(&self) -> Result<(String, String)>;
    async fn set_current_branch(&self, branch_name: &str, commit_id: &str) -> Result<()>;
    async fn get_staged_changes(&self) -> Result<BTreeMap<CanonicalPath, Change>>;
    async fn save_staged_changes(&self, changes: &[Change]) -> Result<()>;
    async fn clear_staged_changes(&self, changes: &[Change]) -> Result<()>;
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
