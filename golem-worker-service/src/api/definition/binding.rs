use golem_wasm_ast::analysis::AnalysedType;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Extended binding types for the worker service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BindingType {
    #[serde(rename = "Default")]
    Default {
        input_type: AnalysedType,
        output_type: AnalysedType,
        options: Option<String>,
    },
    #[serde(rename = "Worker")]
    Worker {
        function_name: String,
        input_type: AnalysedType,
        output_type: AnalysedType,
    },
    #[serde(rename = "FileServer")]
    FileServer {
        root_dir: String,
    },
    /// Swagger UI binding for API documentation
    #[serde(rename = "SwaggerUI")]
    SwaggerUI,
    #[serde(rename = "Static")]
    Static {
        content_type: String,
        content: String,
    },
}

#[derive(Debug, Clone)]
pub enum BaseBindingType {
    Default {
        input_type: AnalysedType,
        output_type: AnalysedType,
        options: Option<String>,
    },
    Worker {
        function_name: String,
        input_type: AnalysedType,
        output_type: AnalysedType,
    },
    FileServer {
        root_dir: String,
    },
    SwaggerUI,
    Static {
        content_type: String,
        content: String,
    },
}

impl From<BaseBindingType> for BindingType {
    fn from(binding: BaseBindingType) -> Self {
        match binding {
            BaseBindingType::Default { input_type, output_type, options } => {
                BindingType::Default {
                    input_type,
                    output_type,
                    options,
                }
            }
            BaseBindingType::Worker { function_name, input_type, output_type } => {
                BindingType::Worker {
                    function_name,
                    input_type,
                    output_type,
                }
            }
            BaseBindingType::FileServer { root_dir } => {
                BindingType::FileServer {
                    root_dir,
                }
            }
            BaseBindingType::Static { content_type, content } => {
                BindingType::Static {
                    content_type,
                    content,
                }
            }
            BaseBindingType::SwaggerUI => BindingType::SwaggerUI,
        }
    }
}

impl From<BindingType> for BaseBindingType {
    fn from(binding: BindingType) -> Self {
        match binding {
            BindingType::Default { input_type, output_type, options } => 
                BaseBindingType::Default { input_type, output_type, options },
            BindingType::Worker { function_name, input_type, output_type } => 
                BaseBindingType::Worker { function_name, input_type, output_type },
            BindingType::FileServer { root_dir } => 
                BaseBindingType::FileServer { root_dir },
            BindingType::SwaggerUI => 
                BaseBindingType::SwaggerUI,
            BindingType::Static { content_type, content } => 
                BaseBindingType::Static { content_type, content },
        }
    }
}

impl Display for BindingType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BindingType::Default { input_type, output_type, options } => {
                write!(f, "Default({:?}, {:?}, {:?})", input_type, output_type, options)
            }
            BindingType::Worker { function_name, input_type, output_type } => {
                write!(f, "Worker({}, {:?}, {:?})", function_name, input_type, output_type)
            }
            BindingType::FileServer { root_dir } => {
                write!(f, "FileServer({})", root_dir)
            }
            BindingType::SwaggerUI => {
                write!(f, "SwaggerUI")
            }
            BindingType::Static { content_type, content } => {
                write!(f, "Static({}, {})", content_type, content)
            }
        }
    }
}

pub fn convert_binding(binding: &BaseBindingType) -> BindingType {
    match binding {
        BaseBindingType::Default { input_type, output_type, options } => {
            BindingType::Default {
                input_type: input_type.clone(),
                output_type: output_type.clone(),
                options: options.clone(),
            }
        }
        BaseBindingType::Worker { function_name, input_type, output_type } => {
            BindingType::Worker {
                function_name: function_name.clone(),
                input_type: input_type.clone(),
                output_type: output_type.clone(),
            }
        }
        BaseBindingType::FileServer { root_dir } => {
            BindingType::FileServer {
                root_dir: root_dir.clone(),
            }
        }
        BaseBindingType::Static { content_type, content } => {
            BindingType::Static {
                content_type: content_type.clone(),
                content: content.clone(),
            }
        }
        BaseBindingType::SwaggerUI => {
            BindingType::SwaggerUI
        }
    }
}
