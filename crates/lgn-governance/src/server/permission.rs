use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::codegen::Context;

use crate::api::{
    common,
    permission::{
        errors::{ErrorExt, Result},
        requests, responses, Api,
    },
};

use super::Server;

#[async_trait]
impl Api for Arc<Server> {
    async fn list_permissions(
        &self,
        _context: &mut Context,
    ) -> Result<responses::ListPermissionsResponse> {
        let permissions = self
            .dal
            .list_permissions()
            .await
            .into_internal_server_error()?;

        Ok(responses::ListPermissionsResponse::Status200(
            permissions.into(),
        ))
    }

    async fn create_permission(
        &self,
        _context: &mut Context,
    ) -> Result<responses::CreatePermissionResponse> {
        Ok(responses::CreatePermissionResponse::Status201(
            common::Permission {
                id: "lol".to_string().into(),
                parent_id: None,
                description: "Some permission".to_string(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            },
        ))
    }

    async fn update_permission(
        &self,
        _context: &mut Context,
        _request: requests::UpdatePermissionRequest,
    ) -> Result<responses::UpdatePermissionResponse> {
        Ok(responses::UpdatePermissionResponse::Status200(
            common::Permission {
                id: "lol".to_string().into(),
                parent_id: None,
                description: "Some permission".to_string(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            },
        ))
    }

    async fn delete_permission(
        &self,
        _context: &mut Context,
        _request: requests::DeletePermissionRequest,
    ) -> Result<responses::DeletePermissionResponse> {
        Ok(responses::DeletePermissionResponse::Status204)
    }
}
