use std::sync::Arc;
use axum::Router;
use axum::Json;
use axum::extract::State;
use crate::service::ServiceError;
use openapiv3::OpenAPI;
use golem_worker_service_base::{
    gateway_binding::GatewayBindingCompiled,
    gateway_api_definition::http::CompiledHttpApiDefinition,
};

pub mod openapi;
pub mod redis;

pub use redis::RedisCache;

pub async fn export_openapi(binding: &GatewayBindingCompiled) -> Result<Json<OpenAPI>, ServiceError> {
    create_openapi_spec(binding).map(Json)
}

pub fn create_api_router() -> Router<Arc<GatewayBindingCompiled>> {
    Router::new()
        .route(
            "/v1/api/definitions/:id/version/:version/export",
            axum::routing::get(|State(binding): State<Arc<GatewayBindingCompiled>>| async move {
                export_openapi(&binding).await
            }),
        )
}

pub fn create_openapi_spec(binding: &GatewayBindingCompiled) -> Result<openapiv3::OpenAPI, String> {
    let converter = openapi::converter::OpenAPIConverter::new();
    let api_def = CompiledHttpApiDefinition::try_from(binding)
        .map_err(|e| format!("Failed to convert binding to API definition: {}", e))?;
    converter.convert_api_definition(&api_def)
}