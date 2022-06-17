mod errors;

use errors::{Error, Result};
use http::{Request, Response};
use hyper::service::Service;

use crate::types::{
    ExtendedUserId, Permission, Role, Space, SpaceId, SpaceUpdate, UserId, UserInfo, Workspace,
};

/// A client for the governance service.
pub struct Client<Inner> {
    permission_client: crate::api::permission::client::Client<Inner>,
    role_client: crate::api::role::client::Client<Inner>,
    user_client: crate::api::user::client::Client<Inner>,
    space_client: crate::api::space::client::Client<Inner>,
    workspace_client: crate::api::workspace::client::Client<Inner>,
}

impl<Inner: Clone> Client<Inner> {
    /// Creates a new client.
    pub fn new(inner: Inner, base_uri: http::Uri) -> Self {
        Self {
            permission_client: crate::api::permission::client::Client::new(
                inner.clone(),
                base_uri.clone(),
            ),
            role_client: crate::api::role::client::Client::new(inner.clone(), base_uri.clone()),
            user_client: crate::api::user::client::Client::new(inner.clone(), base_uri.clone()),
            space_client: crate::api::space::client::Client::new(inner.clone(), base_uri.clone()),
            workspace_client: crate::api::workspace::client::Client::new(inner, base_uri),
        }
    }
}

