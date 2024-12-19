use super::types::{OpenAPISpec, PathItem, ParameterLocation};
use std::collections::HashMap;
use super::error::OpenAPIError;
use openapiv3::{Operation, ReferenceOr, SchemaKind};
use crate::api::definition::patterns::{AllPathPatterns, PathPattern};
use tracing::warn;

pub fn validate_openapi(spec: &OpenAPISpec) -> Result<(), String> {
    validate_paths(&spec.paths)?;
    let schemas = spec.components.as_ref().and_then(|c| c.schemas.as_ref());
    validate_schemas(schemas)?;
    Ok(())
}

fn validate_paths(paths: &HashMap<String, PathItem>) -> Result<(), String> {
    for (path, item) in paths {
        if let Some(op) = &item.get {
            validate_operation(path, op)?;
        }
        if let Some(op) = &item.post {
             validate_operation(path, op)?;
        }
         if let Some(op) = &item.put {
              validate_operation(path, op)?;
        }
         if let Some(op) = &item.delete {
              validate_operation(path, op)?;
        }
    }
    Ok(())
}

fn validate_operation(path: &str, op: &super::types::Operation) -> Result<(), String> {
     if let Some(params) = &op.parameters {
            validate_parameters(path, params)?;
    }
    Ok(())
}


fn validate_parameters(path: &str, params: &Vec<super::types::Parameter>) -> Result<(), String> {
    for p in params.iter() {
        if p.r#in == ParameterLocation::Path {
             validate_path_parameter(path, p)?;
        }
    }
    Ok(())
}


fn validate_path_parameter(path: &str, p: &super::types::Parameter) -> Result<(), String> {
    let path_segments: Vec<&str> = path.split('/').collect();
      let matching_segments = path_segments
            .iter()
            .filter(|segment| segment.starts_with('{') && segment.ends_with('}'))
            .map(|segment| segment[1..segment.len() - 1].to_string()).collect::<Vec<_>>();
    
        if !matching_segments.iter().any(|s| s == &p.name) {
            return Err(format!(
                "Path parameter `{}` not found in path `{}`",
                p.name, path
            ));
        }
        Ok(())
}


fn validate_schemas(schemas: &Option<HashMap<String, crate::api::openapi::OpenAPISchema>>) -> Result<(), String> {
    if let Some(schemas) = schemas {
        for (name, schema) in schemas {
            validate_schema(name, schema)?;
        }
    }
    Ok(())
}

fn validate_schema(name: &str, schema: &crate::api::openapi::OpenAPISchema) -> Result<(), String> {
    match &schema.schema_kind {
        SchemaKind::Type(type_) => {
            // Validate schema type
            validate_type(type_)
        },
        SchemaKind::OneOf { .. } |
        SchemaKind::AnyOf { .. } |
        SchemaKind::AllOf { .. } => {
            warn!("Schema {} uses unsupported composition", name);
            Err(format!("Schema {} uses unsupported composition types", name))
        },
        SchemaKind::Not { .. } => {
            warn!("Schema {} uses unsupported 'not' type", name);
            Err(format!("Schema {} uses unsupported 'not' type", name))
        },
        _ => Ok(())
    }
}

fn validate_type(type_: &openapiv3::Type) -> Result<(), String> {
    match type_ {
        openapiv3::Type::String(_) |
        openapiv3::Type::Number(_) |
        openapiv3::Type::Integer(_) |
        openapiv3::Type::Boolean { .. } => Ok(()),
        openapiv3::Type::Array(array) => {
            if let Some(items) = array.items.as_ref() {
                match items {
                    ReferenceOr::Item(schema) => validate_schema("array_items", schema),
                    ReferenceOr::Reference { reference } => validate_schema_ref(&"array_items".to_string(), reference)
                }
            } else {
                Ok(())
            }
        },
        openapiv3::Type::Object(obj) => {
            // Validate object properties
            for (name, property) in &obj.properties {
                if let ReferenceOr::Item(schema) = property {
                    validate_schema(name, schema)?;
                }
            }
            Ok(())
        }
    }
}

fn validate_schema_ref(_key: &String, reference: &String) -> Result<(), String> {
   if !reference.starts_with("#/components/schemas/") {
        return Err(format!(
            "Schema reference `{}` is invalid",
            reference
        ));
   }
      Ok(())
}

pub(crate) fn validate_path_pattern(path: &str) -> Result<(), OpenAPIError> {
    match AllPathPatterns::parse(path) {
        Ok(pattern) => {
            for p in pattern.path_patterns {
                match p {
                    PathPattern::Var(info) => {
                        if !validate_parameter_name(&info.key_name) {
                            return Err(OpenAPIError::ValidationFailed(
                                format!("Invalid path parameter name: {}", info.key_name)
                            ));
                        }
                    },
                    PathPattern::CatchAllVar(info) => {
                        if !validate_catch_all_name(&info.key_name) {
                            return Err(OpenAPIError::ValidationFailed(
                                format!("Invalid catch-all parameter name: {}", info.key_name)
                            ));
                        }
                    },
                    _ => {}
                }
            }
            Ok(())
        },
        Err(e) => {
            warn!("Invalid path pattern: {}", e);
            Err(OpenAPIError::ValidationFailed(format!("Invalid path pattern: {}", e)))
        }
    }
}

fn validate_parameter_name(name: &str) -> bool {
    !name.is_empty() 
    && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    && !name.starts_with('_')
    && !name.ends_with('_')
}

fn validate_catch_all_name(name: &str) -> bool {
    validate_parameter_name(name) && !name.contains("__")
}

pub(crate) fn validate_operation_types(operation: &Operation) -> Result<(), OpenAPIError> {
    // Must have at least one response
    if operation.responses.responses.is_empty() {
        return Err(OpenAPIError::ValidationFailed(
            "Operation must have at least one response".into()
        ));
    }

    // Validate parameters
    for param in &operation.parameters {
        if let openapiv3::ReferenceOr::Item(param) = param {
            if let Some(schema) = &param.parameter_data().schema {
                validate_parameter_schema(schema)?;
            }
        }
    }

    Ok(())
}

fn validate_parameter_schema(schema: &crate::api::openapi::OpenAPISchema) -> Result<(), OpenAPIError> {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(t) => {
            match t {
                openapiv3::Type::String(_) |
                openapiv3::Type::Number(_) |
                openapiv3::Type::Integer(_) |
                openapiv3::Type::Boolean { .. } => Ok(()),
                _ => {
                    warn!("Unsupported parameter schema type");
                    Err(OpenAPIError::ValidationFailed(
                        "Unsupported parameter schema type".into()
                    ))
                }
            }
        },
        _ => {
            warn!("Only simple types are supported for parameters");
            Err(OpenAPIError::ValidationFailed(
                "Only simple types are supported for parameters".into()
            ))
        }
    }
}

mod tests {
    #[test]
    fn test_parameter_name_validation() {
        assert!(super::validate_parameter_name("user_id"));
        assert!(super::validate_parameter_name("count123"));
        assert!(!super::validate_parameter_name("_hidden"));
        assert!(!super::validate_parameter_name("invalid-name"));
        assert!(!super::validate_parameter_name(""));
    }

    #[test]
    fn test_catch_all_validation() {
        assert!(super::validate_catch_all_name("all_files"));
        assert!(!super::validate_catch_all_name("bad__name"));
        assert!(!super::validate_catch_all_name("_invalid"));
    }
}