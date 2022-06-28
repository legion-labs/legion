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
    async fn list_workspaces(
        &self,
        _request: server::ListWorkspacesRequest,
    ) -> Result<server::ListWorkspacesResponse> {
        Ok(server::ListWorkspacesResponse::Status200(
            vec![workspace::Workspace {
                id: "lol".to_string().into(),
                name: "myws".to_string(),
                description: "lol".to_string(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            }]
            .into(),
        ))
    }

    async fn create_workspace(
        &self,
        _request: server::CreateWorkspaceRequest,
    ) -> Result<server::CreateWorkspaceResponse> {
        Ok(server::CreateWorkspaceResponse::Status200(
            workspace::Workspace {
                id: "lol".to_string().into(),
                name: "myws".to_string(),
                description: "lol".to_string(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            },
        ))
    }

    async fn get_workspace(
        &self,
        _request: server::GetWorkspaceRequest,
    ) -> Result<server::GetWorkspaceResponse> {
        Ok(server::GetWorkspaceResponse::Status200(
            workspace::Workspace {
                id: "lol".to_string().into(),
                name: "myws".to_string(),
                description: "lol".to_string(),
                created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
                last_updated_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                    .into_internal_server_error()?
                    .with_timezone(&Utc),
            },
        ))
    }

    async fn get_workspace_session(
        &self,
        _request: server::GetWorkspaceSessionRequest,
    ) -> Result<server::GetWorkspaceSessionResponse> {
        Ok(server::GetWorkspaceSessionResponse::Status200(
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

    async fn create_workspace_session(
        &self,
        _request: server::CreateWorkspaceSessionRequest,
    ) -> Result<server::CreateWorkspaceSessionResponse> {
        Ok(server::CreateWorkspaceSessionResponse::Status201(
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

    async fn delete_workspace_session(
        &self,
        _request: server::DeleteWorkspaceSessionRequest,
    ) -> Result<server::DeleteWorkspaceSessionResponse> {
        Ok(server::DeleteWorkspaceSessionResponse::Status200(
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
}
