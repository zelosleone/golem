use crate::api::definition::types::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::definition::patterns::{AllPathPatterns, PathPattern};
use openapiv3::{
    OpenAPI as OpenAPISpec, Info, Paths, Operation, PathItem,
    Schema, SecurityScheme, Parameter as OpenApiParameter,
    Components, ReferenceOr, Header, Responses, Tag,
    ParameterData, QueryStyle, ExternalDocumentation,
};
use crate::api::openapi::types::{Parameter, ParameterLocation};
use std::collections::HashMap;
use heck::ToSnakeCase;
use openapiv3::Response;
use openapiv3::RequestBody as OpenApiRequestBody;
use openapiv3::MediaType as OpenApiMediaType;
use openapiv3::{self, MediaType, IndexMap, SchemaObject, StringType, IntegerType, NumberType, ArrayType, BooleanType, HeaderStyle, ObjectType};

// Add conversion traits
trait IntoOpenApi<T> {
    fn into_openapi(self) -> T;
}

impl IntoOpenApi<openapiv3::Schema> for crate::api::openapi::types::Schema {
    fn into_openapi(self) -> openapiv3::Schema {
        match self {
            Self::String { format, enum_values } => {
                let string_format = format.map(|f| match f.as_str() {
                    "date" => openapiv3::StringFormat::Date,
                    "date-time" => openapiv3::StringFormat::DateTime,
                    "binary" => openapiv3::StringFormat::Binary,
                    _ => openapiv3::StringFormat::Default,
                });
                openapiv3::Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::String(openapiv3::StringType {
                        format: string_format.into(),
                        pattern: None,
                        enumeration: enum_values,
                        min_length: None,
                        max_length: None,
                    })),
                }
            },
            Self::Integer { format } => {
                let int_format = format.map(|f| match f.as_str() {
                    "int32" => openapiv3::IntegerFormat::Int32,
                    "int64" => openapiv3::IntegerFormat::Int64,
                    _ => openapiv3::IntegerFormat::Default,
                });
                openapiv3::Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Integer(openapiv3::IntegerType {
                        format: int_format.into(),
                        multiple_of: None,
                        minimum: None,
                        maximum: None,
                        exclusive_minimum: false,
                        exclusive_maximum: false,
                    })),
                }
            },
            Self::Boolean => openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Boolean(Default::default()))
            },
            Self::Array { items } => openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Array(openapiv3::ArrayType {
                    items: Some(ReferenceOr::Item(Box::new(items.into_openapi()))),
                    min_items: None,
                    max_items: None,
                    unique_items: false,
                })),
            },
            Self::Ref { reference } => openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Reference {
                    reference,
                },
            },
            Self::Object { properties, required, additional_properties } => {
                let converted_props = properties.into_iter()
                    .map(|(k, v)| (k, ReferenceOr::Item(v.into_openapi())))
                    .collect();
                let additional = additional_properties.map(|schema| 
                    openapiv3::AdditionalProperties::Schema(Box::new(schema.into_openapi())));
                
                openapiv3::Schema {
                    schema_data: Default::default(),
                    schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                        properties: converted_props,
                        required: required.unwrap_or_default(),
                        additional_properties: additional,
                        min_properties: None,
                        max_properties: None,
                    })),
                }
            },
            Self::Number => openapiv3::Schema {
                schema_data: Default::default(),
                schema_kind: openapiv3::SchemaKind::Type(openapiv3::Type::Number(Default::default()))
            },
        }
    }
}

impl IntoOpenApi<openapiv3::Parameter> for crate::api::openapi::types::Parameter {
    fn into_openapi(self) -> openapiv3::Parameter {
        let parameter_data = openapiv3::ParameterData {
            name: self.name,
            description: self.description,
            required: self.required.unwrap_or(false),
            deprecated: None,
            format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(self.schema.into_openapi())),
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
                style: openapiv3::HeaderStyle::Simple,
            },
            // Add other cases as needed
            _ => unimplemented!("Parameter location not supported"),
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
            tags: Some(vec![]),
            extensions: Default::default(),
            servers: Default::default(),
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

