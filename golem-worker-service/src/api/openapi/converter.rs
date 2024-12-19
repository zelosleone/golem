use crate::api::definition::types::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::definition::patterns::{AllPathPatterns, PathPattern};
use crate::api::openapi::types::{OpenAPISchemaType, Parameter};
use golem_wasm_ast::analysis::{
    AnalysedType,
    TypeStr, TypeBool, TypeList, TypeRecord,
};
use openapiv3::{
    OpenAPI as OpenAPISpec, Info, Paths, Operation, PathItem,
    Schema, Parameter as OpenApiParameter,
    Components, ReferenceOr, Header, Responses,
    ParameterData, QueryStyle, MediaType, StringFormat, IntegerFormat,
    Response, RequestBody as OpenApiRequestBody,
    ArrayType, BooleanType, IntegerType, NumberType, ObjectType, StringType,
    AdditionalProperties, HeaderStyle, VariantOrUnknownOrEmpty,
    StatusCode,
};
use indexmap::IndexMap;
use crate::api::openapi::types::{Parameter, ParameterLocation};
use std::collections::HashMap;
use heck::ToSnakeCase;
use tracing::warn;

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
                        map.insert(StatusCode::Code(200), ReferenceOr::Item(Response {
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

    fn convert_parameters(route: &Route) -> Vec<ReferenceOr<openapiv3::Parameter>> {
        let mut params = Vec::new();
        
        if let Some(path_params) = Self::extract_path_parameters(&route.path) {
            for param in path_params {
                let schema: Schema = param.schema.clone().into();
                let param = openapiv3::Parameter::Path {
                    parameter_data: openapiv3::ParameterData {
                        name: param.name,
                        description: param.description,
                        required: param.required.unwrap_or(true),
                        deprecated: None,
                        format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(schema))),
                        example: None,
                        examples: Default::default(),
                        explode: param.explode.unwrap_or(false),
                        extensions: Default::default(),
                    },
                    style: param.style.map(|s| s.parse().unwrap_or(openapiv3::PathStyle::Simple))
                        .unwrap_or(openapiv3::PathStyle::Simple),
                };
                params.push(ReferenceOr::Item(param));
            }
        }
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

                    // Default to string schema for path params
                    let schema = OpenAPISchemaType::String { format: None, enum_values: None };
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
                        schema: OpenAPISchemaType::Array {
                            items: Box::new(OpenAPISchemaType::String {
                                format: None,
                                enum_values: None
                            })
                        },
                        style: Some("matrix".to_string()),
                        explode: Some(true),
                        description: Some(format![
                            "Multi-segment catch-all parameter for {}",
                            info.key_name
                        ])
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
            let schema = analysed_type_to_schema(&input_type);

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
            StatusCode::Code(200),
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

    fn create_cors_headers(allowed_origins: &str) -> IndexMap<String, ReferenceOr<Header>> {
        let mut headers = IndexMap::new();
        let header = Header {
            description: None,
            required: false,
            deprecated: None,
            style: HeaderStyle::Simple,
            schema_or_content: openapiv3::ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                        format: None,
                        pattern: None,
                        enumeration: vec![Some(allowed_origins.to_string())],
                        min_length: None,
                        max_length: None,
                    }))
                }
            )),
            example: None,
            examples: IndexMap::new(),
            extensions: IndexMap::new(),
        };
        
        headers.insert(
            "Access-Control-Allow-Origin".to_string(),
            ReferenceOr::Item(header.clone())
        );
        headers.insert(
            "Access-Control-Allow-Methods".to_string(), 
            ReferenceOr::Item(header.clone())
        );
        headers.insert(
            "Access-Control-Allow-Headers".to_string(),
            ReferenceOr::Item(header.clone())
        );
        headers.insert(
            "Access-Control-Max-Age".to_string(),
            ReferenceOr::Item(header.clone())
        );
        headers.insert(
            "Access-Control-Allow-Credentials".to_string(),
            ReferenceOr::Item(header.clone())
        );
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
                    // Just set to None if you don't want 'Other("html")'
                    format: None,
                    pattern: None,
                    enumeration: vec![],
                    min_length: None,
                    max_length: None,
                }))
            }),
            BindingType::Http => ReferenceOr::Reference {
                reference: "#/components/schemas/HttpResponse".to_string()
            },
            BindingType::Worker => ReferenceOr::Reference {
                reference: "#/components/schemas/WorkerResponse".to_string()
            },
            BindingType::Proxy => ReferenceOr::Reference {
                reference: "#/components/schemas/ProxyResponse".to_string()
            },
        }
    }
}

