use poem_openapi::types::{ParseFromJSON, ParseFromParameter, ToJSON, Type};
use serde::{Deserialize, Serialize};
use golem_api_grpc::proto::golem::worker;
use golem_api_grpc::proto::golem::common;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiIdempotencyKey {
    pub value: String,
}

impl ApiIdempotencyKey {
    pub fn fresh() -> Self {
        Self {
            value: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiWorkerId {
    pub value: String,
}

impl From<worker::WorkerId> for ApiWorkerId {
    fn from(id: worker::WorkerId) -> Self {
        Self { value: id.value }
    }
}

impl From<ApiWorkerId> for worker::WorkerId {
    fn from(id: ApiWorkerId) -> Self {
        worker::WorkerId { value: id.value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiAccountId {
    pub value: String,
}

impl From<common::AccountId> for ApiAccountId {
    fn from(id: common::AccountId) -> Self {
        Self { value: id.value }
    }
}

impl From<ApiAccountId> for common::AccountId {
    fn from(id: ApiAccountId) -> Self {
        common::AccountId { value: id.value }
    }
}

// Implement OpenAPI traits for wrapper types
impl Type for ApiIdempotencyKey {
    const IS_REQUIRED: bool = true;
    type RawValueType = Self;
    type RawElementValueType = String;

    fn name() -> std::borrow::Cow<'static, str> {
        "IdempotencyKey".into()
    }

    fn schema_ref() -> poem_openapi::registry::MetaSchemaRef {
        String::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter(&self) -> Box<dyn Iterator<Item = &Self::RawElementValueType> + '_> {
        Box::new(std::iter::once(&self.value))
    }
}

impl ParseFromParameter for ApiIdempotencyKey {
    fn parse_from_parameter(value: &str) -> poem_openapi::types::ParseResult<Self> {
        Ok(Self { value: value.to_string() })
    }
}

impl ParseFromJSON for ApiIdempotencyKey {
    fn parse_from_json(value: Option<serde_json::Value>) -> poem_openapi::types::ParseResult<Self> {
        if let Some(value) = value {
            if let Ok(s) = serde_json::from_value::<String>(value) {
                return Ok(Self { value: s });
            }
        }
        Err(poem_openapi::types::ParseError::expected_type(value))
    }
}

impl ToJSON for ApiIdempotencyKey {
    fn to_json(&self) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.value.clone()))
    }
}

// Implement OpenAPI traits for ApiWorkerId
impl Type for ApiWorkerId {
    const IS_REQUIRED: bool = true;
    type RawValueType = Self;
    type RawElementValueType = String;

    fn name() -> std::borrow::Cow<'static, str> {
        "WorkerId".into()
    }

    fn schema_ref() -> poem_openapi::registry::MetaSchemaRef {
        String::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        Some(self)
    }

    fn raw_element_iter(&self) -> Box<dyn Iterator<Item = &Self::RawElementValueType> + '_> {
        Box::new(std::iter::once(&self.value))
    }
}

impl ParseFromParameter for ApiWorkerId {
    fn parse_from_parameter(value: &str) -> poem_openapi::types::ParseResult<Self> {
        Ok(Self { value: value.to_string() })
    }
}

impl ParseFromJSON for ApiWorkerId {
    fn parse_from_json(value: Option<serde_json::Value>) -> poem_openapi::types::ParseResult<Self> {
        if let Some(value) = value {
            if let Ok(s) = serde_json::from_value::<String>(value) {
                return Ok(Self { value: s });
            }
        }
        Err(poem_openapi::types::ParseError::expected_type(value))
    }
}

impl ToJSON for ApiWorkerId {
    fn to_json(&self) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.value.clone()))
    }
}
