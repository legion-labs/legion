use crate::api::{Parameter, Type};
pub use crate::filters::*;
use lazy_static::lazy_static;
use regex::Regex;

const KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "try", "type",
    "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_type(type_: &Type) -> ::askama::Result<String> {
    Ok(match type_ {
        Type::Int32 => "i32".to_string(),
        Type::Int64 => "i64".to_string(),
        Type::String => "String".to_string(),
        Type::Boolean => "bool".to_string(),
        Type::Float32 => "f32".to_string(),
        Type::Float64 => "f64".to_string(),
        Type::Bytes => "Vec<u8>".to_string(),
        Type::DateTime => "chrono::DateTime::<chrono::Utc>".to_string(),
        Type::Date => "chrono::Date::<chrono::Utc>".to_string(),
        Type::Array(inner) => format!("Vec<{}>", fmt_type(inner).unwrap()),
        Type::HashSet(inner) => format!("std::collections::HashSet<{}>", fmt_type(inner).unwrap()),
        Type::Struct(struct_) => format!("crate::models::{}", struct_),
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_field(name: &str) -> ::askama::Result<String> {
    if is_keyword(name) {
        Ok(format!("{}_", snake_case(name).unwrap()))
    } else {
        Ok(snake_case(name).unwrap())
    }
}

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_axum_path(path: &str) -> ::askama::Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{(?P<p>[^/]+)\}").unwrap();
    }

    Ok(RE.replace_all(path, ":$p").to_string())
}

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_rust_path(path: &str) -> ::askama::Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{([^/]+)\}").unwrap();
    }

    Ok(RE.replace_all(path, "{}").to_string())
}

#[allow(clippy::unnecessary_wraps)]
pub fn join_names(params: &[Parameter]) -> ::askama::Result<String> {
    let joined = params
        .iter()
        .map(|param| snake_case(param.name.clone()).unwrap())
        .collect::<Vec<String>>()
        .join(", ");

    if params.len() > 1 {
        Ok(format!("({})", joined))
    } else {
        Ok(joined)
    }
}

#[allow(clippy::unnecessary_wraps)]
pub fn join_types(params: &[Parameter]) -> ::askama::Result<String> {
    let joined = params
        .iter()
        .map(|param| {
            let type_ = fmt_type(&param.type_).unwrap();
            if param.required {
                type_
            } else {
                format!("Option<{}>", type_)
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    if params.len() > 1 {
        Ok(format!("({})", joined))
    } else {
        Ok(joined)
    }
}

fn is_keyword(name: &str) -> bool {
    KEYWORDS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_names() {
        let p1 = Parameter {
            name: "foo".to_string(),
            description: None,
            type_: Type::Int32,
            required: true,
        };
        let p2 = Parameter {
            name: "bar".to_string(),
            description: None,
            type_: Type::String,
            required: false,
        };
        assert_eq!(join_names(&[p1.clone()]).unwrap(), "foo".to_string());
        assert_eq!(join_names(&[p1, p2]).unwrap(), "(foo, bar)".to_string());
    }

    #[test]
    fn test_join_types() {
        let p1 = Parameter {
            name: "foo".to_string(),
            description: None,
            type_: Type::Int32,
            required: true,
        };
        let p2 = Parameter {
            name: "bar".to_string(),
            description: None,
            type_: Type::String,
            required: false,
        };
        assert_eq!(join_types(&[p1.clone()]).unwrap(), "i32".to_string());
        assert_eq!(
            join_types(&[p1, p2]).unwrap(),
            "(i32, Option<String>)".to_string()
        );
    }

    #[test]
    fn test_fmt_axum_path() {
        assert_eq!(fmt_axum_path("/v1/users/{id}").unwrap(), "/v1/users/:id");
        assert_eq!(fmt_axum_path("/v1/users/:id").unwrap(), "/v1/users/:id");
        assert_eq!(
            fmt_axum_path("/v1/users/{my-id}").unwrap(),
            "/v1/users/:my-id"
        );
        assert_eq!(
            fmt_axum_path("/v1/users/{id}/{name}").unwrap(),
            "/v1/users/:id/:name"
        );
    }

    #[test]
    fn test_fmt_rust_path() {
        assert_eq!(fmt_rust_path("/v1/users/{id}").unwrap(), "/v1/users/{}");
        assert_eq!(fmt_rust_path("/v1/users/{my-id}").unwrap(), "/v1/users/{}");
        assert_eq!(
            fmt_rust_path("/v1/users/{id}/{name}").unwrap(),
            "/v1/users/{}/{}"
        );
    }
}
