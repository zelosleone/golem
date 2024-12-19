use indexmap::IndexMap;
use openapiv3::{
    OpenAPI, PathItem, Operation, Parameter, ReferenceOr,
    ParameterData, ParameterSchemaOrContent, Schema,
    Response, Responses, RequestBody, MediaType,
    Components, Type, SchemaKind, StatusCode,
    ParameterStyle,
};
use golem_worker_service_base::{
    gateway_api_definition::http::{CompiledHttpApiDefinition, CompiledRoute},
    gateway_binding::GatewayBindingCompiled,
};
use crate::api::openapi::types::{create_default_openapi_spec, convert_security_scheme};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde_json;

#[derive(Debug)]
pub struct TypeMapper {
    type_cache: Arc<RwLock<HashMap<String, ReferenceOr<Schema>>>>,
}

impl TypeMapper {
    pub fn new() -> Self {
        Self {
            type_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn map_wit_type(&self, wit_type: &AnalysedType) -> Result<ReferenceOr<Schema>, OpenAPIError> {
        use AnalysedType::*;
        
        match wit_type {
            Str(_) => Ok(ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::String(Default::default())),
            })),
            I32(_) => Ok(ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Integer(Default::default())),
                format: Some("int32".to_string()),
            })),
            Record(record) => {
                let mut cache = self.type_cache.write().expect("Lock poisoned");
                if let Some(cached) = cache.get(&record.name) {
                    return Ok(cached.clone());
                }

                let mut properties = IndexMap::new();
                for field in &record.fields {
                    properties.insert(
                        field.name.clone(),
                        self.map_wit_type(&field.ty)?
                    );
                }

                let schema = Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(Type::Object(openapiv3::ObjectType {
                        properties,
                        required: record.fields.iter()
                            .filter(|f| !f.optional)
                            .map(|f| f.name.clone())
                            .collect(),
                        ..Default::default()
                    })),
                };

                let reference = format!("#/components/schemas/{}", record.name);
                cache.insert(record.name.clone(), ReferenceOr::Reference { reference: reference.clone() });
                
                Ok(ReferenceOr::Reference { reference })
            },
            List(inner) => Ok(ReferenceOr::Item(Schema {
                schema_data: Default::default(),
                schema_kind: SchemaKind::Type(Type::Array(openapiv3::ArrayType {
                    items: Box::new(self.map_wit_type(inner)?),
                    ..Default::default()
                })),
            })),
            Option(inner) => {
                let mut schema = match self.map_wit_type(inner)? {
                    ReferenceOr::Item(schema) => schema,
                    ref_schema => return Ok(ref_schema),
                };
                schema.schema_data.nullable = Some(true);
                Ok(ReferenceOr::Item(schema))
            },
            _ => Err(OpenAPIError::UnsupportedType(format!("{:?}", wit_type))),
        }
    }
}

pub struct OpenAPIConverter {
    components: Components,
    type_mapper: TypeMapper,
}

impl OpenAPIConverter {
    pub fn new() -> Self {
        Self {
            components: Components::default(),
            type_mapper: TypeMapper::new(),
        }
    }

    pub fn convert_api_definition<T>(&self, api_def: &CompiledHttpApiDefinition<T>) -> Result<OpenAPI, String> {
        let mut spec = create_default_openapi_spec();

        for route in &api_def.routes {
            let path_str = route.path.to_string();
            let path_item = self.convert_route_to_path_item(route)?;
            spec.paths.paths.insert(path_str, ReferenceOr::Item(path_item));
        }

        Ok(spec)
    }

    pub fn convert_binding_type(&self, binding: &GatewayBindingCompiled) -> Result<OpenAPI, String> {
        let mut spec = create_default_openapi_spec();

        // Add security schemes if available
        if let Some(ref schemes) = binding.security_schemes {
            let mut security_schemes = IndexMap::new();
            for scheme in schemes {
                security_schemes.insert(
                    scheme.name.clone(),
                    ReferenceOr::Item(convert_security_scheme(scheme)),
                );
            }
            if let Some(ref mut components) = spec.components {
                components.security_schemes = security_schemes;
            }
        }

        // Add CORS information if available
        if let Some(ref cors_config) = binding.cors_config {
            spec.extensions.insert(
                "x-cors".to_string(),
                serde_json::json!({
                    "allow_origins": &cors_config.allow_origins,
                    "allow_methods": &cors_config.allow_methods,
                    "allow_headers": &cors_config.allow_headers,
                    "expose_headers": &cors_config.expose_headers,
                    "max_age": cors_config.max_age,
                    "allow_credentials": cors_config.allow_credentials,
                }),
            );
        }

        Ok(spec)
    }

