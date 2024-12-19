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

use golem_common::tracing::init_tracing_with_default_env_filter;
use golem_service_base::migration::MigrationsDir;
use golem_worker_service_base::{
    app_config::WorkerServiceBaseConfig,
    gateway_binding::GatewayBindingCompiled,
    metrics,
};
use opentelemetry::global;
use prometheus::Registry;
use tokio::task::JoinSet;
use golem_worker_service::start_service;

fn main() -> Result<(), anyhow::Error> {
    if std::env::args().any(|arg| arg == "--dump-openapi-yaml") {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(dump_openapi_yaml())
    } else {
        let config = WorkerServiceBaseConfig::default();
        init_tracing_with_default_env_filter(&config.tracing);

        let prometheus = metrics::register_all();

        let exporter = opentelemetry_prometheus::exporter()
            .with_registry(prometheus.clone())
            .build()?;

        global::set_meter_provider(
            opentelemetry_sdk::metrics::MeterProviderBuilder::default()
                .with_reader(exporter)
                .build(),
        );

        Ok(tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async_main(config))?)
    }
}

async fn async_main(config: WorkerServiceBaseConfig) -> Result<(), anyhow::Error> {
    let binding = GatewayBindingCompiled::Worker(Default::default());
    start_service(config, binding).await
}

async fn dump_openapi_yaml() -> Result<(), anyhow::Error> {
    let config = WorkerServiceBaseConfig::default();
    let binding = GatewayBindingCompiled::Worker(Default::default());
    let services = golem_worker_service::service::Services::new(&config, &binding)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create services: {}", e))?;
    let yaml = services.swagger_generator.spec_yaml();
    println!("{yaml}");
    Ok(())
}
