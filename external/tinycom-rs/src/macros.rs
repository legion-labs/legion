// Copyright (c) 2016 com-rs developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

/**
Macro for generating COM interface definitions.

# Usage
```
#[macro_use]
extern crate tinycom;
use tinycom::IUnknown;

iid!(IID_IFOO =
    0x12345678, 0x90AB, 0xCDEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF);

com_interface! {
    interface IFoo: IUnknown {
        iid: IID_IFOO,
        vtable: IFooVtbl,
        fn foo() -> bool;
    }
}
# fn main() { }
```

This example defines an interface called `IFoo`. In this case, the base type is
IUnknown, the root COM type. The IID for the interface must also be defined,
along with the name of the vtable type, `IFooVtbl`. This isn't publicly exposed,
but there is currently no way to generate an ident within a macro so the callee
must define one instead.

The trait `Foo` defines the methods available for the interface, in this case
a single method named `foo`. Note that any methods that return no value
(e.g. the `void` type in C/C++) should return the unit type `()`.

## Inheritance
To define interfaces with a deeper hierarchy, add additional parent identifiers
to the type definitions. e.g:

```
# #[macro_use]
# extern crate tinycom;
# use tinycom::IUnknown;
# iid!(IID_IFOO = 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
# com_interface! {
#     interface IFoo: IUnknown {
#         iid: IID_IFOO,
#         vtable: IFooVtbl,
#         fn foo() -> bool;
#     }
# }
iid!(IID_IBAR =
    0x12345678, 0x90AB, 0xCDEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF);
com_interface! {
    interface IBar: IFoo, IUnknown {
        iid: IID_IBAR,
        vtable: IBarVtbl,
        fn bar(baz: i32) -> ();
    }
}
# fn main() { }
```

This example defines an interface called `IBar` which extends `IFoo` from the
previous example. Note that it is necessary to specify the parent types
for both the interface and trait declarations.

The interface hierarchy automates pointer conversion using the `AsComPtr` trait,
and the trait hierarchy automatically implements the parent methods for the
child interface.
*/
#[macro_export]
macro_rules! com_interface {
    (
        $(#[$iface_attr:meta])*
        interface $iface:ident: $base_iface:ty {
            iid: $iid:ident,
            vtable: $vtable:ident,
            $(
                $(#[$fn_attr:meta])*
                fn $func:ident($($i:ident: $t:ty),*) -> $rt:ty;
            )*
        }
    ) => (
        #[allow(missing_debug_implementations)]
        #[doc(hidden)]
        #[repr(C)]
        pub struct $vtable {
            base: <$base_iface as $crate::ComInterface>::Vtable,
            $($func: extern "stdcall" fn(*const $iface, $($t),*) -> $rt),*
        }

        $(#[$iface_attr])*
        #[derive(Debug)]
        #[repr(C)]
        pub struct $iface {
            vtable: *const $vtable
        }

        impl $iface {
            $($(#[$fn_attr])*
            pub unsafe fn $func(&self, $($i: $t),*) -> $rt {
                ((*self.vtable).$func)(self $(,$i)*)
            })*
        }

        impl ::std::ops::Deref for $iface {
            type Target = $base_iface;
            fn deref(&self) -> &$base_iface {
                unsafe { ::std::mem::transmute(self) }
            }
        }

        unsafe impl $crate::AsComPtr<$iface> for $iface {}
        unsafe impl $crate::AsComPtr<$base_iface> for $iface {}

        unsafe impl $crate::ComInterface for $iface {
            #[doc(hidden)]
            type Vtable = $vtable;
            #[allow(unused_unsafe)]
            fn iid() -> $crate::IID { unsafe { $iid } }
        }
    );

    (
        $(#[$iface_attr:meta])*
        interface $iface:ident: $base_iface:ty, $($extra_base:ty),+ {
            iid: $iid:ident,
            vtable: $vtable:ident,
            $(
                $(#[$fn_attr:meta])*
                fn $func:ident($($i:ident: $t:ty),*) -> $rt:ty;
            )*
        }
    ) => (
        com_interface! {
            $(#[$iface_attr])*
            interface $iface: $base_iface {
                iid: $iid,
                vtable: $vtable,
                $($(#[$fn_attr])* fn $func($($i: $t),*) -> $rt;)*
            }
        }

        $(unsafe impl $crate::AsComPtr<$extra_base> for $iface {})*
    )
}

/**
Helper macro for defining [`IID`](struct.IID.html) constants.

# Usage
```
# #[macro_use]
# extern crate tinycom;
iid!(IID_IFOO = 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
# fn main() {}
```

IIDs are private by default as they are only supposed to be exposed by the
`ComPtr::iid` method. If you want to make them public, just add the `pub`
keyword before the identifier.

```
# #[macro_use]
# extern crate tinycom;
iid!(pub IID_IBAR = 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
# fn main() {}
```

*/
#[macro_export]
macro_rules! iid {
    ($(#[$iid_attr:meta])*
    $name:ident = $d1:expr, $d2:expr, $d3:expr, $($d4:expr),*) => (
        $(#[$iid_attr])*
        const $name: $crate::IID = $crate::IID {
            data1: $d1,
            data2: $d2,
            data3: $d3,
            data4: [$($d4),*],
        };
    );
    ($(#[$iid_attr:meta])*
    pub $name:ident = $d1:expr, $d2:expr, $d3:expr, $($d4:expr),*) => (
        $(#[$iid_attr])*
        pub const $name: $crate::IID = $crate::IID {
            data1: $d1,
            data2: $d2,
            data3: $d3,
            data4: [$($d4),*],
        };
    );
}
