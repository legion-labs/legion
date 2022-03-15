//! Transaction Operation to Update a Resource property through Reflection

use async_trait::async_trait;
use lgn_data_model::json_utils::{get_property_as_json_string, set_property_from_json_string};
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

/// Operation to update a Property Value through reflection
#[derive(Debug)]
pub struct UpdatePropertyOperation {
    resource_id: ResourceTypeAndId,
    new_values: Vec<(String, String)>,
    old_values: Option<Vec<String>>,
}

impl UpdatePropertyOperation {
    /// Return a newly created `UpdatePropertyOperation`
    pub fn new(
        resource_id: ResourceTypeAndId,
        new_values: &[(impl AsRef<str>, impl AsRef<str>)],
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            new_values: new_values
                .iter()
                .map(|(a, b)| (a.as_ref().into(), b.as_ref().into()))
                .collect::<Vec<_>>(),
            old_values: None,
        })
    }
}

#[async_trait]
impl TransactionOperation for UpdatePropertyOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        let resource_handle = ctx.get_or_load(self.resource_id).await?;

        let reflection = ctx
            .resource_registry
            .get_resource_reflection_mut(self.resource_id.kind, &resource_handle)
            .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

        // init old values
        self.old_values.get_or_insert(
            self.new_values
                .iter()
                .map(|(property_name, _new_json)| {
                    let old_json = get_property_as_json_string(reflection, property_name)
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
                    Ok(old_json)
                })
                .collect::<Result<Vec<_>, _>>()?,
        );

        for (path, json_value) in &self.new_values {
            set_property_from_json_string(reflection, path, json_value)
                .map_err(|err| Error::Reflection(self.resource_id, err))?;
        }

        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        if let Some(old_values) = &self.old_values {
            let handle = ctx.get_or_load(self.resource_id).await?;

            let reflection = ctx
                .resource_registry
                .get_resource_reflection_mut(self.resource_id.kind, &handle)
                .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

            if self.new_values.len() == old_values.len() {
                for ((property, _), old_json) in self.new_values.iter().zip(old_values) {
                    set_property_from_json_string(reflection, property, old_json)
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
                }
            }

            ctx.changed_resources.insert(self.resource_id);
        }
        Ok(())
    }
}
