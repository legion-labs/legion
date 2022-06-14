use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::server::{ErrorExt, Result};

use crate::api::space::{self, server, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_spaces(
        &self,
        _request: server::ListSpacesRequest,
    ) -> Result<server::ListSpacesResponse> {
        let spaces = self
            .mysql_dal
            .list_spaces()
            .await
            .into_internal_server_error()?;

        Ok(server::ListSpacesResponse::Status200(
            spaces
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        ))
    }

    async fn create_space(
        &self,
        _request: server::CreateSpaceRequest,
    ) -> Result<server::CreateSpaceResponse> {
        Ok(server::CreateSpaceResponse::Status201(space::Space {
            id: "lol".to_string().into(),
            description: "Some space".to_string(),
            cordoned: false,
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }

    async fn update_space(
        &self,
        _request: server::UpdateSpaceRequest,
    ) -> Result<server::UpdateSpaceResponse> {
        Ok(server::UpdateSpaceResponse::Status200(space::Space {
            id: "lol".to_string().into(),
            description: "Some space".to_string(),
            cordoned: false,
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }

    async fn delete_space(
        &self,
        _request: server::DeleteSpaceRequest,
    ) -> Result<server::DeleteSpaceResponse> {
        Ok(server::DeleteSpaceResponse::Status204)
    }

    async fn cordon_space(
        &self,
        _request: server::CordonSpaceRequest,
    ) -> Result<server::CordonSpaceResponse> {
        Ok(server::CordonSpaceResponse::Status200(space::Space {
            id: "lol".to_string().into(),
            description: "Some space".to_string(),
            cordoned: false,
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }

    async fn uncordon_space(
        &self,
        _request: server::UncordonSpaceRequest,
    ) -> Result<server::UncordonSpaceResponse> {
        Ok(server::UncordonSpaceResponse::Status200(space::Space {
            id: "lol".to_string().into(),
            description: "Some space".to_string(),
            cordoned: false,
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }
}
