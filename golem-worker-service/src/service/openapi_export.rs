use crate::api::definition::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::openapi::{OpenAPIConverter, OpenAPISpec, validate_openapi, OpenAPIError};
use golem_service_base::auth::{EmptyAuthCtx, DefaultNamespace};
use golem_worker_service_base::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use golem_worker_service_base::gateway_api_definition::http::MethodPattern;
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::{
    GatewayBindingCompiled, WorkerBinding, FileServerBinding, SwaggerUIBinding, StaticBinding
};
use golem_wasm_ast::analysis::{AnalysedType, TypeInference};
use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use tracing::{error, info};
use crate::service::api::Cache;

#[derive(Clone)]
pub struct OpenAPIExportConfig {
    pub default_namespace: String,
}

impl Default for OpenAPIExportConfig {
    fn default() -> Self {
        Self {
            default_namespace: "default".to_string(),
        }
    }
}

impl From<OpenAPIError> for StatusCode {
    fn from(err: OpenAPIError) -> Self {
        match err {
            OpenAPIError::InvalidDefinition(_) => StatusCode::BAD_REQUEST,
            OpenAPIError::ValidationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OpenAPIError::CacheError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

fn convert_method(method: &MethodPattern) -> HttpMethod {
    match method {
        MethodPattern::Get => HttpMethod::Get,
        MethodPattern::Post => HttpMethod::Post,
        MethodPattern::Put => HttpMethod::Put,
        MethodPattern::Delete => HttpMethod::Delete,
        MethodPattern::Patch => HttpMethod::Patch,
        MethodPattern::Head => HttpMethod::Head,
        MethodPattern::Options => HttpMethod::Options,
        // Removed Trace and Connect as they are not supported
        // Provide a default case to handle unexpected variants
        _ => {
            error!("Unsupported HTTP method encountered: {:?}", method);
            HttpMethod::Get // Defaulting to GET; adjust as needed
        }
    }
}

fn convert_binding(binding: &GatewayBindingCompiled) -> BindingType {
    match binding {
        GatewayBindingCompiled::Worker(worker) => {
            // Extract type information from Rib script
            let (input_type, output_type) = match &worker.rib_script {
                Some(script) => {
                    let rib_types = script.infer_types();
                    (rib_types.input_type, rib_types.output_type)
                },
                None => (AnalysedType::Any, AnalysedType::Any),
            };

            BindingType::Default {
                input_type,
                output_type,
                options: None,
            }
        },
        GatewayBindingCompiled::FileServer(fs) => {
            BindingType::FileServer {
                root_dir: fs.root_dir.clone(),
                options: None,
            }
        },
        GatewayBindingCompiled::SwaggerUI(swagger) => {
            BindingType::SwaggerUI {
                spec_path: swagger.spec_path.clone(),
                options: None,
            }
        },
        GatewayBindingCompiled::Static(static_binding) => {
            BindingType::Static {
                content_type: static_binding.content_type.clone(),
                content: static_binding.content.clone(),
            }
        },
        _ => {
            error!("Unsupported binding type encountered: {:?}", binding);
            // Default to a safe fallback
            BindingType::Static {
                content_type: "application/json".to_string(),
                content: Vec::new(),
            }
        }
    }
}

pub async fn export_openapi(
    State(services): State<crate::service::Services>,
    Path((id, version)): Path<(String, String)>,
) -> Result<Json<OpenAPISpec>, StatusCode> {
    info!("Requesting OpenAPI spec for API {}, version {}", id, version);

    // Try to get from cache first
    let cache_key = format!("openapi:{}:{}", id, version);
    if let Some(cached_spec) = services
        .cache
        .get::<OpenAPISpec>(&cache_key)
        .await
        .map_err(|e| {
            error!("Cache error: {}", e);
            <OpenAPIError as Into<StatusCode>>::into(OpenAPIError::CacheError(e.to_string()))
        })?
    {
        info!("Returning cached OpenAPI spec for {}", id);
        return Ok(Json(cached_spec));
    }

    // Fetch API definition if not in cache
    let namespace = DefaultNamespace::default(); // Create an instance instead of using the type
    let api_def = services
        .definition_service
        .get(
            &ApiDefinitionId(id.clone()),
            &ApiVersion(version.clone()),
            &namespace,
            &EmptyAuthCtx::default(),
        )
        .await
        .map_err(|e| {
            error!("Failed to fetch API definition: {}", e);
            <OpenAPIError as Into<StatusCode>>::into(OpenAPIError::InvalidDefinition(e.to_string()))
        })?
        .ok_or_else(|| {
            error!("API definition not found");
            <OpenAPIError as Into<StatusCode>>::into(OpenAPIError::InvalidDefinition(
                "API definition not found".to_string(),
            ))
        })?;

    // Convert CompiledHttpApiDefinition to ApiDefinition
    let api_id = api_def.id.0.clone();
    let api_definition = ApiDefinition {
        id: api_id.clone(),
        name: api_def.id.0.clone(), // Using 'id' as 'name' since 'name' field doesn't exist
        version: api_def.version.0.clone(),
        description: "".to_string(), // Providing a default empty description
        routes: api_def
            .routes
            .iter()
            .map(|r| Route {
                path: r.path.to_string(),
                method: convert_method(&r.method),
                description: "".to_string(), // Providing a default empty description
                template_name: "".to_string(), // Providing a default empty template_name
                binding: convert_binding(&r.binding),
            })
            .collect(),
    };

    // Convert to OpenAPI spec
    let spec = OpenAPIConverter::convert(&api_definition);

    // Validate the generated spec
    validate_openapi(&spec).map_err(|e| {
        error!("OpenAPI spec validation failed: {}", e);
        StatusCode::from(OpenAPIError::ValidationFailed(e))
    })?;

    // Cache the valid spec
    services
        .cache
        .set(&cache_key, &spec)
        .await
        .map_err(|e| {
            error!("Failed to cache OpenAPI spec: {}", e);
            StatusCode::from(OpenAPIError::CacheError(e.to_string()))
        })?;

    info!("Successfully generated and cached OpenAPI spec for {}", id);
    Ok(Json(spec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use golem_wasm_ast::analysis::{TypeStr, TypeBool};

    #[test]
    fn test_convert_worker_binding() {
        let rib_script = r#"
            fn handle_request(input: string) -> bool {
                input.length() > 0
            }
        "#;

        let worker_binding = WorkerBinding {
            rib_script: Some(rib_script.to_string()),
            function_name: "handle_request".to_string(),
            options: None,
        };

        let binding = convert_binding(&GatewayBindingCompiled::Worker(worker_binding));
        
        match binding {
            BindingType::Default { input_type, output_type, .. } => {
                assert!(matches!(input_type, AnalysedType::Str(TypeStr)));
                assert!(matches!(output_type, AnalysedType::Bool(TypeBool)));
            },
            _ => panic!("Expected Default binding type"),
        }
    }

    #[test]
    fn test_convert_file_server_binding() {
        let fs_binding = FileServerBinding {
            root_dir: "/test".to_string(),
            options: None,
        };

        let binding = convert_binding(&GatewayBindingCompiled::FileServer(fs_binding));
        
        match binding {
            BindingType::FileServer { root_dir, .. } => {
                assert_eq!(root_dir, "/test");
            },
            _ => panic!("Expected FileServer binding type"),
        }
    }

    #[test]
    fn test_convert_swagger_ui_binding() {
        let swagger_binding = SwaggerUIBinding {
            spec_path: "/api/spec".to_string(),
            options: None,
        };

        let binding = convert_binding(&GatewayBindingCompiled::SwaggerUI(swagger_binding));
        
        match binding {
            BindingType::SwaggerUI { spec_path, .. } => {
                assert_eq!(spec_path, "/api/spec");
            },
            _ => panic!("Expected SwaggerUI binding type"),
        }
    }
}