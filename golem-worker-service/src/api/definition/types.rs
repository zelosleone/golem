use serde::{Deserialize, Serialize};
use golem_worker_service_base::gateway_api_definition::http::{CompiledHttpApiDefinition, MethodPattern};
use golem_worker_service_base::gateway_binding::gateway_binding_compiled::GatewayBindingCompiled;
use golem_worker_service_base::gateway_binding::{StaticBinding, WorkerBinding};
use golem_worker_service_base::id_types::{VersionedComponentId, WorkerNameCompiled};
use std::str::FromStr;

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
                input_type: worker.worker_name
                    .as_ref()
                    .map(|w| w.name.clone())
                    .unwrap_or_else(|| "UnnamedWorker".to_string()),
                output_type: worker.response_type.clone(),
                function_name: worker.component_id.to_string(),
            },
            GatewayBindingCompiled::FileServer(fs_binding) => {
                BindingType::FileServer {
                    root_dir: fs_binding.clone(),
                }
            },
            GatewayBindingCompiled::Static(static_binding) => BindingType::Static {
                content_type: static_binding.content_type.clone(),
                content: static_binding.content.clone(),
            },
        }
    }
}

/// Represents an API definition with routes and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDefinition {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub routes: Vec<Route>,
}

impl ApiDefinition {
    /// Validates the API definition
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("API ID cannot be empty".to_string());
        }
        if self.version.is_empty() {
            return Err("Version cannot be empty".to_string());
        }
        if self.routes.is_empty() {
            return Err("API must have at least one route".to_string());
        }
        Ok(())
    }
}

impl<T> From<&CompiledHttpApiDefinition<T>> for ApiDefinition {
    fn from(compiled: &CompiledHttpApiDefinition<T>) -> Self {
        ApiDefinition {
            id: compiled.id.0.clone(),
            name: compiled.id.0.clone(),
            version: compiled.version.0.clone(),
            description: format!("API Definition {}", compiled.id.0),
            routes: compiled.routes.iter().map(|route| {
                Route {
                    path: route.path.pattern.clone(),
                    method: match route.method {
                        MethodPattern::Get => HttpMethod::Get,
                        MethodPattern::Post => HttpMethod::Post,
                        MethodPattern::Put => HttpMethod::Put,
                        MethodPattern::Delete => HttpMethod::Delete,
                        MethodPattern::Patch => HttpMethod::Patch,
                        MethodPattern::Head => HttpMethod::Head,
                        MethodPattern::Options => HttpMethod::Options,
                    },
                    description: route.metadata.description.clone()
                        .unwrap_or_else(|| "No description available".to_string()),
                    template_name: route.metadata.template_name.clone()
                        .unwrap_or_default(),
                    binding: BindingType::from(&route.binding),
                }
            }).collect(),
        }
    }
}

/// Represents a single route in the API definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// The HTTP path pattern for this route
    pub path: String,
    /// The HTTP method for this route
    pub method: HttpMethod,
    /// Human-readable description of the route
    pub description: String,
    /// Optional template name for the route
    pub template_name: String,
    /// The binding configuration for this route
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

impl FromStr for HttpMethod {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            _ => Err(format!("Invalid HTTP method: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_definition_validation() {
        let invalid_api = ApiDefinition {
            id: "".into(),
            name: "test".into(),
            version: "1.0".into(),
            description: "test".into(),
            routes: vec![],
        };
        assert!(invalid_api.validate().is_err());
    }

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::Get);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }
}