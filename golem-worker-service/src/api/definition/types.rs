use serde::{Deserialize, Serialize};
use golem_worker_service_base::gateway_api_definition::http::CompiledHttpApiDefinition;

/// Base binding types for the API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BindingType {
    Http,
    Worker,
    Proxy,
    #[serde(rename = "Default")]
    Default {
        input_type: String,
        output_type: String,
        function_name: String,
    },
    #[serde(rename = "FileServer")]
    FileServer {
        root_dir: String,
    },
    #[serde(rename = "SwaggerUI")]
    SwaggerUI {
        spec_path: String,
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
            routes: compiled.routes.iter().map(|route| Route {
                path: route.path.pattern.to_string(),
                method: match route.method {
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Get => HttpMethod::Get,
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Post => HttpMethod::Post,
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Put => HttpMethod::Put,
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Delete => HttpMethod::Delete,
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Patch => HttpMethod::Patch,
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Head => HttpMethod::Head,
                    golem_worker_service_base::gateway_api_definition::http::MethodPattern::Options => HttpMethod::Options,
                    _ => HttpMethod::Get, // Default for other methods
                },
                description: route.metadata.description.clone()
                    .unwrap_or_else(|| String::from("No description available")),
                template_name: route.metadata.template_name.clone(),
                binding: match &route.binding {
                    golem_worker_service_base::gateway_binding::GatewayBinding::Default { input_type, output_type, function_name } => {
                        BindingType::Default {
                            input_type: input_type.to_string(),
                            output_type: output_type.to_string(),
                            function_name: function_name.clone(),
                        }
                    },
                    golem_worker_service_base::gateway_binding::GatewayBinding::FileServer { root_dir } => {
                        BindingType::FileServer {
                            root_dir: root_dir.clone(),
                        }
                    },
                    golem_worker_service_base::gateway_binding::GatewayBinding::SwaggerUI { spec_path } => {
                        BindingType::SwaggerUI {
                            spec_path: spec_path.clone(),
                        }
                    },
                    _ => BindingType::Http, // Default for other bindings
                },
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