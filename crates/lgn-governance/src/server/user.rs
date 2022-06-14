use async_trait::async_trait;
use lgn_tracing::{debug, info, warn};

use crate::{
    api::user::{server, Api},
    check_user_global_permissions,
    types::UserId,
};

use super::{Error, Server};

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
            check_user_global_permissions!(self, caller_user_id, USER_READ);
        }

        let user_info = self
            .aws_cognito_dal
            .get_user_info(&user_id.to_string())
            .await?;

        Ok(server::GetUserInfoResponse::Status200(user_info.into()))
    }

    async fn resolve_user_id(
        &self,
        request: server::ResolveUserIdRequest,
    ) -> lgn_online::server::Result<server::ResolveUserIdResponse> {
        let user_id = match self
            .aws_cognito_dal
            .resolve_username_by("email", &request.email)
            .await
        {
            Ok(user_id) => user_id,
            Err(Error::DoesNotExist) => {
                return Ok(server::ResolveUserIdResponse::Status404(request.email))
            }
            Err(err) => return Err(err.into()),
        };

        Ok(server::ResolveUserIdResponse::Status200(user_id.into()))
    }

    async fn list_user_spaces(
        &self,
        request: server::ListUserSpacesRequest,
    ) -> lgn_online::server::Result<server::ListUserSpacesResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let user_id = request.user_id.into();

        if caller_user_id != user_id {
            check_user_global_permissions!(self, caller_user_id, ROOT);
        }

        // This function is a bit special in that it relies on permissions for
        // space-visibility, and so performs their own permission checks.
        //
        // If a user does not have any `SPACE_READ` permission, they simply
        // won't see any spaces.
        let spaces = self.mysql_dal.list_spaces_for_user(&user_id).await?;

        Ok(server::ListUserSpacesResponse::Status200(
            spaces
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        ))
    }

    async fn list_user_roles(
        &self,
        request: server::ListUserRolesRequest,
    ) -> lgn_online::server::Result<server::ListUserRolesResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let user_id = request.user_id.into();

        if caller_user_id != user_id {
            check_user_global_permissions!(self, caller_user_id, USER_ADMIN);
        }

        let spaces = self.mysql_dal.list_all_roles_for_user(&user_id).await?;

        Ok(server::ListUserRolesResponse::Status200(
            spaces
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        ))
    }
}
