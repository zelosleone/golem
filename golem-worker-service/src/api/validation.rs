use crate::api::definition::types::{ApiDefinition, BindingType};
use golem_wasm_ast::analysis::{
    AnalysedType, TypeStr, TypeF32, TypeF64, TypeBool, 
    TypeList, TypeOption, TypeRecord, TypeResult, NameTypePair
};

#[derive(Debug)]
enum TypeConstraint {
    Input,
    Output,
}

pub fn validate_api_definition(api: &ApiDefinition) -> Result<(), String> {
    for route in &api.routes {
        if let BindingType::Default { input_type, output_type, .. } = &route.binding {
            let input = parse_type(input_type)?;
            let output = parse_type(output_type)?;
            validate_wit_binding_types(&input, &output, route.path.as_str())?;
        }
    }
    Ok(())
}

fn parse_type(type_str: &str) -> Result<AnalysedType, String> {
    match type_str {
        "string" => Ok(AnalysedType::Str(TypeStr)),
        "i32" => Ok(AnalysedType::I32),
        "i64" => Ok(AnalysedType::I64),
        "f32" => Ok(AnalysedType::F32(TypeF32)),
        "f64" => Ok(AnalysedType::F64(TypeF64)),
        "bool" => Ok(AnalysedType::Bool(TypeBool)),
        "void" => Ok(AnalysedType::Unit),
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
    input_type: &AnalysedType,
    output_type: &AnalysedType,
    path: &str,
) -> Result<(), String> {
    // Validate input type constraints
    validate_type_constraints(input_type, TypeConstraint::Input)
        .map_err(|e| format!("Invalid input type for path {}: {}", path, e))?;

    // Validate output type constraints
    validate_type_constraints(output_type, TypeConstraint::Output)
        .map_err(|e| format!("Invalid output type for path {}: {}", path, e))?;

    // Validate type compatibility
    if !are_types_compatible(input_type, output_type) {
        return Err(format!(
            "Incompatible types for path {}: input {:?} cannot be used with output {:?}",
            path, input_type, output_type
        ));
    }

    Ok(())
}

fn validate_type_constraints(typ: &AnalysedType, constraint: TypeConstraint) -> Result<(), String> {
    match (typ, constraint) {
        // Validate primitive types
        (AnalysedType::Str(_), _) |
        (AnalysedType::I32, _) |
        (AnalysedType::I64, _) |
        (AnalysedType::F32(_), _) |
        (AnalysedType::F64(_), _) |
        (AnalysedType::Bool(_), _) |
        (AnalysedType::Unit, _) => Ok(()),

        // Validate lists
        (AnalysedType::List(l), c) => validate_type_constraints(&l.inner, c),

        // Validate records
        (AnalysedType::Record(r), c) => {
            for field in &r.fields {
                validate_type_constraints(&field.typ, c)?;
            }
            Ok(())
        },

        // Validate options
        (AnalysedType::Option(o), c) => validate_type_constraints(&o.inner, c),

        // Validate results
        (AnalysedType::Result(r), c) => {
            if let (Some(ok), Some(err)) = (&r.ok, &r.err) {
                validate_type_constraints(ok, c)?;
                validate_type_constraints(err, c)
            } else {
                Err("Result type must have both ok and err types".to_string())
            }
        },

        _ => Err(format!("Unsupported type {:?} for {:?}", typ, constraint))
    }
}

fn are_types_compatible(input: &AnalysedType, output: &AnalysedType) -> bool {
    match (input, output) {
        // Check primitive type compatibility
        (AnalysedType::Str(_), AnalysedType::Str(_)) |
        (AnalysedType::I32, AnalysedType::I32) |
        (AnalysedType::I64, AnalysedType::I64) |
        (AnalysedType::F32(_), AnalysedType::F32(_)) |
        (AnalysedType::F64(_), AnalysedType::F64(_)) |
        (AnalysedType::Bool(_), AnalysedType::Bool(_)) |
        (AnalysedType::Unit, AnalysedType::Unit) => true,

        // Check list compatibility
        (AnalysedType::List(l1), AnalysedType::List(l2)) => 
            are_types_compatible(&l1.inner, &l2.inner),

        // Check record compatibility
        (AnalysedType::Record(r1), AnalysedType::Record(r2)) => {
            r1.fields.len() == r2.fields.len() &&
            r1.fields.iter().zip(&r2.fields).all(|(f1, f2)| {
                f1.name == f2.name && are_types_compatible(&f1.typ, &f2.typ)
            })
        },

        // Other combinations are incompatible
        _ => false
    }
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