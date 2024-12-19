mod types;
mod converter;
mod error;
mod validation;

pub use types::*;
pub use converter::OpenAPIConverter;
pub use error::{OpenAPIError, validate_openapi};

use openapiv3::OpenAPI;
use crate::api::definition::types::ApiDefinition;

impl OpenAPIConverter {
    pub fn convert(api: &ApiDefinition) -> OpenAPI {
        OpenAPI {
            openapi: String::from("3.0.0"),
            info: openapiv3::Info {
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
}