            let path_item = PathItem {
                summary: None,
                get: if route.method == HttpMethod::Get { Some(operation.clone()) } else { None },
                post: if route.method == HttpMethod::Post { Some(operation.clone()) } else { None },
                put: if route.method == HttpMethod::Put { Some(operation.clone()) } else { None },
                delete: if route.method == HttpMethod::Delete { Some(operation.clone()) } else { None },
                options: Some(Operation {
                    tags: vec![route.template_name.clone()],
                    summary: None,
                    description: None,
                    external_docs: None,
                    operation_id: None,
                    parameters: None,
                    request_body: None,
                    responses: Responses {
                        responses: {
                            let mut map = indexmap::IndexMap::new();
                            map.insert("200".to_string(), Response {
                                description: String::new(),
                                content: Default::default(),
                                headers: Self::create_cors_headers("*"),
                                extensions: Default::default(),
                                links: Default::default(),
                            });
                            map
                        },
                        extensions: Default::default(),
                    },
                    callbacks: Default::default(),
                    deprecated: false,
                    security: None,
                    servers: None,
                    extensions: Default::default(),
                }),
                description: None,
                servers: None,
                parameters: None,
                extensions: Default::default(),
                head: None,
                patch: None,
                trace: None,
            };

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
            servers: None,
            callbacks: Default::default(),
            extensions: Default::default(),
        }
    }

    fn convert_parameters(route: &Route) -> Option<Vec<ReferenceOr<OpenApiParameter>>> {
        let mut params = Vec::new();
        
        // Convert path parameters
        if let Some(path_params) = Self::extract_path_parameters(&route.path) {
            for param in path_params {
                params.push(ReferenceOr::Item(param.into_openapi()));
            }
        }

        // Convert query parameters
        for param in Self::extract_query_parameters(route) {
            params.push(ReferenceOr::Item(param.into_openapi()));
        }

        // Convert header parameters
        for param in Self::extract_header_parameters(route) {
            params.push(ReferenceOr::Item(param.into_openapi()));
        }

        if params.is_empty() {
            None
        } else {
            Some(params)
        }
    }

    fn convert_parameter(param: &Parameter) -> OpenApiParameter {
        match param.r#in {
            ParameterLocation::Path => OpenApiParameter::Path {
                parameter_data: Self::create_parameter_data(param),
                style: openapiv3::PathStyle::Simple,
            },
            ParameterLocation::Query => OpenApiParameter::Query {
                parameter_data: Self::create_parameter_data(param),
                style: QueryStyle::Form,
                allow_reserved: false,
                allow_empty_value: Some(false),
            },
            ParameterLocation::Header => OpenApiParameter::Header {
                parameter_data: Self::create_parameter_data(param),
                style: openapiv3::HeaderStyle::Simple,
            },
            // Add other cases as needed
            _ => unimplemented!("Parameter location not supported"),
        }
    }

    fn create_parameter_data(param: &Parameter) -> ParameterData {
        ParameterData {
            name: param.name.clone(),
            description: param.description.clone(),
            required: param.required.unwrap_or(false),
            deprecated: None,
            format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(param.schema.clone())),
            example: None,
            examples: Default::default(),
            extensions: Default::default(),
        }
    }

    fn extract_path_parameters(path: &str) -> Option<Vec<Parameter>> {
        let path_pattern = match AllPathPatterns::parse(path) {
            Ok(pattern) => pattern,
            Err(e) => {
                tracing::warn!("Failed to parse path pattern: {}", e);
                return None;
            }
        };

        let params: Vec<Parameter> = path_pattern.path_patterns
            .iter()
            .filter_map(|pattern| match pattern {
                PathPattern::Var(info) => {
                    if !Self::validate_path_parameter(&info.key_name) {
                        tracing::warn!("Invalid path parameter name: {}", info.key_name);
                        return None;
                    }

                    let (schema, description) = Self::infer_parameter_type(&info.key_name);
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
                        tracing::warn!("Invalid catch-all parameter name: {}", info.key_name);
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
                            "Multi-segment catch-all parameter: matches one or more path segments for {}", 
                            info.key_name
                        ))
                    })
                },
                _ => None
            })
            .collect();

        if params.is_empty() { None } else { Some(params) }
    }

    fn infer_parameter_type(name: &str) -> (crate::api::openapi::types::Schema, String) {
        match name {
            n if n.ends_with("_id") => (
                crate::api::openapi::types::Schema::String { 
                    format: Some("uuid".to_string()),
                    enum_values: None
                },
                format!("Unique identifier for {}", &n[..n.len()-3])
            ),
            n if n.ends_with("_num") || n.ends_with("_count") => (
                crate::api::openapi::types::Schema::Integer { 
                     format: Some("int64".to_string()) 
                 },
                format!("Numeric value for {}", &n[..n.len()-4])
            ),
            n if n.ends_with("_bool") => (
                crate::api::openapi::types::Schema::Boolean,
                format!("Boolean flag for {}", &n[..n.len()-5])
            ),
            n if n.ends_with("_date") => (
                crate::api::openapi::types::Schema::String { 
                   format: Some("date".to_string()),
                    enum_values: None
                },
                format!("Date value for {}", &n[..n.len()-5])
            ),
            _ => (
                crate::api::openapi::types::Schema::String {
                    format: None,
                    enum_values: None
                },
                format!("Path parameter: {}", name)
            )
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

    fn validate_path_parameter_types(params: &[Parameter], wit_types: &HashMap<String, String>) -> Result<(), String> {
        for param in params {
            if let Some(wit_type) = wit_types.get(&param.name) {
                let expected_schema = Self::wit_type_to_schema(wit_type);
                if !Self::schemas_compatible(&param.schema, &expected_schema) {
                    return Err(format!(
                        "Path parameter '{}' schema mismatch: expected {:?}, got {:?}",
                        param.name, expected_schema, param.schema
                    ));
                }
            }
        }
        Ok(())
    }

    fn schemas_compatible(schema1: &openapiv3::Schema, schema2: &openapiv3::Schema) -> bool {
        // TODO: Implement proper schema compatibility checking for openapiv3::Schema
        true // Temporary implementation
    }

    fn extract_query_parameters(route: &Route) -> Vec<Parameter> {
         let mut params = Vec::new();

        if route.path.contains("/workers") && route.method == HttpMethod::Get {
            params.push(
                Parameter {
                    name: "filter".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::Array {
                        items: Box::new(crate::api::openapi::types::Schema::String {
                            format: None,
                            enum_values: None
                        })
                    },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                     description: Some("Filter criteria for workers".to_string()),  // Added description
                }
            );
           params.push(
                Parameter {
                    name: "cursor".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
            params.push(
                Parameter {
                    name: "count".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::Integer { format: Some("uint64".to_string()) },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
            params.push(
                Parameter {
                    name: "precise".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::Boolean,
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
        if route.path.contains("/invoke-and-await") || route.path.contains("/invoke") {
            params.push(
                Parameter {
                    name: "function".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(true),
                    description: None,
                }
            );
        }
        if route.path.contains("/interrupt") {
             params.push(
                Parameter {
                    name: "recovery-immediately".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::Boolean,
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
        if route.path.contains("/oplog") {
            params.push(
                Parameter {
                    name: "from".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::Integer { format: Some("uint64".to_string()) },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
             params.push(
                Parameter {
                    name: "count".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::Integer { format: Some("uint64".to_string()) },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(true),
                    description: None,
                }
            );
             params.push(
                Parameter {
                    name: "cursor".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::Ref {
                        reference: "#/components/schemas/OplogCursor".to_string()
                     },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
              params.push(
                Parameter {
                    name: "query".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
         if route.path.contains("/download") {
              params.push(
                Parameter {
                    name: "version".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::Integer { format: Some("uint64".to_string()) },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
         }
        if route.path.contains("/components") && route.method == HttpMethod::Get {
              params.push(
                Parameter {
                    name: "component-name".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
        if route.path.contains("/api/definitions") && route.method == HttpMethod::Get {
             params.push(
                Parameter {
                    name: "api-definition-id".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
         if route.path.contains("/api/deployments") && route.method == HttpMethod::Get {
             params.push(
                Parameter {
                    name: "api-definition-id".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(true),
                    description: None,
                }
            );
        }

        if route.path.contains("/upload") {
            params.push(
                Parameter {
                    name: "component_type".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: crate::api::openapi::types::Schema::Ref {
                        reference: "#/components/schemas/ComponentType".to_string()
                     },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: Some(
                        "Type of the new version of the component - if not specified, the type of the previous version is used.".to_string()
                    ),
                }
            );
        }
        if route.path.contains("/plugins") && route.method == HttpMethod::Get {
              params.push(
                Parameter {
                    name: "scope".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::Ref {
                        reference: "#/components/schemas/DefaultPluginScope".to_string()
                     },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
         }
         if route.path.contains("/activate-plugin") || route.path.contains("/deactivate-plugin"){
             params.push(
                Parameter {
                    name: "plugin-installation-id".to_string(),
                    r#in: ParameterLocation::Query,
                     schema: crate::api::openapi::types::Schema::String { format: Some("uuid".to_string()), enum_values: None },
                    style: Some("form".to_string()),
                    explode: Some(true),
                    required: Some(true),
                    description: None,
                }
            );
         }
         params
    }


   fn extract_header_parameters(route: &Route) -> Vec<Parameter> {
        let mut params = Vec::new();
        if route.path.contains("/invoke-and-await") || route.path.contains("/invoke") {
            params.push(
                Parameter {
                    name: "Idempotency-Key".to_string(),
                    r#in: ParameterLocation::Header,
                    schema: crate::api::openapi::types::Schema::String { format: None, enum_values: None },
                    style: Some("simple".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
        params
    }

    fn convert_schema_to_openapi(schema: &crate::api::openapi::types::Schema) -> openapiv3::Schema {
        match schema {
            crate::api::openapi::types::Schema::String { format, enum_values } => {
                openapiv3::Schema::String(openapiv3::StringType {
                    format: format.clone(),
                    enum_values: enum_values.clone(),
                    ..Default::default()
                })
            },
            crate::api::openapi::types::Schema::Integer { format } => {
                openapiv3::Schema::Integer(openapiv3::IntegerType {
                    format: format.clone(),
                    ..Default::default()
                })
            },
            crate::api::openapi::types::Schema::Boolean => {
                openapiv3::Schema::Boolean(openapiv3::BooleanType::default())
            },
            crate::api::openapi::types::Schema::Array { items } => {
                openapiv3::Schema::Array(openapiv3::ArrayType {
                    items: Box::new(Self::convert_schema_to_openapi(items)),
                    ..Default::default()
                })
            },
            crate::api::openapi::types::Schema::Ref { reference } => {
                openapiv3::Schema::Reference {
                    reference: reference.clone()
                }
            },
            crate::api::openapi::types::Schema::Object { properties, required, additional_properties } => {
                openapiv3::Schema::Object(openapiv3::ObjectType {
                    properties: properties.iter().map(|(k, v)| {
                        (k.clone(), Self::convert_schema_to_openapi(v))
                    }).collect(),
                    required: required.clone(),
                    additional_properties: additional_properties.clone(),
                    ..Default::default()
                })
            },
            crate::api::openapi::types::Schema::Number => {
                openapiv3::Schema::Number(openapiv3::NumberType::default())
            }
        }
    }

    fn create_header() -> Header {
        Header {
            required: false,
            format: None,
            deprecated: Some(false),
            style: Some(HeaderStyle::Simple),
            description: None,
            example: None,
            examples: Default::default(),
            extensions: Default::default(),
        }
    }

    fn create_media_type(schema: openapiv3::Schema) -> MediaType {
        MediaType {
            schema: Some(ReferenceOr::Item(schema)),
            example: None,
            examples: Default::default(),
            encoding: Default::default(),
            extensions: Default::default(),
        }
    }

    fn create_cors_headers(_cors_allowed_origins: &str) -> IndexMap<String, ReferenceOr<Header>> {
        let mut headers = IndexMap::new();
        let cors_header = Self::create_header();
        
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

    fn create_request_body(route: &Route) -> Option<OpenApiRequestBody> {
        match &route.binding {
            BindingType::Default { input_type, .. } => {
                let schema = Self::convert_schema_to_openapi(&Self::wit_type_to_schema(input_type));
                let mut content = indexmap::IndexMap::new();
                
                content.insert(
                    "application/json; charset=utf-8".to_string(),
                    Self::create_media_type(schema)
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
            responses: IndexMap::new(),
            extensions: Default::default(),
        };

        // Success response
        let response_schema = Self::get_response_schema(route);
        let content = if route.path.ends_with("/file-contents/{file_name}") && route.method == HttpMethod::Get {
            let mut map = IndexMap::new();
            map.insert(
                "application/octet-stream".to_string(),
                Self::create_media_type(response_schema)
            );
            map
        } else {
            let mut map = IndexMap::new();
            map.insert(
                "application/json; charset=utf-8".to_string(),
                Self::create_media_type(response_schema)
            );
            map
        };

        responses.responses.insert(
            "200".to_string(),
           Response {
                description: String::new(),
                content: content                ,
                headers: Self::create_cors_headers("*"),
                extensions: Default::default(),
                links: Default::default(),
           }
       );

       // Standard error responses
       Self::add_error_responses(&mut responses.responses);

       responses
   }

   fn add_error_responses(responses: &mut indexmap::IndexMap<String, Response>) {
       let error_codes = ["400", "401", "403", "404", "409", "500"];
         let error_schemas = [
           "#/components/schemas/ErrorsBody",
           "#/components/schemas/ErrorBody",
           "#/components/schemas/ErrorBody",
           "#/components/schemas/ErrorBody",
           "#/components/schemas/ErrorBody",
           "#/components/schemas/GolemErrorBody"
       ];
       for (code, schema) in error_codes.iter().zip(error_schemas.iter()) {
           responses.insert(
               code.to_string(),
              Response {
                   description: String::new(),
                   content: Some(IndexMap::from([(
                       "application/json; charset=utf-8".to_string(),
                       OpenApiMediaType {
                           schema: Some(Schema::Ref {
                               reference: schema.to_string()
                           }),
                           example: None,
                          encoding: Default::default(),
                          examples: Default::default(),
                           extensions: Default::default()
                       }
                   )])),
                    headers: Self::create_cors_headers("*"),
                   extensions: Default::default(),
                   links: Default::default()
               }
           );
       }
   }


   fn create_components(routes: &[Route]) -> Components {
       let mut components = Components {
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
       };

       // Add standard error schemas
       components.schemas.insert(
           "ErrorsBody".to_string(),
           ReferenceOr::Item(Schema::Object {
               properties: HashMap::from([
                   ("errors".to_string(), Schema::Array {
                       items: Box::new(Schema::String {
                           format: None,
                           enum_values: None
                       })
                   })
               ]),
               required: Some(vec!["errors".to_string()]),
               additional_properties: None,
           })
       );

       components.schemas.insert(
           "ErrorBody".to_string(),
           ReferenceOr::Item(Schema::Object {
               properties: HashMap::from([
                   ("error".to_string(), Schema::String {
                       format: None,
                       enum_values: None
                   })
               ]),
               required: Some(vec!["error".to_string()]),
               additional_properties: None,
           })
       );

       components.schemas.insert(
           "GolemErrorBody".to_string(),
           ReferenceOr::Item(Schema::Object {
               properties: HashMap::from([
                   ("golemError".to_string(), Schema::Ref {
                       reference: "#/components/schemas/GolemError".to_string()
                   })
               ]),
               required: Some(vec!["golemError".to_string()]),
               additional_properties: None,
           })
       );

      // Add WorkersMetadataResponse
       components.schemas.insert(
           "WorkersMetadataResponse".to_string(),
           ReferenceOr::Item(Schema::Object {
               properties: HashMap::from([
                   ("workers".to_string(), Schema::Array {
                       items: Box::new(Schema::Ref {
                           reference: "#/components/schemas/WorkerMetadata".to_string()
                       })
                   }),
                   ("cursor".to_string(), Schema::String {  // Match yaml exactly
                       format: None,
                       enum_values: None
                   })
               ]),
               required: Some(vec!["workers".to_string()]),
               additional_properties: None
           })
       );
        components.schemas.insert(
           "HttpApiDefinitionRequest".to_string(),
            ReferenceOr::Item(Schema::Object {
                 properties: HashMap::from([
                   ("id".to_string(), Schema::String { format: None, enum_values: None }),
                   ("version".to_string(), Schema::String { format: None, enum_values: None }),
                   ("security".to_string(), Schema::Array { items: Box::new(Schema::String { format: None, enum_values: None }) }),
                    ("routes".to_string(), Schema::Array {
                       items: Box::new(Schema::Ref {
                           reference: "#/components/schemas/RouteRequestData".to_string()
                       })
                    }),
                     ("draft".to_string(), Schema::Boolean)
                ]),
                required: Some(vec![
                   "id".to_string(),
                   "version".to_string(),
                   "routes".to_string(),
                   "draft".to_string()
               ]),
                additional_properties: None
           })
       );
        components.schemas.insert(
           "HttpApiDefinitionResponseData".to_string(),
            ReferenceOr::Item(Schema::Object {
                 properties: HashMap::from([
                   ("id".to_string(), Schema::String { format: None, enum_values: None }),
                   ("version".to_string(), Schema::String { format: None, enum_values: None }),
                    ("routes".to_string(), Schema::Array {
                       items: Box::new(Schema::Ref {
                           reference: "#/components/schemas/RouteResponseData".to_string()
                       })
                    }),
                     ("draft".to_string(), Schema::Boolean),
                   ("createdAt".to_string(), Schema::String { format: Some("date-time".to_string()), enum_values: None }),
                ]),
                required: Some(vec![
                   "id".to_string(),
                   "version".to_string(),
                   "routes".to_string(),
                    "draft".to_string(),
               ]),
                additional_properties: None
           })
       );
       // Add other schemas if necessary
       let mut type_set = std::collections::HashSet::new();
       for route in routes {
           if let BindingType::Default { input_type, output_type, .. } = &route.binding {
               Self::extract_custom_types(input_type, &mut type_set);
               Self::extract_custom_types(output_type, &mut type_set);
           }
       }

       for type_name in type_set {
           if !type_name.starts_with("record{") && !type_name.starts_with("list<")
               && type_name != "binary" && type_name != "string" && type_name != "i32" 
               && type_name != "i64" && type_name != "f32" && type_name != "f64" 
               && type_name != "bool" {
               components.schemas.insert(
                   type_name.clone(),
                   ReferenceOr::Item(Schema::Object {
                       properties: Self::parse_record_fields(&format!("record{{{}}}", type_name)),
                       required: None,
                       additional_properties: None,
                   })
               );
           }
       }

       // Add security schemes
       components.security_schemes.insert(
           "bearerAuth".to_string(),
           ReferenceOr::Item(SecurityScheme::HTTP {
               scheme: "bearer".to_string(),
               bearer_format: Some("JWT".to_string()),
               description: Some("JWT Authorization header".to_string()),
               extensions: Default::default()
           })
       );

       // Add common parameters
       components.parameters.insert(
           "filter".to_string(),
           ReferenceOr::Item(openapiv3::Parameter::Query {
               parameter_data: openapiv3::ParameterData {
                   name: "filter".to_string(),
                   description: Some("Filter criteria".to_string()),
                   required: false,
                   deprecated: Some(false),
                   format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(Schema::Array {
                       items: Box::new(Schema::String {
                           format: None,
                           enum_values: None
                       })
                   })),
                   example: None,
                   examples: Default::default(),
                   extensions: Default::default(),
               },
               style: openapiv3::QueryStyle::Form,
               allow_reserved: false,
               allow_empty_value: Some(false),
           })
       );

       components
   }

   fn create_common_parameters() -> HashMap<String, Parameter> {
       let mut params = HashMap::new();
        // Add the 'filter' parameter as requested
       params.insert(
           "filter".to_string(),
           Parameter {
               name: "filter".to_string(),
               r#in: ParameterLocation::Query,
                schema: crate::api::openapi::types::Schema::Array {
                   items: Box::new(crate::api::openapi::types::Schema::String {
                       format: None,
                       enum_values: None
                   })
               },
               style: Some("form".to_string()),
               explode: Some(true),
               required: Some(false),
                description: Some("Filter criteria".to_string()),
           }
       );
      // Add other common parameters similarly if needed
       // e.g. cursor, count, precise, etc. matching the YAML.

       params
   }


   fn wit_type_to_schema(wit_type: &str) -> crate::api::openapi::types::Schema {
       match wit_type {
           "string" => crate::api::openapi::types::Schema::String { format: None, enum_values: None },
           "i32" | "i64" => crate::api::openapi::types::Schema::Integer { format: None },
           "f32" | "f64" => crate::api::openapi::types::Schema::Number,
           "bool" => crate::api::openapi::types::Schema::Boolean,
           t if t.starts_with("list<") => {
                let inner_type = &t[5..t.len()-1];
                 crate::api::openapi::types::Schema::Array {
                   items: Box::new(Self::wit_type_to_schema(inner_type)),
               }
           },
            t if t.starts_with("record{") => {
               crate::api::openapi::types::Schema::Object {
                   properties: Self::parse_record_fields(t),
                   required: None,
                   additional_properties: None,
               }
           },
           _ => crate::api::openapi::types::Schema::Ref {
               reference: format!("#/components/schemas/{}", wit_type),
           },
       }
   }


   fn parse_record_fields(record_type: &str) -> HashMap<String, crate::api::openapi::types::Schema> {
         let mut properties = HashMap::new();
        if let Some(fields_str) = record_type
           .strip_prefix("record{")
           .and_then(|s| s.strip_suffix("}"))
       {
           for field in fields_str.split(',').map(str::trim) {
               if let Some((name, type_str)) = field.split_once(':') {
                   let name = name.trim().to_string();
                   let type_str = type_str.trim();
                  properties.insert(name, Self::wit_type_to_schema(type_str));
               }
           }
       }
       properties
   }

   fn collect_common_schemas(routes: &[Route], schemas: &mut HashMap<String, Schema>) {
       let mut type_set = std::collections::HashSet::new();

        for route in routes {
           match &route.binding {
               BindingType::Default { input_type, output_type, .. } => {
                   Self::extract_custom_types(input_type, &mut type_set);
                    Self::extract_custom_types(output_type, &mut type_set);
               }
               _ => {}
           }
       }
       for type_name in type_set {
           if !type_name.starts_with("record{") && !type_name.starts_with("list<")
               && type_name != "binary" && type_name != "string" && type_name != "i32" && type_name != "i64"
               && type_name != "f32" && type_name != "f64" && type_name != "bool" {
                schemas.insert(
                   type_name.clone(),
                   Schema::Object {
                        properties: Self::parse_record_fields(&format!("record{{{}}}", type_name)),
                       required: None,
                       additional_properties: None,
                   }
               );
           }
       }
   }

   fn extract_custom_types(wit_type: &str, type_set: &mut std::collections::HashSet<String>) {
       match wit_type {
           "string" | "i32" | "i64" | "f32" | "f64" | "bool" | "binary" => {} ,
           t if t.starts_with("list<") => {
               let inner_type = &t[5..t.len()-1];
               Self::extract_custom_types(inner_type, type_set);
           },
           t if t.starts_with("record{") => {
               if let Some(fields_str) = t.strip_prefix("record{").and_then(|s| s.strip_suffix("}")) {
                   for field in fields_str.split(',').map(str::trim) {
                       if let Some((_, type_str)) = field.split_once(':') {
                           Self::extract_custom_types(type_str.trim(), type_set);
                       }
                   }
               }
           },
           t => {
               type_set.insert(t.to_string());
           }
       }
   }

   fn get_response_schema(route: &Route) -> Schema {
       match &route.binding {
           BindingType::Default { output_type, .. } => {
               if (output_type == "binary") {
                   Schema::String {
                       format: Some("binary".to_string()),
                       enum_values: None,
                   }
               } else  {
                   Schema::Ref {
                       reference: format!("#/components/schemas/{}",
                           Self::get_response_type_name(route))
                   }
               }
           },
           BindingType::FileServer { .. } => Schema::String {
               format: Some("binary".to_string()),
               enum_values: None,
           },
           BindingType::SwaggerUI { .. } => Schema::String {
               format: Some("html".to_string()),
               enum_values: None,
           },
           BindingType::Http => Schema::Ref {
               reference: format!("#/components/schemas/{}",
                   Self::get_response_type_name(route))
           },
           BindingType::Worker => Schema::Ref {
               reference: format!("#/components/schemas/{}",
                   Self::get_response_type_name(route))
           },
           BindingType::Proxy => Schema::Ref {
               reference: format!("#/components/schemas/{}",
                   Self::get_response_type_name(route))
           },
       }
   }


   fn get_response_type_name(route: &Route) -> String {
       if route.path.ends_with("/workers") && route.method == HttpMethod::Get {
           "WorkersMetadataResponse".to_string()
       } else if route.path.ends_with("/complete") && route.method == HttpMethod::Post {
           "boolean".to_string()
       } else {
           match &route.binding {
               BindingType::Default { output_type, .. } => output_type.clone(),
               BindingType::FileServer { .. } => "binary".to_string(),
               BindingType::SwaggerUI { .. } => "html".to_string(),
               BindingType::Http => "HttpResponse".to_string(),
               BindingType::Worker => "WorkerResponse".to_string(),
               BindingType::Proxy => "ProxyResponse".to_string(),
           }
       }
   }
}

#[cfg(test)]
mod tests {
   use super::*;

   #[test]
   fn test_path_parameter_validation() {
       assert!(OpenAPIConverter::validate_path_parameter("user_id"));
       assert!(OpenAPIConverter::validate_path_parameter("count123"));
       assert!(!OpenAPIConverter::validate_path_parameter("_hidden"));
       assert!(!OpenAPIConverter::validate_path_parameter("invalid-name"));
       assert!(!OpenAPIConverter::validate_path_parameter(""));
   }

   #[test]
   fn test_catch_all_parameter_validation() {
       assert!(OpenAPIConverter::validate_catch_all_parameter("path"));
       assert!(OpenAPIConverter::validate_catch_all_parameter("file_path"));
       assert!(!OpenAPIConverter::validate_catch_all_parameter("invalid__name"));
       assert!(!OpenAPIConverter::validate_catch_all_parameter("_path"));
   }

   #[test]
   fn test_parameter_type_inference() {
       let (schema, desc) = OpenAPIConverter::infer_parameter_type("user_id");
       assert!(matches!(schema, crate::api::openapi::types::Schema::String { format: Some(f), .. } if f == "uuid"));
       assert!(desc.contains("identifier"));

       let (schema, desc) = OpenAPIConverter::infer_parameter_type("item_count");
       assert!(matches!(schema, crate::api::openapi::types::Schema::Integer { .. }));
       assert!(desc.contains("Numeric"));
   }
}