    fn convert_route_to_path_item(&self, route: &CompiledRoute) -> Result<PathItem, String> {
        let mut path_item = PathItem::default();
        let operation = self.create_operation_from_route(route)?;

        match route.method.to_string().as_str() {
            "GET" => path_item.get = Some(operation),
            "POST" => path_item.post = Some(operation),
            "PUT" => path_item.put = Some(operation),
            "DELETE" => path_item.delete = Some(operation),
            "PATCH" => path_item.patch = Some(operation),
            method => return Err(format!("Unsupported HTTP method: {}", method)),
        }

        Ok(path_item)
    }

    fn create_operation_from_route(&self, route: &CompiledRoute) -> Result<Operation, String> {
        let mut operation = Operation::default();

        // Set operation ID and summary
        let method = route.method.to_string().to_lowercase();
        operation.operation_id = Some(format!("{}_{}", method, route.path.to_string()));
        operation.summary = Some(format!("{} {}", route.method, route.path.to_string()));

        // Add path parameters
        let path_str = route.path.to_string();
        let path_params: Vec<&str> = path_str
            .split('/')
            .filter(|s| s.starts_with('{') && s.ends_with('}'))
            .map(|s| &s[1..s.len()-1])
            .collect();

        operation.parameters = path_params.iter()
            .map(|&param| {
                ReferenceOr::Item(Parameter::Path {
                    parameter_data: ParameterData {
                        name: param.to_string(),
                        description: None,
                        required: true,
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(
                            Schema {
                                schema_data: Default::default(),
                                schema_kind: SchemaKind::Type(Type::String(Default::default())),
                            }
                        )),
                        example: None,
                        examples: IndexMap::new(),
                        explode: None,
                        extensions: IndexMap::new(),
                    },
                    style: ParameterStyle::Label, // Update parameter style to Label
                })
            })
            .collect();

        // Add request body for POST, PUT, PATCH
        if matches!(route.method.to_string().as_str(), "POST" | "PUT" | "PATCH") {
            operation.request_body = Some(ReferenceOr::Item(RequestBody {
                description: None,
                content: {
                    let mut content = IndexMap::new();
                    content.insert(
                        "application/json".to_string(),
                        MediaType {
                            schema: Some(ReferenceOr::Item(Schema {
                                schema_data: Default::default(),
                                schema_kind: SchemaKind::Type(Type::Object(Default::default())),
                            })),
                            example: None,
                            examples: IndexMap::new(),
                            encoding: IndexMap::new(),
                            extensions: IndexMap::new(),
                        },
                    );
                    content
                },
                required: false,
                extensions: IndexMap::new(),
            }));
        }

        // Add responses
        operation.responses = Responses {
            default: None,
            responses: {
                let mut responses = IndexMap::new();
                responses.insert(
                    StatusCode::Code(200),
                    ReferenceOr::Item(Response {
                        description: "Successful operation".to_string(),
                        headers: IndexMap::new(),
                        content: {
                            let mut content = IndexMap::new();
                            content.insert(
                                "application/json".to_string(),
                                MediaType {
                                    schema: Some(ReferenceOr::Item(Schema {
                                        schema_data: Default::default(),
                                        schema_kind: SchemaKind::Type(Type::Object(Default::default())),
                                    })),
                                    example: None,
                                    examples: IndexMap::new(),
                                    encoding: IndexMap::new(),
                                    extensions: IndexMap::new(),
                                },
                            );
                            content
                        },
                        links: IndexMap::new(),
                        extensions: IndexMap::new(),
                    }),
                );
                responses
            },
            extensions: IndexMap::new(),
        };

        Ok(operation)
    }
}