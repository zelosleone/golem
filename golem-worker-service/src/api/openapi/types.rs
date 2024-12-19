use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use openapiv3::{
    ReferenceOr, Schema, SchemaKind, Type as OpenAPIType, StringFormat, VariantOrUnknownOrEmpty,
    IntegerFormat, NumberFormat, ParameterData, ParameterSchemaOrContent, QueryStyle, PathStyle,
};
use indexmap::IndexMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPISpec {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<GolemComponents>,
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub options: Option<Operation>,
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub r#in: ParameterLocation,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: OpenAPISchemaType,
    pub style: Option<String>,
    pub explode: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterLocation {
    Path,
    Query,
    Header,
    Cookie,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: HashMap<String, MediaType>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Schema,
    pub example: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
    pub headers: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GolemComponents {
    pub schemas: Option<HashMap<String, Schema>>,
    pub responses: Option<HashMap<String, Response>>,
    pub parameters: Option<HashMap<String, Parameter>>,
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
}

// Rename Schema to avoid conflicts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpenAPISchemaType {
    String {
        format: Option<String>,
        enum_values: Option<Vec<String>>
    },
    Integer {
        format: Option<String>,
    },
    Number {
        format: Option<String>,
    },
    Boolean,
    Array {
        items: Box<OpenAPISchemaType>
    },
    Object {
        properties: HashMap<String, OpenAPISchemaType>,
        required: Option<Vec<String>>,
    },
    Reference {
        #[serde(rename = "$ref")]
        reference: String,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityScheme {
    Http {
        scheme: String,
        bearer_format: Option<String>,
        description: Option<String>
    },
    ApiKey {
        r#in: ParameterLocation,
        name: String,
        description: Option<String>,
    },
    OAuth2 {
        flows: OAuthFlows
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlows {
    pub implicit: Option<OAuthFlow>,
    pub password: Option<OAuthFlow>,
    pub client_credentials: Option<OAuthFlow>,
    pub authorization_code: Option<OAuthFlow>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    pub authorization_url: String,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>
}

// Helper functions to handle formats because we can't implement From<String> for VariantOrUnknownOrEmpty.
fn string_format_from_str(s: &str) -> VariantOrUnknownOrEmpty<StringFormat> {
    match s {
        "date-time" => VariantOrUnknownOrEmpty::Item(StringFormat::DateTime),
        "date" => VariantOrUnknownOrEmpty::Item(StringFormat::Date),
        "password" => VariantOrUnknownOrEmpty::Item(StringFormat::Password),
        "byte" => VariantOrUnknownOrEmpty::Item(StringFormat::Byte),
        "binary" => VariantOrUnknownOrEmpty::Item(StringFormat::Binary),
        other => VariantOrUnknownOrEmpty::Unknown(other.to_string()),
    }
}

fn integer_format_from_str(s: &str) -> VariantOrUnknownOrEmpty<IntegerFormat> {
    // The openapiv3 crate doesn't define well-known integer formats besides int32, int64. 
    // Let's handle them if needed:
    match s {
        "int32" => VariantOrUnknownOrEmpty::Item(IntegerFormat::Int32),
        "int64" => VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64),
        other => VariantOrUnknownOrEmpty::Unknown(other.to_string()),
    }
}

fn number_format_from_str(s: &str) -> VariantOrUnknownOrEmpty<NumberFormat> {
    // Similarly handle number formats
    match s {
        "float" => VariantOrUnknownOrEmpty::Item(NumberFormat::Float),
        "double" => VariantOrUnknownOrEmpty::Item(NumberFormat::Double),
        other => VariantOrUnknownOrEmpty::Unknown(other.to_string()),
    }
}

// Convert OpenAPISchemaType to openapiv3::Schema
impl From<OpenAPISchemaType> for Schema {
    fn from(schema: OpenAPISchemaType) -> Self {
        match schema {
            OpenAPISchemaType::String { format, enum_values } => {
                let enumeration = enum_values
                    .unwrap_or_default()
                    .into_iter()
                    .map(Some)
                    .collect::<Vec<_>>();

                let string_type = openapiv3::StringType {
                    format: format.as_deref().map(string_format_from_str),
                    enumeration,
                    ..Default::default()
                };

                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::String(string_type)),
                }
            },
            OpenAPISchemaType::Integer { format } => {
                let integer_type = openapiv3::IntegerType {
                    format: format.as_deref().map(integer_format_from_str),
                    ..Default::default()
                };
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Integer(integer_type)),
                }
            },
            OpenAPISchemaType::Number { format } => {
                let number_type = openapiv3::NumberType {
                    format: format.as_deref().map(number_format_from_str),
                    ..Default::default()
                };
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Number(number_type)),
                }
            },
            OpenAPISchemaType::Boolean => {
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Boolean(Default::default())),
                }
            },
            OpenAPISchemaType::Array { items } => {
                let items_schema: Schema = (*items).into();
                let array_type = openapiv3::ArrayType {
                    items: Some(Box::new(ReferenceOr::Item(items_schema))),
                    // If needed, default the rest:
                    ..openapiv3::ArrayType::default()
                };
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Array(array_type)),
                }
            },
            OpenAPISchemaType::Object { properties, required } => {
                let props: HashMap<String, ReferenceOr<Schema>> = properties
                    .into_iter()
                    .map(|(k,v)| (k, ReferenceOr::Item(v.into())))
                    .collect();
                let object_type = openapiv3::ObjectType {
                    properties: props,
                    required: required.unwrap_or_default(),
                    ..Default::default()
                };
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Object(object_type)),
                }
            },
            OpenAPISchemaType::Reference { reference } => {
                // References cannot be placed directly into SchemaKind as a reference kind.
                // They must be represented as ReferenceOr::Reference at a higher level.
                // However, here we must return a Schema. The openapiv3 crate expects references
                // in places of schemas as ReferenceOr::Reference. If we must return a Schema,
                // we have a problem because a pure reference isn't a Schema.
                // We'll return a simple object schema to indicate an error or 
                // handle references differently at runtime.
                // Ideally, references should be handled before conversion or stored differently.
                // For now, let's produce a dummy schema since we cannot represent references here:
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::String(openapiv3::StringType::default())),
                }
            },
        }
    }
}

