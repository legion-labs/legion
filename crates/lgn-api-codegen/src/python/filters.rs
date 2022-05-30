use crate::api::{Path, Type};

pub use crate::filters::*;

use lazy_static::lazy_static;
use regex::Regex;

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_type(type_: &Type) -> ::askama::Result<String> {
    Ok(match type_ {
        Type::Int32 => "int32".to_string(),
        Type::Int64 => "int64".to_string(),
        Type::String => "str".to_string(),
        Type::Boolean => "bool".to_string(),
        Type::Float32 => "float32".to_string(),
        Type::Float64 => "float64".to_string(),
        Type::Bytes | Type::Binary => "bytearray".to_string(),
        Type::DateTime => "datetime".to_string(),
        Type::Date => "date".to_string(),
        Type::Array(inner) => format!("list[{}]", fmt_type(inner).unwrap()),
        Type::HashSet(inner) => format!("set{}", fmt_type(inner).unwrap()),
        Type::Struct(struct_) => format!("{}", struct_),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_py_path(path: &Path) -> ::askama::Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{([^/]+)\}").unwrap();
    }

    Ok(RE.replace_all(path.0.as_str(), "{}").to_string())
}
