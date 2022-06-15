use async_trait::async_trait;
use lgn_online::server::Result;

use crate::api::session::{server, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_current_user_sessions(
        &self,
        _request: server::ListCurrentUserSessionsRequest,
    ) -> Result<server::ListCurrentUserSessionsResponse> {
        Ok(server::ListCurrentUserSessionsResponse::Status200(
            vec![].into(),
        ))
    }
}
