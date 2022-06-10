use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::server::{ErrorExt, Result};

use crate::api::{
    session,
    workspace::{self, requests, responses, Api},
};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_current_user_workspaces(
        &self,
        _parts: http::request::Parts,
        _request: requests::ListCurrentUserWorkspacesRequest,
    ) -> Result<responses::ListCurrentUserWorkspacesResponse> {
        Ok(responses::ListCurrentUserWorkspacesResponse::Status200(
            vec![].into(),
        ))
    }

    async fn get_current_user_workspace(
        &self,
        _parts: http::request::Parts,
        _request: requests::GetCurrentUserWorkspaceRequest,
    ) -> Result<responses::GetCurrentUserWorkspaceResponse> {
        Ok(responses::GetCurrentUserWorkspaceResponse::Status200(
            workspace::Workspace {
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
        _parts: http::request::Parts,
        _request: requests::GetCurrentUserWorkspaceSessionRequest,
    ) -> Result<responses::GetCurrentUserWorkspaceSessionResponse> {
        Ok(
            responses::GetCurrentUserWorkspaceSessionResponse::Status200(session::Session {
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
        _parts: http::request::Parts,
        _request: requests::CreateCurrentUserWorkspaceSessionRequest,
    ) -> Result<responses::CreateCurrentUserWorkspaceSessionResponse> {
        Ok(
            responses::CreateCurrentUserWorkspaceSessionResponse::Status201(session::Session {
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
        _parts: http::request::Parts,
        _request: requests::DeleteCurrentUserWorkspaceSessionRequest,
    ) -> Result<responses::DeleteCurrentUserWorkspaceSessionResponse> {
        Ok(
            responses::DeleteCurrentUserWorkspaceSessionResponse::Status200(session::Session {
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
