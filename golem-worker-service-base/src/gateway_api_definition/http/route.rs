use crate::gateway_binding::{GatewayBinding, GatewayBindingCompiled};
use crate::gateway_middleware::{HttpCors, HttpMiddlewares};
use crate::gateway_security::SecuritySchemeReference;

use super::{AllPathPatterns, MethodPattern};

#[derive(Debug, Clone, PartialEq)]
pub struct Route {
    pub method: MethodPattern,
    pub path: AllPathPatterns,
    pub binding: GatewayBinding,
    pub middlewares: Option<HttpMiddlewares>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RouteRequest {
    pub method: MethodPattern,
    pub path: AllPathPatterns,
    pub binding: GatewayBinding,
    pub security: Option<SecuritySchemeReference>,
    pub cors: Option<HttpCors>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledRoute {
    pub method: MethodPattern,
    pub path: AllPathPatterns,
    pub binding: GatewayBindingCompiled,
    pub middlewares: Option<HttpMiddlewares>,
}
