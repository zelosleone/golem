use openapiv3::{OpenAPI, PathItem, Parameter, ReferenceOr};
use golem_worker_service_base::gateway_api_definition::http::CompiledHttpApiDefinition;
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    DuplicatePathParameter(String, String),
    MissingPathParameter(String, String),
    UnusedPathParameter(String, String),
    InvalidParameterType(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::DuplicatePathParameter(path, param) => {
                write!(f, "Duplicate path parameter '{}' in path '{}'", param, path)
            }
            ValidationError::MissingPathParameter(path, param) => {
                write!(f, "Missing path parameter '{}' in path '{}'", param, path)
            }
            ValidationError::UnusedPathParameter(path, param) => {
                write!(f, "Unused path parameter '{}' in path '{}'", param, path)
            }
            ValidationError::InvalidParameterType(msg) => {
                write!(f, "Invalid parameter type: {}", msg)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_openapi_spec(spec: &OpenAPI) -> Result<(), ValidationError> {
    for (path, path_item) in &spec.paths.paths {
        if let ReferenceOr::Item(path_item) = path_item {
            validate_path_item(path, path_item)?;
        }
    }
    Ok(())
}

fn validate_path_item(path: &str, item: &PathItem) -> Result<(), ValidationError> {
    // Extract all operations from the path item
    let operations = [
        item.get.as_ref(),
        item.post.as_ref(),
        item.put.as_ref(),
        item.delete.as_ref(),
        item.patch.as_ref(),
    ];

    // Validate each operation
    for operation in operations.iter().filter_map(|op| *op) {
        validate_operation(path, operation)?;
    }

    Ok(())
}

fn validate_operation(path: &str, operation: &openapiv3::Operation) -> Result<(), ValidationError> {
    let mut path_params = std::collections::HashSet::new();

    // Extract path parameters from the path
    let path_segments: Vec<&str> = path.split('/').collect();
    for segment in path_segments {
        if segment.starts_with('{') && segment.ends_with('}') {
            let param_name = &segment[1..segment.len() - 1];
            path_params.insert(param_name.to_string());
        }
    }

    // Check operation parameters
    let mut used_params = std::collections::HashSet::new();
    for param in &operation.parameters {
        if let ReferenceOr::Item(param) = param {
            match param {
                Parameter::Path { parameter_data, .. } => {
                    if !path_params.contains(&parameter_data.name) {
                        return Err(ValidationError::UnusedPathParameter(
                            path.to_string(),
                            parameter_data.name.clone(),
                        ));
                    }
                    if !used_params.insert(parameter_data.name.clone()) {
                        return Err(ValidationError::DuplicatePathParameter(
                            path.to_string(),
                            parameter_data.name.clone(),
                        ));
                    }
                }
                _ => validate_parameter(path, param)?,
            }
        }
    }

    // Check for missing path parameters
    for param in path_params {
        if !used_params.contains(&param) {
            return Err(ValidationError::MissingPathParameter(
                path.to_string(),
                param,
            ));
        }
    }

    Ok(())
}

fn validate_parameter(path: &str, param: &Parameter) -> Result<(), ValidationError> {
    match param {
        Parameter::Path { parameter_data, .. } => {
            if !parameter_data.required {
                return Err(ValidationError::InvalidParameterType(
                    format!("Path parameter '{}' must be required", parameter_data.name)
                ));
            }
            Ok(())
        }
        Parameter::Query { parameter_data, .. } |
        Parameter::Header { parameter_data, .. } |
        Parameter::Cookie { parameter_data, .. } => {
            if parameter_data.name.is_empty() {
                return Err(ValidationError::InvalidParameterType(
                    format!("Parameter name cannot be empty in path '{}'", path)
                ));
            }
            Ok(())
        }
    }
}

pub fn validate_api_definition<T>(api_def: &CompiledHttpApiDefinition<T>) -> Result<(), ValidationError> {
    // Validate each route in the API definition
    for route in &api_def.routes {
        // Extract path parameters from the path
        let path_str = route.path.to_string();
        let path_params: Vec<&str> = path_str
            .split('/')
            .filter(|s| s.starts_with('{') && s.ends_with('}'))
            .map(|s| &s[1..s.len()-1])
            .collect();

        // Check for duplicate path parameters
        let mut seen_params = std::collections::HashSet::new();
        for &param in &path_params {
            if !seen_params.insert(param) {
                return Err(ValidationError::DuplicatePathParameter(
                    path_str.clone(),
                    param.to_string(),
                ));
            }
        }
    }
    Ok(())
}