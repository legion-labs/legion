use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::codegen::Context;

use crate::api::session::{
    errors::{self, ErrorExt},
    models, requests, responses, Api,
};

use super::Server;

#[async_trait]
impl Api for Arc<Server> {
    async fn list_current_user_sessions(
        &self,
        _context: &mut Context,
        _request: requests::ListCurrentUserSessionsRequest,
    ) -> errors::Result<responses::ListCurrentUserSessionsResponse> {
        Ok(responses::ListCurrentUserSessionsResponse::Status200(
            vec![].into(),
        ))
    }

    async fn list_current_user_workspaces(
        &self,
        _context: &mut Context,
        _request: requests::ListCurrentUserWorkspacesRequest,
    ) -> errors::Result<responses::ListCurrentUserWorkspacesResponse> {
        Ok(responses::ListCurrentUserWorkspacesResponse::Status200(
            vec![].into(),
        ))
    }

    async fn get_current_user_workspace(
        &self,
        _context: &mut Context,
        _request: requests::GetCurrentUserWorkspaceRequest,
    ) -> errors::Result<responses::GetCurrentUserWorkspaceResponse> {
        Ok(responses::GetCurrentUserWorkspaceResponse::Status200(
            models::Workspace {
                id: "lol".to_string().into(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            },
        ))
    }

    async fn get_current_user_workspace_session(
        &self,
        _context: &mut Context,
        _request: requests::GetCurrentUserWorkspaceSessionRequest,
    ) -> errors::Result<responses::GetCurrentUserWorkspaceSessionResponse> {
        Ok(
            responses::GetCurrentUserWorkspaceSessionResponse::Status200(models::Session {
                user_id: "lol".to_string().into(),
                workspace_id: "lol".to_string().into(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            }),
        )
    }

    async fn create_current_user_workspace_session(
        &self,
        _context: &mut Context,
        _request: requests::CreateCurrentUserWorkspaceSessionRequest,
    ) -> errors::Result<responses::CreateCurrentUserWorkspaceSessionResponse> {
        Ok(
            responses::CreateCurrentUserWorkspaceSessionResponse::Status201(models::Session {
                user_id: "lol".to_string().into(),
                workspace_id: "lol".to_string().into(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            }),
        )
    }

    async fn delete_current_user_workspace_session(
        &self,
        _context: &mut Context,
        _request: requests::DeleteCurrentUserWorkspaceSessionRequest,
    ) -> errors::Result<responses::DeleteCurrentUserWorkspaceSessionResponse> {
        Ok(
            responses::DeleteCurrentUserWorkspaceSessionResponse::Status200(models::Session {
                user_id: "lol".to_string().into(),
                workspace_id: "lol".to_string().into(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            }),
        )
    }
}
