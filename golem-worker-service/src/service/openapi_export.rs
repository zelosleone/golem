use crate::api::definition::{HttpMethod, BindingType, ApiDefinition, WorkerBinding, StaticBinding};
use crate::api::openapi::{OpenAPIConverter, validate_openapi, OpenAPIError};
use crate::service::api::CacheError;
use golem_worker_service_base::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::GatewayBindingCompiled;
use golem_service_base::auth::EmptyAuthCtx;
use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use tracing::{error, info};
use crate::service::api::Cache;
use openapiv3::OpenAPI;

// Custom wrapper type for StatusCode to implement From for external types
#[derive(Debug)]
struct ApiStatusCode(StatusCode);

impl From<ApiStatusCode> for StatusCode {
    fn from(code: ApiStatusCode) -> Self {
        code.0
    }
}

impl From<CacheError> for ApiStatusCode {
    fn from(err: CacheError) -> Self {
        error!("Cache error: {}", err);
        ApiStatusCode(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<ApiDefinitionError> for ApiStatusCode {
    fn from(err: ApiDefinitionError) -> Self {
        error!("API definition error: {}", err);
        ApiStatusCode(StatusCode::NOT_FOUND)
    }
}

impl From<OpenAPIError> for ApiStatusCode {
    fn from(err: OpenAPIError) -> Self {
        let status = match err {
            OpenAPIError::InvalidDefinition(_) => StatusCode::BAD_REQUEST,
            OpenAPIError::ValidationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OpenAPIError::SchemaMismatch { .. } => StatusCode::BAD_REQUEST,
        };
        error!("OpenAPI error: {}", err);
        ApiStatusCode(status)
    }
}

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

/// Converts a MethodPattern to an HttpMethod
/// 
/// # Arguments
/// * `method` - The HTTP method pattern to convert
/// 
/// # Returns
/// Converted HttpMethod, with fallback to GET for unsupported methods
fn convert_method(method: &MethodPattern) -> HttpMethod {
    match method {
        MethodPattern::Get => HttpMethod::Get,
        MethodPattern::Post => HttpMethod::Post,
        MethodPattern::Put => HttpMethod::Put,
        MethodPattern::Delete => HttpMethod::Delete,
        MethodPattern::Patch => HttpMethod::Patch,
        MethodPattern::Head => HttpMethod::Head,
        MethodPattern::Options => HttpMethod::Options,
        MethodPattern::Connect => {
            error!("Connect method not supported in OpenAPI export");
            HttpMethod::Get // Fallback
        },
        MethodPattern::Trace => {
            error!("Trace method not supported in OpenAPI export"); 
            HttpMethod::Get // Fallback
        }
    }
}

fn convert_binding(binding: &GatewayBindingCompiled) -> BindingType {
    match binding {
        GatewayBindingCompiled::Worker(worker) => {
            BindingType::Worker {
                input_type: worker.request_type.to_string(),
                output_type: worker.response_type.to_string(),
                function_name: worker.name.clone(),
            }
        }
        GatewayBindingCompiled::Static(static_binding) => {
            BindingType::Static {
                content_type: static_binding.mime_type.clone(),
                content: static_binding.data.clone(),
            }
        }
        GatewayBindingCompiled::FileServer(fs) => {
            BindingType::FileServer {
                root_dir: fs.path.clone(),
            }
        }
    }
}

/// Exports an API definition as an OpenAPI specification
/// 
/// # Arguments
/// * `services` - Service container with cache and API definition service
/// * `id` - API definition ID
/// * `version` - API version
/// 
/// # Returns
/// JSON response containing OpenAPI specification
pub async fn export_openapi(
    State(services): State<crate::service::Services>,
    Path((id, version)): Path<(String, String)>,
) -> Result<Json<OpenAPI>, StatusCode> {
    info!("Requesting OpenAPI spec for API {}, version {}", id, version);
    
    let namespace = services.config.openapi.default_namespace.clone();
    
    // Try to get from cache first
    let cache_key = format!("openapi:{}:{}:{}", namespace, id, version);
    let cached_spec = services.cache.get(&cache_key).await
        .map_err(|e| ApiStatusCode(StatusCode::INTERNAL_SERVER_ERROR))?;
    
    if let Some(spec) = cached_spec {
        return Ok(Json(spec));
    }

    // Convert API definition to OpenAPI spec
    let api_def = services.definition_service.get(
        &ApiDefinitionId(id.clone()),
        &ApiVersion(version.clone()),
        &EmptyAuthCtx,
    ).await
    .map_err(Into::into)?
    .ok_or_else(|| ApiStatusCode(StatusCode::NOT_FOUND))?;

    // Convert CompiledHttpApiDefinition to ApiDefinition
    let converted_def = ApiDefinition::from(&api_def);
    let spec = OpenAPIConverter::new().convert(&converted_def);

    // Validate the spec
    validate_openapi(&spec.clone())
        .map_err(Into::into)?;

    // Cache the valid spec
    services.cache.set(&cache_key, &spec).await
        .map_err(|e| ApiStatusCode(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(spec))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::definition::HttpMethod;

    #[test]
    fn test_convert_method() {
        assert_eq!(convert_method(&MethodPattern::Get), HttpMethod::Get);
        assert_eq!(convert_method(&MethodPattern::Post), HttpMethod::Post);
        assert_eq!(convert_method(&MethodPattern::Put), HttpMethod::Put);
        assert_eq!(convert_method(&MethodPattern::Delete), HttpMethod::Delete);
        assert_eq!(convert_method(&MethodPattern::Patch), HttpMethod::Patch);
        assert_eq!(convert_method(&MethodPattern::Head), HttpMethod::Head);
        assert_eq!(convert_method(&MethodPattern::Options), HttpMethod::Options);
        // Test fallback cases
        assert_eq!(convert_method(&MethodPattern::Connect), HttpMethod::Get);
        assert_eq!(convert_method(&MethodPattern::Trace), HttpMethod::Get);
    }

    #[test]
    fn test_convert_binding() {
        let worker_binding = GatewayBindingCompiled::Worker(WorkerBinding {
            name: "test_worker".to_string(),
            request_type: "Request".to_string(),
            response_type: "Response".to_string(),
        });
        
        let static_binding = GatewayBindingCompiled::Static(StaticBinding {
            mime_type: "text/plain".to_string(),
            data: vec![1, 2, 3],
        });

        match convert_binding(&worker_binding) {
            BindingType::Worker { function_name, input_type, output_type } => {
                assert_eq!(function_name, "test_worker");
                assert_eq!(input_type, "Request");
                assert_eq!(output_type, "Response");
            },
            _ => panic!("Expected Worker binding"),
        }

        match convert_binding(&static_binding) {
            BindingType::Static { content_type, content } => {
                assert_eq!(content_type, "text/plain");
                assert_eq!(content, vec![1, 2, 3]);
            },
            _ => panic!("Expected Static binding"),
        }
    }
}