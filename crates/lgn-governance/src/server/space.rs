use async_trait::async_trait;
use lgn_online::server::Result;

use crate::{
    api::space::{server, Api},
    check_user_global_permissions, check_user_space_permissions, user_has_space_permissions,
};

use super::{Error, Server};

#[async_trait]
impl Api for Server {
    async fn list_spaces(
        &self,
        request: server::ListSpacesRequest,
    ) -> Result<server::ListSpacesResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        // This function is a bit special in that it relies on permissions for
        // space-visibility, and so performs their own permission checks.
        //
        // If a user does not have any `SPACE_READ` permission, they simply
        // won't see any spaces.
        let spaces = self.mysql_dal.list_spaces_for_user(&caller_user_id).await?;

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
        request: server::CreateSpaceRequest,
    ) -> Result<server::CreateSpaceResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;

        check_user_global_permissions!(self, caller_user_id, ROOT);

        let space_id = request.body.id.into();

        match self
            .mysql_dal
            .create_space(&space_id, &request.body.description)
            .await
        {
            Ok(space) => Ok(server::CreateSpaceResponse::Status201(space.into())),
            Err(Error::AlreadyExists) => Ok(server::CreateSpaceResponse::Status409(
                self.mysql_dal.get_space(&space_id).await?.into(),
            )),
            Err(err) => Err(err.into()),
        }
    }

    async fn get_space(
        &self,
        request: server::GetSpaceRequest,
    ) -> Result<server::GetSpaceResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let space_id = request.space_id.into();

        // A lack of space visibility will treat the call as if the space did not exist.
        if !user_has_space_permissions!(self, caller_user_id, space_id, SPACE_READ) {
            Ok(server::GetSpaceResponse::Status404(space_id.into()))
        } else {
            match self.mysql_dal.get_space(&space_id).await {
                Ok(space) => Ok(server::GetSpaceResponse::Status200(space.into())),
                Err(Error::DoesNotExist) => {
                    Ok(server::GetSpaceResponse::Status404(space_id.into()))
                }
                Err(err) => Err(err.into()),
            }
        }
    }

    async fn update_space(
        &self,
        request: server::UpdateSpaceRequest,
    ) -> Result<server::UpdateSpaceResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let space_id = request.space_id.into();

        check_user_space_permissions!(self, caller_user_id, space_id, SPACE_ADMIN);

        match self
            .mysql_dal
            .update_space(&space_id, request.body.into())
            .await
        {
            Ok(space) => Ok(server::UpdateSpaceResponse::Status200(space.into())),
            Err(Error::DoesNotExist) => Ok(server::UpdateSpaceResponse::Status404(space_id.into())),
            Err(err) => Err(err.into()),
        }
    }

    async fn delete_space(
        &self,
        request: server::DeleteSpaceRequest,
    ) -> Result<server::DeleteSpaceResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let space_id = request.space_id.into();

        check_user_global_permissions!(self, caller_user_id, ROOT);

        match self.mysql_dal.delete_space(&space_id).await {
            Ok(space) => Ok(server::DeleteSpaceResponse::Status200(space.into())),
            Err(Error::DoesNotExist) => Ok(server::DeleteSpaceResponse::Status404(space_id.into())),
            Err(Error::Conflict) => Ok(server::DeleteSpaceResponse::Status409(
                self.mysql_dal.get_space(&space_id).await?.into(),
            )),
            Err(err) => Err(err.into()),
        }
    }

    async fn cordon_space(
        &self,
        request: server::CordonSpaceRequest,
    ) -> Result<server::CordonSpaceResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let space_id = request.space_id.into();

        check_user_global_permissions!(self, caller_user_id, ROOT);

        match self.mysql_dal.cordon_space(&space_id).await {
            Ok(space) => Ok(server::CordonSpaceResponse::Status200(space.into())),
            Err(Error::DoesNotExist) => Ok(server::CordonSpaceResponse::Status404(space_id.into())),
            Err(err) => Err(err.into()),
        }
    }

    async fn uncordon_space(
        &self,
        request: server::UncordonSpaceRequest,
    ) -> Result<server::UncordonSpaceResponse> {
        let caller_user_id = Self::get_caller_user_id_from_parts(&request.parts)?;
        let space_id = request.space_id.into();

        check_user_global_permissions!(self, caller_user_id, ROOT);

        match self.mysql_dal.uncordon_space(&space_id).await {
            Ok(space) => Ok(server::UncordonSpaceResponse::Status200(space.into())),
            Err(Error::DoesNotExist) => {
                Ok(server::UncordonSpaceResponse::Status404(space_id.into()))
            }
            Err(err) => Err(err.into()),
        }
    }
}
