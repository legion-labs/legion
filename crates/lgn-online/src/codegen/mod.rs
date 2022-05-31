mod bytes;
mod context;
pub mod encoding;

pub use self::bytes::Bytes;
pub use context::Context;

#[macro_export]
macro_rules! include_api {
    ($api_name:ident) => {
        pub mod $api_name {
            include!(concat!(env!("OUT_DIR"), "/", stringify!($api_name), ".rs"));
        }
    };
}

#[macro_export]
macro_rules! include_apis {
    ($api_name:ident, $($api_names:ident)+) => {
        lgn_online::include_api!($api_name);
        lgn_online::include_apis!($($api_names)+);
    };
    ($api_name:ident) => {
        lgn_online::include_api!($api_name);
    };
}
