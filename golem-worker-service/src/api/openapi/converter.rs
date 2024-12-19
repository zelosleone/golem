use crate::api::definition::types::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::definition::patterns::{AllPathPatterns, PathPattern};
use crate::api::openapi::types::{OpenAPISchemaType, Parameter, ParameterLocation};
use golem_wasm_ast::analysis::{
    AnalysedType,
    TypeStr, TypeBool, TypeS32, TypeS64,
};
use openapiv3::{
    OpenAPI as OpenAPISpec, Info, Paths, Operation, PathItem,
    Schema, Components, ReferenceOr, Header, Responses,
    MediaType, StringFormat,
    Response, RequestBody as OpenApiRequestBody,
    BooleanType, IntegerType, NumberType, StringType,
    HeaderStyle, StatusCode,
};
use indexmap::IndexMap;
use heck::ToSnakeCase;
use tracing::warn;

pub struct OpenAPIConverter;

impl OpenAPIConverter {
    pub fn convert(api: &ApiDefinition) -> OpenAPISpec {
        Self::convert_to_spec(api)
    }

    fn convert_to_spec(api: &ApiDefinition) -> OpenAPISpec {
        OpenAPISpec {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: api.name.clone(),
                version: api.version.clone(),
                description: Some(api.description.clone()),
                ..Default::default()
            },
            paths: Self::convert_paths(&api.routes),
            components: Some(Self::create_components(&api.routes)),
            ..Default::default()
        }
    }

    fn convert_paths(routes: &[Route]) -> Paths {
        let mut paths = Paths::default();
        
        for route in routes {
            let mut path_item = PathItem {
                summary: Some(route.description.clone()),
                ..Default::default()
            };

            // Create the main operation
            let operation = Self::generate_operation(route);
            match route.method {
                HttpMethod::Get => path_item.get = Some(operation),
                HttpMethod::Post => path_item.post = Some(operation),
                HttpMethod::Put => path_item.put = Some(operation),
                HttpMethod::Delete => path_item.delete = Some(operation),
                HttpMethod::Patch => path_item.patch = Some(operation),
                HttpMethod::Head => path_item.head = Some(operation),
                HttpMethod::Options => path_item.options = Some(operation),
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
                let schema = openapi_schema_type_to_schema(&param.schema);

                params.push(ReferenceOr::Item(openapiv3::Parameter::Path {
                    parameter_data: openapiv3::ParameterData {
                        name: param.name,
                        description: param.description,
                        required: true,
                        deprecated: None,
                        allow_empty_value: None,
                        style: openapiv3::ParameterStyle::Simple,
                        explode: Some(false),
                        allow_reserved: None,
                        schema: Some(ReferenceOr::Item(schema)),
                        example: None,
                        examples: Default::default(),
                        extensions: Default::default(),
                    },
                }));
            }
        }
        params
    }

    fn extract_path_parameters(path: &str) -> Option<Vec<Parameter>> {
        match AllPathPatterns::parse(path) {
            Ok(patterns) => {
                let params = patterns.parameters();
                if params.is_empty() {
                    None
                } else {
                    Some(
                        params
                            .into_iter()
                            .map(|name| Parameter {
                                name,
                                location: ParameterLocation::Path,
                                required: true,
                                description: None,
                                schema: Some(OpenAPISchemaType::String),
                                style: None,
                                explode: None,
                                deprecated: None,
                            })
                            .collect(),
                    )
                }
            }
            Err(e) => {
                warn!("Failed to parse path pattern: {}", e);
                None
            }
        }
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
        match &route.binding {
            BindingType::Default { input_type, .. } | BindingType::Worker { input_type, .. } => {
                let schema = analysed_type_to_schema(input_type);
                let mut content = IndexMap::new();
                content.insert(
                    "application/json".to_string(),
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
            },
            _ => None
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

        let schema = Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                format: None,
                pattern: None,
                enumeration: vec![Some(allowed_origins.to_string())],
                min_length: None,
                max_length: None,
            }))
        };

        let header = Header {
            description: None,
            deprecated: None,
            allow_empty_value: None,
            style: HeaderStyle::Simple,
            explode: None,
            allow_reserved: None,
            schema: Some(ReferenceOr::Item(schema)),
            example: None,
            examples: IndexMap::new(),
            content: IndexMap::new(),
            extensions: IndexMap::new(),
        };

        headers.insert("Access-Control-Allow-Origin".to_string(), ReferenceOr::Item(header.clone()));
        headers.insert("Access-Control-Allow-Methods".to_string(), ReferenceOr::Item(header.clone()));
        headers.insert("Access-Control-Allow-Headers".to_string(), ReferenceOr::Item(header.clone()));
        headers.insert("Access-Control-Max-Age".to_string(), ReferenceOr::Item(header.clone()));
        headers.insert("Access-Control-Allow-Credentials".to_string(), ReferenceOr::Item(header));

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
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType::default()))
            }),
            BindingType::Worker { output_type, .. } => {
                analysed_type_to_schema(output_type)
            },
            BindingType::Http => ReferenceOr::Reference {
                reference: "#/components/schemas/HttpResponse".to_string()
            },
            BindingType::Proxy => ReferenceOr::Reference {
                reference: "#/components/schemas/ProxyResponse".to_string()
            },
            BindingType::Static { .. } => ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType::default()))
            }),
        }
    }

    fn create_options_operation(route: &Route) -> Operation {
        Operation {
            tags: vec![route.template_name.clone()],
            responses: Responses {
                responses: {
                    let mut map = IndexMap::new();
                    map.insert(StatusCode::Code(200), ReferenceOr::Item(Response {
                        description: "CORS support".to_string(),
                        headers: Self::create_cors_headers("*"),
                        ..Default::default()
                    }));
                    map
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

fn analysed_type_to_schema(typ: &AnalysedType) -> ReferenceOr<Schema> {
    let schema = match typ {
        AnalysedType::Bool(_) => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(BooleanType {
                enumeration: vec![]
            }))
        },
        AnalysedType::S32(_) | AnalysedType::S64(_) => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(IntegerType {
                format: None,
                enumeration: vec![],
                ..Default::default()
            }))
        },
        AnalysedType::Str(_) => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                format: None,
                enumeration: vec![],
                ..Default::default()
            }))
        },
        _ => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Any(Default::default())
        },
    };
    ReferenceOr::Item(schema)
}

