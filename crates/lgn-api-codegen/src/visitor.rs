use std::collections::HashMap;

use indexmap::IndexMap;

use super::api::{
    Api, Enum, Field, Method, Model, Parameter, RequestBody, Response, Route, StatusCode, Struct,
    Type,
};
use crate::{
    api::{Content, Parameters},
    openapi_ext::{OpenAPIExt, OpenAPIPath},
    Error, Result,
};

pub fn visit(oas: &openapiv3::OpenAPI) -> Result<Api> {
    Visitor::new(oas).visit()
}

#[derive(Debug)]
struct Visitor<'a> {
    pub oas: &'a openapiv3::OpenAPI,
    pub api: Api,
}

impl<'a> Visitor<'a> {
    fn new(oas: &'a openapiv3::OpenAPI) -> Self {
        Self {
            oas,
            api: Api {
                title: oas.info.title.clone(),
                description: oas.info.description.clone(),
                version: oas.info.version.clone(),
                models: Vec::new(),
                paths: HashMap::new(),
            },
        }
    }

    fn visit(mut self) -> Result<Api> {
        self.visit_schemas()?;
        self.visit_paths()?;
        Ok(self.api)
    }

    fn visit_schemas(&mut self) -> Result<()> {
        if self.oas.components.is_some() {
            for (name, schema_ref) in &self.oas.components.as_ref().unwrap().schemas {
                let path = OpenAPIPath::from(name.as_str());

                match schema_ref {
                    openapiv3::ReferenceOr::Item(schema) => {
                        // Mandatory call to generate all models during visit.
                        self.resolve_type(&path, schema)?;
                    }
                    openapiv3::ReferenceOr::Reference { reference } => {
                        return Err(Error::Unsupported(format!("reference: {}", reference)));
                    }
                }
            }
        }

        Ok(())
    }

    fn visit_paths(&mut self) -> Result<()> {
        for (path, path_item_ref) in &self.oas.paths.paths {
            self.api.paths.insert(path.as_str().into(), Vec::new());
            let path = OpenAPIPath::from(path.as_str());

            let path_item = match path_item_ref {
                openapiv3::ReferenceOr::Item(path_item) => path_item,
                openapiv3::ReferenceOr::Reference { reference } => {
                    return Err(Error::Unsupported(format!("reference: {:?}", reference)));
                }
            };

            for (method, operation) in path_item.iter() {
                let method: Method = method.parse()?;
                self.visit_operation(&path, method, operation)?;
            }
        }
        Ok(())
    }

