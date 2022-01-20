use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceRegistration {
    pub id: String,
    pub owner: String,
}

impl From<WorkspaceRegistration> for lgn_source_control_proto::WorkspaceRegistration {
    fn from(workspace_registration: WorkspaceRegistration) -> Self {
        Self {
            id: workspace_registration.id,
            owner: workspace_registration.owner,
        }
    }
}

impl From<lgn_source_control_proto::WorkspaceRegistration> for WorkspaceRegistration {
    fn from(workspace_registration: lgn_source_control_proto::WorkspaceRegistration) -> Self {
        Self {
            id: workspace_registration.id,
            owner: workspace_registration.owner,
        }
    }
}

impl WorkspaceRegistration {
    pub fn new(owner: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            owner,
        }
    }

    pub fn new_with_current_user() -> Self {
        let owner = whoami::username();

        Self::new(owner)
    }
}
