// Copyright 2024 Golem Cloud
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use error::ServiceError;
pub use openapi_export::export_openapi;

use std::sync::Arc;
use axum::Router;
use golem_worker_service_base::{
    config::worker::Config as WorkerServiceConfig,
    gateway_binding::GatewayBindingCompiled,
    service::worker::WorkerServiceDefault,
    routing::table::RoutingTableServiceDefault,
};
use golem_common::model::RetryConfig;
use golem_worker_service_base::routing::RoutingTableServiceDefault;

pub mod error;
pub mod openapi_export;
pub mod swagger;
pub mod swagger_ui;

use crate::api::RedisCache;
use crate::service::swagger::SwaggerGenerator;

pub struct Services {
    pub worker_service: Arc<WorkerServiceDefault>,
    pub swagger_generator: Arc<SwaggerGenerator>,
    pub cache: Arc<RedisCache>,
    binding: Arc<GatewayBindingCompiled>,
    pub router: Router,
}

impl Services {
    pub async fn new(config: &WorkerServiceConfig, binding: GatewayBindingCompiled) -> Result<Self, error::ServiceError> {
        let binding = Arc::new(binding);

        let worker_service = Arc::new(WorkerServiceDefault::new(
            config.clone(),
            RetryConfig::default(),
            Arc::new(RoutingTableServiceDefault::new()),
        ));

        let swagger_generator = Arc::new(SwaggerGenerator::new("/swagger".to_string()));
        let cache = Arc::new(RedisCache);

        // Create base router without state
        let base_router = Router::new()
            .merge(swagger_generator.create_router());

        // Create API router with state
        let api_router = crate::api::create_api_router()
            .with_state(Arc::clone(&binding));

        // Combine routers
        let router = base_router.nest("/api", api_router);

        Ok(Self {
            worker_service,
            swagger_generator,
            cache,
            binding,
            router,
        })
    }

    pub async fn start(self) -> Result<(), String> {
        Ok(())
    }
}