    fn visit_operation(
        &mut self,
        path: &OpenAPIPath,
        method: Method,
        operation: &'a openapiv3::Operation,
    ) -> Result<()> {
        if operation.security.is_some() {
            return Err(Error::Unsupported(format!(
                "security: {:?}",
                operation.security
            )));
        }

        // We enforce an operation id for now.
        let operation_name = match &operation.operation_id {
            Some(name) => name,
            None => {
                return Err(Error::MissingOperationID(path.to_string()));
            }
        };

        // Visit parameters.
        let mut parameters = Parameters::default();
        for parameter_ref in &operation.parameters {
            let parameter = match parameter_ref {
                openapiv3::ReferenceOr::Item(parameter) => parameter,
                openapiv3::ReferenceOr::Reference { reference } => {
                    self.oas.find_parameter(reference)?.1
                }
            };

            match parameter {
                openapiv3::Parameter::Path { parameter_data, .. } => {
                    parameters
                        .path
                        .push(self.visit_parameter(path, parameter_data, None)?);
                }
                openapiv3::Parameter::Query { parameter_data, .. } => {
                    parameters
                        .query
                        .push(self.visit_parameter(path, parameter_data, None)?);
                }
                openapiv3::Parameter::Header { parameter_data, .. } => {
                    // Only string header parameters are supported for now.
                    // There is no standard on how to parse them so we just
                    // forward the raw string value to the implementor.
                    let allowed_types = Some(vec![Type::String]);
                    parameters.header.push(self.visit_parameter(
                        path,
                        parameter_data,
                        allowed_types,
                    )?);
                }
                // We don't support cookie parameters for now.
                openapiv3::Parameter::Cookie { parameter_data, .. } => {
                    return Err(Error::Unsupported(format!(
                        "cookie parameter: {}",
                        parameter_data.name
                    )));
                }
            };
        }

        // Visit request body.
        let request_body = match &operation.request_body {
            Some(request_body) => {
                let request_body = match request_body {
                    openapiv3::ReferenceOr::Item(request_body) => request_body,
                    openapiv3::ReferenceOr::Reference { reference } => {
                        self.oas.find_request_body(reference)?.1
                    }
                };

                let (type_, media_type) = match request_body.content.len() {
                    0 => return Err(Error::Invalid(format!("schema: {}", path))),
                    1 => {
                        let (media_type, media_type_data) =
                            request_body.content.iter().next().unwrap();

                        let type_ = match &media_type_data.schema {
                            Some(schema_ref) => match &schema_ref {
                                openapiv3::ReferenceOr::Item(schema) => {
                                    // Use the operation id and the body suffix to generate the type name.
                                    let mut path = OpenAPIPath::from(operation_name.as_str());
                                    path.push("body");
                                    self.resolve_type(&path, schema)?
                                }
                                openapiv3::ReferenceOr::Reference { reference } => {
                                    self.resolve_schema_ref(path, reference)?
                                }
                            },
                            None => return Err(Error::Invalid(format!("schema: {}", path))),
                        };

                        (type_, media_type)
                    }
                    _ => {
                        return Err(Error::Unsupported(format!(
                            "multiple media type on request body: {} {}",
                            path, method
                        )));
                    }
                };

                Some(RequestBody {
                    description: request_body.description.clone(),
                    required: request_body.required,
                    content: Content {
                        media_type: media_type.as_str().try_into()?,
                        type_,
                    },
                })
            }
            None => None,
        };

        // Visit responses.
        let mut responses = IndexMap::new();
        for (status_code, response_ref) in &operation.responses.responses {
            let response = match response_ref {
                openapiv3::ReferenceOr::Item(response) => response,
                openapiv3::ReferenceOr::Reference { reference } => {
                    self.oas.find_response(reference)?.1
                }
            };

            let (media_type, type_) = match response.content.len() {
                0 => (None, None),
                1 => {
                    let (media_type, media_type_data) = response.content.iter().next().unwrap();

                    let type_ = match &media_type_data.schema {
                        Some(schema_ref) => Some(match &schema_ref {
                            openapiv3::ReferenceOr::Item(schema) => {
                                // Use the operation id and the response suffix to generate the type name.
                                let mut path = OpenAPIPath::from(operation_name.as_str());
                                path.push("response");
                                self.resolve_type(&path, schema)?
                            }
                            openapiv3::ReferenceOr::Reference { reference } => {
                                self.resolve_schema_ref(path, reference)?
                            }
                        }),
                        None => None,
                    };

                    (Some(media_type), type_)
                }
                _ => {
                    return Err(Error::Unsupported(format!(
                        "multiple media type on response: {} {}",
                        path, method
                    )));
                }
            };

            let status_code: StatusCode = match status_code {
                openapiv3::StatusCode::Code(v) => http::StatusCode::from_u16(*v)
                    .map_err(|e| Error::Invalid(format!("status code: {}", e)))?
                    .into(),
                openapiv3::StatusCode::Range(_) => {
                    return Err(Error::Unsupported(format!(
                        "status code ranges: {}",
                        status_code
                    )));
                }
            };

            responses.insert(
                status_code,
                Response {
                    description: response.description.clone(),
                    content: match media_type {
                        Some(media_type) => Some(Content {
                            media_type: media_type.as_str().try_into()?,
                            type_: type_.ok_or_else(|| {
                                Error::Invalid("content should have a schema".to_string())
                            })?,
                        }),
                        None => None,
                    },
                },
            );
        }

        let route = Route {
            name: operation_name.clone(),
            method,
            summary: operation.summary.clone(),
            request_body,
            parameters,
            responses,
        };

        self.api
            .paths
            .get_mut(&path.to_string().as_str().into())
            .unwrap()
            .push(route);
        Ok(())
    }

    fn visit_parameter(
        &mut self,
        path: &OpenAPIPath,
        parameter_data: &'a openapiv3::ParameterData,
        allowed_types: Option<Vec<Type>>,
    ) -> Result<Parameter> {
        let type_ = match &parameter_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => match schema_ref {
                openapiv3::ReferenceOr::Item(schema) => self.resolve_type(path, schema)?,
                openapiv3::ReferenceOr::Reference { reference } => {
                    self.resolve_schema_ref(path, reference)?
                }
            },
            openapiv3::ParameterSchemaOrContent::Content(_) => {
                return Err(Error::Unsupported(format!(
                    "parameter content: {}",
                    parameter_data.name
                )));
            }
        };

