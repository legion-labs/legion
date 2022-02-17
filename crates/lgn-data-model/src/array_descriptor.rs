use crate::{utils::ReflectionError, BaseDescriptor, TypeDefinition, TypeReflection};

type DeleteElementFunction = unsafe fn(
    array: *mut (),
    index: usize,
    serializer: Option<&mut dyn ::erased_serde::Serializer>,
) -> Result<(), ReflectionError>;

type DeleteValueFunction = unsafe fn(
    array: *mut (),
    value_to_delete: &mut dyn ::erased_serde::Deserializer<'_>,
    old_value: Option<&mut dyn ::erased_serde::Serializer>,
) -> Result<usize, ReflectionError>;

/// Define the reflection of a Array typpe
pub struct ArrayDescriptor {
    /// Base Descriptor
    pub base_descriptor: BaseDescriptor,
    /// Type of the Vec Element
    pub inner_type: TypeDefinition,
    /// Function to return the array size
    pub len: unsafe fn(array: *const ()) -> usize,
    /// Function to return an element raw pointer
    pub get: unsafe fn(array: *const (), index: usize) -> Result<*const (), ReflectionError>,
    /// Function to return an element mutable raw pointer
    pub get_mut: unsafe fn(array: *mut (), index: usize) -> Result<*mut (), ReflectionError>,
    /// Function to clear an array
    pub clear: unsafe fn(array: *mut ()),

    /// Function to insert a new defualt element at the specified index
    pub insert_element: unsafe fn(
        array: *mut (),
        index: Option<usize>,
        deserializer: &mut dyn ::erased_serde::Deserializer<'_>,
    ) -> Result<(), ReflectionError>,

    /// Function to insert a new element
    pub delete_element: DeleteElementFunction,

    /// Function to reorder an element with an array
    pub reorder_element: unsafe fn(
        array: *mut (),
        old_index: usize,
        new_index: usize,
    ) -> Result<(), ReflectionError>,

    /// Function to search and delete a value in an array
    pub delete_value: DeleteValueFunction,
}

///
pub fn array_remove_value<InnerType: PartialEq>(
    array: &mut Vec<InnerType>,
    value_to_delete: &InnerType,
) -> Option<(InnerType, usize)> {
    for (index, value) in array.iter().enumerate() {
        if *value == *value_to_delete {
            return Some((array.remove(index), index));
        }
    }
    None
}

/// Macro to implement array descriptor
#[macro_export]
macro_rules! implement_array_descriptor {
    ($type_id:ty) => {
        lazy_static::lazy_static! {
            static ref ARRAY_DESCRIPTOR: $crate::ArrayDescriptor = $crate::ArrayDescriptor {
                base_descriptor : $crate::create_base_descriptor!(Vec<$type_id>, concat!("Vec<",stringify!($type_id),">").into()),
                inner_type: <$type_id as $crate::TypeReflection>::get_type_def(),
                len: |array: *const ()| unsafe { (*(array as *const Vec<$type_id>)).len() },
                get: |array: *const (), index: usize| unsafe {
                    (*(array as *const Vec<$type_id>))
                        .get(index).ok_or_else(|| $crate::ReflectionError::InvalidArrayIndex(index, concat!("Vec<",stringify!($type_id),">")))
                        .and_then(|value| Ok((value as *const $type_id).cast::<()>()))
                    },
                get_mut:|array: *mut (), index: usize| unsafe {
                    (*(array as *mut Vec<$type_id>))
                        .get_mut(index).ok_or_else(|| $crate::ReflectionError::InvalidArrayIndex(index, concat!("Vec<",stringify!($type_id),">")))
                        .and_then(|value| Ok((value as *mut $type_id).cast::<()>()))
                },
                clear:|array: *mut ()| unsafe {
                    (*(array as *mut Vec<$type_id>)).clear();
                },

                insert_element : |array: *mut(), index : Option<usize>, deserializer: &mut dyn::erased_serde::Deserializer<'_>| unsafe {
                    let array = &mut (*(array as *mut Vec<$type_id>));
                    let new_element : $type_id = ::erased_serde::deserialize(deserializer)
                        .map_err(|err|$crate::utils::ReflectionError::ErrorErasedSerde(err))?;

                    let index = index.unwrap_or(array.len());
                    if index > array.len() {
                        return Err($crate::ReflectionError::InvalidArrayIndex(index, concat!("Vec<",stringify!($type_id),">")));
                    }
                    array.insert(index, new_element);
                    Ok(())
                },
                delete_element : |array: *mut(), index : usize, old_value_ser:  Option<&mut dyn::erased_serde::Serializer> | unsafe {
                    let array = &mut (*(array as *mut Vec<$type_id>));
                    if index >= array.len() {
                        return Err($crate::ReflectionError::InvalidArrayIndex(index, concat!("Vec<",stringify!($type_id),">")));
                    }
                    let old_value = array.remove(index);
                    if let Some(serializer) = old_value_ser {
                       ::erased_serde::serialize(&old_value, serializer)
                       .map_err(|err| $crate::utils::ReflectionError::ErrorErasedSerde(err))?;
                    }
                    Ok(())
                },
                reorder_element : |array: *mut(), old_index : usize, new_index : usize  | unsafe {
                    let array = &mut (*(array as *mut Vec<$type_id>));
                    if old_index >= array.len() {
                        return Err($crate::ReflectionError::InvalidArrayIndex(old_index, concat!("Vec<",stringify!($type_id),">")));
                    }
                    if new_index > array.len() {
                        return Err($crate::ReflectionError::InvalidArrayIndex(new_index, concat!("Vec<",stringify!($type_id),">")));
                    }
                    let value = array.remove(old_index);
                    array.insert(new_index, value);
                    Ok(())
                },

                delete_value : | array: *mut(), value_de: &mut dyn::erased_serde::Deserializer<'_>, old_value_ser: Option<&mut dyn::erased_serde::Serializer> | unsafe {
                    let value_to_delete = ::erased_serde::deserialize::<$type_id>(value_de)
                        .map_err(|err| $crate::ReflectionError::ErrorErasedSerde(err))?;

                    let array = &mut (*(array as *mut Vec<$type_id>));
                    if let Some((old_value,index)) = $crate::array_remove_value(array,&value_to_delete) {
                        if let Some(serializer) = old_value_ser {
                            ::erased_serde::serialize(&old_value, serializer)
                            .map_err(|err| $crate::ReflectionError::ErrorErasedSerde(err))?;

                        }
                        return Ok(index);
                    }
                    Err($crate::ReflectionError::InvalidArrayValue(concat!("Vec<",stringify!($type_id),">")))
                }
            };
        }
    };
}

impl<T: TypeReflection> TypeReflection for Vec<T> {
    fn get_type(&self) -> TypeDefinition {
        Self::get_type_def()
    }

    fn get_type_def() -> TypeDefinition {
        T::get_array_def()
    }
}
