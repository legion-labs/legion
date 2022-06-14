use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::server::{ErrorExt, Result};

use crate::api::permission::{self, requests, responses, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_permissions(
        &self,
        _parts: http::request::Parts,
    ) -> Result<responses::ListPermissionsResponse> {
        let permissions = self
            .mysql_dal
            .list_permissions()
            .await
            .into_internal_server_error()?;

        Ok(responses::ListPermissionsResponse::Status200(
            permissions.into(),
        ))
    }

    async fn create_permission(
        &self,
        _parts: http::request::Parts,
    ) -> Result<responses::CreatePermissionResponse> {
        Ok(responses::CreatePermissionResponse::Status201(
            permission::Permission {
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
        _parts: http::request::Parts,
        _request: requests::UpdatePermissionRequest,
    ) -> Result<responses::UpdatePermissionResponse> {
        Ok(responses::UpdatePermissionResponse::Status200(
            permission::Permission {
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
        _parts: http::request::Parts,
        _request: requests::DeletePermissionRequest,
    ) -> Result<responses::DeletePermissionResponse> {
        Ok(responses::DeletePermissionResponse::Status204)
    }
}
