use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::server::{ErrorExt, Result};

use crate::api::{
    session,
    workspace::{self, server, Api},
};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_current_user_workspaces(
        &self,
        _request: server::ListCurrentUserWorkspacesRequest,
    ) -> Result<server::ListCurrentUserWorkspacesResponse> {
        Ok(server::ListCurrentUserWorkspacesResponse::Status200(
            vec![].into(),
        ))
    }

    async fn get_current_user_workspace(
        &self,
        _request: server::GetCurrentUserWorkspaceRequest,
    ) -> Result<server::GetCurrentUserWorkspaceResponse> {
        Ok(server::GetCurrentUserWorkspaceResponse::Status200(
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
        _request: server::GetCurrentUserWorkspaceSessionRequest,
    ) -> Result<server::GetCurrentUserWorkspaceSessionResponse> {
        Ok(server::GetCurrentUserWorkspaceSessionResponse::Status200(
            session::Session {
                user_id: "lol".to_string().into(),
                workspace_id: "lol".to_string().into(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            },
        ))
    }

    async fn create_current_user_workspace_session(
        &self,
        _request: server::CreateCurrentUserWorkspaceSessionRequest,
    ) -> Result<server::CreateCurrentUserWorkspaceSessionResponse> {
        Ok(
            server::CreateCurrentUserWorkspaceSessionResponse::Status201(session::Session {
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
        _request: server::DeleteCurrentUserWorkspaceSessionRequest,
    ) -> Result<server::DeleteCurrentUserWorkspaceSessionResponse> {
        Ok(
            server::DeleteCurrentUserWorkspaceSessionResponse::Status200(session::Session {
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