impl<Inner, ResBody> Client<Inner>
where
    Inner: Service<Request<hyper::Body>, Response = Response<ResBody>> + Send + Sync + Clone,
    Inner::Error: Into<lgn_online::client::Error>,
    Inner::Future: Send,
    ResBody: hyper::body::HttpBody + Send,
    ResBody::Data: Send,
    ResBody::Error: std::error::Error,
{
    /// Initialize the stack.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions or if the stack was already initialized.
    pub async fn init_stack(&self, init_key: &str) -> Result<()> {
        use crate::api::user::client::{InitStackRequest, InitStackResponse};

        match self
            .user_client
            .init_stack(InitStackRequest {
                x_init_key: init_key.to_string(),
            })
            .await?
        {
            InitStackResponse::Status200 { .. } => Ok(()),
            InitStackResponse::Status403 { .. } => Err(Error::Unauthorized),
            InitStackResponse::Status409 { .. } => Err(Error::StackAlreadyInitialized),
        }
    }

    /// Get user information.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions or if the user does not exist.
    pub async fn get_user_info(&self, user_id: &ExtendedUserId) -> Result<UserInfo> {
        use crate::api::user::client::{GetUserInfoRequest, GetUserInfoResponse};

        match self
            .user_client
            .get_user_info(GetUserInfoRequest {
                user_id: user_id.clone().into(),
            })
            .await?
        {
            GetUserInfoResponse::Status200 { body, .. } => Ok(body.into()),
            GetUserInfoResponse::Status404 { body, .. } => {
                Err(Error::UserNotFound(body.try_into()?))
            }
        }
    }

    /// Resolve a user id from an email address.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions or if the user does not exist.
    pub async fn resolve_user_id(&self, user_id: &ExtendedUserId) -> Result<UserId> {
        use crate::api::user::client::{ResolveUserIdRequest, ResolveUserIdResponse};

        match self
            .user_client
            .resolve_user_id(ResolveUserIdRequest {
                user_id: user_id.clone().into(),
            })
            .await?
        {
            ResolveUserIdResponse::Status200 { body, .. } => Ok(body.into()),
            ResolveUserIdResponse::Status404 { body, .. } => {
                Err(Error::UserNotFound(body.try_into()?))
            }
        }
    }

    /// List all the permissions known to the system.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn list_permissions(&self) -> Result<Vec<Permission>> {
        use crate::api::permission::client::ListPermissionsResponse;

        match self.permission_client.list_permissions().await? {
            ListPermissionsResponse::Status200 { body, .. } => Ok(body
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<crate::types::Result<_>>()?),
        }
    }

    /// List all the roles known to the system.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn list_roles(&self) -> Result<Vec<Role>> {
        use crate::api::role::client::ListRolesResponse;

        match self.role_client.list_roles().await? {
            ListRolesResponse::Status200 { body, .. } => Ok(body
                .0
                .into_iter()
                .map(TryInto::try_into)
                .collect::<crate::types::Result<_>>()?),
        }
    }

    /// List all the spaces the user has access to.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn list_spaces(&self) -> Result<Vec<Space>> {
        use crate::api::space::client::ListSpacesResponse;

        match self.space_client.list_spaces().await? {
            ListSpacesResponse::Status200 { body, .. } => {
                Ok(body.0.into_iter().map(Into::into).collect())
            }
        }
    }

    /// Get a specific space.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn get_space(&self, id: impl Into<SpaceId>) -> Result<Space> {
        use crate::api::space::client::{GetSpaceRequest, GetSpaceResponse};

        let space_id = id.into();

        let request = GetSpaceRequest {
            space_id: space_id.into(),
        };

        match self.space_client.get_space(request).await? {
            GetSpaceResponse::Status200 { body, .. } => Ok(body.into()),
            GetSpaceResponse::Status404 { body, .. } => Err(Error::SpaceDoesNotExist(body.into())),
        }
    }

    /// Create a new space.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn create_space(&self, id: impl Into<SpaceId>, description: &str) -> Result<Space> {
        use crate::api::space::client::{CreateSpaceRequest, CreateSpaceResponse};

        let new_space = crate::api::space::NewSpace {
            id: id.into().into(),
            description: description.to_string(),
        };

        let request = CreateSpaceRequest { body: new_space };

        match self.space_client.create_space(request).await? {
            CreateSpaceResponse::Status201 { body, .. } => Ok(body.into()),
            CreateSpaceResponse::Status409 { body, .. } => {
                Err(Error::SpaceAlreadyExists(body.into()))
            }
        }
    }

    /// Update a space.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn update_space(&self, id: impl Into<SpaceId>, update: SpaceUpdate) -> Result<Space> {
        use crate::api::space::client::{UpdateSpaceRequest, UpdateSpaceResponse};

        let request = UpdateSpaceRequest {
            space_id: id.into().into(),
            body: update.into(),
        };

        match self.space_client.update_space(request).await? {
            UpdateSpaceResponse::Status200 { body, .. } => Ok(body.into()),
            UpdateSpaceResponse::Status404 { body, .. } => {
                Err(Error::SpaceDoesNotExist(body.into()))
            }
        }
    }

    /// Delete a space.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn delete_space(&self, id: impl Into<SpaceId>) -> Result<Space> {
        use crate::api::space::client::{DeleteSpaceRequest, DeleteSpaceResponse};

        let space_id = id.into();

        let request = DeleteSpaceRequest {
            space_id: space_id.into(),
        };

        match self.space_client.delete_space(request).await? {
            DeleteSpaceResponse::Status200 { body, .. } => Ok(body.into()),
            DeleteSpaceResponse::Status404 { body, .. } => {
                Err(Error::SpaceDoesNotExist(body.into()))
            }
            DeleteSpaceResponse::Status409 { body, .. } => Err(Error::SpaceInUse(body.into())),
        }
    }

    /// Cordon a space.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn cordon_space(&self, id: impl Into<SpaceId>) -> Result<Space> {
        use crate::api::space::client::{CordonSpaceRequest, CordonSpaceResponse};

        let space_id = id.into();

        let request = CordonSpaceRequest {
            space_id: space_id.into(),
        };

        match self.space_client.cordon_space(request).await? {
            CordonSpaceResponse::Status200 { body, .. } => Ok(body.into()),
            CordonSpaceResponse::Status404 { body, .. } => {
                Err(Error::SpaceDoesNotExist(body.into()))
            }
        }
    }

    /// Uncordon a space.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn uncordon_space(&self, id: impl Into<SpaceId>) -> Result<Space> {
        use crate::api::space::client::{UncordonSpaceRequest, UncordonSpaceResponse};

        let space_id = id.into();

        let request = UncordonSpaceRequest {
            space_id: space_id.into(),
        };

        match self.space_client.uncordon_space(request).await? {
            UncordonSpaceResponse::Status200 { body, .. } => Ok(body.into()),
            UncordonSpaceResponse::Status404 { body, .. } => {
                Err(Error::SpaceDoesNotExist(body.into()))
            }
        }
    }

    /// List all the workspaces visible to the user.
    ///
    /// # Errors
    ///
    /// This function will return an error if the client does not have the
    /// appropriate permissions.
    pub async fn list_workspaces(&self, space_id: &SpaceId) -> Result<Vec<Workspace>> {
        use crate::api::workspace::client::ListWorkspacesResponse;

        let request = crate::api::workspace::client::ListWorkspacesRequest {
            space_id: space_id.clone().into(),
        };

        match self.workspace_client.list_workspaces(request).await? {
            ListWorkspacesResponse::Status200 { body, .. } => {
                Ok(body.0.into_iter().map(Into::into).collect())
            }
        }
    }
}
