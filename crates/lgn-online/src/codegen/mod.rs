pub mod encoding;

mod bytes;

pub use self::bytes::Bytes;

#[macro_export]
macro_rules! include_api {
    () => {
        include!(concat!(env!("OUT_DIR"), "/api.rs"));
    };
}
