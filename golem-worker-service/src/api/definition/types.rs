use serde::{Deserialize, Serialize};
use golem_wasm_ast::analysis::AnalysedType;

/// Base binding types for the API Gateway
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BindingType {
    Default {
        function_name: String,
        input_type: AnalysedType,
        output_type: AnalysedType,
    },
    Worker {
        function_name: String,
        input_type: AnalysedType,
        output_type: AnalysedType,
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
            BindingType::Default { function_name, input_type, output_type } => {
                write!(f, "Default({}, {:?}, {:?})", function_name, input_type, output_type)
            },
            BindingType::Worker { function_name, input_type, output_type } => {
                write!(f, "Worker({}, {:?}, {:?})", function_name, input_type, output_type)
            },
            BindingType::FileServer { root_dir } => {
                write!(f, "FileServer({})", root_dir)
            },
            BindingType::SwaggerUI { spec_path } => {
                write!(f, "SwaggerUI({})", spec_path)
            },
            BindingType::Static { content_type, content } => {
                write!(f, "Static({}, {:?})", content_type, content)
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