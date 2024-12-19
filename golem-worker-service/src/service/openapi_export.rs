use crate::api::definition::{ApiDefinition, Route, HttpMethod, BindingType};
use crate::api::openapi::{OpenAPIConverter, OpenAPISpec, validate_openapi, OpenAPIError};
use golem_service_base::auth::{EmptyAuthCtx, DefaultNamespace};
use golem_worker_service_base::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use golem_worker_service_base::gateway_api_definition::http::MethodPattern;
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::GatewayBindingCompiled;
use axum::{
    extract::{Path, State},
    Json,
    http::StatusCode,
};
use tracing::{error, info};
use crate::service::api::Cache;
use openapiv3::OpenAPI;

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
        // Provide a default case to handle unexpected variants
        _ => {
            error!("Unsupported HTTP method encountered: {:?}", method);
            HttpMethod::Get // Defaulting to GET; adjust as needed
        }
    }
}

fn convert_binding(binding: &GatewayBindingCompiled) -> BindingType {
    match binding {
        GatewayBindingCompiled::Worker(_) => BindingType::Worker,
        // Provide a default case to handle unexpected variants
        _ => {
            error!("Unsupported binding type encountered: {:?}", binding);
            BindingType::Worker // Defaulting to Worker; adjust as needed
        }
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
        &namespace,
        &EmptyAuthCtx::default(),
    ).await?;

    let spec = OpenAPIConverter::convert(&api_def);

    // Validate the spec
    validate_openapi(&spec)?;

    // Cache the valid spec
    services.cache.set(&cache_key, &spec).await?;

    Ok(Json(spec))
}