use lazy_static::lazy_static;
use regex::{Captures, Regex};

pub use crate::filters::*;
use crate::{
    api_types::{Parameter, Path, Type},
    errors::Error,
};

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_type(type_: &Type) -> ::askama::Result<String> {
    Ok(match type_ {
        Type::Int32 | Type::Float32 => "number".to_string(),
        Type::Int64 | Type::Float64 => "bigint".to_string(),
        Type::String => "string".to_string(),
        Type::Boolean => "boolean".to_string(),
        Type::Bytes | Type::Binary => "Blob".to_string(),
        Type::DateTime | Type::Date => "Date".to_string(),
        Type::Array(inner) => format!("{}[]", fmt_type(inner).unwrap()),
        Type::HashSet(inner) => format!("Set<{}>", fmt_type(inner).unwrap()),
        Type::Named(struct_) => format!("{}Model", struct_),
        Type::Enum { .. } | Type::Struct { .. } | Type::OneOf { .. } => {
            return Err(askama::Error::Custom(Box::new(Error::UnsupportedType(
                "complex types cannot be formatted".to_string(),
            ))))
        }
    })
}

#[allow(clippy::unnecessary_wraps)]
pub fn fmt_ts_path(path: &Path, parameters: &[Parameter]) -> ::askama::Result<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\{([^/]+)\}").unwrap();
    }

    Ok(RE
        .replace_all(path.0.as_str(), |captures: &Captures<'_>| {
            let capture_name = captures.get(1).map(|match_| match_.as_str());

            if let Some(capture_name) = capture_name {
                if parameters
                    .iter()
                    .any(|parameter| parameter.name == capture_name)
                {
                    return format!("${{input.params[\"{}\"]}}", capture_name);
                }
            }

            "unknown".to_string()
        })
        .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_ts_path() {
        assert_eq!(
            fmt_ts_path(
                &"/v1/users/{id}".into(),
                &[Parameter {
                    name: "id".into(),
                    description: None,
                    type_: Type::String,
                    required: true,
                }]
            )
            .unwrap(),
            "/v1/users/${input.params[\"id\"]}"
        );
        assert_eq!(
            fmt_ts_path(
                &"/v1/users/{my-id}".into(),
                &[Parameter {
                    name: "my-id".into(),
                    description: None,
                    type_: Type::String,
                    required: true,
                }]
            )
            .unwrap(),
            "/v1/users/${input.params[\"my-id\"]}"
        );
        assert_eq!(
            fmt_ts_path(
                &"/v1/users/{id}/{name}".into(),
                &[
                    Parameter {
                        name: "id".into(),
                        description: None,
                        type_: Type::String,
                        required: true,
                    },
                    Parameter {
                        name: "name".into(),
                        description: None,
                        type_: Type::String,
                        required: true,
                    }
                ]
            )
            .unwrap(),
            "/v1/users/${input.params[\"id\"]}/${input.params[\"name\"]}"
        );
    }
}
