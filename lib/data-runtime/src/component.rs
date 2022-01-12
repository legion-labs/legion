use std::any::{Any, TypeId};
use std::ops::DerefMut;

use lgn_data_model::{implement_box_dyn_reflection, TypeReflection};

/// Component Interface
#[typetag::serde]
pub trait Component: Any + Sync + Send + TypeReflection {
    /// Compare to dynamic Component instance is they are the same
    fn eq(&self, other: &dyn Component) -> bool;
}

/// Note: Based on impl of dyn Any
impl dyn Component {
    /// Returns `true` if the boxed type is the same as `T`.
    /// (See [`std::any::Any::is`](https://doc.rust-lang.org/std/any/trait.Any.html#method.is))
    #[inline]
    pub fn is<T: Component>(&self) -> bool {
        TypeId::of::<T>() == self.type_id()
    }

    /// Returns some reference to the boxed value if it is of type `T`, or
    /// `None` if it isn't.
    /// (See [`std::any::Any::downcast_ref`](https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref))
    #[inline]
    pub fn downcast_ref<T: Component>(&self) -> Option<&T> {
        if self.is::<T>() {
            #[allow(unsafe_code)]
            unsafe {
                Some(&*((self as *const dyn Component).cast::<T>()))
            }
        } else {
            None
        }
    }
}

impl PartialEq for dyn Component {
    fn eq(&self, other: &dyn Component) -> bool {
        self.eq(other)
    }
}

implement_box_dyn_reflection!(dyn Component);
