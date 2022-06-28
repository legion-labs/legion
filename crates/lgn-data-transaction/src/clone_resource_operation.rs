//! Transaction Operation to Clone a Resource

use async_trait::async_trait;
#[allow(unused_imports)]
use lgn_data_model::{json_utils::set_property_from_json_string, ReflectionError};
#[allow(unused_imports)]
use lgn_data_runtime::{AssetRegistryReader, ResourceTypeAndId};

use crate::{Error, LockContext, TransactionOperation};

/// Clone a Resource Operation
#[allow(dead_code)]
#[derive(Debug)]
pub struct CloneResourceOperation {
    source_resource_id: ResourceTypeAndId,
    clone_resource_id: ResourceTypeAndId,
    target_parent_id: Option<ResourceTypeAndId>,
}

impl CloneResourceOperation {
    /// Create a new Clone a Resource Operation
    pub fn new(
        source_resource_id: ResourceTypeAndId,
        clone_resource_id: ResourceTypeAndId,
        target_parent_id: Option<ResourceTypeAndId>,
    ) -> Box<Self> {
        Box::new(Self {
            source_resource_id,
            clone_resource_id,
            target_parent_id,
        })
    }
}

#[async_trait]
impl TransactionOperation for CloneResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        let mut resource = ctx
            .project
            .load_resource_untyped(self.source_resource_id)
            .await?;

        let mut source_raw_name = lgn_data_offline::get_meta(resource.as_ref()).name.clone();
        source_raw_name.replace_parent_info(self.target_parent_id, None);
        source_raw_name = ctx.project.get_incremental_name(&source_raw_name).await;

        if let Some(entity_name) = source_raw_name.to_string().rsplit('/').next() {
            // Try to set the name component field
            if let Err(err) = set_property_from_json_string(
                resource.as_reflect_mut(),
                "components[Name].name",
                &serde_json::json!(entity_name).to_string(),
            ) {
                match err {
                    ReflectionError::FieldNotFoundOnStruct(_, _)
                    | ReflectionError::ArrayKeyNotFound(_, _) => {} // ignore missing name components
                    _ => return Err(Error::Reflection(self.clone_resource_id, err)),
                }
            }
        }

        let meta = lgn_data_offline::get_meta_mut(resource.as_mut());
        meta.type_id = self.clone_resource_id;
        meta.name = source_raw_name;

        ctx.project
            .add_resource_with_id(self.clone_resource_id, resource.as_ref())
            .await?;

        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        ctx.project.delete_resource(self.clone_resource_id).await?;
        Ok(())
    }
}
