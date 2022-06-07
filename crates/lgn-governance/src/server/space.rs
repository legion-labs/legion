use async_trait::async_trait;
use chrono::{DateTime, Utc};
use lgn_online::codegen::Context;

use crate::api::space::{
    self,
    errors::{ErrorExt, Result},
    requests, responses, Api,
};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_spaces(&self, _context: &mut Context) -> Result<responses::ListSpacesResponse> {
        let spaces = self.dal.list_spaces().await.into_internal_server_error()?;

        Ok(responses::ListSpacesResponse::Status200(
            spaces
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        ))
    }

    async fn create_space(&self, _context: &mut Context) -> Result<responses::CreateSpaceResponse> {
        Ok(responses::CreateSpaceResponse::Status201(space::Space {
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
        _context: &mut Context,
        _request: requests::UpdateSpaceRequest,
    ) -> Result<responses::UpdateSpaceResponse> {
        Ok(responses::UpdateSpaceResponse::Status200(space::Space {
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
        _context: &mut Context,
        _request: requests::DeleteSpaceRequest,
    ) -> Result<responses::DeleteSpaceResponse> {
        Ok(responses::DeleteSpaceResponse::Status204)
    }

    async fn cordon_space(
        &self,
        _context: &mut Context,
        _request: requests::CordonSpaceRequest,
    ) -> Result<responses::CordonSpaceResponse> {
        Ok(responses::CordonSpaceResponse::Status200(space::Space {
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
        _context: &mut Context,
        _request: requests::UncordonSpaceRequest,
    ) -> Result<responses::UncordonSpaceResponse> {
        Ok(responses::UncordonSpaceResponse::Status200(space::Space {
            id: "lol".to_string().into(),
            description: "Some space".to_string(),
            cordoned: false,
            created_at: DateTime::parse_from_rfc3339("2020-01-01T00:00:00Z")
                .into_internal_server_error()?
                .with_timezone(&Utc),
        }))
    }
}
