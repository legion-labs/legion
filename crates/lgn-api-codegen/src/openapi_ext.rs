use crate::{Error, Result};
use convert_case::{Case, Casing};

const HEADERS_PREFIX: &str = "#/components/headers/";
const PARAMETERS_PREFIX: &str = "#/components/parameters/";
const REQUEST_BODIES_PREFIX: &str = "#/components/requestBodies/";
const RESPONSES_PREFIX: &str = "#/components/responses/";
const SCHEMAS_PREFIX: &str = "#/components/schemas/";

#[derive(Debug, Default, Clone)]
pub struct OpenAPIPath(Vec<String>);

impl OpenAPIPath {
    pub fn push(&mut self, s: impl Into<String>) {
        self.0.push(s.into());
    }

    pub fn to_pascal_case(&self) -> String {
        self.0.join("_").to_case(Case::Pascal)
    }
}

impl std::fmt::Display for OpenAPIPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = self.0.join("/");
        write!(f, "{}", joined)
    }
}

impl From<&str> for OpenAPIPath {
    fn from(s: &str) -> Self {
        let parts = s.split('/').map(ToOwned::to_owned).collect();
        Self(parts)
    }
}

pub(crate) trait OpenAPIExt {
    fn find_header(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Header)>;
    fn find_parameter(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Parameter)>;
    fn find_response(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Response)>;
    fn find_request_body(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::RequestBody)>;
    fn find_schema(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Schema)>;
}

impl OpenAPIExt for openapiv3::OpenAPI {
    fn find_header(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Header)> {
        if self.components.is_some() {
            let header_name = reference.trim_start_matches(HEADERS_PREFIX);
            let header_ref = self.components.as_ref().unwrap().headers.get(header_name);

            if let Some(header_ref) = header_ref {
                let schema = match header_ref {
                    openapiv3::ReferenceOr::Item(header) => header,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return self.find_header(reference);
                    }
                };

                return Ok((OpenAPIPath::from(header_name), schema));
            }
        }

        Err(Error::Invalid(format!("reference: {}", reference)))
    }

    fn find_parameter(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Parameter)> {
        if self.components.is_some() {
            let parameter_name = reference.trim_start_matches(PARAMETERS_PREFIX);
            let parameter_ref = self
                .components
                .as_ref()
                .unwrap()
                .parameters
                .get(parameter_name);

            if let Some(parameter_ref) = parameter_ref {
                let schema = match parameter_ref {
                    openapiv3::ReferenceOr::Item(parameter) => parameter,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return self.find_parameter(reference);
                    }
                };

                return Ok((OpenAPIPath::from(parameter_name), schema));
            }
        }

        Err(Error::Invalid(format!("reference: {}", reference)))
    }

    fn find_response(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Response)> {
        if self.components.is_some() {
            let response_name = reference.trim_start_matches(RESPONSES_PREFIX);
            let response_ref = self
                .components
                .as_ref()
                .unwrap()
                .responses
                .get(response_name);

            if let Some(response_ref) = response_ref {
                let response = match response_ref {
                    openapiv3::ReferenceOr::Item(response) => response,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return self.find_response(reference);
                    }
                };

                return Ok((OpenAPIPath::from(response_name), response));
            }
        }

        Err(Error::Invalid(format!("reference: {}", reference)))
    }

    fn find_request_body(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::RequestBody)> {
        if self.components.is_some() {
            let request_body_name = reference.trim_start_matches(REQUEST_BODIES_PREFIX);
            let request_body_ref = self
                .components
                .as_ref()
                .unwrap()
                .request_bodies
                .get(request_body_name);

            if let Some(request_body_ref) = request_body_ref {
                let request_body = match request_body_ref {
                    openapiv3::ReferenceOr::Item(request_body) => request_body,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return self.find_request_body(reference);
                    }
                };

                return Ok((OpenAPIPath::from(request_body_name), request_body));
            }
        }

        Err(Error::Invalid(format!("reference: {}", reference)))
    }

    fn find_schema(&self, reference: &str) -> Result<(OpenAPIPath, &openapiv3::Schema)> {
        if self.components.is_some() {
            let schema_name = reference.trim_start_matches(SCHEMAS_PREFIX);
            let schema_ref = self.components.as_ref().unwrap().schemas.get(schema_name);

            if let Some(schema_ref) = schema_ref {
                let schema = match schema_ref {
                    openapiv3::ReferenceOr::Item(schema) => schema,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return self.find_schema(reference);
                    }
                };

                return Ok((OpenAPIPath::from(schema_name), schema));
            }
        }

        Err(Error::Invalid(format!("reference: {}", reference)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_schema() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"    
            schemas:
              User:
                type: object
            "#,
        )
        .unwrap();

        let openapi = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };

        let expected_schema = openapiv3::Schema {
            schema_data: openapiv3::SchemaData {
                ..openapiv3::SchemaData::default()
            },
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(
                openapiv3::ObjectType {
                    ..openapiv3::ObjectType::default()
                },
            )),
        };

        let result = openapi.find_schema("#/components/schemas/User");

        assert!(result.is_ok());
        let (path, schema) = result.unwrap();

        assert_eq!(path.to_string(), "User");
        assert_eq!(schema, &expected_schema);
    }
}
