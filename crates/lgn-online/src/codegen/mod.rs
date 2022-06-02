mod bytes;
mod context;
pub mod encoding;

pub use self::bytes::Bytes;
pub use context::Context;

#[macro_export]
macro_rules! include_api {
    () => {
        pub mod api {
            include!(concat!(env!("OUT_DIR"), "/api.rs"));
        }
    };
}
