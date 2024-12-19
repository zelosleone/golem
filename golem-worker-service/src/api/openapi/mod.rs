mod converter;
mod error;
mod validation;
mod types;

pub use error::{OpenAPIError, validate_openapi};
pub use types::OpenAPISpec;
pub use converter::OpenAPIConverter;

// Re-export openapiv3::Schema for external use 
pub use openapiv3::Schema as OpenAPISchema;

pub use openapiv3::{
    Schema, SchemaKind, Type as OpenAPIType,
    VariantOrUnknownOrEmpty, StringFormat, Parameter,
    PathStyle, ParameterData,
};

use openapiv3::OpenAPI;
use crate::api::definition::types::ApiDefinition;

impl OpenAPIConverter {
    pub fn convert(api: &ApiDefinition) -> OpenAPI {
        Self::convert_to_spec(api)
    }
}