        if allowed_types.is_some() && !allowed_types.unwrap().contains(&type_) {
            return Err(Error::Unsupported(format!(
                "parameter type: {}",
                parameter_data.name,
            )));
        }

        Ok(Parameter {
            name: parameter_data.name.clone(),
            description: parameter_data.description.clone(),
            type_,
            required: parameter_data.required,
        })
    }

    fn new_model_from_object(
        &mut self,
        path: &OpenAPIPath,
        schema_data: &'a openapiv3::SchemaData,
        object_type: &'a openapiv3::ObjectType,
    ) -> Result<Model> {
        if object_type.additional_properties.is_some() {
            return Err(Error::Unsupported(format!(
                "additional_properties: {}",
                path
            )));
        }

        let mut properties = Vec::new();
        for (property_name, property_ref) in &object_type.properties {
            let mut path = path.clone();
            path.push(property_name.to_string());

            let required = object_type.required.contains(property_name);
            let property = match property_ref {
                openapiv3::ReferenceOr::Item(schema) => Field {
                    name: property_name.to_string(),
                    description: schema.schema_data.description.clone(),
                    type_: self.resolve_type(&path, schema)?,
                    required,
                },
                openapiv3::ReferenceOr::Reference { reference } => Field {
                    name: property_name.to_string(),
                    description: None,
                    type_: self.resolve_schema_ref(&path, reference)?,
                    required,
                },
            };
            properties.push(property);
        }
        let struct_ = Struct {
            name: path.to_pascal_case(),
            description: schema_data.description.clone(),
            fields: properties,
        };

        Ok(Model::Struct(struct_))
    }

    fn resolve_schema_ref(&mut self, path: &OpenAPIPath, reference: &str) -> Result<Type> {
        let (schema_path, schema) = self.oas.find_schema(reference)?;
        if matches!(
            schema.schema_kind,
            openapiv3::SchemaKind::Type(openapiv3::Type::Object(_))
        ) {
            Ok(Type::Struct(schema_path.to_pascal_case()))
        } else {
            self.resolve_type(path, schema)
        }
    }

    fn resolve_type(&mut self, path: &OpenAPIPath, schema: &'a openapiv3::Schema) -> Result<Type> {
        Ok(match &schema.schema_kind {
            openapiv3::SchemaKind::Type(type_) => match type_ {
                openapiv3::Type::Boolean {} => Type::Boolean,
                openapiv3::Type::Integer(t) => Visitor::resolve_integer(path, t)?,
                openapiv3::Type::Number(t) => Visitor::resolve_number(path, t)?,
                openapiv3::Type::String(t) => self.resolve_string(path, &schema.schema_data, t)?,
                openapiv3::Type::Array(t) => self.resolve_array(path, t)?,
                openapiv3::Type::Object(t) => self.resolve_object(path, &schema.schema_data, t)?,
            },
            _ => {
                return Err(Error::Unsupported(format!(
                    "schema kind: {:?}",
                    &schema.schema_kind
                )));
            }
        })
    }

    fn resolve_integer(_path: &OpenAPIPath, integer_type: &openapiv3::IntegerType) -> Result<Type> {
        if !integer_type.enumeration.is_empty() {
            return Err(Error::Unsupported(format!(
                "integer enum: {:?}",
                integer_type.enumeration
            )));
        }

        Ok(match &integer_type.format {
            openapiv3::VariantOrUnknownOrEmpty::Item(format) => match format {
                openapiv3::IntegerFormat::Int32 => Type::Int32,
                openapiv3::IntegerFormat::Int64 => Type::Int64,
            },
            _ => Type::Int32,
        })
    }

    fn resolve_number(_path: &OpenAPIPath, number_type: &openapiv3::NumberType) -> Result<Type> {
        if !number_type.enumeration.is_empty() {
            return Err(Error::Unsupported(format!(
                "number enum: {:?}",
                number_type.enumeration
            )));
        }

        Ok(match &number_type.format {
            openapiv3::VariantOrUnknownOrEmpty::Item(format) => match format {
                openapiv3::NumberFormat::Float => Type::Float32,
                openapiv3::NumberFormat::Double => Type::Float64,
            },
            _ => Type::Float64,
        })
    }

    fn resolve_array(
        &mut self,
        path: &OpenAPIPath,
        array_type: &'a openapiv3::ArrayType,
    ) -> Result<Type> {
        if array_type.items.is_none() {
            return Err(Error::Invalid(format!("array items: {:?}", array_type)));
        }

        let inner_schema_ref = array_type.items.as_ref().unwrap();
        let type_ = match inner_schema_ref {
            openapiv3::ReferenceOr::Item(schema) => self.resolve_type(path, schema)?,
            openapiv3::ReferenceOr::Reference { reference } => {
                self.resolve_schema_ref(path, reference)?
            }
        };

        if array_type.unique_items {
            return Ok(Type::HashSet(Box::new(type_)));
        }

        Ok(Type::Array(Box::new(type_)))
    }

    fn resolve_string(
        &mut self,
        path: &OpenAPIPath,
        schema_data: &'a openapiv3::SchemaData,
        string_type: &'a openapiv3::StringType,
    ) -> Result<Type> {
        if !string_type.enumeration.is_empty() {
            let enum_ = Model::Enum(Enum {
                name: path.to_pascal_case(),
                description: schema_data.description.clone(),
                variants: string_type
                    .enumeration
                    .iter()
                    .filter(|&s| s.is_some())
                    .map(|s| s.clone().unwrap())
                    .collect(),
            });

            let model_name = enum_.name().to_owned();
            self.api.models.push(enum_);
            return Ok(Type::Struct(model_name));
        }

        Ok(match &string_type.format {
            openapiv3::VariantOrUnknownOrEmpty::Item(format) => match format {
                openapiv3::StringFormat::Byte => Type::Bytes,
                openapiv3::StringFormat::Binary => Type::Binary,
                openapiv3::StringFormat::Date => Type::Date,
                openapiv3::StringFormat::DateTime => Type::DateTime,
                openapiv3::StringFormat::Password => {
                    return Err(Error::Unsupported(format!("format: {:?}", format)))
                }
            },
            _ => Type::String,
        })
    }

    fn resolve_object(
        &mut self,
        path: &OpenAPIPath,
        schema_data: &'a openapiv3::SchemaData,
        object_type: &'a openapiv3::ObjectType,
    ) -> Result<Type> {
        // New model is created on the fly for each object type.
        let model = self.new_model_from_object(path, schema_data, object_type)?;
        let model_name = model.name().to_owned();
        self.api.models.push(model);
        Ok(Type::Struct(model_name))
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{Content, MediaType};
    use indexmap::IndexMap;

    use super::*;

    #[test]
    fn test_resolve_integer() {
        let int = serde_yaml::from_str::<openapiv3::IntegerType>(
            r#"
            type: integer
            "#,
        )
        .unwrap();
        let int32 = serde_yaml::from_str::<openapiv3::IntegerType>(
            r#"
            type: integer
            format: int32
            "#,
        )
        .unwrap();
        let int64 = serde_yaml::from_str::<openapiv3::IntegerType>(
            r#"
            type: integer
            format: int64
            "#,
        )
        .unwrap();

        assert_eq!(
            Visitor::resolve_integer(&"my_int".into(), &int).unwrap(),
            Type::Int32
        );
        assert_eq!(
            Visitor::resolve_integer(&"my_int32".into(), &int32).unwrap(),
            Type::Int32
        );
        assert_eq!(
            Visitor::resolve_integer(&"my_int64".into(), &int64).unwrap(),
            Type::Int64
        );
    }

    #[test]
    fn test_resolve_number() {
        let num = serde_yaml::from_str::<openapiv3::NumberType>(
            r#"
            type: number
            "#,
        )
        .unwrap();
        let num32 = serde_yaml::from_str::<openapiv3::NumberType>(
            r#"
            type: number
            format: float
            "#,
        )
        .unwrap();
        let num64 = serde_yaml::from_str::<openapiv3::NumberType>(
            r#"
            type: number
            format: double
            "#,
        )
        .unwrap();

        assert_eq!(
            Visitor::resolve_number(&"my_num".into(), &num).unwrap(),
            Type::Float64
        );
        assert_eq!(
            Visitor::resolve_number(&"my_num32".into(), &num32).unwrap(),
            Type::Float32
        );
        assert_eq!(
            Visitor::resolve_number(&"my_num64".into(), &num64).unwrap(),
            Type::Float64
        );
    }

    #[test]
    fn test_resolve_string() {
        let str = serde_yaml::from_str::<openapiv3::StringType>(
            r#"
            type: string
            "#,
        )
        .unwrap();
        let str_date = serde_yaml::from_str::<openapiv3::StringType>(
            r#"
            type: string
            format: date
            "#,
        )
        .unwrap();
        let str_date_time = serde_yaml::from_str::<openapiv3::StringType>(
            r#"
            type: string
            format: date-time
            "#,
        )
        .unwrap();
        let str_bytes = serde_yaml::from_str::<openapiv3::StringType>(
            r#"
            type: string
            format: byte
            "#,
        )
        .unwrap();
        let str_binary = serde_yaml::from_str::<openapiv3::StringType>(
            r#"
            type: string
            format: binary
            "#,
        )
        .unwrap();

        let data = openapiv3::SchemaData::default();
        let oas = openapiv3::OpenAPI::default();
        let mut v = Visitor::new(&oas);

        assert_eq!(
            v.resolve_string(&"my_str".into(), &data, &str).unwrap(),
            Type::String
        );
        assert_eq!(
            v.resolve_string(&"my_str_date".into(), &data, &str_date)
                .unwrap(),
            Type::Date
        );
        assert_eq!(
            v.resolve_string(&"my_str_date_time".into(), &data, &str_date_time)
                .unwrap(),
            Type::DateTime
        );
        assert_eq!(
            v.resolve_string(&"my_str_bytes".into(), &data, &str_bytes)
                .unwrap(),
            Type::Bytes
        );
        assert_eq!(
            v.resolve_string(&"my_str_binary".into(), &data, &str_binary)
                .unwrap(),
            Type::Binary
        );
    }

    #[test]
    fn test_resolve_string_enum() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"    
            schemas:
              MyStruct:
                type: object
                properties:
                  my_enum:
                    type: string
                    enum:
                      - foo
                      - bar
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "MyStruct".to_string(),
            fields: vec![Field {
                name: "my_enum".to_string(),
                description: None,
                type_: Type::Struct("MyStructMyEnum".to_string()),
                required: false,
            }],
            ..Struct::default()
        });

        let expected_enum = Model::Enum(Enum {
            name: "MyStructMyEnum".to_string(),
            variants: vec!["foo".to_string(), "bar".to_string()],
            ..Enum::default()
        });

        assert_eq!(api.models.len(), 2);
        assert!(api.models.contains(&expected_struct));
        assert!(api.models.contains(&expected_enum));
    }

    #[test]
    fn test_resolve_array() {
        let array = serde_yaml::from_str::<openapiv3::ArrayType>(
            r#"
            type: array
            items:
              type: string
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI::default();
        let mut v = Visitor::new(&oas);

        assert_eq!(
            v.resolve_array(&"my_array".into(), &array).unwrap(),
            Type::Array(Box::new(Type::String))
        );
    }

    #[test]
    fn test_resolve_hashset() {
        let array = serde_yaml::from_str::<openapiv3::ArrayType>(
            r#"
            type: array
            items:
              type: string
            uniqueItems: true
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI::default();
        let mut v = Visitor::new(&oas);

        assert_eq!(
            v.resolve_array(&"my_hashset".into(), &array).unwrap(),
            Type::HashSet(Box::new(Type::String))
        );
    }

    #[test]
    fn test_resolve_array_ref() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"    
            schemas:
              Category:
                type: string
              Tag:
                type: string
              MyStruct:
                type: object
                properties:
                  category:
                    $ref: '#/components/schemas/Category'
                  tags:
                    type: array
                    items:
                      $ref: '#/components/schemas/Tag'
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "MyStruct".to_string(),
            fields: vec![
                Field {
                    name: "category".to_string(),
                    description: None,
                    type_: Type::String,
                    required: false,
                },
                Field {
                    name: "tags".to_string(),
                    description: None,
                    type_: Type::Array(Box::new(Type::String)),
                    required: false,
                },
            ],
            ..Struct::default()
        });

        assert_eq!(api.models.len(), 1);
        assert!(api.models.contains(&expected_struct));
    }

    #[test]
    fn test_resolve_object() {
        let object = serde_yaml::from_str::<openapiv3::ObjectType>(
            r#"
            type: object
            "#,
        )
        .unwrap();

        let data = openapiv3::SchemaData::default();
        let oas = openapiv3::OpenAPI::default();
        let mut v = Visitor::new(&oas);

        assert_eq!(
            v.resolve_object(&"my_obj".into(), &data, &object).unwrap(),
            Type::Struct("MyObj".to_string())
        );
    }

    #[test]
    fn test_resolve_struct() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"    
            schemas:
              MyStruct:
                type: object
                properties:
                  my_prop1:
                    type: string
                  my_prop2:
                    type: integer
                required:
                  - my_prop1
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "MyStruct".to_string(),
            fields: vec![
                Field {
                    name: "my_prop1".to_string(),
                    description: None,
                    type_: Type::String,
                    required: true,
                },
                Field {
                    name: "my_prop2".to_string(),
                    description: None,
                    type_: Type::Int32,
                    required: false,
                },
            ],
            ..Struct::default()
        });

        assert_eq!(api.models.len(), 1);
        assert!(api.models.contains(&expected_struct));
    }

    #[test]
    fn test_resolve_inner_struct() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"    
            schemas:
              MyStruct:
                type: object
                properties:
                  my_inner_struct:
                    type: object
                    properties:
                      my_inner_prop:
                        type: string
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "MyStruct".to_string(),
            fields: vec![Field {
                name: "my_inner_struct".to_string(),
                description: None,
                type_: Type::Struct("MyStructMyInnerStruct".to_string()),
                required: false,
            }],
            ..Struct::default()
        });

        let expected_inner_struct = Model::Struct(Struct {
            name: "MyStructMyInnerStruct".to_string(),
            fields: vec![Field {
                name: "my_inner_prop".to_string(),
                description: None,
                type_: Type::String,
                required: false,
            }],
            ..Struct::default()
        });

        assert_eq!(api.models.len(), 2);
        assert!(api.models.contains(&expected_struct));
        assert!(api.models.contains(&expected_inner_struct));
    }

    #[test]
    fn test_resolve_get_operation() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"    
            schemas:
              Pet:
                type: object
                properties:
                    name:
                      type: string
            "#,
        )
        .unwrap();

        let paths = serde_yaml::from_str::<openapiv3::Paths>(
            r#"    
            /pets:
              get:
                summary: Finds pets by tags.
                operationId: findPetsByTags
                parameters:
                  - name: tags
                    in: query
                    description: Tags to filter by
                    required: true
                    schema:
                      type: array
                      items:
                        type: string
                responses:
                  '200':
                    description: Successful
                    content:
                      application/json:
                        schema:
                          type: array
                          items:
                            $ref: '#/components/schemas/Pet'
                       "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "Pet".to_string(),
            fields: vec![Field {
                name: "name".to_string(),
                description: None,
                type_: Type::String,
                required: false,
            }],
            ..Struct::default()
        });

        let expected_route = Route {
            name: "findPetsByTags".to_string(),
            method: Method::Get,
            summary: Some("Finds pets by tags.".to_string()),
            request_body: None,
            parameters: Parameters {
                query: vec![Parameter {
                    name: "tags".to_string(),
                    description: Some("Tags to filter by".to_string()),
                    required: true,
                    type_: Type::Array(Box::new(Type::String)),
                }],
                ..Parameters::default()
            },
            responses: IndexMap::from([(
                http::StatusCode::OK.into(),
                Response {
                    description: "Successful".to_string(),
                    content: Some(Content {
                        media_type: MediaType::Json,
                        type_: Type::Array(Box::new(Type::Struct("Pet".to_string()))),
                    }),
                },
            )]),
        };

        assert_eq!(api.models.len(), 1);
        assert!(api.models.contains(&expected_struct));

        assert_eq!(api.paths.len(), 1);
        assert_eq!(api.paths.get(&"/pets".into()), Some(&vec![expected_route]));
    }

    #[test]
    fn test_resolve_request_body() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"  
            requestBodies:
              Pet:
                content:
                  application/json:
                    schema:
                      type: object
                      properties:
                        pet_data:
                          type: object
                          properties:
                            name:
                              type: string
            "#,
        )
        .unwrap();

        let paths = serde_yaml::from_str::<openapiv3::Paths>(
            r#"    
            /pets:
              post:
                operationId: addPet
                responses:
                  '200':
                    description: Successful
                requestBody:
                  $ref: '#/components/requestBodies/Pet'
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "AddPetBody".to_string(),
            fields: vec![Field {
                name: "pet_data".to_string(),
                description: None,
                type_: Type::Struct("AddPetBodyPetData".to_string()),
                required: false,
            }],
            ..Struct::default()
        });

        let expected_inner_struct = Model::Struct(Struct {
            name: "AddPetBodyPetData".to_string(),
            fields: vec![Field {
                name: "name".to_string(),
                description: None,
                type_: Type::String,
                required: false,
            }],
            ..Struct::default()
        });

        let expected_route = Route {
            name: "addPet".to_string(),
            method: Method::Post,
            summary: None,
            request_body: Some(RequestBody {
                description: None,
                required: false,
                content: Content {
                    media_type: MediaType::Json,
                    type_: Type::Struct("AddPetBody".to_string()),
                },
            }),
            parameters: Parameters::default(),
            responses: IndexMap::from([(
                http::StatusCode::OK.into(),
                Response {
                    description: "Successful".to_string(),
                    content: None,
                },
            )]),
        };

        assert_eq!(api.models.len(), 2);
        assert!(api.models.contains(&expected_struct));
        assert!(api.models.contains(&expected_inner_struct));

        assert_eq!(api.paths.len(), 1);
        assert_eq!(api.paths.get(&"/pets".into()), Some(&vec![expected_route]));
    }

    #[test]
    fn test_resolve_post_operation() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"  
            requestBodies:
              Pet:
                content:
                  application/json:
                    schema:
                      $ref: '#/components/schemas/Pet'  
            schemas:
              Pet:
                type: object
                properties:
                    name:
                      type: string
            "#,
        )
        .unwrap();

        let paths = serde_yaml::from_str::<openapiv3::Paths>(
            r#"    
            /pets:
              post:
                summary: Add a new pet to the store.
                operationId: addPet
                responses:
                  '200':
                    description: Successful
                    content:
                      application/json:
                        schema:
                          $ref: '#/components/schemas/Pet'
                  '405':
                    description: Invalid input
                requestBody:
                  $ref: '#/components/requestBodies/Pet'
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let api = visit(&oas).unwrap();

        let expected_struct = Model::Struct(Struct {
            name: "Pet".to_string(),
            fields: vec![Field {
                name: "name".to_string(),
                description: None,
                type_: Type::String,
                required: false,
            }],
            ..Struct::default()
        });

        let expected_route = Route {
            name: "addPet".to_string(),
            method: Method::Post,
            summary: Some("Add a new pet to the store.".to_string()),
            request_body: Some(RequestBody {
                description: None,
                required: false,
                content: Content {
                    media_type: MediaType::Json,
                    type_: Type::Struct("Pet".to_string()),
                },
            }),
            parameters: Parameters::default(),
            responses: IndexMap::from([
                (
                    http::StatusCode::OK.into(),
                    Response {
                        description: "Successful".to_string(),
                        content: Some(Content {
                            media_type: MediaType::Json,
                            type_: Type::Struct("Pet".to_string()),
                        }),
                    },
                ),
                (
                    http::StatusCode::METHOD_NOT_ALLOWED.into(),
                    Response {
                        description: "Invalid input".to_string(),
                        content: None,
                    },
                ),
            ]),
        };

        assert_eq!(api.models.len(), 1);
        assert!(api.models.contains(&expected_struct));

        assert_eq!(api.paths.len(), 1);
        assert_eq!(api.paths.get(&"/pets".into()), Some(&vec![expected_route]));
    }

    #[test]
    fn test_unsupported_media_type() {
        let paths = serde_yaml::from_str::<openapiv3::Paths>(
            r#"
            /pets:
              post:
                operationId: addPet
                responses:
                  '200':
                    description: Successful
                requestBody:
                  content:
                    test/test:
                      schema:
                        type: string
            "#,
        )
        .unwrap();

        let oas = openapiv3::OpenAPI {
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let result = visit(&oas);
        assert!(result.is_err());
    }
}
