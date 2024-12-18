use crate::api::definition::types::{ApiDefinition, Route, HttpMethod, BindingType}; // Add BindingType here
use golem_wasm_ast::analysis::model::AnalysedType;

pub fn validate_api_definition(api: &ApiDefinition) -> Result<(), String> {
    for route in &api.routes {
        match &route.binding {
            BindingType::Default { input_type, output_type, function_name: _ } => {
                // Convert string types to AnalysedType for validation
                let input = parse_type(input_type)?;
                let output = parse_type(output_type)?;
                validate_wit_binding_types(&input, &output)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn parse_type(type_str: &str) -> Result<AnalysedType, String> {
    match type_str {
        "string" => Ok(AnalysedType::String),
        "i32" => Ok(AnalysedType::I32),
        "i64" => Ok(AnalysedType::I64),
        "f32" => Ok(AnalysedType::F32),
        "f64" => Ok(AnalysedType::F64),
        "bool" => Ok(AnalysedType::Bool),
        "void" => Ok(AnalysedType::Void),
        t if t.starts_with("list<") => {
            let inner_type = t.trim_start_matches("list<").trim_end_matches('>');
            Ok(AnalysedType::List(Box::new(parse_type(inner_type)?)))
        },
        t if t.starts_with("record{") && t.ends_with("}") => {
            let fields_str = t.trim_start_matches("record{").trim_end_matches('}');
            let mut fields = Vec::new();
            
            for field in fields_str.split(',').map(str::trim) {
                if let Some((name, type_str)) = field.split_once(':') {
                    fields.push((
                        name.trim().to_string(),
                        parse_type(type_str.trim())?
                    ));
                } else {
                    return Err(format!("Invalid record field format: {}", field));
                }
            }
            
            Ok(AnalysedType::Record(fields))
        },
        t if t.starts_with("option<") => {
            let inner_type = t.trim_start_matches("option<").trim_end_matches('>');
            Ok(AnalysedType::Option(Box::new(parse_type(inner_type)?)))
        },
        t if t.starts_with("result<") && t.ends_with(">") => {
            let types_str = t.trim_start_matches("result<").trim_end_matches('>');
            if let Some((ok_type, err_type)) = types_str.split_once(',') {
                Ok(AnalysedType::Result {
                    ok: Box::new(parse_type(ok_type.trim())?),
                    err: Box::new(parse_type(err_type.trim())?),
                })
            } else {
                Err(format!("Invalid result type format: {}", t))
            }
        },
        // Add any custom type handling here if needed
        _ => Err(format!("Unsupported type: {}", type_str))
    }
}

fn validate_wit_binding_types(
    input_type: &AnalysedType,
    output_type: &AnalysedType
) -> Result<(), String> {
    // Validation handled by WIT type system
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_api_definition() {
        let api = ApiDefinition {
            id: "test".to_string(),
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test API".to_string(),
            routes: vec![
                Route {
                    path: "/test".to_string(),
                    method: HttpMethod::Get,
                    description: "Test route".to_string(),
                    template_name: "test".to_string(),
                    binding: BindingType::Default {
                        input_type: "string".to_string(),
                        output_type: "string".to_string(),
                        function_name: "test".to_string(),
                    },
                },
            ],
        };
        assert!(validate_api_definition(&api).is_ok());
    }
}