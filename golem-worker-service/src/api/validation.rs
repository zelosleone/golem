use crate::api::definition::types::{ApiDefinition, BindingType};
use golem_wasm_ast::analysis::AnalysedType;

// Remove incorrect imports
// use wasm_ast::analysis::model::{TypeI32, TypeI64, TypeF32, TypeF64, TypeUnit}; 

#[derive(Debug, PartialEq)]
enum TypeConstraint {
    Input,
    Output,
}

pub fn validate_api_definition(api: &ApiDefinition) -> Result<(), String> {
    for route in &api.routes {
        if let BindingType::Default { input_type, output_type, .. } = &route.binding {
            // Now input_type and output_type are already AnalysedType,
            // so we can directly validate them.
            validate_wit_binding_types(input_type, output_type, route.path.as_str())?;
        }
    }
    Ok(())
}

// Replace validate_wit_binding_types function:
fn validate_wit_binding_types(
    input_type: &str,
    output_type: &str,
    path: &str,
) -> Result<(), String> {
    // For now, just validate that they're not empty
    if input_type.is_empty() {
        return Err(format!("Empty input type for path {}", path));
    }
    if output_type.is_empty() {
        return Err(format!("Empty output type for path {}", path));
    }
    Ok(())
}

fn validate_type_constraints(typ: &AnalysedType, constraint: TypeConstraint) -> Result<(), String> {
    match (typ, constraint) {
        (AnalysedType::Str(_), _) |
        (AnalysedType::S32(_), _) |
        (AnalysedType::S64(_), _) |
        (AnalysedType::F32(_), _) |
        (AnalysedType::F64(_), _) |
        (AnalysedType::Bool(_), _) |
        (AnalysedType::Unit(_), _) => Ok(()),

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
        (AnalysedType::S32(_), AnalysedType::S32(_)) |
        (AnalysedType::S64(_), AnalysedType::S64(_)) |
        (AnalysedType::F32(_), AnalysedType::F32(_)) |
        (AnalysedType::F64(_), AnalysedType::F64(_)) |
        (AnalysedType::Bool(_), AnalysedType::Bool(_)) |
        (AnalysedType::Unit(_), AnalysedType::Unit(_)) => true,

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
    use crate::api::definition::types::{Route, HttpMethod, ApiDefinition, BindingType};
    use golem_wasm_ast::analysis::{AnalysedType, TypeStr};

    #[test]
    fn test_valid_api_definition() {
        // Assume we have a route with already analyzed types:
        let input_type = AnalysedType::Str(TypeStr);
        let output_type = AnalysedType::Str(TypeStr);

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
                        input_type,
                        output_type,
                        function_name: "test".to_string(),
                    },
                },
            ],
        };
        assert!(validate_api_definition(&api).is_ok());
    }

    #[test]
    fn test_incompatible_types_api_definition() {
        let input_type = AnalysedType::Str(TypeStr);
        // For example, output is a record while input is a string
        let output_type = AnalysedType::Record(TypeRecord { fields: vec![] });

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
                        input_type,
                        output_type,
                        function_name: "test".to_string(),
                    },
                },
            ],
        };
        assert!(validate_api_definition(&api).is_err());
    }
}