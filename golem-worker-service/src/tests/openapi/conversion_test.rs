use crate::api::{
    definition::types::{ApiDefinition, Route, HttpMethod, BindingType},
    openapi::{OpenAPIConverter, Schema, SchemaKind, Type, PathItem, Operation, ReferenceOr, StatusCode},
};
use std::collections::HashMap;

fn create_test_route(path: &str, method: HttpMethod, input: &str, output: &str) -> Route {
    Route {
        path: path.to_string(),
        method,
        description: "Test route".to_string(),
        template_name: "test".to_string(),
        binding: BindingType::Default {
            input_type: input.to_string(),
            output_type: output.to_string(),
            function_name: "test_function".to_string(),
        },
    }
}

#[test]
fn test_primitive_type_conversion() {
    let types = [
        ("string", Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::String(Default::default())),
        }),
        ("i32", Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Integer(Default::default())),
        }),
        ("i64", Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Integer(Default::default())),
        }),
        ("f32", Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Number(Default::default())),
        }),
        ("f64", Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Number(Default::default())),
        }),
        ("bool", Schema {
            schema_data: Default::default(),
            schema_kind: SchemaKind::Type(Type::Boolean(Default::default())),
        }),
    ];

    for (wit_type, expected_schema) in types {
        let route = create_test_route(
            "/test",
            HttpMethod::Get,
            wit_type,
            wit_type,
        );
        let api = ApiDefinition {
            id: "test".to_string(),
            name: "Test API".to_string(),
            version: "1.0".to_string(),
            description: "Test API".to_string(),
            routes: vec![route],
        };

        let spec = OpenAPIConverter::convert(&api);
        let path_item = spec.paths.get("/test").unwrap();
        if let Some(operation) = &path_item.get {
            if let Some(request_body) = &operation.request_body {
                let schema = &request_body.content["application/json"].schema;
                assert_eq!(&expected_schema, schema);
            }
        }
    }
}

#[test]
fn test_complex_type_conversion() {
    let route = create_test_route(
        "/complex",
        HttpMethod::Post,
        "record{name: string, age: i32, tags: list<string>}",
        "record{id: string, data: record{value: f64, valid: bool}}",
    );

    let api = ApiDefinition {
        id: "test".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API".to_string(),
        routes: vec![route],
    };

    let spec = OpenAPIConverter::convert(&api);
    let path_item = spec.paths.get("/complex").unwrap();
    
    // Verify request body schema
    if let Some(operation) = &path_item.post {
        let schema = &operation.request_body.as_ref().unwrap().content["application/json"].schema;
        match schema {
            ReferenceOr::Item(Schema { schema_data: _, schema_kind }) => {
                match schema_kind {
                    SchemaKind::Type(Type::Object(obj)) => {
                        // Check top-level object properties
                        assert!(obj.properties.contains_key("name"));
                        assert!(obj.properties.contains_key("age"));
                        assert!(obj.properties.contains_key("tags"));
                        
                        // Check tags array
                        match &obj.properties["tags"] {
                            ReferenceOr::Item(Schema { schema_data: _, schema_kind: SchemaKind::Type(Type::Array(array_type)) }) => {
                                // Verify array items are strings
                                match &*array_type.items {
                                    ReferenceOr::Item(Schema { schema_data: _, schema_kind: SchemaKind::Type(Type::String(_)) }) => (),
                                    _ => panic!("Expected string type for array items"),
                                }
                            },
                            _ => panic!("Expected array schema for tags"),
                        }
                    },
                    _ => panic!("Expected object schema"),
                }
            },
            _ => panic!("Expected schema"),
        }

        // Verify response schema
        let response_schema = &operation.responses.responses[&StatusCode::Code(200)]
            .content.as_ref().unwrap()["application/json"].schema;
        
        match response_schema {
            ReferenceOr::Item(Schema { schema_data: _, schema_kind }) => {
                match schema_kind {
                    SchemaKind::Type(Type::Object(obj)) => {
                        // Check top-level response object properties
                        assert!(obj.properties.contains_key("id"));
                        assert!(obj.properties.contains_key("data"));
                        
                        // Check nested data object
                        match &obj.properties["data"] {
                            ReferenceOr::Item(Schema { schema_data: _, schema_kind: SchemaKind::Type(Type::Object(nested_obj)) }) => {
                                assert!(nested_obj.properties.contains_key("value"));
                                assert!(nested_obj.properties.contains_key("valid"));
                                
                                // Verify property types
                                match &nested_obj.properties["value"] {
                                    ReferenceOr::Item(Schema { schema_data: _, schema_kind: SchemaKind::Type(Type::Number(_)) }) => (),
                                    _ => panic!("Expected number type for value"),
                                }
                                match &nested_obj.properties["valid"] {
                                    ReferenceOr::Item(Schema { schema_data: _, schema_kind: SchemaKind::Type(Type::Boolean(_)) }) => (),
                                    _ => panic!("Expected boolean type for valid"),
                                }
                            },
                            _ => panic!("Expected object schema for data"),
                        }
                    },
                    _ => panic!("Expected object schema"),
                }
            },
            _ => panic!("Expected schema"),
        }
    }
}

