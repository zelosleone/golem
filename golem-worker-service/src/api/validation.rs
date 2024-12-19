use crate::api::definition::types::BindingType;
use crate::api::ApiDefinition;
use golem_wasm_ast::analysis::analysed_type::AnalysedType;

pub fn validate_api_definition(api: &ApiDefinition) -> Result<(), String> {
    for route in &api.routes {
        match &route.binding {
            BindingType::Default { input_type, output_type, .. } | 
            BindingType::Worker { input_type, output_type, .. } => {
                // Validate input type
                if !is_valid_type(input_type) {
                    return Err(format!("Invalid input type for route {}: {:?}", route.path, input_type));
                }
                // Validate output type
                if !is_valid_type(output_type) {
                    return Err(format!("Invalid output type for route {}: {:?}", route.path, output_type));
                }
            },
            BindingType::FileServer { .. } | 
            BindingType::SwaggerUI { .. } | 
            BindingType::Static { .. } => {
                // These bindings don't need type validation
            }
        }
    }
    Ok(())
}

fn is_valid_type(typ: &AnalysedType) -> bool {
    // Add your type validation logic here
    true // For now, accept all types
}

#[cfg(test)]
mod tests {
    use super::*;
    use golem_wasm_ast::analysis::analysed_type::{AnalysedType, TypeId};
    
    #[test]
    fn test_valid_api_definition() {
        let api = ApiDefinition {
            routes: vec![
                Route {
                    path: "/test".to_string(),
                    method: HttpMethod::Get,
                    binding: BindingType::Default {
                        input_type: AnalysedType::String,
                        output_type: AnalysedType::Record(vec![
                            ("field".to_string(), AnalysedType::I32)
                        ]),
                        options: None,
                    },
                },
            ],
        };
        assert!(validate_api_definition(&api).is_ok());
    }
}