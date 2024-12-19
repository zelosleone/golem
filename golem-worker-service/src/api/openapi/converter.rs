use crate::api::definition::types::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::definition::patterns::{AllPathPatterns, PathPattern};
use golem_wasm_ast::analysis::{
    AnalysedType, TypeStr, TypeI32, TypeI64, TypeF32, TypeF64, TypeBool, 
    TypeList, TypeOption, TypeRecord, TypeResult
};
use wasm_ast::analysis::model::{TypeUnit};
use openapiv3::{
    self, 
    OpenAPI as OpenAPISpec, Info, Paths, Operation, PathItem,
    Schema, SecurityScheme, Parameter as OpenApiParameter,
    Components, ReferenceOr, Header, Responses, 
    ParameterData, QueryStyle,
    MediaType, StringFormat, IntegerFormat,
    Response, RequestBody as OpenApiRequestBody,
    ArrayType, BooleanType, IntegerType, NumberType, ObjectType, StringType,
    AdditionalProperties, HeaderStyle, Callback, Link, Server,
};
use indexmap::IndexMap;
use crate::api::openapi::types::{Parameter, ParameterLocation};
use std::collections::HashMap;
use heck::ToSnakeCase;
use tracing::warn;

trait IntoOpenApi<T> {
    fn into_openapi(self) -> T;
}

impl IntoOpenApi<ReferenceOr<Schema>> for crate::api::openapi::types::Schema {
    fn into_openapi(self) -> ReferenceOr<Schema> {
        match self {
            Self::String { format, enum_values } => {
                let fmt = format.map(|f| match f.as_str() {
                    "date" => StringFormat::Date,
                    "date-time" => StringFormat::DateTime,
                    "binary" => StringFormat::Binary,
                    other => StringFormat::Other(other.to_string()),
                });
                let enumeration = if let Some(vals) = enum_values {
                    vals.into_iter().map(Some).collect()
                } else {
                    vec![]
                };

                ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                        format: fmt,
                        pattern: None,
                        enumeration,
                        min_length: None,
                        max_length: None,
                    })),
                })
            },
            Self::Integer { format } => {
                let fmt = format.map(|f| match f.as_str() {
                    "int32" => IntegerFormat::Int32,
                    "int64" => IntegerFormat::Int64,
                    other => IntegerFormat::Other(other.to_string()),
                });

                ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(IntegerType {
                        format: fmt,
                        multiple_of: None,
                        minimum: None,
                        maximum: None,
                        exclusive_minimum: false,
                        exclusive_maximum: false,
                        enumeration: vec![],
                    })),
                })
            },
            Self::Boolean => ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(BooleanType {}))
            }),
            Self::Array { items } => {
                let item_schema = items.into_openapi(); // ReferenceOr<Schema>
                ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(ArrayType {
                        items: Some(Box::new(item_schema)),
                        min_items: None,
                        max_items: None,
                        unique_items: false,
                    })),
                })
            },
            Self::Ref { reference } => {
                ReferenceOr::Reference {
                    reference
                }
            },
            Self::Object { properties, required, additional_properties } => {
                let converted_props: IndexMap<String, ReferenceOr<Schema>> = properties
                    .into_iter()
                    .map(|(k, v)| (k, v.into_openapi()))
                    .collect();

                let additional = additional_properties.map(|schema| 
                    AdditionalProperties::Schema(Box::new(schema.into_openapi()))
                );

                ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(ObjectType {
                        properties: converted_props,
                        required: required.unwrap_or_default(),
                        additional_properties: additional,
                        min_properties: None,
                        max_properties: None,
                    })),
                })
            },
            Self::Number => ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Number(NumberType {
                    multiple_of: None,
                    minimum: None,
                    maximum: None,
                    exclusive_minimum: false,
                    exclusive_maximum: false,
                    enumeration: vec![],
                }))
            }),
        }
    }
}

impl IntoOpenApi<openapiv3::Parameter> for crate::api::openapi::types::Parameter {
    fn into_openapi(self) -> openapiv3::Parameter {
        let schema = self.schema.into_openapi(); // ReferenceOr<Schema>
        let parameter_data = ParameterData {
            name: self.name,
            description: self.description,
            required: self.required.unwrap_or(false),
            deprecated: None,
            explode: false,
            format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(schema)),
            example: None,
            examples: Default::default(),
            extensions: Default::default(),
        };

