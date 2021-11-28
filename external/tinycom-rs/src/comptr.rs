// Copyright (c) 2016 com-rs developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::mem;
use std::ops::Deref;
use std::os::raw::c_void;
use std::ptr;

use crate::{IUnknown, IID};

/// Wrapper type for COM interface pointers.
///
/// # Usage
/// ## Passing COM pointers to/from FFI methods
/// `ComPtr<T>` has the following methods for accessing the underlying pointer:
///
/// * `as_ptr` returns the raw pointer `*const T`
/// * `as_mut_ptr` returns a mutable reference to the raw pointer `&mut *mut T`
///
/// The `AsComPtr` trait defines which pointer types can be returned by these
/// methods. These methods should be used carefully to ensure the returned pointers
/// do not outlive the `ComPtr` object.
///
/// ```
/// extern crate tinycom;
/// use tinycom::*;
///
/// fn create_iunknown_object(p: *mut *mut IUnknown) { }
/// fn use_iunknown_object(p: *const IUnknown) { }
///
/// fn main() {
///     let mut unknown: ComPtr<IUnknown> = ComPtr::new();
///     create_iunknown_object(unknown.as_mut_ptr());
///     use_iunknown_object(unknown.as_ptr());
/// }
/// ```
///
/// ## Reference Counting
/// `ComPtr` implements the `Clone` and `Drop` traits, which call the
/// `IUnknown::add_ref` and `IUnknown::release` methods respectively to handle the
/// internal reference counting.
///
/// ## Accessing COM interface methods
/// `ComPtr<T>` coerces into `T` using the `Deref` trait, allowing interface methods
/// to be called directly. However, dereferencing a `ComPtr` containing a null
/// pointer in this way results in a panic. All method calls should be guarded with
/// `is_null` checks to prevent this.
///
/// ```
/// # use tinycom::*;
/// # fn create_iunknown_object(p: *mut *mut IUnknown) { }
/// let mut ptr: ComPtr<IUnknown> = ComPtr::new();
/// create_iunknown_object(ptr.as_mut_ptr());
/// if !ptr.is_null() {
///     // This is just for demonstration, don't call these directly
///     unsafe { ptr.add_ref() };
///     unsafe { ptr.release() };
/// }
/// ```
///
/// ## Conversion using `From`
/// `ComPtr<T>` also implements the `From` trait for conversion between different
/// COM interfaces. This is a wrapper around the `IUnknown::query_interface` method
/// which automatically uses the IID of the target type.
///
/// ```
/// # use tinycom::*;
/// # fn create_iunknown_object(p: *mut *mut IUnknown) { }
/// # type IFoobar = IUnknown;
/// let mut unknown: ComPtr<IUnknown> = ComPtr::new();
/// create_iunknown_object(unknown.as_mut_ptr());
/// let other: ComPtr<IFoobar> = ComPtr::from(&unknown);
/// ```
/// This will try to query the `IFoobar` interface on the unknown object. If the
/// interface is unavailable (or `unknown` is null), the returned object will be
/// null.
#[derive(Debug)]
pub struct ComPtr<T: ComInterface> {
    ptr: *mut T,
}

/// Helper trait for `ComPtr`. Implemented automatically by the
/// `com_interface!` macro.
pub unsafe trait ComInterface: AsComPtr<IUnknown> {
    #[doc(hidden)]
    type Vtable;
    /// Get the IID associated with a COM interface struct.
    fn iid() -> IID;
}

/// Helper trait for `ComPtr`. Defines which types of raw pointer can be
/// returned by `as_ptr`/`as_mut_ptr`.
pub unsafe trait AsComPtr<T> {}

impl<T: ComInterface> Default for ComPtr<T> {
    fn default() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }
}

impl<T: ComInterface> ComPtr<T> {
    /// Constructs a new `ComPtr<T>`.
    pub fn new() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }

    /// Returns the raw pointer as type `U`. Depends on the `AsComPtr` trait.
    pub fn as_ptr<U>(&self) -> *const U
    where
        T: AsComPtr<U>,
    {
        self.ptr as *const U
    }

    /// Returns a mutable reference to the raw pointer.
    /// Depends on the `AsComPtr` trait.
    pub fn as_mut_ptr<U>(&mut self) -> &mut *mut U
    where
        T: AsComPtr<U>,
    {
        unsafe { mem::transmute(&mut self.ptr) }
    }

    /// Returns true if the contained interface pointer is null. This should
    /// always be checked before calling any methods on the contained interface.
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Return the IID associated with type `T`.
    pub fn iid(&self) -> IID {
        T::iid()
    }
}

/// All types can be cast into `c_void` pointers.
unsafe impl<T: ComInterface> AsComPtr<c_void> for T {}

impl<'a, T, U> From<&'a ComPtr<T>> for ComPtr<U>
where
    T: ComInterface,
    U: ComInterface + AsComPtr<c_void>,
{
    /// Create a `ComPtr` of a different interface type `U`. Calls
    /// `IUnknown::query_interface` and returns a new `ComPtr<U>` object.
    /// If the requested interface is unavailable, the returned `ComPtr<U>`
    /// will contain a null pointer.
    fn from(other: &'a ComPtr<T>) -> Self {
        let mut new: Self = Self::new();
        if !other.is_null() {
            unsafe {
                (*other.as_ptr()).query_interface(&U::iid(), new.as_mut_ptr());
            }
        }
        new
    }
}

impl<T: ComInterface> Deref for ComPtr<T> {
    type Target = T;
    /// Dereference into contained interface `T` to call methods directly.
    ///
    /// # Panics
    /// If the contained pointer is null, any dereference will result in a
    /// panic. Use the [`is_null`](#method.is_null) method before dereferencing.
    fn deref(&self) -> &T {
        assert!(!self.is_null(), "dereferenced null ComPtr");
        unsafe { &*self.ptr }
    }
}

impl<T: ComInterface> Clone for ComPtr<T> {
    /// Clones the `ComPtr<T>`. Increments the internal reference counter by
    /// calling `IUnknown::add_ref` on the contained COM object
    /// (if the pointer is non-null).
    fn clone(&self) -> Self {
        if !self.is_null() {
            unsafe { (*self.as_ptr()).add_ref() };
        }
        Self { ptr: self.ptr }
    }
}

impl<T: ComInterface> Drop for ComPtr<T> {
    /// Drops the `ComPtr<T>`. Decrements the internal reference counter by
    /// calling `IUnknown::release` on the contained COM object
    /// (if the pointer is non-null).
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe { (*self.as_ptr()).release() };
        }
    }
}
