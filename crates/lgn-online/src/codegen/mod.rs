pub mod byte_array;
pub mod extra;

pub use byte_array::ByteArray;
pub use extra::Extra;

#[macro_export]
macro_rules! include_api {
    ($api_name:literal) => {
        include!(concat!(env!("OUT_DIR"), "/", $api_name, ".rs"));
    };
}
