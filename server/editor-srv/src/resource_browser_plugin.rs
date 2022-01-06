#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]

use lgn_app::prelude::*;
//use lgn_ecs::prelude::*;
use lgn_editor_proto::resource_browser::{
    CloneResourceRequest, CloneResourceResponse, DeleteResourceRequest, DeleteResourceResponse,
    GetResourceTypeNamesRequest, GetResourceTypeNamesResponse, ImportResourceRequest,
    ImportResourceResponse, RenameResourceRequest, RenameResourceResponse, SearchResourcesRequest,
};
//use lgn_tasks::IoTaskPool;

use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::{ResourceId, ResourceType, ResourceTypeAndId};
use lgn_data_transaction::{
    CloneResourceOperation, CreateResourceOperation, DataManager, DeleteResourceOperation,
    LockContext, RenameResourceOperation, Transaction,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{codegen::http::status, Request, Response, Status};

struct ResourceBrowserRPC {
    data_manager: Arc<Mutex<DataManager>>,
}

#[derive(Default)]
pub(crate) struct ResourceBrowserPlugin {}

fn parse_resource_id(value: &str) -> Result<ResourceTypeAndId, Status> {
    value
        .parse::<ResourceTypeAndId>()
        .map_err(|_err| Status::internal(format!("Invalid ResourceID format: {}", value)))
}

impl ResourceBrowserPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

use lgn_editor_proto::resource_browser::{
    resource_browser_server::{ResourceBrowser, ResourceBrowserServer},
    CreateResourceRequest, CreateResourceResponse, ResourceDescription, SearchResourcesResponse,
};

impl Plugin for ResourceBrowserPlugin {
    fn build(&self, app: &mut App) {
        let data_manager = app
            .world
            .get_resource::<Arc<Mutex<DataManager>>>()
            .expect("ResourceBrowser requires DataManager resource");

        let resource_browser_service = ResourceBrowserServer::new(ResourceBrowserRPC {
            data_manager: data_manager.clone(),
        });

        app.world
            .get_resource_mut::<lgn_grpc::GRPCPluginSettings>()
            .expect("the editor plugin requires the gRPC plugin")
            .into_inner()
            .register_service(resource_browser_service);
    }
}

#[tonic::async_trait]
impl ResourceBrowser for ResourceBrowserRPC {
    /// Search for all resources
    async fn search_resources(
        &self,
        request: Request<SearchResourcesRequest>,
    ) -> Result<Response<SearchResourcesResponse>, Status> {
        let request = request.get_ref();
        let data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let descriptors = ctx
            .project
            .resource_list()
            .filter_map(|resource_id| {
                let path: String = ctx
                    .project
                    .resource_name(resource_id)
                    .unwrap_or_else(|_err| "".into())
                    .to_string();

                // Basic Filter
                if !request.search_token.is_empty() {
                    path.find(&request.search_token)?;
                }

                Some(ResourceDescription {
                    id: ResourceTypeAndId::to_string(&resource_id),
                    path,
                    version: 1,
                })
            })
            .collect::<Vec<ResourceDescription>>();

        Ok(Response::new(SearchResourcesResponse {
            next_search_token: "".to_string(),
            total: descriptors.len() as u64,
            resource_descriptions: descriptors,
        }))
    }

    /// Create a new resource
    async fn create_resource(
        &self,
        request: Request<CreateResourceRequest>,
    ) -> Result<Response<CreateResourceResponse>, Status> {
        let request = request.into_inner();

        let resource_type = ResourceType::new(request.resource_type.as_bytes());
        let new_res_id = ResourceTypeAndId {
            kind: resource_type,
            id: ResourceId::new(),
        };

        let transaction = Transaction::new().add_operation(CreateResourceOperation::new(
            new_res_id,
            ResourcePathName::new(request.resource_path.as_str()),
        ));

        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(CreateResourceResponse {
            new_id: new_res_id.to_string(),
        }))
    }

    /// Get the list of all the resources types available (for creation dialog)
    async fn get_resource_type_names(
        &self,
        _request: Request<GetResourceTypeNamesRequest>,
    ) -> Result<Response<GetResourceTypeNamesResponse>, Status> {
        let mut data_manager = self.data_manager.lock().await;
        let ctx = LockContext::new(&data_manager).await;
        let res_types = ctx.resource_registry.get_resource_types();
        Ok(Response::new(GetResourceTypeNamesResponse {
            resource_types: res_types
                .into_iter()
                .map(|(_k, v)| String::from(v))
                .collect(),
        }))
    }

    /// Import a new resource from an existing local file
    async fn import_resource(
        &self,
        _request: Request<ImportResourceRequest>,
    ) -> Result<Response<ImportResourceResponse>, Status> {
        Err(Status::internal(""))
    }

    /// Delete a Resource
    async fn delete_resource(
        &self,
        request: Request<DeleteResourceRequest>,
    ) -> Result<Response<DeleteResourceResponse>, Status> {
        let request = request.get_ref();
        let resource_id = parse_resource_id(request.id.as_str())?;

        let mut transaction =
            Transaction::new().add_operation(DeleteResourceOperation::new(resource_id));
        let mut data_manager = self.data_manager.lock().await;
        data_manager
            .commit_transaction(transaction)
            .await
            .map_err(|err| Status::internal(err.to_string()))?;

        Ok(Response::new(DeleteResourceResponse {}))
    }

    /// Rename a Resource
    async fn rename_resource(
        &self,
        request: Request<RenameResourceRequest>,
    ) -> Result<Response<RenameResourceResponse>, Status> {
        let request = request.get_ref();
        let resource_id = parse_resource_id(request.id.as_str())?;
        let mut transaction = Transaction::new().add_operation(RenameResourceOperation::new(
            resource_id,
            ResourcePathName::new(request.new_path.as_str()),
        ));
        {
            let mut data_manager = self.data_manager.lock().await;
            data_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        Ok(Response::new(RenameResourceResponse {}))
    }

    /// Clone a Resource
    async fn clone_resource(
        &self,
        request: Request<CloneResourceRequest>,
    ) -> Result<Response<CloneResourceResponse>, Status> {
        let request = request.get_ref();
        let source_resource_id = parse_resource_id(request.source_id.as_str())?;
        let clone_res_id = ResourceTypeAndId {
            kind: source_resource_id.kind,
            id: ResourceId::new(),
        };

        let mut transaction = Transaction::new().add_operation(CloneResourceOperation::new(
            source_resource_id,
            clone_res_id,
            ResourcePathName::new(request.clone_path.as_str()),
        ));
        {
            let mut data_manager = self.data_manager.lock().await;
            data_manager
                .commit_transaction(transaction)
                .await
                .map_err(|err| Status::internal(err.to_string()))?;
        }

        Ok(Response::new(CloneResourceResponse {
            new_id: clone_res_id.to_string(),
        }))
    }
}

#[cfg(test)]
mod test {

    use super::{DataManager, ResourceBrowser, ResourceBrowserRPC};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use tonic::{Request, Response, Status};

    use generic_data::offline::TestEntity;
    use lgn_content_store::ContentStoreAddr;
    use lgn_data_build::DataBuildOptions;
    use lgn_data_compiler::compiler_reg::CompilerRegistryOptions;
    use lgn_data_offline::resource::{
        Project, ResourcePathName, ResourceRegistry, ResourceRegistryOptions,
    };
    use lgn_data_runtime::{
        manifest::Manifest, AssetRegistryOptions, Resource, ResourceId, ResourceTypeAndId,
    };

    use lgn_data_transaction::BuildManager;
    use lgn_editor_proto::resource_browser::{
        CloneResourceRequest, CreateResourceRequest, CreateResourceResponse, DeleteResourceRequest,
        GetResourceTypeNamesRequest, GetResourceTypeNamesResponse, RenameResourceRequest,
    };
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_scene_explorer() -> anyhow::Result<()> {
        let project_dir = tempfile::tempdir().unwrap();
        let build_dir = project_dir.path().join("temp");
        std::fs::create_dir(&build_dir).unwrap();

        let project = Project::create_new(&project_dir).expect("failed to create a project");
        let mut registry = ResourceRegistryOptions::new();
        registry = generic_data::offline::register_resource_types(registry);
        let registry = registry.create_async_registry();
        let project = Arc::new(Mutex::new(project));

        let asset_registry = AssetRegistryOptions::new().create();
        let compilers = CompilerRegistryOptions::default()
            .add_compiler(&lgn_compiler_testentity::COMPILER_INFO);

        let options = DataBuildOptions::new(&build_dir, compilers)
            .content_store(&ContentStoreAddr::from(build_dir.as_path()));

        let build_manager = BuildManager::new(options, &project_dir, Manifest::default()).unwrap();

        {
            let data_manager = Arc::new(Mutex::new(DataManager::new(
                project.clone(),
                registry.clone(),
                asset_registry,
                build_manager,
            )));
            let resource_browser = ResourceBrowserRPC {
                data_manager: data_manager.clone(),
            };

            // Read all Resoruce Type registered
            let response = resource_browser
                .get_resource_type_names(Request::new(GetResourceTypeNamesRequest {}))
                .await?
                .into_inner();

            // Validate that sceneEntity should be in the list
            assert!(
                response
                    .resource_types
                    .iter()
                    .filter(|res_type| res_type.as_str() == TestEntity::TYPENAME)
                    .count()
                    == 1
            );

            // Create new resource
            let new_id = resource_browser
                .create_resource(Request::new(CreateResourceRequest {
                    resource_type: TestEntity::TYPENAME.into(),
                    resource_path: "test/root_entity.ent".into(),
                }))
                .await?
                .into_inner()
                .new_id;

            // Rename the created resource
            resource_browser
                .rename_resource(Request::new(RenameResourceRequest {
                    id: new_id.clone(),
                    new_path: "test2/root_entity.ent".into(),
                }))
                .await?;

            // Clone it
            let clone_id = resource_browser
                .clone_resource(Request::new(CloneResourceRequest {
                    source_id: new_id.clone(),
                    clone_path: "test2/clone_entity.ent".into(),
                }))
                .await?
                .into_inner()
                .new_id;

            resource_browser
                .delete_resource(Request::new(DeleteResourceRequest { id: new_id.clone() }))
                .await?;

            resource_browser
                .delete_resource(Request::new(DeleteResourceRequest {
                    id: clone_id.clone(),
                }))
                .await?;

            {
                let mut guard = data_manager.lock().await;
                guard.undo_transaction().await?;
                guard.undo_transaction().await?;
            }
        }
        Ok(())
    }
}
