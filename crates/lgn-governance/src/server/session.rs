use async_trait::async_trait;
use lgn_online::{codegen::Context, server::Result};

use crate::api::session::{requests, responses, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn list_current_user_sessions(
        &self,
        _context: &mut Context,
        _request: requests::ListCurrentUserSessionsRequest,
    ) -> Result<responses::ListCurrentUserSessionsResponse> {
        Ok(responses::ListCurrentUserSessionsResponse::Status200(
            vec![].into(),
        ))
    }
}
