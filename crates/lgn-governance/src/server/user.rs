use std::sync::Arc;

use async_trait::async_trait;
use lgn_online::codegen::Context;

use crate::api::user::{
    errors::{self},
    responses, Api,
};

use super::Server;

#[async_trait]
impl Api for Arc<Server> {
    async fn list_current_user_spaces(
        &self,
        _context: &mut Context,
    ) -> errors::Result<responses::ListCurrentUserSpacesResponse> {
        Ok(responses::ListCurrentUserSpacesResponse::Status200(
            vec![].into(),
        ))
    }
}
