use crate::gateway_binding::GatewayBinding;
use crate::gateway_middleware::{HttpCors, HttpMiddlewares};
use crate::gateway_security::SecuritySchemeReference;
use golem_service_base::model::{Component, VersionedComponentId};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub struct HttpApiDefinition {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<Route>,
    pub draft: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl HttpApiDefinition {
    pub async fn from_http_api_definition_request<Namespace: Display>(
        namespace: &Namespace,
        request: HttpApiDefinitionRequest,
        created_at: chrono::DateTime<chrono::Utc>,
        security_scheme_service: &Arc<dyn SecuritySchemeService<Namespace> + Send + Sync>,
    ) -> Result<Self, ApiDefinitionError> {
        let mut routes = Vec::new();
        
        for route_request in request.routes {
            let middlewares = if let Some(security) = route_request.security {
                let security_middleware = security_scheme_service
                    .get(namespace, &security)
                    .await
                    .map_err(ApiDefinitionError::SecuritySchemeError)?;

                Some(HttpMiddlewares {
                    cors: route_request.cors,
                    security: Some(security_middleware),
                })
            } else {
                route_request.cors.map(|cors| HttpMiddlewares {
                    cors: Some(cors),
                    security: None,
                })
            };

            routes.push(Route {
                method: route_request.method,
                path: route_request.path,
                binding: route_request.binding,
                middlewares,
            });
        }

        Ok(Self {
            id: request.id,
            version: request.version,
            routes,
            draft: true,
            created_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledHttpApiDefinition<Namespace> {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<CompiledRoute>,
    pub draft: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub namespace: Namespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpApiDefinitionRequest {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<RouteRequest>,
}

impl HttpApiDefinitionRequest {
    pub fn new(id: ApiDefinitionId, version: ApiVersion, routes: Vec<RouteRequest>) -> Self {
        Self {
            id,
            version,
            routes,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComponentMetadataDictionary {
    metadata: std::collections::HashMap<VersionedComponentId, Component>,
}

impl ComponentMetadataDictionary {
    pub fn from_components(components: &[Component]) -> Self {
        let metadata = components
            .iter()
            .map(|c| (c.id.clone(), c.clone()))
            .collect();
        Self { metadata }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarInfo {
    pub key_name: String,
}

impl VarInfo {
    pub fn new(key_name: String) -> Self {
        Self { key_name }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryInfo {
    pub key_name: String,
}

impl QueryInfo {
    pub fn new(key_name: String) -> Self {
        Self { key_name }
    }
}

impl Display for QueryInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key_name)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathPattern {
    Literal(LiteralPattern),
    Var(VarInfo),
    CatchAllVar(VarInfo),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiteralPattern(pub String);

#[derive(Debug, Clone)]
pub enum RouteCompilationErrors {
    RibCompilationError(String),
    MetadataNotFoundError(VersionedComponentId),
}

impl From<String> for RouteCompilationErrors {
    fn from(error: String) -> Self {
        Self {
            rib_compilation_error: error,
            metadata_not_found_error: None,
        }
    }
}

impl From<VersionedComponentId> for RouteCompilationErrors {
    fn from(component_id: VersionedComponentId) -> Self {
        Self {
            rib_compilation_error: String::new(),
            metadata_not_found_error: Some(component_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkerIdGenerateError(pub String);

impl From<WorkerIdGenerateError> for String {
    fn from(value: WorkerIdGenerateError) -> Self {
        value.0
    }
}
