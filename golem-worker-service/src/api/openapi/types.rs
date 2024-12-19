use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use openapiv3::{ReferenceOr, Schema, SchemaKind, Type as OpenAPIType};
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

pub trait IntoRaw<T> {
    fn into_raw(self) -> T;
}

impl IntoRaw<openapiv3::Parameter> for Parameter {
    fn into_raw(self) -> openapiv3::Parameter {
        openapiv3::Parameter::Query {
            parameter_data: openapiv3::ParameterData {
                name: self.name,
                description: self.description,
                required: self.required.unwrap_or(false),
                deprecated: None,
                format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(
                    self.schema.into_raw(),
                ))),
                example: None,
                examples: IndexMap::new(),
                extensions: IndexMap::new(),
            },
            style: self.style.map(|s| s.into()),
            allow_reserved: false,
            allow_empty_value: None,
        }
    }
}

// Add complete From implementation for OpenAPISchemaType
impl From<OpenAPISchemaType> for Schema {
    fn from(schema: OpenAPISchemaType) -> Self {
        let schema_kind = match schema {
            OpenAPISchemaType::String { format, enum_values } => {
                SchemaKind::Type(openapiv3::Type::String(openapiv3::StringType {
                    format: format.map(|f| f.into()),
                    enumeration: enum_values.map(|v| v.into_iter().map(Some).collect()).unwrap_or_default(),
                    ..Default::default()
                }))
            },
            OpenAPISchemaType::Integer { format } => {
                SchemaKind::Type(openapiv3::Type::Integer(openapiv3::IntegerType {
                    format: format.map(|f| f.into()),
                    ..Default::default()
                }))
            },
            OpenAPISchemaType::Number { format } => {
                SchemaKind::Type(openapiv3::Type::Number(openapiv3::NumberType {
                    format: format.map(|f| f.into()),
                    ..Default::default()
                }))
            },
            OpenAPISchemaType::Boolean => {
                SchemaKind::Type(openapiv3::Type::Boolean(openapiv3::BooleanType::default()))
            },
            OpenAPISchemaType::Array { items } => {
                SchemaKind::Type(openapiv3::Type::Array(openapiv3::ArrayType {
                    items: Some(Box::new(ReferenceOr::Item((*items).into()))),
                    ..Default::default()
                }))
            },
            OpenAPISchemaType::Object { properties, required } => {
                let properties = properties.into_iter()
                    .map(|(k, v)| (k, ReferenceOr::Item(v.into())))
                    .collect();
                SchemaKind::Type(openapiv3::Type::Object(openapiv3::ObjectType {
                    properties,
                    required: required.unwrap_or_default(),
                    ..Default::default()
                }))
            },
            OpenAPISchemaType::Reference { reference } => {
                SchemaKind::Reference { reference }
            }
        };
        Schema {
            schema_kind,
            schema_data: Default::default(),
        }
    }
}

impl From<Parameter> for openapiv3::Parameter {
    fn from(param: Parameter) -> Self {
        openapiv3::Parameter::Path {
            parameter_data: openapiv3::ParameterData {
                name: param.name,
                description: param.description,
                required: param.required.unwrap_or(true),
                deprecated: None,
                format: openapiv3::ParameterSchemaOrContent::Schema(Box::new(ReferenceOr::Item(
                    param.schema.into()
                ))),
                example: None,
                examples: Default::default(),
                explode: param.explode.unwrap_or(false),
                extensions: Default::default(),
            },
            style: param.style.map(|s| s.parse().unwrap_or(openapiv3::PathStyle::Simple))
                .unwrap_or(openapiv3::PathStyle::Simple),
        }
    }
}