        match self.r#in {
            ParameterLocation::Path => openapiv3::Parameter::Path {
                parameter_data,
                style: openapiv3::PathStyle::Simple,
            },
            ParameterLocation::Query => openapiv3::Parameter::Query {
                parameter_data,
                style: QueryStyle::Form,
                allow_reserved: false,
                allow_empty_value: Some(false),
            },
            ParameterLocation::Header => openapiv3::Parameter::Header {
                parameter_data,
                style: HeaderStyle::Simple,
            },
            ParameterLocation::Cookie => openapiv3::Parameter::Cookie {
                parameter_data,
                style: openapiv3::CookieStyle::Form,
            },
        }
    }
}

pub struct OpenAPIConverter;

impl OpenAPIConverter {
    pub fn convert_to_spec(api: &ApiDefinition) -> OpenAPISpec {
        OpenAPISpec {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: "Golem API".to_string(),
                version: "1.0".to_string(),
                description: None,
                terms_of_service: None,
                contact: None,
                license: None,
                extensions: Default::default(),
            },
            paths: Self::convert_paths(&api.routes),
            components: Some(Self::create_components(&api.routes)),
            security: None,
            tags: vec![],
            extensions: Default::default(),
            servers: vec![],
            external_docs: None,
        }
    }

    pub fn convert_paths(routes: &[Route]) -> Paths {
        let mut paths = Paths {
            paths: Default::default(),
            extensions: Default::default(),
        };

        for route in routes {
            let operation = Self::generate_operation(route);

            // Create an OPTIONS operation for CORS, if desired
            let options_operation = Operation {
                tags: vec![route.template_name.clone()],
                summary: None,
                description: None,
                external_docs: None,
                operation_id: None,
                parameters: vec![],
                request_body: None,
                responses: Responses {
                    default: None,
                    responses: {
                        let mut map = IndexMap::new();
                        // Insert a 200 response for OPTIONS
                        map.insert(http::StatusCode::OK, ReferenceOr::Item(Response {
                            description: String::new(),
                            content: IndexMap::new(),
                            headers: Self::create_cors_headers("*"),
                            links: IndexMap::new(),
                            extensions: Default::default(),
                        }));
                        map
                    },
                    extensions: Default::default(),
                },
                callbacks: Default::default(),
                deprecated: false,
                security: None,
                servers: vec![],
                extensions: Default::default(),
            };

            let mut path_item = PathItem {
                summary: None,
                description: None,
                get: None,
                put: None,
                post: None,
                delete: None,
                options: Some(options_operation),
                head: None,
                patch: None,
                trace: None,
                servers: vec![],
                parameters: vec![],
                extensions: Default::default(),
            };

            match route.method {
                HttpMethod::Get => path_item.get = Some(operation),
                HttpMethod::Post => path_item.post = Some(operation),
                HttpMethod::Put => path_item.put = Some(operation),
                HttpMethod::Delete => path_item.delete = Some(operation),
            }

            paths.paths.insert(route.path.clone(), ReferenceOr::Item(path_item));
        }

        paths
    }

    fn generate_operation(route: &Route) -> Operation {
        Operation {
            tags: vec![route.template_name.clone()],
            summary: Some(route.description.clone()),
            description: None,
            external_docs: None,
            operation_id: Some(format!("{}_{}", 
                route.template_name.to_snake_case(),
                route.method.to_string().to_lowercase()
            )),
            parameters: Self::convert_parameters(route),
            request_body: Self::create_request_body(route).map(ReferenceOr::Item),
            responses: Self::create_responses(route),
            deprecated: false,
            security: None,
            servers: vec![],
            callbacks: Default::default(),
            extensions: Default::default(),
        }
    }

    fn convert_parameters(route: &Route) -> Vec<ReferenceOr<OpenApiParameter>> {
        let mut params = Vec::new();
        
        // Convert path parameters
        if let Some(path_params) = Self::extract_path_parameters(&route.path) {
            for param in path_params {
                params.push(ReferenceOr::Item(param.into_openapi()));
            }
        }

        // If you have query or header parameters strongly typed, convert them similarly.

        params
    }

    fn extract_path_parameters(path: &str) -> Option<Vec<Parameter>> {
        let path_pattern = match AllPathPatterns::parse(path) {
            Ok(pattern) => pattern,
            Err(e) => {
                warn!("Failed to parse path pattern: {}", e);
                return None;
            }
        };

        let params: Vec<Parameter> = path_pattern.path_patterns
            .iter()
            .filter_map(|pattern| match pattern {
                PathPattern::Var(info) => {
                    if !Self::validate_path_parameter(&info.key_name) {
                        warn!("Invalid path parameter name: {}", info.key_name);
                        return None;
                    }

                    // If you know the WIT type of this parameter from analysis, 
                    // you would call `analysed_type_to_schema` here instead of `infer_parameter_type`.
                    // For now, let's assume a simple default:
                    let schema = crate::api::openapi::types::Schema::String { format: None, enum_values: None };
                    let description = format!("Path parameter: {}", info.key_name);

                    Some(Parameter {
                        name: info.key_name.clone(),
                        r#in: ParameterLocation::Path,
                        required: Some(true),
                        schema,
                        style: Some("simple".to_string()),
                        explode: Some(false),
                        description: Some(description)
                    })
                },
                PathPattern::CatchAllVar(info) => {
                    if !Self::validate_catch_all_parameter(&info.key_name) {
                        warn!("Invalid catch-all parameter name: {}", info.key_name);
                        return None;
                    }

                    Some(Parameter {
                        name: info.key_name.clone(),
                        r#in: ParameterLocation::Path,
                        required: Some(true),
                        schema: crate::api::openapi::types::Schema::Array {
                            items: Box::new(crate::api::openapi::types::Schema::String {
                                format: None,
                                enum_values: None
                            })
                        },
                        style: Some("matrix".to_string()),
                        explode: Some(true),
                        description: Some(format!(
                            "Multi-segment catch-all parameter for {}",
                            info.key_name
                        ))
                    })
                },
                _ => None
            })
            .collect();

        if params.is_empty() { None } else { Some(params) }
    }

    fn validate_path_parameter(name: &str) -> bool {
        !name.is_empty() 
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            && !name.starts_with('_')
            && !name.ends_with('_')
    }

    fn validate_catch_all_parameter(name: &str) -> bool {
        Self::validate_path_parameter(name) && !name.contains("__")
    }

    fn create_request_body(route: &Route) -> Option<OpenApiRequestBody> {
        if let BindingType::Default { input_type, .. } = &route.binding {
            // Now `input_type` is an AnalysedType. Convert it directly:
            let schema = analysed_type_to_schema(input_type);

            let mut content = IndexMap::new();
            content.insert(
                "application/json; charset=utf-8".to_string(),
                MediaType {
                    schema: Some(schema),
                    example: None,
                    examples: Default::default(),
                    encoding: Default::default(),
                    extensions: Default::default(),
                }
            );

            Some(OpenApiRequestBody {
                description: None,
                content,
                required: true,
                extensions: Default::default()
            })
        } else {
            None
        }
    }

    fn create_responses(route: &Route) -> Responses {
        let mut responses = Responses {
            default: None,
            responses: IndexMap::new(),
            extensions: Default::default(),
        };

        let schema = Self::get_response_schema(route);

        let mut content = IndexMap::new();
        content.insert(
            "application/json; charset=utf-8".to_string(),
            MediaType {
                schema: Some(schema),
                example: None,
                examples: Default::default(),
                encoding: Default::default(),
                extensions: Default::default(),
            }
        );

        responses.responses.insert(
            http::StatusCode::OK,
            ReferenceOr::Item(Response {
                description: String::new(),
                content,
                headers: Self::create_cors_headers("*"),
                links: IndexMap::new(),
                extensions: Default::default(),
            })
        );

        responses
    }

    fn create_cors_headers(_cors_allowed_origins: &str) -> IndexMap<String, ReferenceOr<Header>> {
        let mut headers = IndexMap::new();
        let cors_header = Header {
            description: None,
            required: false,
            deprecated: None,
            style: HeaderStyle::Simple,
            explode: false,
            schema: None,
            example: None,
            examples: Default::default(),
            extensions: Default::default(),
        };
        
        for header_name in [
            "Access-Control-Allow-Origin",
            "Access-Control-Allow-Methods",
            "Access-Control-Allow-Headers",
            "Access-Control-Max-Age",
            "Access-Control-Expose-Headers",
        ] {
            headers.insert(
                header_name.to_string(),
                ReferenceOr::Item(cors_header.clone())
            );
        }
        headers
    }

    fn create_components(_routes: &[Route]) -> Components {
        Components {
            schemas: IndexMap::new(),
            responses: IndexMap::new(),
            parameters: IndexMap::new(),
            examples: IndexMap::new(),
            request_bodies: IndexMap::new(),
            headers: IndexMap::new(),
            security_schemes: IndexMap::new(),
            links: IndexMap::new(),
            callbacks: IndexMap::new(),
            extensions: Default::default(),
        }
    }

    fn get_response_schema(route: &Route) -> ReferenceOr<Schema> {
        match &route.binding {
            BindingType::Default { output_type, .. } => {
                analysed_type_to_schema(output_type)
            },
            BindingType::FileServer { .. } => ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                    format: Some(StringFormat::Binary),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }),
            BindingType::SwaggerUI { .. } => ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                    format: Some(StringFormat::Other("html".to_string())),
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }),
            BindingType::Http => {
                // If HTTP binding returns a known type:
                ReferenceOr::Reference {
                    reference: "#/components/schemas/HttpResponse".to_string()
                }
            },
            BindingType::Worker => {
                ReferenceOr::Reference {
                    reference: "#/components/schemas/WorkerResponse".to_string()
                }
            },
            BindingType::Proxy => {
                ReferenceOr::Reference {
                    reference: "#/components/schemas/ProxyResponse".to_string()
                }
            },
        }
    }
}