#[test]
fn test_path_parameters() {
    let route = create_test_route(
        "/users/{id}/posts/{postId}",
        HttpMethod::Get,
        "string",
        "string",
    );

    let api = ApiDefinition {
        id: "test".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API".to_string(),
        routes: vec![route],
    };

    let spec = OpenAPIConverter::convert(&api);
    let path_item = spec.paths.get("/users/{id}/posts/{postId}").unwrap();
    
    if let Some(parameters) = &path_item.parameters {
        assert_eq!(parameters.len(), 2);
        
        // Check first parameter
        match &parameters[0] {
            ReferenceOr::Item(Parameter::Path { parameter_data, .. }) => {
                assert_eq!(parameter_data.name, "id");
                assert_eq!(parameter_data.required, true);
                match &parameter_data.format {
                    ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema { schema_data: _, schema_kind })) => {
                        assert!(matches!(schema_kind, SchemaKind::Type(Type::String(_))));
                    },
                    _ => panic!("Expected string schema for parameter"),
                }
            },
            _ => panic!("Expected path parameter"),
        }

        // Check second parameter
        match &parameters[1] {
            ReferenceOr::Item(Parameter::Path { parameter_data, .. }) => {
                assert_eq!(parameter_data.name, "postId");
                assert_eq!(parameter_data.required, true);
                match &parameter_data.format {
                    ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema { schema_data: _, schema_kind })) => {
                        assert!(matches!(schema_kind, SchemaKind::Type(Type::String(_))));
                    },
                    _ => panic!("Expected string schema for parameter"),
                }
            },
            _ => panic!("Expected path parameter"),
        }
    } else {
        panic!("Expected path parameters");
    }
}

#[test]
fn test_cors_headers() {
    let route = create_test_route(
        "/test",
        HttpMethod::Get,
        "string",
        "string",
    );

    let api = ApiDefinition {
        id: "test".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API".to_string(),
        routes: vec![route],
    };

    let spec = OpenAPIConverter::convert(&api);
    let path_item = spec.paths.get("/test").unwrap();
    
    // Verify CORS options
    if let Some(options) = &path_item.options {
        match &options.responses.responses[&StatusCode::Code(200)] {
            ReferenceOr::Item(response) => {
                let headers = response.headers.as_ref().unwrap();
                
                assert!(headers.contains_key("Access-Control-Allow-Origin"));
                assert!(headers.contains_key("Access-Control-Allow-Methods"));
                assert!(headers.contains_key("Access-Control-Allow-Headers"));
            },
            _ => panic!("Expected response"),
        }
    } else {
        panic!("Expected OPTIONS operation");
    }
}

#[test]
fn test_file_server_binding() {
    let route = Route {
        path: "/files/{path}".to_string(),
        method: HttpMethod::Get,
        description: "Serve files".to_string(),
        template_name: "files".to_string(),
        binding: BindingType::FileServer {
            root_dir: "/static".to_string(),
        },
    };

    let api = ApiDefinition {
        id: "test".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API".to_string(),
        routes: vec![route],
    };

    let spec = OpenAPIConverter::convert(&api);
    let path_item = spec.paths.get("/files/{path}").unwrap();
    
    if let Some(operation) = &path_item.get {
        let response = &operation.responses.responses[&StatusCode::Code(200)];
        let content = response.content.as_ref().unwrap();
        assert!(content.contains_key("*/*"));
        
        match &content["*/*"].schema {
            ReferenceOr::Item(Schema { schema_data: _, schema_kind }) => {
                match schema_kind {
                    SchemaKind::Type(Type::String(string_type)) => {
                        assert_eq!(string_type.format.as_deref(), Some("binary"));
                    },
                    _ => panic!("Expected string schema with binary format"),
                }
            },
            _ => panic!("Expected schema"),
        }
    } else {
        panic!("Expected GET operation");
    }
}

#[test]
fn test_swagger_ui_binding() {
    let route = Route {
        path: "/docs".to_string(),
        method: HttpMethod::Get,
        description: "API Documentation".to_string(),
        template_name: "docs".to_string(),
        binding: BindingType::SwaggerUI {
            spec_path: "/api/openapi/my-api/v1".to_string(),
        },
    };

    let api = ApiDefinition {
        id: "my-api".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API".to_string(),
        routes: vec![route],
    };

    let spec = OpenAPIConverter::convert(&api);
    let path_item = spec.paths.get("/docs").unwrap();
    
    // Verify SwaggerUI route is converted correctly
    if let Some(operation) = &path_item.get {
        assert_eq!(
            operation.summary,
            Some("API Documentation".to_string())
        );
        match &operation.responses.responses[&StatusCode::Code(200)] {
            ReferenceOr::Item(response) => {
                assert!(response.content.is_some());
            },
            _ => panic!("Expected response"),
        }
    } else {
        panic!("Expected GET operation for SwaggerUI");
    }
}

#[test]
fn test_multi_segment_path_parameters() {
    let route = create_test_route(
        "/files/{path..}",
        HttpMethod::Get,
        "string",
        "string",
    );

    let api = ApiDefinition {
        id: "test".to_string(),
        name: "Test API".to_string(),
        version: "1.0".to_string(),
        description: "Test API".to_string(),
        routes: vec![route],
    };

    let spec = OpenAPIConverter::convert(&api);
    let path_item = spec.paths.get("/files/{path}").unwrap();
    
    if let Some(parameters) = &path_item.parameters {
        assert_eq!(parameters.len(), 1);
        match &parameters[0] {
            ReferenceOr::Item(Parameter::Path { parameter_data, .. }) => {
                assert_eq!(parameter_data.name, "path");
                assert_eq!(parameter_data.required, true);
                assert!(parameter_data.description.as_ref().unwrap().contains("multi-segment"));
                
                match &parameter_data.format {
                    ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema { schema_data: _, schema_kind })) => {
                        assert!(matches!(schema_kind, SchemaKind::Type(Type::String(_))));
                    },
                    _ => panic!("Expected string schema for parameter"),
                }
            },
            _ => panic!("Expected path parameter"),
        }
    } else {
        panic!("Expected path parameters");
    }
}
