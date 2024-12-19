use std::collections::HashSet;
use crate::api::definition::types::{ApiDefinition, BindingType, Route, HttpMethod};
use crate::api::errors::{ValidationError, ValidationResult};
use golem_wasm_ast::analysis::{AnalysedType, TypeStr, TypeRecord};

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConstraint {
    Serializable,
    Deserializable,
}

pub fn validate_api_definition(api: &ApiDefinition) -> ValidationResult<()> {
    let mut errors = Vec::new();

    if let Err(err) = validate_route_uniqueness(&api.routes) {
        errors.push(err);
    }

    for route in &api.routes {
        if let BindingType::Default { input_type, output_type, .. } = &route.binding {
            // Validate input type is deserializable
            if let Err(err) = validate_type_constraints(input_type, TypeConstraint::Deserializable) {
                errors.push(ValidationError::Type(
                    format!("Route {} input: {}", route.path, err)
                ));
            }
            // Validate output type is serializable
            if let Err(err) = validate_type_constraints(output_type, TypeConstraint::Serializable) {
                errors.push(ValidationError::Type(
                    format!("Route {} output: {}", route.path, err)
                ));
            }
            // Validate type compatibility
            if let Err(err) = validate_type_compatibility(input_type, output_type) {
                errors.push(ValidationError::Type(
                    format!("Route {}: {}", route.path, err)
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationError::Multiple(errors))
    }
}

fn validate_type_constraints(typ: &AnalysedType, constraint: TypeConstraint) -> Result<(), String> {
    match (typ, constraint) {
        // Basic types are always valid
        (AnalysedType::Str(_), _) |
        (AnalysedType::S32(_), _) |
        (AnalysedType::S64(_), _) |
        (AnalysedType::F32(_), _) |
        (AnalysedType::F64(_), _) |
        (AnalysedType::Bool(_), _) => Ok(()),

        // Validate container types recursively
        (AnalysedType::List(l), c) => validate_type_constraints(&l.inner, c),
        
        (AnalysedType::Record(r), c) => {
            for field in &r.fields {
                validate_type_constraints(&field.typ, c)?;
            }
            Ok(())
        },

        (AnalysedType::Option(o), c) => validate_type_constraints(&o.inner, c),

        (AnalysedType::Result(r), c) => {
            if let (Some(ok), Some(err)) = (&r.ok, &r.err) {
                validate_type_constraints(ok, c)?;
                validate_type_constraints(err, c)
            } else {
                Err("Result type must have both ok and err types".to_string())
            }
        },

        _ => Err(format!("Unsupported type {:?} for constraint {:?}", typ, constraint))
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
        (AnalysedType::Bool(_), AnalysedType::Bool(_)) => true,

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

fn validate_route_uniqueness(routes: &[Route]) -> ValidationResult<()> {
    let mut seen_routes = HashSet::new();
    
    for route in routes {
        let route_key = format!("{} {}", route.method, route.path);
        if !seen_routes.insert(route_key.clone()) {
            return Err(ValidationError::Route(format!("Duplicate route: {}", route_key)));
        }
    }
    
    Ok(())
}

fn validate_type_compatibility(input: &AnalysedType, output: &AnalysedType) -> ValidationResult<()> {
    if !are_types_compatible(input, output) {
        return Err(ValidationError::Type(
            format!("Incompatible types: input {:?}, output {:?}", input, output)
        ));
    }
    Ok(())
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

    #[test]
    fn test_duplicate_routes() {
        let routes = vec![
            Route {
                path: "/test".to_string(),
                method: HttpMethod::Get,
                description: "Test 1".to_string(),
                template_name: "test1".to_string(),
                binding: BindingType::Default {
                    input_type: AnalysedType::Str(TypeStr),
                    output_type: AnalysedType::Str(TypeStr),
                    function_name: "test1".to_string(),
                },
            },
            Route {
                path: "/test".to_string(),
                method: HttpMethod::Get,
                description: "Test 2".to_string(),
                template_name: "test2".to_string(),
                binding: BindingType::Default {
                    input_type: AnalysedType::Str(TypeStr),
                    output_type: AnalysedType::Str(TypeStr),
                    function_name: "test2".to_string(),
                },
            },
        ];

        assert!(matches!(
            validate_route_uniqueness(&routes),
            Err(ValidationError::Route(_))
        ));
    }

    #[test]
    fn test_type_constraints() {
        let str_type = AnalysedType::Str(TypeStr);
        assert!(validate_type_constraints(&str_type, TypeConstraint::Serializable).is_ok());
        
        let record_type = AnalysedType::Record(TypeRecord {
            fields: vec![]
        });
        assert!(validate_type_constraints(&record_type, TypeConstraint::Deserializable).is_ok());
    }
}