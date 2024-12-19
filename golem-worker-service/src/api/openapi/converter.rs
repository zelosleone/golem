use crate::api::definition::types::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::definition::patterns::{AllPathPatterns, PathPattern};
use openapiv3::{
    OpenAPI as OpenAPISpec, Info, Paths, PathItem, Operation, 
    Schema, MediaType, RequestBody, SecurityScheme,
    Components, ReferenceOr, Header, Responses,
};
use crate::api::openapi::types::{Parameter, ParameterLocation};
use indexmap::IndexMap;
use std::collections::HashMap;
use heck::ToSnakeCase;
use openapiv3::PathItem;
use openapiv3::Response;
use openapiv3::Parameter as OpenApiParameter;
use openapiv3::RequestBody as OpenApiRequestBody;
use openapiv3::Schema as OpenApiSchema;
use openapiv3::MediaType as OpenApiMediaType;

pub struct OpenAPIConverter;

impl OpenAPIConverter {
    pub fn convert_to_spec(api: &ApiDefinition) -> OpenAPISpec {
        OpenAPISpec {
            openapi: "3.0.0".to_string(),
            info: Info {
                title: "Golem API".to_string(),
                version: "1.0".to_string(),
                description: None,
                contact: None,
                license: None,
                extensions: Default::default(),
            },
            paths: Self::convert_paths(&api.routes),
            components: Some(Self::create_components(&api.routes)),
             security: None,
             extensions: Default::default(),
             servers: Default::default(),
             external_docs: None,
        }
    }

    pub fn convert_paths(routes: &[Route]) -> Paths {
       let mut paths_map: IndexMap<String, ReferenceOr<PathItem>> = IndexMap::new();

        for route in routes {
            let operation = Self::generate_operation(route);

            let path_item = PathItem {
                get: if route.method == HttpMethod::Get { Some(operation.clone()) } else { None },
                post: if route.method == HttpMethod::Post { Some(operation.clone()) } else { None },
                put: if route.method == HttpMethod::Put { Some(operation.clone()) } else { None },
                delete: if route.method == HttpMethod::Delete { Some(operation.clone()) } else { None },
                options: Some(Operation {
                    responses: {
                        let mut map = IndexMap::new();
                         map.insert("200".to_string(), Response {
                            description: String::new(),
                            content: Default::default(),
                            headers: Self::create_cors_headers("*"),
                            extensions: Default::default(),
                            links: Default::default(),
                        });
                        map
                    },
                       callbacks: Default::default(),
                      deprecated: false,
                      extensions: Default::default(),
                       security: Default::default(),
                        servers: Default::default(),
                        ..operation
                }),
                  description: None,
                  extensions: Default::default(),
                  parameters: Default::default(),
                  head: None,
                   trace: None,
                   patch: None,
                   servers: Default::default()
            };

            paths_map.insert(route.path.clone(), ReferenceOr::Item(path_item));
        }


        Paths {
            paths: paths_map,
             extensions: Default::default()
        }
    }

