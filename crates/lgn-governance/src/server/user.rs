use async_trait::async_trait;
use lgn_online::codegen::Context;

use crate::api::user::{requests, responses, Api};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn get_user_info(
        &self,
        context: &mut Context,
        _request: requests::GetUserInfoRequest,
    ) -> lgn_online::server::Result<responses::GetUserInfoResponse> {
        let caller_user_info = Self::get_caller_user_info_from_context(context)?;

        Ok(responses::GetUserInfoResponse::Status200(
            caller_user_info.into(),
        ))
    }

    async fn list_current_user_spaces(
        &self,
        _context: &mut Context,
        _request: requests::ListCurrentUserSpacesRequest,
    ) -> lgn_online::server::Result<responses::ListCurrentUserSpacesResponse> {
        Ok(responses::ListCurrentUserSpacesResponse::Status200(
            vec![].into(),
        ))
    }
}
