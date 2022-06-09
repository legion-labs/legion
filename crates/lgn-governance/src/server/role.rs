use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::{
    codegen::Context,
    server::{ErrorExt, Result},
};

use crate::api::role::{self, requests, responses, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_roles(&self, _context: &mut Context) -> Result<responses::ListRolesResponse> {
        let roles = self.dal.list_roles().await.into_internal_server_error()?;

        Ok(responses::ListRolesResponse::Status200(roles.into()))
    }

    async fn create_role(&self, _context: &mut Context) -> Result<responses::CreateRoleResponse> {
        Ok(responses::CreateRoleResponse::Status201(role::Role {
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
        _context: &mut Context,
        _request: requests::UpdateRoleRequest,
    ) -> Result<responses::UpdateRoleResponse> {
        Ok(responses::UpdateRoleResponse::Status200(role::Role {
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
        _context: &mut Context,
        _request: requests::DeleteRoleRequest,
    ) -> Result<responses::DeleteRoleResponse> {
        Ok(responses::DeleteRoleResponse::Status204)
    }
}
