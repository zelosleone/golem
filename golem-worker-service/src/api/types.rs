use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use golem_wasm_ast::analysis::AnalysedType;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAPISpec {
    pub openapi: String,
    pub info: Info,
    pub paths: HashMap<String, PathItem>,
    pub components: Option<Components>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub title: String,
    pub description: Option<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub parameters: Vec<Parameter>,
    pub request_body: Option<RequestBody>,
    pub responses: HashMap<String, Response>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub r#in: String,
    pub description: Option<String>,
    pub required: bool,
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: bool,
    pub content: HashMap<String, MediaType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub description: String,
    pub content: Option<HashMap<String, MediaType>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub r#type: Option<String>,
    pub format: Option<String>,
    pub properties: Option<HashMap<String, Schema>>,
    pub items: Option<Box<Schema>>,
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
    pub parameters: Option<HashMap<String, Parameter>>,
    pub responses: Option<HashMap<String, Response>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        content: String,
    },
}

impl std::fmt::Display for BindingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindingType::Default { function_name, input_type, output_type } => {
                write!(f, "Default({}, {:?}, {:?})", function_name, input_type, output_type)
            }
            BindingType::Worker { function_name, input_type, output_type } => {
                write!(f, "Worker({}, {:?}, {:?})", function_name, input_type, output_type)
            }
            BindingType::FileServer { root_dir } => {
                write!(f, "FileServer({})", root_dir)
            }
            BindingType::SwaggerUI { spec_path } => {
                write!(f, "SwaggerUI({})", spec_path)
            }
            BindingType::Static { content_type, content } => {
                write!(f, "Static({}, {})", content_type, content)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingOptions {
    pub auth: Option<String>,
    pub cache: Option<String>,
    pub cors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileServerOptions {
    pub cache: Option<String>,
    pub cors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwaggerUIOptions {
    pub title: Option<String>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeInfo {
    pub type_name: String,
    pub analysed_type: AnalysedType,
}