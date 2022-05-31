use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use indexmap::IndexMap;

use super::api::{
    Api, Field, Method, Model, Parameter, RequestBody, Response, Route, StatusCode, Type,
};
use crate::{
    api::{Content, Header, Parameters},
    openapi_loader::OpenApiRef,
    openapi_path::OpenAPIPath,
    Error, OpenApi, OpenApiElement, Result,
};

impl TryFrom<OpenApi<'_>> for Api {
    type Error = Error;

    fn try_from(value: OpenApi<'_>) -> Result<Self, Self::Error> {
        Visitor::default().visit(&value)
    }
}

#[derive(Debug, Default)]
struct Visitor {
    pub api: Api,
}

impl Visitor {
    fn visit(mut self, oas: &OpenApi<'_>) -> Result<Api> {
        self.api = Api {
            title: oas.info.title.clone(),
            description: oas.info.description.clone(),
            version: oas.info.version.clone(),
            models: BTreeMap::new(),
            paths: IndexMap::new(),
        };

        // Let's first resolve schemas.
        if oas.components.is_some() {
            for (name, schema_ref) in &oas.components.as_ref().unwrap().schemas {
                let schema = oas.resolve_reference_or(schema_ref)?;
                self.register_model_from_schema(name, &schema)?;
            }
        }

        // Then resolve paths.
        for (path, path_item_ref) in &oas.paths.paths {
            self.api.paths.insert(path.as_str().into(), Vec::new());
            let path = OpenAPIPath::from(path.as_str());

            let path_item = oas.resolve_reference_or(path_item_ref)?;

            for (method, operation) in path_item.iter() {
                let method: Method = method.parse()?;
                self.register_operation(&path, &path_item, method, &oas.as_element_ref(operation))?;
            }
        }

        Ok(self.api)
    }