fn analysed_type_from_string(typ_str: &str) -> Result<AnalysedType, String> {
    match typ_str {
        "string" => Ok(AnalysedType::Str(TypeStr)),
        "bool" => Ok(AnalysedType::Bool(TypeBool)),
        _ => Err(format!("Unsupported type: {}", typ_str))
    }
}

// Conversion from `OpenAPISchemaType` to `Schema`
fn openapi_schema_type_to_schema(t: &OpenAPISchemaType) -> Schema {
    match t {
        OpenAPISchemaType::Boolean => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(BooleanType {
                enumeration: vec![]
            }))
        },
        OpenAPISchemaType::Integer { format } => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(IntegerType {
                format: format.clone(),
                multiple_of: None,
                minimum: None,
                maximum: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                enumeration: vec![]
            }))
        },
        OpenAPISchemaType::Number { format } => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Number(NumberType {
                format: format.clone(),
                multiple_of: None,
                minimum: None,
                maximum: None,
                exclusive_minimum: false,
                exclusive_maximum: false,
                enumeration: vec![]
            }))
        },
        OpenAPISchemaType::String { format, enum_values } => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(StringType {
                format: format.clone(),
                pattern: None,
                enumeration: enum_values.clone().unwrap_or_default().into_iter().map(Some).collect(),
                min_length: None,
                max_length: None
            }))
        },
        OpenAPISchemaType::Array { items } => Schema {
            schema_data: Default::default(),
            schema_kind: openapiv3::SchemaKind::Type(
                openapiv3::Type::Array(openapiv3::ArrayType {
                    items: Some(Box::new(ReferenceOr::Item(openapi_schema_type_to_schema(items)))),
                    min_items: None,
                    max_items: None,
                    unique_items: None
                })
            )
        },
        OpenAPISchemaType::Object { properties, required } => {
            let mut props = IndexMap::new();
            for (k,v) in properties {
                props.insert(k.clone(), ReferenceOr::Item(openapi_schema_type_to_schema(v)));
            }
            Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(
                    openapiv3::Type::Object(openapiv3::ObjectType {
                        properties: props,
                        required: required.clone(),
                        additional_properties: None,
                        min_properties: None,
                        max_properties: None
                    })
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use golem_wasm_ast::analysis::{TypeStr, TypeBool, TypeS32, TypeS64};
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
                    options: None,
                },
            }],
        };

        let spec = OpenAPIConverter::convert_to_spec(&api);
        assert!(spec.paths.paths.contains_key("/test"));
        
        if let Some(ReferenceOr::Item(path_item)) = spec.paths.paths.get("/test") {
            if let Some(operation) = &path_item.get {
                assert!(operation.request_body.is_some());
                assert!(!operation.responses.responses.is_empty());
            } else {
                panic!("GET operation not found");
            }
        } else {
            panic!("Path item not found");
        }
    }

    #[test]
    fn test_all_binding_types() {
        let bindings = vec![
            BindingType::Default {
                input_type: AnalysedType::Str(TypeStr),
                output_type: AnalysedType::Str(TypeStr),
                options: None,
            },
            BindingType::Worker {
                input_type: AnalysedType::Bool(TypeBool),
                output_type: AnalysedType::S32(TypeS32),
                options: None,
            },
            BindingType::FileServer {
                root_dir: "/test".to_string(),
                options: None,
            },
            BindingType::SwaggerUI {
                spec_path: "/openapi.json".to_string(),
                options: None,
            },
            BindingType::Http,
            BindingType::Proxy,
            BindingType::Static {
                content_type: "text/plain".to_string(),
                content: vec![],
            },
        ];

        for binding in bindings {
            let route = Route {
                path: "/test".to_string(),
                method: HttpMethod::Get,
                description: "Test route".to_string(),
                template_name: "test".to_string(),
                binding,
            };

            let schema = OpenAPIConverter::get_response_schema(&route);
            assert!(matches!(schema, ReferenceOr::Item(_)) || 
                   matches!(schema, ReferenceOr::Reference { .. }));
        }
    }
}