use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::server::{ErrorExt, Result};

use crate::api::role::{self, server, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_roles(
        &self,
        _request: server::ListRolesRequest,
    ) -> Result<server::ListRolesResponse> {
        let roles = self
            .mysql_dal
            .list_roles()
            .await
            .into_internal_server_error()?;

        Ok(server::ListRolesResponse::Status200(roles.into()))
    }

    async fn create_role(
        &self,
        _request: server::CreateRoleRequest,
    ) -> Result<server::CreateRoleResponse> {
        Ok(server::CreateRoleResponse::Status201(role::Role {
            id: "lol".to_string().into(),
            description: "Some role".to_string(),
            permissions: vec![],
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }

    async fn update_role(
        &self,
        _request: server::UpdateRoleRequest,
    ) -> Result<server::UpdateRoleResponse> {
        Ok(server::UpdateRoleResponse::Status200(role::Role {
            id: "lol".to_string().into(),
            description: "Some role".to_string(),
            permissions: vec![],
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }

    async fn delete_role(
        &self,
        _request: server::DeleteRoleRequest,
    ) -> Result<server::DeleteRoleResponse> {
        Ok(server::DeleteRoleResponse::Status204)
    }
}
