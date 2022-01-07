use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::sql::execute_sql;

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
    pub(crate) fn new(owner: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            owner,
        }
    }
}

pub async fn init_workspace_registrations_database(
    sql_connection: &mut sqlx::AnyConnection,
) -> Result<()> {
    let sql = "CREATE TABLE workspace_registrations(id VARCHAR(255), owner VARCHAR(255));
               CREATE UNIQUE INDEX workspace_registration_id on workspace_registrations(id);";

    execute_sql(sql_connection, sql)
        .await
        .context("error creating workspace registrations table and index")
}
