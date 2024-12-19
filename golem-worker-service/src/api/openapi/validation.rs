use openapiv3::{OpenAPI, PathItem, Parameter, ReferenceOr, Components, Schema, Type, SchemaKind};
use golem_worker_service_base::gateway_api_definition::http::CompiledHttpApiDefinition;
use std::fmt;
use std::collections::HashSet;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Duplicate path parameter '{param}' in path '{path}'")]
    DuplicatePathParameter { path: String, param: String },

    #[error("Missing path parameter '{param}' in path '{path}'")]
    MissingPathParameter { path: String, param: String },

    #[error("Invalid parameter type: {0}")]
    InvalidParameterType(String),

    #[error("Schema validation error: {0}")]
    SchemaValidation(String),

    #[error("Security scheme validation error: {0}")]
    SecuritySchemeValidation(String),

    #[error("CORS configuration error: {0}")]
    CorsValidation(String),
}

pub struct OpenAPIValidator {
    strict_mode: bool,
    validate_examples: bool,
}

impl OpenAPIValidator {
    pub fn new() -> Self {
        Self {
            strict_mode: false,
            validate_examples: true,
        }
    }

    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    pub fn with_example_validation(mut self, validate: bool) -> Self {
        self.validate_examples = validate;
        self
    }

    pub fn validate(&self, spec: &OpenAPI) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate paths
        for (path, item) in spec.paths.paths.iter() {
            if let Err(e) = self.validate_path_item(path, item) {
                errors.extend(e);
            }
        }

        // Validate components
        if let Err(e) = self.validate_components(&spec.components) {
            errors.extend(e);
        }

        // Validate security schemes
        if let Some(security) = &spec.security {
            if let Err(e) = self.validate_security_requirements(security, &spec.components) {
                errors.extend(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_path_item(&self, path: &str, item: &ReferenceOr<PathItem>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        let path_item = match item {
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { reference } => {
                if self.strict_mode {
                    errors.push(ValidationError::SchemaValidation(
                        format!("References not allowed in strict mode: {}", reference)
                    ));
                }
                return if errors.is_empty() { Ok(()) } else { Err(errors) };
            }
        };

        // Extract path parameters
        let path_params: HashSet<_> = path
            .split('/')
            .filter(|s| s.starts_with('{') && s.ends_with('}'))
            .map(|s| s[1..s.len()-1].to_string())
            .collect();

        // Validate parameters
        let declared_params: HashSet<_> = path_item
            .parameters
            .iter()
            .filter_map(|p| match p {
                ReferenceOr::Item(p) if p.parameter_data.location == "path" => {
                    Some(p.parameter_data.name.clone())
                }
                _ => None,
            })
            .collect();

        // Check for missing parameters
        for param in path_params.difference(&declared_params) {
            errors.push(ValidationError::MissingPathParameter {
                path: path.to_string(),
                param: param.clone(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_components(&self, components: &Option<Components>) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        
        if let Some(components) = components {
            // Validate schemas
            for (name, schema) in components.schemas.iter() {
                if let ReferenceOr::Item(schema) = schema {
                    if self.validate_examples && schema.example.is_some() {
                        if let Err(e) = self.validate_example(name, schema) {
                            errors.push(e);
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_example(&self, name: &str, schema: &Schema) -> Result<(), ValidationError> {
        if let Some(example) = &schema.example {
            match &schema.schema_kind {
                SchemaKind::Type(Type::String(_)) if matches!(example, Value::String(_)) => Ok(()),
                SchemaKind::Type(Type::Integer(_)) if matches!(example, Value::Number(n)) && n.is_i64() => Ok(()),
                SchemaKind::Type(Type::Number(_)) if matches!(example, Value::Number(_)) => Ok(()),
                SchemaKind::Type(Type::Boolean(_)) if matches!(example, Value::Bool(_)) => Ok(()),
                SchemaKind::Type(Type::Array(_)) if matches!(example, Value::Array(_)) => Ok(()),
                SchemaKind::Type(Type::Object(_)) if matches!(example, Value::Object(_)) => Ok(()),
                _ => Err(ValidationError::SchemaValidation(
                    format!("Example for '{}' does not match schema type", name)
                )),
            }
        } else {
            Ok(())
        }
    }

    fn validate_security_requirements(
        &self,
        security: &[SecurityRequirement],
        components: &Option<Components>,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Some(components) = components {
            for req in security {
                for scheme_name in req.keys() {
                    if !components.security_schemes.contains_key(scheme_name) {
                        errors.push(ValidationError::SecuritySchemeValidation(
                            format!("Security scheme '{}' not found in components", scheme_name)
                        ));
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn validate_openapi_spec(spec: &OpenAPI) -> Result<(), Vec<ValidationError>> {
    let validator = OpenAPIValidator::new();
    validator.validate(spec)
}

pub fn validate_api_definition<T>(api_def: &CompiledHttpApiDefinition<T>) -> Result<(), Vec<ValidationError>> {
    let validator = OpenAPIValidator::new();
    let spec = OpenAPI {
        openapi: "3.0.0".to_string(),
        info: None,
        servers: None,
        paths: api_def.routes.iter().map(|route| {
            let path = route.path.clone();
            let item = PathItem {
                get: None,
                put: None,
                post: None,
                delete: None,
                options: None,
                head: None,
                patch: None,
                parameters: vec![],
            };
            (path, ReferenceOr::Item(item))
        }).collect(),
        components: None,
        security_schemes: None,
        tags: None,
        external_docs: None,
    };
    validator.validate(&spec)
}