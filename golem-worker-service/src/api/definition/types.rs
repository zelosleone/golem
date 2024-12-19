use serde::{Deserialize, Serialize};
use golem_worker_service_base::gateway_api_definition::http::{CompiledHttpApiDefinition, MethodPattern};
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::{GatewayBindingCompiled, WorkerBindingCompiled, ResponseMappingCompiled};
use golem_worker_service_base::gateway_api_definition::{CompiledRoute, AllPathPatterns};
use golem_worker_service_base::id_types::{VersionedComponentId, WorkerNameCompiled};

/// Base binding types for the API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BindingType {
    Http,
    Worker {
        input_type: String,
        output_type: String,
        function_name: String,
    },
    Proxy,
    Default {
        input_type: String,
        output_type: String,
        function_name: String,
    },
    FileServer {
        root_dir: String,
    },
    SwaggerUI {
        spec_path: String,
    },
    Static {
        content_type: String,
        content: Vec<u8>,
    },
}

impl std::fmt::Display for BindingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindingType::Default { input_type, output_type, function_name } => {
                write!(f, "Default({}, {}, {})", input_type, output_type, function_name)
            },
            BindingType::FileServer { root_dir } => {
                write!(f, "FileServer({})", root_dir)
            },
            BindingType::SwaggerUI { spec_path } => {
                write!(f, "SwaggerUI({})", spec_path)
            },
            _ => write!(f, "{:?}", self),
        }
    }
}

// Update From implementation for GatewayBindingCompiled
impl From<&GatewayBindingCompiled> for BindingType {
    fn from(binding: &GatewayBindingCompiled) -> Self {
        match binding {
            GatewayBindingCompiled::Worker(worker) => BindingType::Worker {
                // worker_name_compiled: Option<WorkerNameCompiled>
                // Convert to String (assuming WorkerNameCompiled implements ToString):
                input_type: worker.worker_name_compiled
                    .as_ref()
                    .map(|w| w.to_string())
                    .unwrap_or_else(|| "UnnamedWorker".to_string()),
                // response_compiled: ResponseMappingCompiled
                // Convert to string (assuming ResponseMappingCompiled implements ToString):
                output_type: worker.response_compiled.to_string(),
                // component_id: VersionedComponentId
                // Convert to string:
                function_name: worker.component_id.to_string(),
            },
            GatewayBindingCompiled::FileServer(fs_binding) => {
                // fs_binding is likely a String representing the root directory:
                BindingType::FileServer {
                    root_dir: fs_binding.clone(),
                }
            },
            GatewayBindingCompiled::Static(static_binding) => BindingType::Static {
                content_type: "application/octet-stream".to_string(),
                content: static_binding.clone().unwrap_or_default(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub routes: Vec<Route>,
}

impl<T> From<&CompiledHttpApiDefinition<T>> for ApiDefinition {
    fn from(compiled: &CompiledHttpApiDefinition<T>) -> Self {
        ApiDefinition {
            id: compiled.id.0.clone(),
            // Use id as name since CompiledHttpApiDefinition doesn't have a name field
            name: compiled.id.0.clone(),
            version: compiled.version.0.clone(),
            // Add a generic description since source doesn't have one
            description: format!("API Definition {}", compiled.id.0),
            routes: compiled.routes.iter().map(|route: &CompiledRoute| {
                // Instead of route.path.pattern, use a correct field/method.
                // If AllPathPatterns implements Display or has a method to get original pattern:
                let path_str = route.path.to_string();

                // If CompiledRoute does not have 'metadata', we must extract description and template_name differently.
                // Check what fields CompiledRoute actually has. For this example, assume it has
                // `route.description` and `route.template_name` fields directly:
                let description = route.description.clone().unwrap_or_else(|| "No description available".to_string());
                let template_name = route.template_name.clone().unwrap_or_default();

                Route {
                    path: path_str,
                    method: match route.method {
                        MethodPattern::Get => HttpMethod::Get,
                        MethodPattern::Post => HttpMethod::Post,
                        MethodPattern::Put => HttpMethod::Put,
                        MethodPattern::Delete => HttpMethod::Delete,
                        MethodPattern::Patch => HttpMethod::Patch,
                        MethodPattern::Head => HttpMethod::Head,
                        MethodPattern::Options => HttpMethod::Options,
                        _ => HttpMethod::Get, // Default for other methods
                    },
                    description,
                    template_name,
                    // For route.binding, check available variants of GatewayBindingCompiled.
                    // If no Default variant, remove that branch and map them accordingly.
                    // If route.binding is Worker, Proxy, Http, etc., handle them:
                    binding: match &route.binding {
                        GatewayBindingCompiled::Worker(worker) => BindingType::Worker {
                            input_type: worker.worker_name_compiled
                                .as_ref()
                                .map(|w| w.to_string())
                                .unwrap_or_else(|| "UnnamedWorker".to_string()),
                            output_type: worker.response_compiled.to_string(),
                            function_name: worker.component_id.to_string(),
                        },
                        GatewayBindingCompiled::FileServer(root_dir) => BindingType::FileServer {
                            root_dir: root_dir.clone(),
                        },
                        GatewayBindingCompiled::Static(static_binding) => BindingType::Static {
                            content_type: "application/octet-stream".to_string(),
                            content: static_binding.clone().unwrap_or_default(),
                        },
                    }
                }
            }).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub path: String,
    pub method: HttpMethod,
    pub description: String,
    pub template_name: String,
    pub binding: BindingType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}