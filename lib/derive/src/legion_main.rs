use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[allow(clippy::needless_pass_by_value)]
pub fn legion_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    assert!(
        !(input.sig.ident != "main"),
        "`legion_main` can only be used on a function called 'main'."
    );

    TokenStream::from(quote! {
        #[no_mangle]
        #[cfg(target_os = "android")]
        unsafe extern "C" fn ANativeActivity_onCreate(
            activity: *mut std::os::raw::c_void,
            saved_state: *mut std::os::raw::c_void,
            saved_state_size: usize,
        ) {
            legion::ndk_glue::init(
                activity as _,
                saved_state as _,
                saved_state_size as _,
                main,
            );
        }

        #[no_mangle]
        #[cfg(target_os = "ios")]
        extern "C" fn main_rs() {
            main();
        }

        #[allow(unused)]
        #input
    })
}
