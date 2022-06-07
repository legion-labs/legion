use crate::api::{Path, Type};

pub use crate::filters::*;

use lazy_static::lazy_static;
use regex::Regex;

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_type(type_: &Type) -> ::askama::Result<String> {
    Ok(match type_ {
        Type::Int32 | Type::Int64 => "int".to_string(),
        Type::String | Type::Bytes | Type::Binary => "str".to_string(), // at the moment the binary is passed as string
        Type::Boolean => "bool".to_string(),
        Type::Float32 | Type::Float64 => "float".to_string(),
        Type::DateTime => "datetime".to_string(),
        Type::Date => "date".to_string(),
        Type::Array(inner) | Type::HashSet(inner) => format!("list[{}]", fmt_type(inner).unwrap()),
        Type::Struct(struct_) => struct_.clone(),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_py_path(path: &Path) -> ::askama::Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{([^/]+)\}").unwrap();
    }

    Ok(RE.replace_all(path.0.as_str(), "{}").to_string())
}
