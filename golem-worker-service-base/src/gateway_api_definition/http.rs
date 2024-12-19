use crate::gateway_binding::gateway_binding_compiled::GatewayBindingCompiled;

#[derive(Debug, Clone)]
pub struct CompiledRoute {
    pub path: String,
    pub method: MethodPattern,
    pub binding: GatewayBindingCompiled,
    description: Option<String>,
    template_name: Option<String>,
}

impl CompiledRoute {
    pub fn get_description(&self) -> Option<String> {
        self.description.clone()
    }

    pub fn get_template_name(&self) -> Option<String> {
        self.template_name.clone()
    }
}

impl std::fmt::Display for CompiledRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.method, self.path)
    }
}
