use crate::api::definition::{HttpMethod, BindingType};
use crate::api::openapi::{OpenAPIConverter, validate_openapi, OpenAPIError};
use golem_service_base::auth::EmptyAuthCtx;
use golem_worker_service_base::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use golem_worker_service_base::gateway_api_definition::http::MethodPattern;
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::GatewayBindingCompiled;
use golem_worker_service_base::service::gateway::api_definition::ApiDefinitionError;
use golem_service_base::cache::CacheError;
use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use tracing::{error, info};
use crate::service::api::Cache;
use openapiv3::OpenAPI;

#[derive(Debug)]
struct ApiError(StatusCode);

impl From<ApiDefinitionError> for ApiError {
    fn from(err: ApiDefinitionError) -> Self {
        error!("API definition error: {}", err);
        ApiError(StatusCode::NOT_FOUND)
    }
}

impl From<CacheError> for ApiError {
    fn from(err: CacheError) -> Self {
        error!("Cache error: {}", err);
        ApiError(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl From<ApiError> for StatusCode {
    fn from(err: ApiError) -> Self {
        err.0
    }
}

impl From<CacheError> for StatusCode {
    fn from(err: CacheError) -> Self {
        error!("Cache error: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<ApiDefinitionError> for StatusCode {
    fn from(err: ApiDefinitionError) -> Self {
        error!("API definition error: {}", err);
        StatusCode::NOT_FOUND
    }
}

impl From<OpenAPIError> for StatusCode {
    fn from(err: OpenAPIError) -> Self {
        match err {
            OpenAPIError::InvalidDefinition(_) => StatusCode::BAD_REQUEST,
            OpenAPIError::ValidationFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OpenAPIError::CacheError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            OpenAPIError::SchemaMismatch { .. } => StatusCode::BAD_REQUEST,
        }
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
    }
}

pub async fn export_openapi(
    State(services): State<crate::service::Services>,
    Path((id, version)): Path<(String, String)>,
) -> Result<Json<OpenAPI>, StatusCode> {
    info!("Requesting OpenAPI spec for API {}, version {}", id, version);

    // Try to get from cache first
    let cache_key = format!("openapi:{}:{}", id, version);
    if let Some(cached_spec) = services.cache.get(&cache_key).await? {
        return Ok(Json(cached_spec));
    }

    // Convert API definition to OpenAPI spec
    let api_def = services.definition_service.get(
        &ApiDefinitionId(id.clone()),
        &ApiVersion(version.clone()),
        &EmptyAuthCtx::default(),
        &EmptyAuthCtx::default(),
    ).await?;

    let spec = OpenAPIConverter::convert(&api_def);

    // Validate the spec
    validate_openapi(&spec).map_err(|e| {
        error!("OpenAPI validation failed: {}", e);
        StatusCode::from(e)
    })?;

    // Cache the valid spec
    services.cache.set(&cache_key, &spec).await?;

    Ok(Json(spec))
}