    fn generate_operation(route: &Route) -> Operation {
        match &route.binding {
            BindingType::Default { .. } => {
                Operation {
                    summary: Some(route.description.clone()),
                    description: None,
                    operation_id: Some(format!("{}_{}",
                        route.template_name.to_snake_case(),
                        route.method.to_string().to_lowercase())),
                   parameters: {
                        let mut params: Vec<ReferenceOr<OpenApiParameter>> = Self::extract_path_parameters(&route.path)
                            .unwrap_or_default()
                            .into_iter()
                            .map(ReferenceOr::Item)
                            .collect();
                       params.extend(Self::extract_query_parameters(route).into_iter().map(ReferenceOr::Item));
                        params.extend(Self::extract_header_parameters(route).into_iter().map(ReferenceOr::Item));
                       if params.is_empty() { None } else { Some(params) }
                    },
                    request_body: Self::create_request_body(route).map(ReferenceOr::Item),
                   responses: {
                        let mut map = Self::create_responses(route);
                        // Add CORS headers to all responses
                        for response in map.values_mut() {
                            response.headers = Self::create_cors_headers("*");
                        }
                         map
                    },
                    security: None,
                    tags: Some(vec![route.template_name.clone()]),
                    callbacks: Default::default(),
                    deprecated: false,
                     extensions: Default::default(),
                     servers: Default::default()
                }
            },
            BindingType::FileServer { .. } => {
                Operation {
                    summary: Some(route.description.clone()),
                    description: None,
                    operation_id: Some(format!("{}_{}",
                        route.template_name.to_snake_case(),
                        route.method.to_string().to_lowercase())),
                   parameters: {
                        let mut params: Vec<ReferenceOr<OpenApiParameter>> = Self::extract_path_parameters(&route.path)
                            .unwrap_or_default()
                            .into_iter()
                            .map(ReferenceOr::Item)
                            .collect();
                       params.extend(Self::extract_query_parameters(route).into_iter().map(ReferenceOr::Item));
                        params.extend(Self::extract_header_parameters(route).into_iter().map(ReferenceOr::Item));
                       if params.is_empty() { None } else { Some(params) }
                    },
                    request_body: Self::create_request_body(route).map(ReferenceOr::Item),
                    responses: {
                        let mut map = Self::create_responses(route);
                        // Add CORS headers to all responses
                        for response in map.values_mut() {
                            response.headers = Self::create_cors_headers("*");
                        }
                         map
                    },
                    security: None,
                    tags: Some(vec![route.template_name.clone()]),
                     callbacks: Default::default(),
                     deprecated: false,
                     extensions: Default::default(),
                     servers: Default::default()
                }
            },
            BindingType::SwaggerUI { .. } => {
                Operation {
                    summary: Some(route.description.clone()),
                    description: None,
                    operation_id: Some(format!("{}_{}",
                        route.template_name.to_snake_case(),
                        route.method.to_string().to_lowercase())),
                    parameters: {
                        let mut params: Vec<ReferenceOr<OpenApiParameter>> = Self::extract_path_parameters(&route.path)
                            .unwrap_or_default()
                            .into_iter()
                            .map(ReferenceOr::Item)
                            .collect();
                       params.extend(Self::extract_query_parameters(route).into_iter().map(ReferenceOr::Item));
                        params.extend(Self::extract_header_parameters(route).into_iter().map(ReferenceOr::Item));
                       if params.is_empty() { None } else { Some(params) }
                    },
                    request_body: Self::create_request_body(route).map(ReferenceOr::Item),
                    responses: {
                        let mut map = Self::create_responses(route);
                        // Add CORS headers to all responses
                        for response in map.values_mut() {
                            response.headers = Self::create_cors_headers("*");
                        }
                         map
                    },
                    security: None,
                    tags: Some(vec![route.template_name.clone()]),
                    callbacks: Default::default(),
                    deprecated: false,
                     extensions: Default::default(),
                     servers: Default::default()
                }
            },
            BindingType::Http => Operation {
                summary: Some(route.description.clone()),
                description: None,
                operation_id: Some(format!("{}_{}",
                    route.template_name.to_snake_case(),
                    route.method.to_string().to_lowercase())),
               parameters: {
                        let mut params: Vec<ReferenceOr<OpenApiParameter>> = Self::extract_path_parameters(&route.path)
                            .unwrap_or_default()
                            .into_iter()
                            .map(ReferenceOr::Item)
                            .collect();
                       params.extend(Self::extract_query_parameters(route).into_iter().map(ReferenceOr::Item));
                        params.extend(Self::extract_header_parameters(route).into_iter().map(ReferenceOr::Item));
                       if params.is_empty() { None } else { Some(params) }
                    },
                request_body: Self::create_request_body(route).map(ReferenceOr::Item),
                responses: {
                    let mut map = Self::create_responses(route);
                    // Add CORS headers to all responses
                    for response in map.values_mut() {
                        response.headers = Self::create_cors_headers("*");
                    }
                     map
                },
                security: None,
                tags: Some(vec![route.template_name.clone()]),
                 callbacks: Default::default(),
                 deprecated: false,
                 extensions: Default::default(),
                 servers: Default::default()
            },
            BindingType::Worker => Operation {
                summary: Some(route.description.clone()),
                description: None,
                operation_id: Some(format!("{}_{}",
                    route.template_name.to_snake_case(),
                    route.method.to_string().to_lowercase())),
               parameters: {
                        let mut params: Vec<ReferenceOr<OpenApiParameter>> = Self::extract_path_parameters(&route.path)
                            .unwrap_or_default()
                            .into_iter()
                            .map(ReferenceOr::Item)
                            .collect();
                       params.extend(Self::extract_query_parameters(route).into_iter().map(ReferenceOr::Item));
                        params.extend(Self::extract_header_parameters(route).into_iter().map(ReferenceOr::Item));
                       if params.is_empty() { None } else { Some(params) }
                    },
                request_body: Self::create_request_body(route).map(ReferenceOr::Item),
                responses: {
                    let mut map = Self::create_responses(route);
                    // Add CORS headers to all responses
                    for response in map.values_mut() {
                        response.headers = Self::create_cors_headers("*");
                    }
                    map
                },
                security: None,
                tags: Some(vec![route.template_name.clone()]),
                 callbacks: Default::default(),
                 deprecated: false,
                 extensions: Default::default(),
                 servers: Default::default()
            },
            BindingType::Proxy => Operation {
                summary: Some(route.description.clone()),
                description: None,
                operation_id: Some(format!("{}_{}",
                    route.template_name.to_snake_case(),
                    route.method.to_string().to_lowercase())),
               parameters: {
                        let mut params: Vec<ReferenceOr<OpenApiParameter>> = Self::extract_path_parameters(&route.path)
                            .unwrap_or_default()
                            .into_iter()
                            .map(ReferenceOr::Item)
                            .collect();
                       params.extend(Self::extract_query_parameters(route).into_iter().map(ReferenceOr::Item));
                        params.extend(Self::extract_header_parameters(route).into_iter().map(ReferenceOr::Item));
                       if params.is_empty() { None } else { Some(params) }
                    },
                request_body: Self::create_request_body(route).map(ReferenceOr::Item),
                 responses: {
                    let mut map = Self::create_responses(route);
                    // Add CORS headers to all responses
                    for response in map.values_mut() {
                        response.headers = Self::create_cors_headers("*");
                    }
                    map
                },
                security: None,
                tags: Some(vec![route.template_name.clone()]),
                callbacks: Default::default(),
                deprecated: false,
                 extensions: Default::default(),
                 servers: Default::default()
            },
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
                        schema: Schema::Array {
                            items: Box::new(Schema::String {
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

    fn infer_parameter_type(name: &str) -> (Schema, String) {
        match name {
            n if n.ends_with("_id") => (
                Schema::String { 
                    format: Some("uuid".to_string()),
                    enum_values: None
                },
                format!("Unique identifier for {}", &n[..n.len()-3])
            ),
            n if n.ends_with("_num") || n.ends_with("_count") => (
                Schema::Integer { 
                     format: Some("int64".to_string()) 
                 },
                format!("Numeric value for {}", &n[..n.len()-4])
            ),
            n if n.ends_with("_bool") => (
                Schema::Boolean,
                format!("Boolean flag for {}", &n[..n.len()-5])
            ),
            n if n.ends_with("_date") => (
                Schema::String { 
                   format: Some("date".to_string()),
                    enum_values: None
                },
                format!("Date value for {}", &n[..n.len()-5])
            ),
            _ => (
                Schema::String {
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
            if let Some(wit_type) = wit_types.get(¶m.name) {
                let expected_schema = Self::wit_type_to_schema(wit_type);
                if !Self::schemas_compatible(¶m.schema, &expected_schema) {
                    return Err(format!(
                        "Path parameter '{}' schema mismatch: expected {:?}, got {:?}",
                        param.name, expected_schema, param.schema
                    ));
                }
            }
        }
        Ok(())
    }

    fn schemas_compatible(schema1: &Schema, schema2: &Schema) -> bool {
        match (schema1, schema2) {
            (Schema::String { .. }, Schema::String { .. }) => true,
            (Schema::Integer { .. }, Schema::Integer { .. }) => true,
            (Schema::Boolean, Schema::Boolean) => true,
            (Schema::Array { items: i1 }, Schema::Array { items: i2 }) => 
                Self::schemas_compatible(i1, i2),
            _ => false
        }
    }

    fn extract_query_parameters(route: &Route) -> Vec<Parameter> {
         let mut params = Vec::new();

        if route.path.contains("/workers") && route.method == HttpMethod::Get {
            params.push(
                Parameter {
                    name: "filter".to_string(),
                    r#in: ParameterLocation::Query,
                    schema: Schema::Array {
                        items: Box::new(Schema::String {
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
                    schema: Schema::String { format: None, enum_values: None },
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
                    schema: Schema::Integer { format: Some("uint64".to_string()) },
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
                    schema: Schema::Boolean,
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
                    schema: Schema::String { format: None, enum_values: None },
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
                     schema: Schema::Boolean,
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
                    schema: Schema::Integer { format: Some("uint64".to_string()) },
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
                     schema: Schema::Integer { format: Some("uint64".to_string()) },
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
                     schema: Schema::Ref {
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
                    schema: Schema::String { format: None, enum_values: None },
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
                     schema: Schema::Integer { format: Some("uint64".to_string()) },
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
                     schema: Schema::String { format: None, enum_values: None },
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
                     schema: Schema::String { format: None, enum_values: None },
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
                     schema: Schema::String { format: None, enum_values: None },
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
                    schema: Schema::Ref {
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
                     schema: Schema::Ref {
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
                     schema: Schema::String { format: Some("uuid".to_string()), enum_values: None },
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
                    schema: Schema::String { format: None, enum_values: None },
                    style: Some("simple".to_string()),
                    explode: Some(true),
                    required: Some(false),
                    description: None,
                }
            );
        }
        params
    }

    fn create_request_body(route: &Route) -> Option<OpenApiRequestBody> {
         match &route.binding {
            BindingType::Default { input_type, .. } => {
                 let schema = Self::wit_type_to_schema(input_type);

                // Check if route allows x-yaml
                let allows_yaml = route.path.starts_with("/v1/api/definitions")
                    && (route.method == HttpMethod::Put || route.method == HttpMethod::Post);

                 let mut content = IndexMap::new();

                // JSON content type
                 if route.path.contains("/components") && route.method == HttpMethod::Post {

                    let mut properties = HashMap::new();

                    properties.insert("name".to_string(), Schema::String { format: None, enum_values: None });

                    properties.insert("componentType".to_string(), Schema::Ref {
                        reference: "#/components/schemas/ComponentType".to_string()
                    });


                    properties.insert("component".to_string(), Schema::String {
                        format: Some("binary".to_string()),
                        enum_values: None
                    });
                    properties.insert("filesPermissions".to_string(), Schema::Ref {
                        reference: "#/components/schemas/ComponentFilePathWithPermissionsList".to_string()
                    });
                      properties.insert("files".to_string(), Schema::String {
                        format: Some("binary".to_string()),
                        enum_values: None
                    });


                    content.insert(
                        "multipart/form-data".to_string(),
                        OpenApiMediaType {
                            schema: Some(Schema::Object {
                                properties,
                                required: Some(vec!["name".to_string(), "component".to_string()]),
                                additional_properties: None,
                            }),
                            example: None,
                            encoding: Default::default(),
                            examples: Default::default(),
                            extensions: Default::default()
                         },
                    );
                     Some(OpenApiRequestBody {
                        description: None,
                        content,
                        required: true,
                        extensions: Default::default()
                    })
                 } else if route.path.contains("/updates") && route.method == HttpMethod::Post {
                       let mut properties = HashMap::new();
                      properties.insert("componentType".to_string(), Schema::Ref {
                        reference: "#/components/schemas/ComponentType".to_string()
                    });
                      properties.insert("component".to_string(), Schema::String {
                        format: Some("binary".to_string()),
                        enum_values: None
                    });
                        properties.insert("filesPermissions".to_string(), Schema::Ref {
                        reference: "#/components/schemas/ComponentFilePathWithPermissionsList".to_string()
                    });
                      properties.insert("files".to_string(), Schema::String {
                        format: Some("binary".to_string()),
                        enum_values: None
                    });

                     content.insert(
                        "multipart/form-data".to_string(),
                        OpenApiMediaType {
                           schema: Some(Schema::Object {
                                properties,
                                required: Some(vec!["component".to_string()]),
                                additional_properties: None,
                            }),
                            example: None,
                              encoding: Default::default(),
                            examples: Default::default(),
                             extensions: Default::default()
                        },
                    );
                    Some(OpenApiRequestBody {
                        description: None,
                        content,
                        required: true,
                         extensions: Default::default()
                    })
                 } else if route.path.contains("/upload") && route.method == HttpMethod::Put {
                      content.insert(
                            "application/octet-stream".to_string(),
                            OpenApiMediaType {
                                 schema: Some(Schema::String {
                                        format: Some("binary".to_string()),
                                        enum_values: None
                                 }),
                                example: None,
                                  encoding: Default::default(),
                                examples: Default::default(),
                                 extensions: Default::default()
                            },
                        );
                         Some(OpenApiRequestBody {
                            description: None,
                            content,
                            required: true,
                             extensions: Default::default()
                        })
                 } else {
                    content.insert(
                        "application/json; charset=utf-8".to_string(),
                        OpenApiMediaType {
                            schema: Some(schema.clone()),
                            example: None,
                             encoding: Default::default(),
                            examples: Default::default(),
                             extensions: Default::default()
                        },
                    );
                    if allows_yaml {
                        content.insert(
                            "application/x-yaml".to_string(),
                            OpenApiMediaType {
                                schema: Some(schema),
                                example: None,
                                encoding: Default::default(),
                                examples: Default::default(),
                                 extensions: Default::default()
                            },
                        );
                    }
                      Some(OpenApiRequestBody {
                        description: None,
                        content,
                        required: true,
                         extensions: Default::default()
                    })
                }

            },
             _ => None,
        }
    }

    fn create_cors_headers(cors_allowed_origins: &str) -> IndexMap<String, ReferenceOr<Header>> {
        let mut headers = IndexMap::new();
        headers.insert(
            "Access-Control-Allow-Origin".to_string(),
            ReferenceOr::Item(Header {
                description: None,
                schema: None,
                extensions: Default::default(),
                example: None,
                examples: Default::default(),
                deprecated: false,
                 style: None,
                
            }),
        );
        headers.insert(
            "Access-Control-Allow-Methods".to_string(),
            ReferenceOr::Item(Header {
                description: None,
                schema: None,
                 extensions: Default::default(),
                 example: None,
                 examples: Default::default(),
                 deprecated: false,
                 style: None,
            }),
        );
        headers.insert(
            "Access-Control-Allow-Headers".to_string(),
           ReferenceOr::Item(Header {
               description: None,
                schema: None,
                extensions: Default::default(),
               example: None,
               examples: Default::default(),
               deprecated: false,
               style: None,
            }),
        );
         headers.insert(
            "Access-Control-Max-Age".to_string(),
           ReferenceOr::Item(Header {
                description: None,
                schema: None,
                 extensions: Default::default(),
                example: None,
                examples: Default::default(),
                deprecated: false,
                style: None,
            }),
        );
       headers.insert(
            "Access-Control-Expose-Headers".to_string(),
           ReferenceOr::Item(Header {
                description: None,
                schema: None,
                extensions: Default::default(),
                example: None,
               examples: Default::default(),
               deprecated: false,
               style: None,
            }),
        );
        headers
    }

    fn create_responses(route: &Route) -> IndexMap<String, Response> {
        let mut responses: IndexMap<String, Response> = IndexMap::new();

        // Success response
        let response_schema = Self::get_response_schema(route);
         let content = if route.path.ends_with("/file-contents/{file_name}") && route.method == HttpMethod::Get {
                Some(IndexMap::from([(
                     "application/octet-stream".to_string(),
                       OpenApiMediaType {
                            schema: Some(response_schema),
                            example: None,
                            encoding: Default::default(),
                            examples: Default::default(),
                            extensions: Default::default()
                        }
                )]))
        } else {
             Some(IndexMap::from([(
                "application/json; charset=utf-8".to_string(),
                OpenApiMediaType {
                    schema: Some(response_schema),
                    example: None,
                    encoding: Default::default(),
                    examples: Default::default(),
                    extensions: Default::default()
                }
            )]))
        };

        responses.insert(
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
       Self::add_error_responses(&mut responses);

       responses
   }

   fn add_error_responses(responses: &mut IndexMap<String, Response>) {
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
           ReferenceOr::Item(OpenApiParameter {
               name: "filter".to_string(),
               location: openapiv3::ParameterLocation::Query,
               schema: Some(Schema::Array {
                   items: Box::new(Schema::String {
                       format: None,
                       enum_values: None
                   })
               }),
               style: Some(openapiv3::ParameterStyle::Form),
               explode: Some(true),
               required: false,
               description: Some("Filter criteria".to_string()),
               deprecated: false,
               allow_empty_value: false,
               allow_reserved: false,
                extensions: Default::default(),
                example: None,
                 examples: Default::default(),
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
                schema: Schema::Array {
                   items: Box::new(Schema::String {
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


   fn wit_type_to_schema(wit_type: &str) -> Schema {
       match wit_type {
           "string" => Schema::String { format: None, enum_values: None },
           "i32" | "i64" => Schema::Integer { format: None },
           "f32" | "f64" => Schema::Number { format: None },
           "bool" => Schema::Boolean,
           t if t.starts_with("list<") => {
                let inner_type = &t[5..t.len()-1];
                 Schema::Array {
                   items: Box::new(Self::wit_type_to_schema(inner_type)),
               }
           },
            t if t.starts_with("record{") => {
               Schema::Object {
                   properties: Self::parse_record_fields(t),
                   required: None,
                   additional_properties: None,
               }
           },
           _ => Schema::Ref {
               reference: format!("#/components/schemas/{}", wit_type),
           },
       }
   }


   fn parse_record_fields(record_type: &str) -> HashMap<String, Schema> {
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
               if output_type == "binary" {
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
       assert!(matches!(schema, Schema::String { format: Some(f), .. } if f == "uuid"));
       assert!(desc.contains("identifier"));

       let (schema, desc) = OpenAPIConverter::infer_parameter_type("item_count");
       assert!(matches!(schema, Schema::Integer { .. }));
       assert!(desc.contains("Numeric"));
   }
}