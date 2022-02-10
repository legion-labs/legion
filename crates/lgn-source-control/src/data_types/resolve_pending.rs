#[derive(Debug)]
pub struct ResolvePending {
    pub relative_path: String,
    pub base_commit_id: String,
    pub theirs_commit_id: String,
}

impl ResolvePending {
    pub fn new(
        canonical_relative_path: String,
        base_commit_id: String,
        theirs_commit_id: String,
    ) -> Self {
        Self {
            relative_path: canonical_relative_path,
            base_commit_id,
            theirs_commit_id,
        }
    }
}