fn analysed_type_to_schema(typ: &AnalysedType) -> ReferenceOr<Schema> {
    let schema = match typ {
        AnalysedType::Bool(_) => OpenAPISchemaType::Boolean,
        AnalysedType::S32(_) | AnalysedType::S64(_) => OpenAPISchemaType::Integer {
            format: None,
        },
        AnalysedType::F32(_) | AnalysedType::F64(_) => OpenAPISchemaType::Number {
            format: None,
        },
        AnalysedType::Chr(_) => OpenAPISchemaType::String {
            format: None,
            enum_values: None,
        },
        AnalysedType::Str(_) => OpenAPISchemaType::String {
            format: None,
            enum_values: None,
        },
        AnalysedType::List(l) => OpenAPISchemaType::Array {
            items: Box::new(analysed_type_to_schema(&l.inner).into()),
        },
        AnalysedType::Tuple(_) => OpenAPISchemaType::Array {
            items: Box::new(OpenAPISchemaType::String {
                format: None,
                enum_values: None,
            }),
        },
        AnalysedType::Record(r) => {
            let mut properties = IndexMap::new();
            for field in &r.fields {
                properties.insert(field.name.clone(), analysed_type_to_schema(&field.typ).into());
            }
            OpenAPISchemaType::Object {
                properties,
                required: r.fields.iter().map(|f| f.name.clone()).collect(),
            }
        },
        AnalysedType::Flags(_) => OpenAPISchemaType::String {
            format: None,
            enum_values: None,
        },
        AnalysedType::Enum(e) => OpenAPISchemaType::String {
            format: None,
            enum_values: Some(e.variants.iter().map(|v| v.name.clone()).collect()),
        },
    };
    ReferenceOr::Item(schema.into())
}

// Update schema creation functions to use OpenAPISchemaType
fn boolean_schema() -> ReferenceOr<Schema> {
    ReferenceOr::Item(Schema {
        schema_data: Default::default(),
        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(BooleanType {
            enumeration: vec![],
        })),
    })
}

fn integer_schema(_format: Option<&str>) -> ReferenceOr<Schema> {
    ReferenceOr::Item(Schema {
        schema_data: Default::default(),
        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(IntegerType {
            format: None,
            multiple_of: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: false,
            exclusive_maximum: false,
            enumeration: vec![],
        })),
    })
}

fn number_schema(_format: Option<&str>) -> ReferenceOr<Schema> {
    ReferenceOr::Item(Schema {
        schema_data: Default::default(),
        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Number(NumberType {
            format: None,
            multiple_of: None,
            minimum: None,
            maximum: None,
            exclusive_minimum: false,
            exclusive_maximum: false,
            enumeration: vec![],
        })),
    })
}

fn string_schema(_format: Option<&str>) -> ReferenceOr<Schema> {
    ReferenceOr::Item(Schema {
        schema_data: Default::default(),
        schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
            format: None,
            pattern: None,
            enumeration: vec![],
            min_length: None,
            max_length: None,
        })),
    })
}

#[cfg(test)]
mod tests {
   use super::*;
   use golem_wasm_ast::analysis::{TypeStr, TypeRecord, NameTypePair};
   use crate::api::definition::types::{Route, HttpMethod, BindingType, ApiDefinition};

   #[test]
   fn test_simple_string_input_output() {
       let input_type = AnalysedType::Str(TypeStr);
       let output_type = AnalysedType::Str(TypeStr);

       let api = ApiDefinition {
           id: "test".to_string(),
           name: "test".to_string(),
           version: "1.0".to_string(),
           description: "Test API".to_string(),
           routes: vec![Route {
               path: "/test".to_string(),
               method: HttpMethod::Get,
               description: "Test route".to_string(),
               template_name: "test".to_string(),
               binding: BindingType::Default {
                   input_type,
                   output_type,
                   function_name: "test".to_string(),
               },
           }],
       };

       let spec = OpenAPIConverter::convert_to_spec(&api);
       assert!(spec.paths.paths.contains_key("/test"));
   }
}