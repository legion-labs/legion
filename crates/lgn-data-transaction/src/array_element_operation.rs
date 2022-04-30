//! Transaction Operation to Modify an Array (Add,Remove,Reorder element)

use async_trait::async_trait;
use lgn_data_model::TypeDefinition;
use lgn_data_model::{
    json_utils::get_property_as_json_string, json_utils::reflection_apply_json_edit,
    utils::find_property_mut,
};
use lgn_data_runtime::ResourceTypeAndId;

use crate::{Error, LockContext, TransactionOperation};

#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
enum ArrayOpType {
    InsertElement(Option<usize>, Option<String>),
    DeleteElement(usize, Option<Vec<u8>>),
    DeleteValue(String, Option<(Vec<u8>, usize)>),
    ReorderElement(usize, usize),
}

/// Operation to modify an array Property
#[derive(Debug)]
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
        value_json: Option<impl AsRef<str>>,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            array_path: array_path.into(),
            operation_type: ArrayOpType::InsertElement(
                index,
                value_json.map(|s| s.as_ref().into()),
            ),
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
        value_json: impl AsRef<str>,
    ) -> Box<Self> {
        Box::new(Self {
            resource_id,
            array_path: array_path.into(),
            operation_type: ArrayOpType::DeleteValue(value_json.as_ref().into(), None),
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
    async fn apply_operation(&mut self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        let resource_handle = ctx.get_or_load(self.resource_id).await?;

        let mut reflection = ctx
            .asset_registry
            .get_resource_reflection_mut(self.resource_id.kind, &resource_handle)
            .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

        let array_value = find_property_mut(&mut *reflection, self.array_path.as_str())
            .map_err(|err| Error::Reflection(self.resource_id, err))?;
        if let TypeDefinition::Array(array_desc) = array_value.type_def {
            match &mut self.operation_type {
                ArrayOpType::InsertElement(index, json_value) => {
                    // If a Json value is specify, use it
                    let json_value = if let Some(json_value) = json_value {
                        let mut json_value = serde_json::from_str::<serde_json::Value>(json_value)
                            .map_err(|err| {
                                Error::Reflection(
                                    self.resource_id,
                                    lgn_data_model::ReflectionError::ErrorSerde(err),
                                )
                            })?;

                        // If we insert a BoxDyn, we cannot apply the json_values as is because there's potentially
                        // missing fields. We spawn a new instance using the BoxDescriptor, apply the Values
                        // and serialize as string
                        if let TypeDefinition::BoxDyn(box_type) = array_desc.inner_type {
                            if let serde_json::Value::Object(obj) = &mut json_value {
                                if let Some((key, values)) = obj.iter_mut().next() {
                                    if let Some(mut boxed_instance) = (box_type.new_instance)(key) {
                                        reflection_apply_json_edit(boxed_instance.as_mut(), values)
                                            .map_err(|err| {
                                                Error::Reflection(self.resource_id, err)
                                            })?;

                                        let merged_json = get_property_as_json_string(
                                            boxed_instance.as_ref(),
                                            "",
                                        )
                                        .map_err(|err| Error::Reflection(self.resource_id, err))?;

                                        *values =
                                            serde_json::from_str(&merged_json).map_err(|err| {
                                                Error::Reflection(
                                                    self.resource_id,
                                                    lgn_data_model::ReflectionError::ErrorSerde(
                                                        err,
                                                    ),
                                                )
                                            })?;
                                    }
                                }
                            }
                        }
                        json_value
                    } else {
                        // Try to use the default from the Descriptor
                        let mut buffer = Vec::new();
                        let mut json = serde_json::Serializer::new(&mut buffer);
                        array_desc
                            .inner_type
                            .serialize_default(&mut <dyn erased_serde::Serializer>::erase(
                                &mut json,
                            ))
                            .map_err(|err| Error::Reflection(self.resource_id, err))?;
                        serde_json::from_slice::<serde_json::Value>(buffer.as_slice()).map_err(
                            |err| {
                                Error::Reflection(
                                    self.resource_id,
                                    lgn_data_model::ReflectionError::ErrorSerde(err),
                                )
                            },
                        )?
                    };

                    (array_desc.insert_element)(
                        array_value.base,
                        *index,
                        &mut <dyn erased_serde::Deserializer<'_>>::erase(json_value),
                    )
                    .map_err(|err| Error::Reflection(self.resource_id, err))?;
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
                        )
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
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

                ArrayOpType::ReorderElement(old_index, new_index) => {
                    (array_desc.reorder_element)(array_value.base, *old_index, *new_index)
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
                }
            }
        }
        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }

    #[allow(unsafe_code)]
    async fn rollback_operation(&self, ctx: &mut LockContext<'_>) -> Result<(), Error> {
        let handle = ctx.get_or_load(self.resource_id).await?;

        let mut reflection = ctx
            .asset_registry
            .get_resource_reflection_mut(self.resource_id.kind, &handle)
            .ok_or(Error::InvalidTypeReflection(self.resource_id))?;

        let array_value = find_property_mut(&mut *reflection, self.array_path.as_str())
            .map_err(|err| Error::Reflection(self.resource_id, err))?;

        if let TypeDefinition::Array(array_desc) = array_value.type_def {
            match &self.operation_type {
                ArrayOpType::InsertElement(index, _json_value) => unsafe {
                    (array_desc.delete_element)(
                        array_value.base,
                        index.unwrap_or((array_desc.len)(array_value.base) - 1),
                        None,
                    )
                    .map_err(|err| Error::Reflection(self.resource_id, err))?;
                },
                ArrayOpType::DeleteElement(index, saved_value) => {
                    if let Some(saved_value) = saved_value {
                        let mut json = serde_json::Deserializer::from_slice(saved_value);
                        let mut deserializer =
                            <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
                        (array_desc.insert_element)(
                            array_value.base,
                            Some(*index),
                            &mut deserializer,
                        )
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
                    }
                }

                ArrayOpType::DeleteValue(_json_value, saved_value) => {
                    if let Some((saved_value, saved_index)) = saved_value {
                        let mut json = serde_json::Deserializer::from_slice(saved_value);
                        let mut deserializer =
                            <dyn erased_serde::Deserializer<'_>>::erase(&mut json);
                        (array_desc.insert_element)(
                            array_value.base,
                            Some(*saved_index),
                            &mut deserializer,
                        )
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
                    }
                }

                ArrayOpType::ReorderElement(old_index, new_index) => {
                    (array_desc.reorder_element)(array_value.base, *new_index, *old_index)
                        .map_err(|err| Error::Reflection(self.resource_id, err))?;
                }
            }
        }

        ctx.changed_resources.insert(self.resource_id);
        Ok(())
    }
}
