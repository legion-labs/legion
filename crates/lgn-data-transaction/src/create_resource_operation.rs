//! Transaction Operation to Create a Resource

use async_trait::async_trait;
use lgn_data_offline::resource::ResourcePathName;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Operation to Create a new Resource
pub struct CreateResourceOperation {
    resource_id: ResourceTypeAndId,
    resource_path: ResourcePathName,
    auto_increment_name: bool,
}

impl CreateResourceOperation {
    /// Create a new `CreateResourceOperation`
    pub fn new(
        resource_id: ResourceTypeAndId,
        resource_path: ResourcePathName,
        auto_increment_name: bool,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            resource_path,
            auto_increment_name,
        })
    }
}

// From a specific resource_path, validate that the resource doesn't already exists
// or increment the suffix number until resource name is not used
// Ex: /world/sample => /world/sample1
// Ex: /world/instance1099 => /world/instance1100
fn assign_resource_path(
    resource_path: &ResourcePathName,
    project: &lgn_data_offline::resource::Project,
) -> ResourcePathName {
    let mut name: String = resource_path.to_string();

    // extract the current suffix number if avaiable
    let mut suffix = String::new();
    name.chars()
        .rev()
        .take_while(|c| c.is_digit(10))
        .for_each(|c| suffix.insert(0, c));

    name = name.trim_end_matches(suffix.as_str()).into();
    let mut index = suffix.parse::<u32>().unwrap_or(1);
    loop {
        // Check if the resource_name exists, if not increment index
        let new_path: ResourcePathName = format!("{}{}", name, index).into();
        if !project.exists_named(&new_path) {
            return new_path;
        }
        index += 1;
    }
}

#[async_trait]
impl TransactionOperation for CreateResourceOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let handle = ctx
            .resource_registry
            .new_resource(self.resource_id.kind)
            .ok_or(Error::ResourceCreationFailed(self.resource_id.kind))?;

        // Validate duplicate id/name
        if ctx.project.exists(self.resource_id.id) {
            return Err(Error::ResourceIdAlreadyExist(self.resource_id).into());
        }

        let mut requested_resource_path = self.resource_path.clone();
        if ctx.project.exists_named(&requested_resource_path) {
            if !self.auto_increment_name {
                return Err(Error::ResourcePathAlreadyExist(self.resource_path.clone()).into());
            }
            requested_resource_path = assign_resource_path(&requested_resource_path, &ctx.project);
        }

        if let Some(resource_type_name) = ctx
            .resource_registry
            .get_resource_type_name(self.resource_id.kind)
        {
            ctx.project.add_resource_with_id(
                requested_resource_path,
                resource_type_name,
                self.resource_id.kind,
                self.resource_id,
                &handle,
                &mut ctx.resource_registry,
            )?;
            ctx.loaded_resource_handles.insert(self.resource_id, handle);
        }
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        if let Some(_handle) = ctx.loaded_resource_handles.remove(self.resource_id) {
            ctx.project.delete_resource(self.resource_id.id)?;
        }
        Ok(())
    }
}
