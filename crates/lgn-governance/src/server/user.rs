use async_trait::async_trait;
use lgn_tracing::{debug, info, warn};

use crate::{
    api::user::{server, Api},
    check_user_global_permissions,
    types::RoleAssignationPatch,
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

        debug!(
            "{} is querying user information for {}",
            caller_user_id, request.user_id.0
        );

        let user_id = match self
            .resolve_api_extended_user_id(request.user_id.clone(), &caller_user_id)
            .await?
        {
            Some(user_id) => user_id,
            None => return Ok(server::GetUserInfoResponse::Status404(request.user_id)),
        };

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
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        let user_id = match self
            .resolve_api_extended_user_id(request.user_id.clone(), &caller_user_id)
            .await?
        {
            Some(user_id) => user_id,
            None => return Ok(server::ResolveUserIdResponse::Status404(request.user_id)),
        };

        Ok(server::ResolveUserIdResponse::Status200(user_id.into()))
    }

    async fn list_user_spaces(
        &self,
        request: server::ListUserSpacesRequest,
    ) -> lgn_online::server::Result<server::ListUserSpacesResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let user_id = match self
            .resolve_api_extended_user_id(request.user_id.clone(), &caller_user_id)
            .await?
        {
            Some(user_id) => user_id,
            None => return Ok(server::ListUserSpacesResponse::Status404(request.user_id)),
        };

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
        let user_id = match self
            .resolve_api_extended_user_id(request.user_id.clone(), &caller_user_id)
            .await?
        {
            Some(user_id) => user_id,
            None => return Ok(server::ListUserRolesResponse::Status404(request.user_id)),
        };

        if caller_user_id != user_id {
            check_user_global_permissions!(self, caller_user_id, USER_ADMIN);
        }

        let roles = self.mysql_dal.list_all_roles_for_user(&user_id).await?;

        Ok(server::ListUserRolesResponse::Status200(
            roles.into_iter().map(Into::into).collect::<Vec<_>>().into(),
        ))
    }

    async fn patch_user_roles(
        &self,
        request: server::PatchUserRolesRequest,
    ) -> lgn_online::server::Result<server::PatchUserRolesResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let user_id = match self
            .resolve_api_extended_user_id(request.user_id.clone(), &caller_user_id)
            .await?
        {
            Some(user_id) => user_id,
            None => return Ok(server::PatchUserRolesResponse::Status404(request.user_id)),
        };

        if caller_user_id != user_id {
            check_user_global_permissions!(self, caller_user_id, USER_ADMIN);
        }

        let patch: RoleAssignationPatch = request.body.try_into().map_err(|err| {
            lgn_online::server::Error::bad_request(format!(
                "invalid role assignation patch: {}",
                err
            ))
        })?;

        let roles = self
            .mysql_dal
            .patch_roles_for_user(&user_id, &patch)
            .await?;

        Ok(server::PatchUserRolesResponse::Status200(
            roles.into_iter().map(Into::into).collect::<Vec<_>>().into(),
        ))
    }

    async fn list_users_aliases(
        &self,
        request: server::ListUsersAliasesRequest,
    ) -> lgn_online::server::Result<server::ListUsersAliasesResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        check_user_global_permissions!(self, caller_user_id, USER_ADMIN);

        let user_aliases = self.mysql_dal.list_user_aliases().await?;

        Ok(server::ListUsersAliasesResponse::Status200(
            user_aliases
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        ))
    }

    async fn register_user_alias(
        &self,
        request: server::RegisterUserAliasRequest,
    ) -> lgn_online::server::Result<server::RegisterUserAliasResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        check_user_global_permissions!(self, caller_user_id, USER_ADMIN);

        let user_alias = request.user_alias.into();
        let user_id = match self
            .resolve_api_extended_user_id(request.body.clone(), &caller_user_id)
            .await?
        {
            Some(user_id) => user_id,
            None => return Ok(server::RegisterUserAliasResponse::Status404(request.body)),
        };

        Ok(
            match self
                .mysql_dal
                .register_user_alias(&user_alias, &user_id)
                .await
            {
                Ok(user_aliases) => server::RegisterUserAliasResponse::Status200(
                    user_aliases
                        .into_iter()
                        .map(Into::into)
                        .collect::<Vec<_>>()
                        .into(),
                ),
                Err(Error::AlreadyExists) => {
                    server::RegisterUserAliasResponse::Status409(user_alias.into())
                }
                Err(err) => return Err(err.into()),
            },
        )
    }

    async fn unregister_user_alias(
        &self,
        request: server::UnregisterUserAliasRequest,
    ) -> lgn_online::server::Result<server::UnregisterUserAliasResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        check_user_global_permissions!(self, caller_user_id, USER_ADMIN);

        let user_alias = request.user_alias.into();

        match self.mysql_dal.unregister_user_alias(&user_alias).await {
            Ok(user_aliases) => Ok(server::UnregisterUserAliasResponse::Status200(
                user_aliases
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<_>>()
                    .into(),
            )),
            Err(Error::DoesNotExist) => Ok(server::UnregisterUserAliasResponse::Status404(
                user_alias.into(),
            )),
            Err(err) => return Err(err.into()),
        }
    }
}
