use crate::TypeDefinition;
/// Define the reflection of a Box<dyn Type>
pub struct BoxDynDescriptor {
    /// Return the type name of the Box inner dyn type
    pub type_name: String,
    /// Return the type of the instance inside the Box
    pub get_inner_type: unsafe fn(box_base: *const ()) -> TypeDefinition,
    /// Return a raw pointer of the Box content
    pub get_inner: unsafe fn(box_base: *const ()) -> *const (),
    /// Return a raw pointer of the Box content
    pub get_inner_mut: unsafe fn(box_base: *mut ()) -> *mut (),
}

/// Macro to implement reflection for Box<dyn Type>
#[macro_export]
macro_rules! implement_box_dyn_reflection {
    ($type_id:ty) => {
        #[allow(clippy::cast_ptr_alignment)]
        impl $crate::TypeReflection for Box<$type_id> {
            fn get_type(&self) -> $crate::TypeDefinition {
                Self::get_type_def()
            }

            fn get_type_def() -> $crate::TypeDefinition {
                lazy_static::lazy_static! {
                    static ref TYPE_DESCRIPTOR: $crate::BoxDynDescriptor = $crate::BoxDynDescriptor {
                        type_name: concat!("Box<", stringify!($type_id), ">").into(),
                        get_inner_type: |box_base: *const ()| unsafe {
                            let boxed = &*(box_base.cast::<Box<$type_id>>());
                            (*(*boxed)).get_type()
                        },
                        get_inner: |box_base: *const ()| unsafe {
                            ((&(*(*(box_base.cast::<Box<$type_id>>())))) as *const $type_id)
                                .cast::<()>()
                        },
                        get_inner_mut: |box_base: *mut ()| unsafe {
                            ((*box_base.cast::<Box<$type_id>>()).deref_mut() as *mut $type_id)
                                .cast::<()>()
                        },
                    };
                }
                $crate::TypeDefinition::BoxDyn(&TYPE_DESCRIPTOR)
            }
            fn get_option_def() -> $crate::TypeDefinition {
                $crate::implement_option_descriptor!(Box<$type_id>);
                $crate::TypeDefinition::Option(&OPTION_DESCRIPTOR)
            }

            fn get_array_def() -> $crate::TypeDefinition {
                $crate::implement_array_descriptor!(Box<$type_id>);
                $crate::TypeDefinition::Array(&ARRAY_DESCRIPTOR)
            }
        }
    };
}
