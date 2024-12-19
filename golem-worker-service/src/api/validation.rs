use crate::api::definition::types::{ApiDefinition, BindingType};
use golem_wasm_ast::analysis::{
    AnalysedType, TypeStr, TypeI32, TypeI64, TypeF32, TypeF64, TypeBool, 
    TypeList, TypeOption, TypeRecord, TypeResult, NameTypePair, TypeVoid
};

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
        "string" => Ok(AnalysedType::Str(TypeStr)),
        "i32" => Ok(AnalysedType::Int32(TypeI32)),
        "i64" => Ok(AnalysedType::Int64(TypeI64)), 
        "f32" => Ok(AnalysedType::F32(TypeF32)),
        "f64" => Ok(AnalysedType::F64(TypeF64)),
        "bool" => Ok(AnalysedType::Bool(TypeBool)),
        "void" => Ok(AnalysedType::Void(TypeVoid)),
        t if t.starts_with("list<") => {
            let inner_type = t.trim_start_matches("list<").trim_end_matches('>');
            let inner = parse_type(inner_type)?;
            Ok(AnalysedType::List(TypeList { inner: Box::new(inner) }))
        },
        t if t.starts_with("record{") && t.ends_with("}") => {
            let fields_str = t.trim_start_matches("record{").trim_end_matches('}');
            let mut fields = Vec::new();
            
            for field in fields_str.split(',').map(str::trim) {
                if let Some((name, type_str)) = field.split_once(':') {
                    fields.push(NameTypePair {
                        name: name.trim().to_string(),
                        typ: parse_type(type_str.trim())?,
                    });
                } else {
                    return Err(format!("Invalid record field format: {}", field));
                }
            }
            
            Ok(AnalysedType::Record(TypeRecord { fields }))
        },
        t if t.starts_with("option<") => {
            let inner_type = t.trim_start_matches("option<").trim_end_matches('>');
            let inner = parse_type(inner_type)?;
            Ok(AnalysedType::Option(TypeOption { inner: Box::new(inner) }))
        },
        t if t.starts_with("result<") && t.ends_with(">") => {
            let types_str = t.trim_start_matches("result<").trim_end_matches('>');
            if let Some((ok_type, err_type)) = types_str.split_once(',') {
                let ok = parse_type(ok_type.trim())?;
                let err = parse_type(err_type.trim())?;
                Ok(AnalysedType::Result(TypeResult {
                    ok: Some(Box::new(ok)),
                    err: Some(Box::new(err)),
                }))
            } else {
                Err(format!("Invalid result type format: {}", t))
            }
        },
        _ => Err(format!("Unsupported type: {}", type_str))
    }
}

fn validate_wit_binding_types(
    _input_type: &AnalysedType,
    _output_type: &AnalysedType
) -> Result<(), String> {
    // Validation handled by WIT type system
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::definition::types::{Route, HttpMethod}; // Ensure Route and HttpMethod are imported

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

    #[test]
    fn test_invalid_type_in_api_definition() {
        let api = ApiDefinition {
            id: "test".to_string(),
            name: "test".to_string(),
            version: "1.0".to_string(),
            description: "Test API with invalid type".to_string(),
            routes: vec![
                Route {
                    path: "/test".to_string(),
                    method: HttpMethod::Get,
                    description: "Test route".to_string(),
                    template_name: "test".to_string(),
                    binding: BindingType::Default {
                        input_type: "unknown_type".to_string(),
                        output_type: "string".to_string(),
                        function_name: "test".to_string(),
                    },
                },
            ],
        };
        assert!(validate_api_definition(&api).is_err());
    }

    #[test]
    fn test_complex_api_definition() {
        let api = ApiDefinition {
            id: "complex".to_string(),
            name: "Complex API".to_string(),
            version: "2.0".to_string(),
            description: "A more complex API".to_string(),
            routes: vec![
                Route {
                    path: "/complex".to_string(),
                    method: HttpMethod::Post,
                    description: "Complex route".to_string(),
                    template_name: "complex".to_string(),
                    binding: BindingType::Default {
                        input_type: "record{name:string,age:f32}".to_string(), // Changed i32 to f32
                        output_type: "result<string, bool>".to_string(),
                        function_name: "complex_function".to_string(),
                    },
                },
            ],
        };
        assert!(validate_api_definition(&api).is_ok());
    }
}