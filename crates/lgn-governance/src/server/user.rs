use async_trait::async_trait;
use lgn_tracing::{debug, info, warn};

use crate::{
    api::user::{server, Api},
    types::{PermissionId, UserId},
};

use super::Server;

#[async_trait]
impl Api for Server {
    async fn init_stack(
        &self,
        request: server::InitStackRequest,
    ) -> lgn_online::server::Result<server::InitStackResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        if request.x_init_key != self.init_key {
            warn!(
                "{} attempted to initialize the stack with an invalid init key",
                caller_user_id
            );

            Ok(server::InitStackResponse::Status403)
        } else if self.mysql_dal.init_stack(&caller_user_id).await? {
            info!(
                "{} initialized the stack and has now superadmin privileges",
                caller_user_id
            );

            self.permissions_cache.clear().await;

            Ok(server::InitStackResponse::Status200)
        } else {
            warn!(
                "{} attempted to initialize the stack but it was already initialized",
                caller_user_id
            );
            Ok(server::InitStackResponse::Status409)
        }
    }

    async fn get_user_info(
        &self,
        request: server::GetUserInfoRequest,
    ) -> lgn_online::server::Result<server::GetUserInfoResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        let user_id: UserId = {
            if request.user_id.0 == "@me" {
                caller_user_id.clone()
            } else {
                request.user_id.into()
            }
        };

        debug!(
            "{} is querying user information for {}",
            caller_user_id, user_id
        );

        if user_id != caller_user_id {
            self.permissions_cache
                .check_user_permissions(&caller_user_id, None, &[PermissionId::USER_READ])
                .await?;

            let roles_assignations = self
                .mysql_dal
                .list_roles_for_user(&caller_user_id, None)
                .await?;

            println!("LOOOL: {:?}", roles_assignations);
            // TODO: Check permissions.
        }

        let user_info = self
            .aws_cognito_dal
            .get_user_info(&user_id.to_string())
            .await?;

        Ok(server::GetUserInfoResponse::Status200(user_info.into()))
    }

    async fn list_current_user_spaces(
        &self,
        _request: server::ListCurrentUserSpacesRequest,
    ) -> lgn_online::server::Result<server::ListCurrentUserSpacesResponse> {
        Ok(server::ListCurrentUserSpacesResponse::Status200(
            vec![].into(),
        ))
    }
}
