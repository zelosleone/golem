use thiserror::Error;
use openapiv3::OpenAPI;
use golem_wasm_ast::analysis::AnalysedType;

#[derive(Error, Debug)]
pub enum OpenAPIError {
    #[error("Invalid API definition: {0}")]
    InvalidDefinition(String),
    #[error("OpenAPI validation failed: {0}")]
    ValidationFailed(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Invalid WIT type mapping: {0}")]
    InvalidWitType(String),
    #[error("Schema validation failed: {expected} != {found}")]
    SchemaMismatch { expected: String, found: String },
}

pub fn validate_openapi(spec: &OpenAPI) -> Result<(), OpenAPIError> {
    // Validate OpenAPI spec structure
    validate_spec_structure(spec)?;
    
    // Validate all schemas against WIT types
    if let Some(components) = &spec.components {
        if let Some(schemas) = &components.schemas {
            for (name, schema) in schemas {
                validate_schema_wit_types(name, schema)?;
            }
        }
    }

    // Validate all operations' request/response types
    for (path, item) in &spec.paths.paths {
        validate_path_operations(&path, item)?;
    }

    Ok(())
}

fn validate_spec_structure(spec: &OpenAPI) -> Result<(), OpenAPIError> {
    if spec.openapi != "3.0.0" {
        return Err(OpenAPIError::ValidationFailed(
            "Only OpenAPI 3.0.0 is supported".to_string()
        ));
    }

    if spec.paths.paths.is_empty() {
        return Err(OpenAPIError::ValidationFailed(
            "API must contain at least one path".to_string()
        ));
    }

    Ok(())
}

fn validate_schema_wit_types(name: &str, schema: &openapiv3::ReferenceOr<openapiv3::Schema>) -> Result<(), OpenAPIError> {
    match schema {
        openapiv3::ReferenceOr::Item(schema) => {
            // Attempt to convert schema to AnalysedType for validation
            let wit_type = schema_to_analysed_type(schema).ok_or_else(|| {
                OpenAPIError::InvalidWitType(format!("Cannot convert schema {} to WIT type", name))
            })?;

            // Verify the WIT type is valid
            validate_wit_type(&wit_type)
        },
        _ => Ok(()) // References are validated elsewhere
    }
}

fn validate_path_operations(path: &str, item: &openapiv3::ReferenceOr<openapiv3::PathItem>) -> Result<(), OpenAPIError> {
    if let openapiv3::ReferenceOr::Item(item) = item {
        // Validate GET operation
        if let Some(op) = &item.get {
            validate_operation(path, "GET", op)?;
        }
        // Validate POST operation
        if let Some(op) = &item.post {
            validate_operation(path, "POST", op)?;
        }
        // ...validate other operations...
    }
    Ok(())
}

fn validate_operation(path: &str, method: &str, op: &openapiv3::Operation) -> Result<(), OpenAPIError> {
    // Validate request body if present
    if let Some(body) = &op.request_body {
        validate_request_body(path, method, body)?;
    }

    // Validate responses
    for (code, response) in &op.responses.responses {
        validate_response(path, method, &code, response)?;
    }

    Ok(())
}

fn validate_wit_type(wit_type: &AnalysedType) -> Result<(), OpenAPIError> {
    match wit_type {
        AnalysedType::Str(_) | AnalysedType::Int32(_) | AnalysedType::Int64(_) 
        | AnalysedType::F32(_) | AnalysedType::F64(_) | AnalysedType::Bool(_) 
        | AnalysedType::Void(_) => Ok(()),
        AnalysedType::List(t) => validate_wit_type(&t.inner),
        AnalysedType::Option(t) => validate_wit_type(&t.inner),
        AnalysedType::Result(t) => {
            if let (Some(ok), Some(err)) = (&t.ok, &t.err) {
                validate_wit_type(ok)?;
                validate_wit_type(err)
            } else {
                Err(OpenAPIError::InvalidWitType(
                    "Result type must have both ok and err types".to_string()
                ))
            }
        },
        AnalysedType::Record(r) => {
            for field in &r.fields {
                validate_wit_type(&field.typ)?;
            }
            Ok(())
        },
        // Add other WIT type validations as needed
        _ => Err(OpenAPIError::InvalidWitType(
            format!("Unsupported WIT type: {:?}", wit_type)
        ))
    }
}

// Helper function to convert OpenAPI schema to AnalysedType
fn schema_to_analysed_type(schema: &openapiv3::Schema) -> Option<AnalysedType> {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(t) => match t {
            openapiv3::Type::String(_) => Some(AnalysedType::Str(Default::default())),
            openapiv3::Type::Number(_) => Some(AnalysedType::F64(Default::default())),
            openapiv3::Type::Integer(_) => Some(AnalysedType::Int64(Default::default())),
            openapiv3::Type::Boolean {} => Some(AnalysedType::Bool(Default::default())),
            openapiv3::Type::Array(arr) => {
                schema_to_analysed_type(&arr.items).map(|inner| {
                    AnalysedType::List(golem_wasm_ast::analysis::TypeList {
                        inner: Box::new(inner)
                    })
                })
            },
            openapiv3::Type::Object(obj) => {
                let fields = obj.properties.iter().filter_map(|(name, schema)| {
                    schema_to_analysed_type(&schema).map(|typ| {
                        golem_wasm_ast::analysis::NameTypePair {
                            name: name.clone(),
                            typ,
                        }
                    })
                }).collect();
                Some(AnalysedType::Record(golem_wasm_ast::analysis::TypeRecord { fields }))
            }
        },
        _ => None
    }
}