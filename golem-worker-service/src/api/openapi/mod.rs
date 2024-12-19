mod converter;
mod error;
mod validation;
mod types;

pub use error::{OpenAPIError, validate_openapi};
pub use types::OpenAPISpec;
pub use converter::OpenAPIConverter;

// Re-export openapiv3 types for external use
pub use openapiv3::{
    Schema, SchemaKind, Type as OpenAPIType,
    VariantOrUnknownOrEmpty, StringFormat, Parameter,
    PathStyle, ParameterData,
    OpenAPI,
};

use crate::api::definition::types::ApiDefinition;

impl OpenAPIConverter {
    pub fn convert(api: &ApiDefinition) -> OpenAPI {
        Self::convert_to_spec(api)
    }
}