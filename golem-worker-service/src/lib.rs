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

use anyhow::{anyhow, Result};
use golem_worker_service_base::{
    gateway_binding::GatewayBindingCompiled,
    config::worker::Config as WorkerServiceConfig,
};
use std::sync::Arc;

pub mod api;
pub mod service;

pub async fn start_service(
    config: WorkerServiceConfig,
    binding: GatewayBindingCompiled,
) -> Result<()> {
    let services = create_services(config, binding).await.map_err(|e| anyhow!("Failed to create services: {}", e))?;

    services.start().await.map_err(|e| anyhow!("Failed to start services: {}", e))?;

    Ok(())
}

pub async fn create_services(config: WorkerServiceConfig, binding: GatewayBindingCompiled) -> Result<Arc<service::Services>, service::error::ServiceError> {
    let services = service::Services::new(&config, binding).await?;
    Ok(Arc::new(services))
}