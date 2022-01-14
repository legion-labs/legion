//! Transaction Operation to Modify an Array (Add,Remove,Reorder element)

use async_trait::async_trait;
use lgn_data_model::utils::find_property_mut;
use lgn_data_model::TypeDefinition;
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

#[allow(clippy::enum_variant_names)]
enum ArrayOpType {
    InsertElement(Option<usize>, String),
    DeleteElement(usize, Option<Vec<u8>>),
    DeleteValue(String, Option<(Vec<u8>, usize)>),
    ReorderElement(usize, usize),
}

/// Operation to modify an array Property
pub struct ArrayOperation {
    resource_id: ResourceTypeAndId,
    array_path: String,
    operation_type: ArrayOpType,
}

impl ArrayOperation {
    /// Return a new operation to insert a new element within an array
    pub fn insert_element(
        resource_id: ResourceTypeAndId,
        array_path: &str,
        index: Option<usize>,
        value_json: &str,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            array_path: array_path.into(),
            operation_type: ArrayOpType::InsertElement(index, value_json.into()),
        })
    }

    /// Return a new operation to delete an element within an array
    pub fn delete_element(
        resource_id: ResourceTypeAndId,
        array_path: &str,
        index: usize,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            array_path: array_path.into(),
            operation_type: ArrayOpType::DeleteElement(index, None),
        })
    }

    /// Return a new operation to delete a value within an array (using linear search)
    pub fn delete_value(
        resource_id: ResourceTypeAndId,
        array_path: &str,
        value_json: &str,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            array_path: array_path.into(),
            operation_type: ArrayOpType::DeleteValue(value_json.into(), None),
        })
    }

    /// Return a new operation to reorder an element within an array
    pub fn reorder_element(
        resource_id: ResourceTypeAndId,
        array_path: &str,
        old_index: usize,
        new_index: usize,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            array_path: array_path.into(),
            operation_type: ArrayOpType::ReorderElement(old_index, new_index),
        })
    }
}

#[async_trait]
impl TransactionOperation for ArrayOperation {
    #[allow(unsafe_code)]
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let resource_handle = ctx
            .loaded_resource_handles
            .get(self.resource_id)
            .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

        let reflection = ctx
            .resource_registry
            .get_resource_reflection_mut(self.resource_id.kind, resource_handle)
            .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

        let array_value = find_property_mut(reflection, self.array_path.as_str())?;
        if let TypeDefinition::Array(array_desc) = array_value.type_def {
            match &mut self.operation_type {
                ArrayOpType::InsertElement(index, json_value) => {
                    let mut json = serde_json::Deserializer::from_str(json_value);
                    let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
                    index.get_or_insert(unsafe { (array_desc.len)(array_value.base) });

                    unsafe {
                        (array_desc.insert_element)(
                            array_value.base,
                            index.unwrap(),
                            &mut deserializer,
                        )?;
                    }
                }
                ArrayOpType::DeleteElement(index, old_value) => {
                    let mut buffer = Vec::<u8>::new();
                    let mut json = serde_json::Serializer::new(&mut buffer);
                    let mut serializer = <dyn erased_serde::Serializer>::erase(&mut json);
                    unsafe {
                        (array_desc.delete_element)(
                            array_value.base,
                            *index,
                            Some(&mut serializer),
                        )?;
                    }
                    *old_value = Some(buffer);
                }

                ArrayOpType::DeleteValue(json_value, old_value) => {
                    let mut value_to_delete = serde_json::Deserializer::from_str(json_value);
                    let mut value_to_delete_de =
                        <dyn erased_serde::Deserializer<'_>>::erase(&mut value_to_delete);

                    let mut old_value_buffer = Vec::<u8>::new();
                    let mut old_value_ser = serde_json::Serializer::new(&mut old_value_buffer);
                    let mut old_value_ser =
                        <dyn erased_serde::Serializer>::erase(&mut old_value_ser);

                    if let Ok(old_index) = unsafe {
                        (array_desc.delete_value)(
                            array_value.base,
                            &mut value_to_delete_de,
                            Some(&mut old_value_ser),
                        )
                    } {
                        *old_value = Some((old_value_buffer, old_index));
                    }
                }

                ArrayOpType::ReorderElement(old_index, new_index) => unsafe {
                    (array_desc.reorder_element)(array_value.base, *old_index, *new_index)?;
                },
            }
        }
        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }

    #[allow(unsafe_code)]
    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> anyhow::Result<()> {
        let handle = ctx
            .loaded_resource_handles
            .get(self.resource_id)
            .ok_or(Error::InvalidResource(self.resource_id))?;

        let reflection = ctx
            .resource_registry
            .get_resource_reflection_mut(self.resource_id.kind, handle)
            .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

        let array_value = find_property_mut(reflection, self.array_path.as_str())?;

        if let TypeDefinition::Array(array_desc) = array_value.type_def {
            match &self.operation_type {
                ArrayOpType::InsertElement(index, _json_value) => unsafe {
                    (array_desc.delete_element)(array_value.base, index.unwrap(), None)?;
                },
                ArrayOpType::DeleteElement(index, saved_value) => {
                    if let Some(saved_value) = saved_value {
                        let mut json = serde_json::Deserializer::from_slice(saved_value);
                        let mut deserializer =
                            <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
                        unsafe {
                            (array_desc.insert_element)(
                                array_value.base,
                                *index,
                                &mut deserializer,
                            )?;
                        };
                    }
                }

                ArrayOpType::DeleteValue(_json_value, saved_value) => {
                    if let Some((saved_value, saved_index)) = saved_value {
                        let mut json = serde_json::Deserializer::from_slice(saved_value);
                        let mut deserializer =
                            <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
                        unsafe {
                            (array_desc.insert_element)(
                                array_value.base,
                                *saved_index,
                                &mut deserializer,
                            )?;
                        };
                    }
                }

                ArrayOpType::ReorderElement(old_index, new_index) => unsafe {
                    (array_desc.reorder_element)(array_value.base, *new_index, *old_index)?;
                },
            }
        }

        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }
}
