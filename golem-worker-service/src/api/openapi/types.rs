use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use openapiv3::{
    ReferenceOr, Schema, SchemaKind, Type as OpenAPIType, StringFormat,
    VariantOrUnknownOrEmpty, IntegerFormat, NumberFormat, ParameterData, ParameterStyle,
};
use indexmap::IndexMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAPISpec {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<GolemComponents>,
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDocs {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OpenAPISchemaType {
    String {
        format: Option<String>,
        enum_values: Option<Vec<String>>,
        pattern: Option<String>,
        min_length: Option<u32>,
        max_length: Option<u32>,
    },
    Integer {
        format: Option<String>,
        minimum: Option<i64>,
        maximum: Option<i64>,
        multiple_of: Option<i64>,
        exclusive_minimum: Option<bool>,
        exclusive_maximum: Option<bool>,
    },
    Number {
        format: Option<String>,
        minimum: Option<f64>,
        maximum: Option<f64>,
        multiple_of: Option<f64>,
        exclusive_minimum: Option<bool>,
        exclusive_maximum: Option<bool>,
    },
    Boolean,
    Array {
        items: Box<OpenAPISchemaType>,
        min_items: Option<u32>,
        max_items: Option<u32>,
        unique_items: Option<bool>,
    },
    Object {
        properties: HashMap<String, OpenAPISchemaType>,
        required: Option<Vec<String>>,
        additional_properties: Option<Box<OpenAPISchemaType>>,
        min_properties: Option<u32>,
        max_properties: Option<u32>,
    },
    Reference {
        #[serde(rename = "$ref")]
        reference: String,
    }
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
    pub authorization_code: Option<OAuthFlow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthFlow {
    pub authorization_url: String,
    pub token_url: Option<String>,
    pub refresh_url: Option<String>,
    pub scopes: HashMap<String, String>
}

// Helper functions for formats
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
    match s {
        "int32" => VariantOrUnknownOrEmpty::Item(IntegerFormat::Int32),
        "int64" => VariantOrUnknownOrEmpty::Item(IntegerFormat::Int64),
        other => VariantOrUnknownOrEmpty::Unknown(other.to_string()),
    }
}

fn number_format_from_str(s: &str) -> VariantOrUnknownOrEmpty<NumberFormat> {
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
            OpenAPISchemaType::String { format, enum_values, pattern, min_length, max_length } => {
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::String(openapiv3::StringType {
                        format: format.as_deref().map(string_format_from_str),
                        pattern,
                        enumeration: enum_values.unwrap_or_default().into_iter().map(Some).collect(),
                        min_length,
                        max_length,
                    })),
                }
            },
            OpenAPISchemaType::Integer { format, minimum, maximum, multiple_of, exclusive_minimum, exclusive_maximum } => {
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Integer(openapiv3::IntegerType {
                        format: format.as_deref().map(integer_format_from_str),
                        multiple_of: multiple_of.map(|x| x as f64),
                        minimum,
                        maximum,
                        exclusive_minimum: exclusive_minimum.unwrap_or(false),
                        exclusive_maximum: exclusive_maximum.unwrap_or(false),
                        enumeration: vec![],
                    })),
                }
            },
            OpenAPISchemaType::Number { format, minimum, maximum, multiple_of, exclusive_minimum, exclusive_maximum } => {
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Number(openapiv3::NumberType {
                        format: format.as_deref().map(number_format_from_str),
                        multiple_of,
                        minimum,
                        maximum,
                        exclusive_minimum: exclusive_minimum.unwrap_or(false),
                        exclusive_maximum: exclusive_maximum.unwrap_or(false),
                        enumeration: vec![],
                    })),
                }
            },
            OpenAPISchemaType::Boolean => {
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Boolean(Default::default())),
                }
            },
            OpenAPISchemaType::Array { items, min_items, max_items, unique_items } => {
                let items_schema: Schema = (*items).into();
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Array(openapiv3::ArrayType {
                        items: Some(Box::new(ReferenceOr::Item(items_schema))),
                        min_items,
                        max_items,
                        unique_items
                    })),
                }
            },
            OpenAPISchemaType::Object { properties, required, additional_properties, min_properties, max_properties } => {
                let mut props = IndexMap::new();
                for (k, v) in properties {
                    props.insert(k, ReferenceOr::Item(v.into()));
                }
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::Type(OpenAPIType::Object(openapiv3::ObjectType {
                        properties: props,
                        required: required.unwrap_or_default(),
                        additional_properties: additional_properties.map(|schema| {
                            Box::new(ReferenceOr::Item((*schema).into()))
                        }),
                        min_properties,
                        max_properties,
                    })),
                }
            },
            OpenAPISchemaType::Reference { reference } => {
                // We use AllOf with a single reference for representation.
                // Another approach might be needed depending on tooling expectations.
                Schema {
                    schema_data: Default::default(),
                    schema_kind: SchemaKind::AllOf {
                        all_of: vec![ReferenceOr::Reference { reference }],
                    },
                }
            },
        }
    }
}