use crate::gateway_api_definition::{ApiDefinitionId, ApiVersion};
use crate::gateway_api_definition::http::route::{CompiledRoute, Route, RouteRequest};
use crate::gateway_security::SecuritySchemeReference;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HttpApiDefinition {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<Route>,
    #[serde(default)]
    pub draft: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HttpApiDefinitionRequest {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub security: Option<Vec<SecuritySchemeReference>>,
    pub routes: Vec<RouteRequest>,
    pub draft: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledHttpApiDefinition<N> {
    pub id: ApiDefinitionId,
    pub version: ApiVersion,
    pub routes: Vec<CompiledRoute>,
    pub draft: bool,
    pub created_at: DateTime<Utc>,
    pub namespace: N,
}