// New function to convert AnalysedType directly to a Schema reference
fn analysed_type_to_schema(typ: &AnalysedType) -> ReferenceOr<Schema> {
    match typ {
        AnalysedType::Str(_) => ReferenceOr::Item(Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                format: None,
                pattern: None,
                enumeration: vec![],
                min_length: None,
                max_length: None,
            })),
        }),
        AnalysedType::I32(_) => int_schema("int32"),
        AnalysedType::I64(_) => int_schema("int64"),
        AnalysedType::F32(_) => num_schema(None),
        AnalysedType::F64(_) => num_schema(None),
        AnalysedType::Bool(_) => ReferenceOr::Item(Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(BooleanType {})),
        }),
        AnalysedType::Empty(_) => {
            // Representing void/empty as a string or empty object is up to you:
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(ObjectType {
                    properties: IndexMap::new(),
                    required: vec![],
                    additional_properties: None,
                    min_properties: None,
                    max_properties: None,
                })),
            })
        },
        AnalysedType::List(list_type) => {
            let inner = analysed_type_to_schema(&list_type.inner);
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(ArrayType {
                    items: Some(Box::new(inner)),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                })),
            })
        },
        AnalysedType::Record(record_type) => {
            let mut properties = IndexMap::new();
            for field in &record_type.fields {
                properties.insert(field.name.clone(), analysed_type_to_schema(&field.typ));
            }
            ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(ObjectType {
                    properties,
                    required: record_type.fields.iter().map(|f| f.name.clone()).collect(),
                    additional_properties: None,
                    min_properties: None,
                    max_properties: None,
                })),
            })
        },
        AnalysedType::Option(option_type) => {
            // OpenAPI doesn't have a direct "optional" field, 
            // typically represent option<T> as a schema that is not required.
            // One way is to use a oneOf with null allowed:
            // For simplicity, treat as nullable schema (OpenAPI 3.1 supports nullable)
            // If using OAS3.0, we might define a union of type with null.
            let inner = analysed_type_to_schema(&option_type.inner);
            // We'll just return the inner type as is; 
            // clients should handle "absence" as `null` if allowed:
            inner
        },
        AnalysedType::Result(result_type) => {
            // A result<ok, err> could be represented as a oneOf with ok and err 
            // objects or a complex schema. This is domain-specific.
            // For simplicity, let's say result<ok,err> is a union (oneOf):
            if let (Some(ok), Some(err)) = (&result_type.ok, &result_type.err) {
                ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::OneOf {
                        one_of: vec![analysed_type_to_schema(ok), analysed_type_to_schema(err)],
                    }
                })
            } else {
                // If result is malformed or incomplete, fallback:
                ReferenceOr::Item(Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Any,
                })
            }
        },
    }
}

