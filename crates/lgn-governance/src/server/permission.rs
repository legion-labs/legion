use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::server::{ErrorExt, Result};

use crate::{
    api::permission::{self, server, Api},
    check_user_global_permissions,
};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_permissions(
        &self,
        request: server::ListPermissionsRequest,
    ) -> Result<server::ListPermissionsResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        // One might think this would require `ROOT` permission, but since
        // `USER_ADMIN` must be able to assign roles (and thus - indirectly -
        // permissions) to users, they need to be able to list those.
        check_user_global_permissions!(self, caller_user_id, USER_ADMIN);

        let permissions = self
            .mysql_dal
            .list_permissions()
            .await
            .into_internal_server_error()?;

        Ok(server::ListPermissionsResponse::Status200(
            permissions.into(),
        ))
    }

    async fn create_permission(
        &self,
        _request: server::CreatePermissionRequest,
    ) -> Result<server::CreatePermissionResponse> {
        Ok(server::CreatePermissionResponse::Status201(
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
        _request: server::UpdatePermissionRequest,
    ) -> Result<server::UpdatePermissionResponse> {
        Ok(server::UpdatePermissionResponse::Status200(
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
        _request: server::DeletePermissionRequest,
    ) -> Result<server::DeletePermissionResponse> {
        Ok(server::DeletePermissionResponse::Status204)
    }
}