    fn register_operation(
        &mut self,
        path: &OpenAPIPath,
        path_item: &OpenApiElement<'_, openapiv3::PathItem>,
        method: Method,
        operation: &OpenApiElement<'_, openapiv3::Operation>,
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

        // Let's iterate over all parameters, in order, with the global path
        // parameters first and remove duplicates.
        let raw_parameters = path_item
            .parameters
            .iter()
            .chain(operation.parameters.iter())
            .map(|parameter_ref| {
                operation
                    .resolve_reference_or(parameter_ref)
                    .map(|parameter| (parameter.parameter_data_ref().name.clone(), parameter))
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        for parameter in raw_parameters.into_values() {
            match parameter.as_ref() {
                openapiv3::Parameter::Path { parameter_data, .. } => {
                    parameters.path.push(self.visit_parameter(
                        path,
                        &parameter.as_element_ref(parameter_data),
                        None,
                    )?);
                }
                openapiv3::Parameter::Query { parameter_data, .. } => {
                    parameters.query.push(self.visit_parameter(
                        path,
                        &parameter.as_element_ref(parameter_data),
                        None,
                    )?);
                }
                openapiv3::Parameter::Header { parameter_data, .. } => {
                    // Only string header parameters are supported for now.
                    // There is no standard on how to parse them so we just
                    // forward the raw string value to the implementor.
                    let allowed_types = Some(vec![Type::String]);
                    parameters.header.push(self.visit_parameter(
                        path,
                        &parameter.as_element_ref(parameter_data),
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
                let request_body = operation.resolve_reference_or(request_body)?;

                let (type_, media_type) = match request_body.content.len() {
                    0 => return Err(Error::Invalid(format!("schema: {}", path))),
                    1 => {
                        let (media_type, media_type_data) =
                            request_body.content.iter().next().unwrap();

                        let type_ = match &media_type_data.schema {
                            Some(schema_ref) => match schema_ref {
                                openapiv3::ReferenceOr::Item(schema) => {
                                    let name =
                                        format!("{}_body", operation_name).to_case(Case::Pascal);
                                    self.resolve_type(
                                        &name.as_str().into(),
                                        &request_body.as_element_ref(schema),
                                    )?
                                }
                                openapiv3::ReferenceOr::Reference { reference } => {
                                    reference.parse().map(OpenApiRef::into_named_type)?
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
            let response = operation.resolve_reference_or(response_ref)?;

            let (media_type, type_) = match response.content.len() {
                0 => (None, None),
                1 => {
                    let (media_type, media_type_data) = response.content.iter().next().unwrap();

                    let type_ = match &media_type_data.schema {
                        Some(schema_ref) => Some(match schema_ref {
                            openapiv3::ReferenceOr::Item(schema) => {
                                let name =
                                    format!("{}_response", operation_name).to_case(Case::Pascal);

                                self.resolve_type(
                                    &name.as_str().into(),
                                    &response.as_element_ref(schema),
                                )?
                            }
                            openapiv3::ReferenceOr::Reference { reference } => {
                                reference.parse().map(OpenApiRef::into_named_type)?
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

            let mut headers = IndexMap::new();
            for (header_name, header_ref) in &response.headers {
                let header = response.resolve_reference_or(header_ref)?;

                headers.insert(
                    header_name.clone(),
                    Header {
                        description: header.description.clone(),
                        type_: match &header.format {
                            openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
                                self.resolve_type_ref(path, &header.as_element_ref(schema_ref))?
                            }
                            openapiv3::ParameterSchemaOrContent::Content(_) => {
                                return Err(Error::Unsupported(format!(
                                    "header content format: {}",
                                    header_name
                                )));
                            }
                        },
                    },
                );
            }

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
                    headers,
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

        // We are guarenteed to have the path key in the map.
        self.api
            .paths
            .get_mut::<crate::api::Path>(&path.into())
            .unwrap()
            .push(route);
        Ok(())
    }

    fn visit_parameter(
        &mut self,
        path: &OpenAPIPath,
        parameter_data: &OpenApiElement<'_, openapiv3::ParameterData>,
        allowed_types: Option<Vec<Type>>,
    ) -> Result<Parameter> {
        let type_ = match &parameter_data.format {
            openapiv3::ParameterSchemaOrContent::Schema(schema_ref) => {
                self.resolve_type_ref(path, &parameter_data.as_element_ref(schema_ref))?
            }
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

    fn register_model_from_schema(
        &mut self,
        name: &str,
        schema: &OpenApiElement<'_, openapiv3::Schema>,
    ) -> Result<Type> {
        let model = Model {
            name: name.to_owned(),
            description: schema.schema_data.description.clone(),
            type_: match &schema.schema_kind {
                openapiv3::SchemaKind::Type(type_) => match type_ {
                    openapiv3::Type::Boolean {} => Type::Boolean,
                    openapiv3::Type::Integer(t) => Self::resolve_integer(t)?,
                    openapiv3::Type::Number(t) => Self::resolve_number(t)?,
                    openapiv3::Type::String(t) => Self::resolve_string(t)?,
                    openapiv3::Type::Array(t) => {
                        self.resolve_array(&name.into(), &schema.as_element_ref(t))?
                    }
                    openapiv3::Type::Object(t) => {
                        self.resolve_object(&name.into(), &schema.as_element_ref(t))?
                    }
                },
                openapiv3::SchemaKind::OneOf { one_of } => self.resolve_one_of(
                    &name.into(),
                    &schema.as_element_ref(&schema.schema_data),
                    one_of,
                )?,
                _ => {
                    return Err(Error::Unsupported(format!(
                        "schema kind: {:?}",
                        &schema.schema_kind
                    )));
                }
            },
        };

        self.register_model(model)
    }

    fn register_model(&mut self, model: Model) -> Result<Type> {
        let type_ = model.to_named_type();

        if let Some(old_model) = self.api.models.get(&model.name) {
            if old_model != &model {
                return Err(Error::ModelAlreadyRegistered(model.name));
            }
        }

        self.api.models.insert(model.name.clone(), model.clone());

        Ok(type_)
    }

    fn resolve_type(
        &mut self,
        path: &OpenAPIPath,
        schema: &OpenApiElement<'_, openapiv3::Schema>,
    ) -> Result<Type> {
        let type_ = match &schema.schema_kind {
            openapiv3::SchemaKind::Type(type_) => match type_ {
                openapiv3::Type::Boolean {} => Type::Boolean,
                openapiv3::Type::Integer(t) => Self::resolve_integer(t)?,
                openapiv3::Type::Number(t) => Self::resolve_number(t)?,
                openapiv3::Type::String(t) => Self::resolve_string(t)?,
                openapiv3::Type::Array(t) => self.resolve_array(path, &schema.as_element_ref(t))?,
                openapiv3::Type::Object(t) => {
                    self.resolve_object(path, &schema.as_element_ref(t))?
                }
            },
            openapiv3::SchemaKind::OneOf { one_of } => {
                self.resolve_one_of(path, &schema.as_element_ref(&schema.schema_data), one_of)?
            }
            _ => {
                return Err(Error::Unsupported(format!(
                    "schema kind: {:?}",
                    &schema.schema_kind
                )));
            }
        };

        if type_.requires_model() {
            let name = path.to_pascal_case();

            let model = Model {
                name,
                description: schema.schema_data.description.clone(),
                type_,
            };

            self.register_model(model)
        } else {
            Ok(type_)
        }
    }

    fn resolve_integer(integer_type: &openapiv3::IntegerType) -> Result<Type> {
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

    fn resolve_number(number_type: &openapiv3::NumberType) -> Result<Type> {
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
        array_type: &OpenApiElement<'_, openapiv3::ArrayType>,
    ) -> Result<Type> {
        if array_type.items.is_none() {
            return Err(Error::Invalid(format!("array items: {:?}", array_type)));
        }

        let type_ = match array_type.items.as_ref().unwrap() {
            openapiv3::ReferenceOr::Item(inner_schema) => {
                self.resolve_type(path, &array_type.as_element_ref(inner_schema))
            }
            openapiv3::ReferenceOr::Reference { reference } => {
                reference.parse().map(OpenApiRef::into_named_type)
            }
        }?;

        if array_type.unique_items {
            return Ok(Type::HashSet(Box::new(type_)));
        }

        Ok(Type::Array(Box::new(type_)))
    }

    fn resolve_string(string_type: &openapiv3::StringType) -> Result<Type> {
        Ok(if !string_type.enumeration.is_empty() {
            Type::Enum {
                variants: string_type
                    .enumeration
                    .iter()
                    .filter(|&s| s.is_some())
                    .map(|s| s.clone().unwrap())
                    .collect(),
            }
        } else {
            match &string_type.format {
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
            }
        })
    }

    fn resolve_object(
        &mut self,
        path: &OpenAPIPath,
        object_type: &OpenApiElement<'_, openapiv3::ObjectType>,
    ) -> Result<Type> {
        // New model is created on the fly for each object type.
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
            let type_ = match property_ref {
                openapiv3::ReferenceOr::Item(inner_schema) => {
                    self.resolve_type(&path, &object_type.as_element_ref(inner_schema))
                }
                openapiv3::ReferenceOr::Reference { reference } => {
                    reference.parse().map(OpenApiRef::into_named_type)
                }
            }?;
            let property = Field {
                name: property_name.to_string(),
                description: None, // TODO: Revisit this.
                type_,
                required,
            };
            properties.push(property);
        }

        Ok(Type::Struct { fields: properties })
    }

    fn resolve_one_of(
        &mut self,
        path: &OpenAPIPath,
        schema_data: &OpenApiElement<'_, openapiv3::SchemaData>,
        one_of: &[openapiv3::ReferenceOr<openapiv3::Schema>],
    ) -> Result<Type> {
        // New enum model is created on the fly for each oneof.
        Ok(Type::OneOf {
            types: one_of
                .iter()
                .map(|schema_ref| {
                    self.resolve_type_ref(path, &schema_data.as_element_ref(schema_ref))
                })
                .collect::<Result<Vec<_>>>()?,
        })
    }

    fn resolve_type_ref(
        &mut self,
        path: &OpenAPIPath,
        schema: &OpenApiElement<'_, openapiv3::ReferenceOr<openapiv3::Schema>>,
    ) -> Result<Type> {
        match schema.as_ref() {
            openapiv3::ReferenceOr::Item(inner_schema) => {
                self.resolve_type(path, &schema.as_element_ref(inner_schema))
            }
            openapiv3::ReferenceOr::Reference { reference } => {
                let ref_: OpenApiRef = reference.parse()?;

                // If we have a reference location, we need to make sure that we
                // resolve the schema and register the associated model.
                if ref_.ref_location().is_some() {
                    let name = ref_.type_name();
                    let schema = schema.resolve_reference::<openapiv3::Schema>(ref_.clone())?;
                    self.register_model_from_schema(name, &schema)
                } else {
                    Ok(ref_.into_named_type())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        api::{Content, MediaType, Path},
        openapi_loader::OpenApiLoader,
    };
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

        assert_eq!(Visitor::resolve_integer(&int).unwrap(), Type::Int32);
        assert_eq!(Visitor::resolve_integer(&int32).unwrap(), Type::Int32);
        assert_eq!(Visitor::resolve_integer(&int64).unwrap(), Type::Int64);
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

        assert_eq!(Visitor::resolve_number(&num).unwrap(), Type::Float64);
        assert_eq!(Visitor::resolve_number(&num32).unwrap(), Type::Float32);
        assert_eq!(Visitor::resolve_number(&num64).unwrap(), Type::Float64);
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

        assert_eq!(Visitor::resolve_string(&str).unwrap(), Type::String);
        assert_eq!(Visitor::resolve_string(&str_date).unwrap(), Type::Date);
        assert_eq!(
            Visitor::resolve_string(&str_date_time).unwrap(),
            Type::DateTime
        );
        assert_eq!(Visitor::resolve_string(&str_bytes).unwrap(), Type::Bytes);
        assert_eq!(Visitor::resolve_string(&str_binary).unwrap(), Type::Binary);
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "MyStruct".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "my_enum".to_string(),
                    description: None,
                    type_: Type::Named("MyStructMyEnum".to_string()),
                    required: false,
                }],
            },
        };

        let expected_enum = Model {
            name: "MyStructMyEnum".to_string(),
            description: None,
            type_: Type::Enum {
                variants: vec!["foo".to_string(), "bar".to_string()],
            },
        };

        assert_eq!(api.models.len(), 2);
        assert_eq!(api.models.get("MyStruct"), Some(&expected_struct));
        assert_eq!(api.models.get("MyStructMyEnum"), Some(&expected_enum));
    }

    #[test]
    fn test_resolve_array() {
        let loader = OpenApiLoader::default();
        let array: OpenApiElement<'_, openapiv3::ArrayType> = loader
            .import_from_yaml(
                r#"
            type: array
            items:
              type: string
            "#,
            )
            .unwrap();

        let mut v = Visitor::default();

        assert_eq!(
            v.resolve_array(&"my_array".into(), &array).unwrap(),
            Type::Array(Box::new(Type::String))
        );
    }

    #[test]
    fn test_resolve_hashset() {
        let loader = OpenApiLoader::default();
        let array: OpenApiElement<'_, openapiv3::ArrayType> = loader
            .import_from_yaml(
                r#"
            type: array
            items:
              type: string
            uniqueItems: true
            "#,
            )
            .unwrap();

        let mut v = Visitor::default();

        assert_eq!(
            v.resolve_array(&"my_hashset".into(), &array).unwrap(),
            Type::HashSet(Box::new(Type::String))
        );
    }

    #[test]
    fn test_resolve_array_ref() {
        let components = serde_yaml::from_str(
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "MyStruct".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![
                    Field {
                        name: "category".to_string(),
                        description: None,
                        type_: Type::Named("Category".to_string()),
                        required: false,
                    },
                    Field {
                        name: "tags".to_string(),
                        description: None,
                        type_: Type::Array(Box::new(Type::Named("Tag".to_string()))),
                        required: false,
                    },
                ],
            },
        };

        assert_eq!(api.models.len(), 3);
        assert_eq!(api.models.get("MyStruct"), Some(&expected_struct));
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "MyStruct".to_string(),
            description: None,
            type_: Type::Struct {
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
            },
        };

        assert_eq!(api.models.len(), 1);
        assert_eq!(api.models.get("MyStruct"), Some(&expected_struct));
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "MyStruct".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "my_inner_struct".to_string(),
                    description: None,
                    type_: Type::Named("MyStructMyInnerStruct".to_string()),
                    required: false,
                }],
            },
        };

        let expected_inner_struct = Model {
            name: "MyStructMyInnerStruct".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "my_inner_prop".to_string(),
                    description: None,
                    type_: Type::String,
                    required: false,
                }],
            },
        };

        assert_eq!(api.models.len(), 2);
        assert_eq!(api.models.get("MyStruct"), Some(&expected_struct));
        assert_eq!(
            api.models.get("MyStructMyInnerStruct"),
            Some(&expected_inner_struct)
        );
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "Pet".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "name".to_string(),
                    description: None,
                    type_: Type::String,
                    required: false,
                }],
            },
        };

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
                        type_: Type::Array(Box::new(Type::Named("Pet".to_string()))),
                    }),
                    headers: IndexMap::new(),
                },
            )]),
        };

        assert_eq!(api.models.len(), 1);
        assert_eq!(api.models.get("Pet"), Some(&expected_struct));

        assert_eq!(api.paths.len(), 1);
        assert_eq!(
            api.paths.get::<Path>(&"/pets".into()),
            Some(&vec![expected_route])
        );
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "AddPetBody".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "pet_data".to_string(),
                    description: None,
                    type_: Type::Named("AddPetBodyPetData".to_string()),
                    required: false,
                }],
            },
        };

        let expected_inner_struct = Model {
            name: "AddPetBodyPetData".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "name".to_string(),
                    description: None,
                    type_: Type::String,
                    required: false,
                }],
            },
        };

        let expected_route = Route {
            name: "addPet".to_string(),
            method: Method::Post,
            summary: None,
            request_body: Some(RequestBody {
                description: None,
                required: false,
                content: Content {
                    media_type: MediaType::Json,
                    type_: Type::Named("AddPetBody".to_string()),
                },
            }),
            parameters: Parameters::default(),
            responses: IndexMap::from([(
                http::StatusCode::OK.into(),
                Response {
                    description: "Successful".to_string(),
                    content: None,
                    headers: IndexMap::new(),
                },
            )]),
        };

        assert_eq!(api.models.len(), 2);
        assert_eq!(api.models.get("AddPetBody"), Some(&expected_struct));
        assert_eq!(
            api.models.get("AddPetBodyPetData"),
            Some(&expected_inner_struct)
        );

        assert_eq!(api.paths.len(), 1);
        assert_eq!(
            api.paths.get::<Path>(&"/pets".into()),
            Some(&vec![expected_route])
        );
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_struct = Model {
            name: "Pet".to_string(),
            description: None,
            type_: Type::Struct {
                fields: vec![Field {
                    name: "name".to_string(),
                    description: None,
                    type_: Type::String,
                    required: false,
                }],
            },
        };

        let expected_route = Route {
            name: "addPet".to_string(),
            method: Method::Post,
            summary: Some("Add a new pet to the store.".to_string()),
            request_body: Some(RequestBody {
                description: None,
                required: false,
                content: Content {
                    media_type: MediaType::Json,
                    type_: Type::Named("Pet".to_string()),
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
                            type_: Type::Named("Pet".to_string()),
                        }),
                        headers: IndexMap::new(),
                    },
                ),
                (
                    http::StatusCode::METHOD_NOT_ALLOWED.into(),
                    Response {
                        description: "Invalid input".to_string(),
                        content: None,
                        headers: IndexMap::new(),
                    },
                ),
            ]),
        };

        assert_eq!(api.models.len(), 1);
        assert_eq!(api.models.get("Pet"), Some(&expected_struct));

        assert_eq!(api.paths.len(), 1);
        assert_eq!(
            api.paths.get::<Path>(&"/pets".into()),
            Some(&vec![expected_route])
        );
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

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let result: Result<Api> = oas.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_one_of() {
        let components = serde_yaml::from_str::<openapiv3::Components>(
            r#"  
            schemas:
              Pet:
                type: object
                properties:
                  name:
                    type: string
              Car:
                type: object
                properties:
                  name:
                    type: string
            "#,
        )
        .unwrap();

        let paths = serde_yaml::from_str::<openapiv3::Paths>(
            r#"
            /test-one-of:
              get:
                operationId: testOneOf
                responses:
                  '200':
                    description: Ok.
                    content:
                      application/json:
                        schema:
                          oneOf:
                            - $ref: '#/components/schemas/Pet'
                            - $ref: '#/components/schemas/Car'
            "#,
        )
        .unwrap();

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: Some(components),
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_one_of = Model {
            name: "TestOneOfResponse".to_string(),
            description: None,
            type_: Type::OneOf {
                types: vec![
                    Type::Named("Pet".to_string()),
                    Type::Named("Car".to_string()),
                ],
            },
        };

        assert_eq!(api.models.len(), 3);
        assert_eq!(api.models.get("TestOneOfResponse"), Some(&expected_one_of));
    }

    #[test]
    fn test_resolve_operation_with_path_level_parameters() {
        let paths = serde_yaml::from_str::<openapiv3::Paths>(
            r#"    
            /foo/{a}/bar/{b}:
              parameters:
                - name: a
                  in: path
                  required: true
                  schema:
                    type: string
                - name: b
                  in: path
                  required: true
                  schema:
                    type: string
              get:
                operationId: foo
                parameters:
                  - name: b
                    in: path
                    required: true
                    schema:
                      type: integer
                  - name: c
                    in: query
                    required: true
                    schema:
                      type: string
                responses:
                  '200':
                    description: Successful
            "#,
        )
        .unwrap();

        let loader = OpenApiLoader::default();
        let api = openapiv3::OpenAPI {
            components: None,
            paths,
            ..openapiv3::OpenAPI::default()
        };
        let oas: OpenApi<'_> = loader.import(&api).unwrap();
        let api: Api = oas.try_into().unwrap();

        let expected_route = Route {
            name: "foo".to_string(),
            method: Method::Get,
            summary: None,
            request_body: None,
            parameters: Parameters {
                path: vec![
                    Parameter {
                        name: "a".to_string(),
                        description: None,
                        required: true,
                        type_: Type::String,
                    },
                    Parameter {
                        name: "b".to_string(),
                        description: None,
                        required: true,
                        type_: Type::Int32,
                    },
                ],
                query: vec![Parameter {
                    name: "c".to_string(),
                    description: None,
                    required: true,
                    type_: Type::String,
                }],
                ..Parameters::default()
            },
            responses: IndexMap::from([(
                http::StatusCode::OK.into(),
                Response {
                    description: "Successful".to_string(),
                    content: None,
                    headers: IndexMap::new(),
                },
            )]),
        };

        assert_eq!(api.paths.len(), 1);
        assert_eq!(
            api.paths.get::<Path>(&"/foo/{a}/bar/{b}".into()),
            Some(&vec![expected_route])
        );
    }
}
