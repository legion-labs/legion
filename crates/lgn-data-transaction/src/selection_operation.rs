//! Transaction Operation to Modify the Active Selection
use crate::{Error, LockContext, TransactionOperation};
use async_trait::async_trait;
use lgn_data_runtime::ResourceTypeAndId;

enum SelectionOpType {
    Set,
    Add,
    Remove,
    Toggle,
}

/// Operation to modify an array Property
pub struct SelectionOperation {
    operation_type: SelectionOpType,
    resource_ids: Vec<ResourceTypeAndId>,
    old_selection: Option<Vec<ResourceTypeAndId>>,
}

impl SelectionOperation {
    /// Return a new operation to set the active selection from a `ResourceId` set
    pub fn set_selection(resource_ids: &[ResourceTypeAndId]) -> Box<Self> {
        Box::new(Self {
            operation_type: SelectionOpType::Set,
            resource_ids: resource_ids.to_vec(),
            old_selection: None,
        })
    }

    /// Return a new operation to add new `ResourceId` to the active selection
    pub fn add_to_selection(resource_ids: &[ResourceTypeAndId]) -> Box<Self> {
        Box::new(Self {
            operation_type: SelectionOpType::Add,
            resource_ids: resource_ids.to_vec(),
            old_selection: None,
        })
    }

    /// Return a new operation to add or remove a item from the selection
    pub fn toggle_selection(resource_ids: &[ResourceTypeAndId]) -> Box<Self> {
        Box::new(Self {
            operation_type: SelectionOpType::Toggle,
            resource_ids: resource_ids.to_vec(),
            old_selection: None,
        })
    }

    /// Return a new operation to remove a set of `ResourceId` from the active selection
    pub fn remove_from_selection(resource_ids: &[ResourceTypeAndId]) -> Box<Self> {
        Box::new(Self {
            operation_type: SelectionOpType::Remove,
            resource_ids: resource_ids.to_vec(),
            old_selection: None,
        })
    }
}

#[async_trait]
impl TransactionOperation for SelectionOperation {
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        match self.operation_type {
            SelectionOpType::Set => {
                self.old_selection = Some(Vec::from_iter(
                    ctx.selection_manager.set_selection(&self.resource_ids),
                ));
            }
            SelectionOpType::Add => ctx.selection_manager.add_to_selection(&self.resource_ids),
            SelectionOpType::Toggle => ctx.selection_manager.toggle_selection(&self.resource_ids),
            SelectionOpType::Remove => ctx
                .selection_manager
                .remove_from_selection(&self.resource_ids),
        }
        Ok(())
    }

    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        match self.operation_type {
            SelectionOpType::Set => {
                if let Some(old_selection) = &self.old_selection {
                    ctx.selection_manager.set_selection(old_selection);
                }
            }
            SelectionOpType::Add => ctx
                .selection_manager
                .remove_from_selection(&self.resource_ids),
            SelectionOpType::Toggle => ctx.selection_manager.toggle_selection(&self.resource_ids),
            SelectionOpType::Remove => ctx.selection_manager.add_to_selection(&self.resource_ids),
        }
        Ok(())
    }
}