fn int_schema(format_str: &str) -> ReferenceOr<Schema> {
    let fmt = match format_str {
        "int32" => Some(IntegerFormat::Int32),
        "int64" => Some(IntegerFormat::Int64),
        _ => Some(IntegerFormat::Other(format_str.to_string())),
    };
    ReferenceOr::Item(Schema {
        schema_data: Default::default(),
        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(IntegerType {
            format: fmt,
            multiple_of: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: false,
            exclusive_maximum: false,
            enumeration: vec![],
        })),
    })
}

fn num_schema(_format: Option<&str>) -> ReferenceOr<Schema> {
    // No special format used for floating numbers
    ReferenceOr::Item(Schema {
        schema_data: Default::default(),
        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Number(NumberType {
            multiple_of: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: false,
            exclusive_maximum: false,
            enumeration: vec![],
        })),
    })
}

#[cfg(test)]
mod tests {
   use super::*;
   use golem_wasm_ast::analysis::AnalysedType;

   #[test]
   fn test_simple_analysed_type_to_schema() {
       let string_schema = analysed_type_to_schema(&AnalysedType::Str(TypeStr));
       if let ReferenceOr::Item(sch) = string_schema {
           match &sch.schema_kind {
               openapiv3::SchemaKind::Type(openapiv3::Type::String(_)) => {},
               _ => panic!("Expected string type"),
           }
       } else {
           panic!("Expected item schema");
       }
   }
}