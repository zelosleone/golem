use crate::api::definition::{HttpMethod, BindingType, ApiDefinition};
use crate::api::openapi::{OpenAPIConverter, validate_openapi, OpenAPIError};
use crate::api::api::CacheError;
use golem_worker_service_base::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use golem_worker_service_base::gateway_api_definition::http::{MethodPattern, CompiledHttpApiDefinition};
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::GatewayBindingCompiled;
use golem_worker_service_base::service::gateway::api_definition::ApiDefinitionError;
use golem_service_base::auth::DefaultNamespace;
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

fn convert_method(method: &MethodPattern) -> HttpMethod {
    match method {
        MethodPattern::Get => HttpMethod::Get,
        MethodPattern::Post => HttpMethod::Post,
        MethodPattern::Put => HttpMethod::Put,
        MethodPattern::Delete => HttpMethod::Delete,
        MethodPattern::Patch => HttpMethod::Patch,
        MethodPattern::Head => HttpMethod::Head,
        MethodPattern::Options => HttpMethod::Options,
        _ => {
            error!("Unsupported HTTP method encountered: {:?}", method);
            HttpMethod::Get
        }
    }
}

fn convert_binding(binding: &GatewayBindingCompiled) -> BindingType {
    match binding {
        GatewayBindingCompiled::Worker(_) => BindingType::Worker,
        GatewayBindingCompiled::Static(_) => BindingType::Worker, // Changed to Worker since Static isn't available
        GatewayBindingCompiled::FileServer(_) => BindingType::Worker,
    }
}

pub async fn export_openapi(
    State(services): State<crate::service::Services>,
    Path((id, version)): Path<(String, String)>,
) -> Result<Json<OpenAPI>, StatusCode> {
    info!("Requesting OpenAPI spec for API {}, version {}", id, version);

    // Try to get from cache first
    let cache_key = format!("openapi:{}:{}", id, version);
    let cached_spec = services.cache.get(&cache_key).await
        .map_err(|e| ApiStatusCode::from(e))?;
    
    if let Some(spec) = cached_spec {
        return Ok(Json(spec));
    }

    // Convert API definition to OpenAPI spec
    let api_def = services.definition_service.get(
        &ApiDefinitionId(id.clone()),
        &ApiVersion(version.clone()),
        &DefaultNamespace::default(),
        &DefaultNamespace::default(),
    ).await
    .map_err(|e| ApiStatusCode::from(e))?
    .ok_or_else(|| ApiStatusCode(StatusCode::NOT_FOUND))?;

    // Convert CompiledHttpApiDefinition to ApiDefinition
    let converted_def = ApiDefinition::from(&api_def);
    let spec = OpenAPIConverter::convert(&converted_def);

    // Validate the spec
    validate_openapi(&spec.clone())
        .map_err(|e| ApiStatusCode::from(e))?;

    // Cache the valid spec
    services.cache.set(&cache_key, &spec).await
        .map_err(|e| ApiStatusCode::from(e))?;

    Ok(Json(spec))
}