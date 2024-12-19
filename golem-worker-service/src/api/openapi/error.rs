use thiserror::Error;
use openapiv3::{OpenAPI, ReferenceOr};
use tracing::warn;

#[derive(Error, Debug)]
pub enum OpenAPIError {
    #[error("Invalid API definition: {0}")]
    InvalidDefinition(String),
    #[error("OpenAPI validation failed: {0}")]
    ValidationFailed(String),
    #[error("Schema validation failed: {expected} != {found}")]
    SchemaMismatch { expected: String, found: String },
}

pub fn validate_openapi(spec: &OpenAPI) -> Result<(), OpenAPIError> {
    // Validate basic structure
    if spec.openapi != "3.0.0" {
        return Err(OpenAPIError::ValidationFailed("Only OpenAPI 3.0.0 is supported".into()));
    }

    // Validate paths
    for (path, item) in &spec.paths.paths {
        validate_path_item(path, item)?;
    }

    // Validate components
    if let Some(components) = &spec.components {
        validate_components(components)?;
    }

    Ok(())
}

fn validate_path_item(path: &str, item: &ReferenceOr<openapiv3::PathItem>) -> Result<(), OpenAPIError> {
    match item {
        ReferenceOr::Item(item) => {
            // Each path must have at least one operation
            if item.get.is_none() && item.post.is_none() && 
               item.put.is_none() && item.delete.is_none() {
                warn!("Path {} has no operations", path);
                return Err(OpenAPIError::ValidationFailed(
                    format!("Path {} must have at least one operation", path)
                ));
            }
            Ok(())
        },
        ReferenceOr::Reference { .. } => {
            warn!("Reference path items are not supported");
            Err(OpenAPIError::ValidationFailed("Reference path items are not supported".into()))
        }
    }
}

fn validate_components(components: &openapiv3::Components) -> Result<(), OpenAPIError> {
    // Validate schemas
    for (name, schema) in &components.schemas {
        if let ReferenceOr::Item(schema) = schema {
            validate_schema(name, schema)?;
        }
    }
    Ok(())
}

fn validate_schema(name: &str, schema: &openapiv3::Schema) -> Result<(), OpenAPIError> {
    // Check for valid schema types
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(_) => Ok(()),
        openapiv3::SchemaKind::OneOf { .. } |
        openapiv3::SchemaKind::AnyOf { .. } |
        openapiv3::SchemaKind::AllOf { .. } => {
            warn!("Schema {} uses unsupported composition", name);
            Err(OpenAPIError::ValidationFailed(
                format!("Schema {} uses unsupported composition types", name)
            ))
        },
        openapiv3::SchemaKind::Not { .. } => {
            warn!("Schema {} uses unsupported 'not' type", name);
            Err(OpenAPIError::ValidationFailed(
                format!("Schema {} uses unsupported 'not' type", name)
            ))
        },
        _ => Ok(())
    }
}