// Convert Parameter to openapiv3::Parameter
impl From<Parameter> for openapiv3::Parameter {
    fn from(param: Parameter) -> Self {
        let schema: Schema = param.schema.into();

        let required = param.required.unwrap_or(false);
        let explode = param.explode.unwrap_or(false);

        // Determine parameter location:
        match param.r#in {
            ParameterLocation::Path => {
                // Path parameters must be required = true by spec, but we use unwrap_or above
                openapiv3::Parameter::Path {
                    parameter_data: ParameterData {
                        name: param.name,
                        description: param.description,
                        required: true, // path params are always required
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(schema))),
                        example: None,
                        examples: Default::default(),
                        explode,
                        extensions: Default::default(),
                    },
                    style: param.style
                        .as_deref()
                        .map(|s| match s {
                            "simple" => PathStyle::Simple,
                            "label" => PathStyle::Label,
                            "matrix" => PathStyle::Matrix,
                            _ => PathStyle::Simple,
                        })
                        .unwrap_or(PathStyle::Simple),
                }
            },
            ParameterLocation::Query => {
                // Query parameters by default:
                openapiv3::Parameter::Query {
                    parameter_data: ParameterData {
                        name: param.name,
                        description: param.description,
                        required,
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(schema))),
                        example: None,
                        examples: Default::default(),
                        explode,
                        extensions: Default::default(),
                    },
                    allow_empty_value: None,
                    allow_reserved: false,
                    style: param.style
                        .as_deref()
                        .map(|s| match s {
                            "form" => QueryStyle::Form,
                            "spaceDelimited" => QueryStyle::SpaceDelimited,
                            "pipeDelimited" => QueryStyle::PipeDelimited,
                            "deepObject" => QueryStyle::DeepObject,
                            _ => QueryStyle::Form,
                        })
                        .unwrap_or(QueryStyle::Form),
                }
            },
            ParameterLocation::Header => {
                openapiv3::Parameter::Header {
                    parameter_data: ParameterData {
                        name: param.name,
                        description: param.description,
                        required,
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(schema))),
                        example: None,
                        examples: Default::default(),
                        explode,
                        extensions: Default::default(),
                    },
                    style: openapiv3::HeaderStyle::Simple,
                }
            },
            ParameterLocation::Cookie => {
                openapiv3::Parameter::Cookie {
                    parameter_data: ParameterData {
                        name: param.name,
                        description: param.description,
                        required,
                        deprecated: None,
                        format: ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(schema))),
                        example: None,
                        examples: Default::default(),
                        explode,
                        extensions: Default::default(),
                    },
                    style: openapiv3::CookieStyle::Form,
                }
            }
        }
    }
}